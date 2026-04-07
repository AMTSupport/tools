/*
 * Copyright (C) 2023-2024. James Draycott me@racci.dev
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, version 3.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
 * See the GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see https://www.gnu.org/licenses/.
 */

use crate::record::Record;
use crate::record::Record::DBRecord;
use crate::RecordDB;
use anyhow::Result;
use async_compression::tokio::bufread::GzipDecoder;
use async_trait::async_trait;
use futures::io::Error as FutureError;
use futures::TryStreamExt;
use keshvar::Alpha2;
use std::net::IpAddr;
use std::str::FromStr;
use std::sync::LazyLock;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_util::compat::FuturesAsyncReadCompatExt;
use tracing::{error, instrument};

/// Parse a single CSV line into a DBRecord.
/// Returns `None` if the line is malformed.
fn parse_csv_line(line: &str) -> Option<Record> {
    let split = line.split(',').map(|s| s.into()).collect::<Vec<String>>();
    if split.len() != 3 {
        error!("Invalid line {line}; skipping");
        return None;
    }

    let start = IpAddr::from_str(&split[0])
        .map_err(|e| error!("Failed to parse start {}; {e}", split[0]))
        .ok()?;
    let end = IpAddr::from_str(&split[1])
        .map_err(|e| error!("Failed to parse end {}; {e}", split[1]))
        .ok()?;
    let alpha = Alpha2::try_from(&*split[2])
        .map_err(|e| error!("Failed to parse country {}; {e}", split[2]))
        .ok()?;

    Some(DBRecord { alpha, start, end })
}

#[derive(Debug, Clone)]
pub struct DB(Vec<Record>);

pub static DB_INSTANCE: LazyLock<DB> = LazyLock::new(|| {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async { DB::new().await }).unwrap()
});

impl DB {
    /// Get the URL to download the current csv file from.
    fn get_dated_url() -> String {
        use chrono::Datelike;

        let year = chrono::Utc::now().year();
        let month = chrono::Utc::now().month();
        format!(
            "https://download.db-ip.com/free/dbip-country-lite-{}-{}.csv.gz",
            year, month
        )
    }

    // TODO - Cache the csv file for reuse.
    #[instrument(level = "TRACE", ret, err)]
    async fn new() -> Result<Self> {
        let response = reqwest::get(Self::get_dated_url()).await?;
        let stream = response
            .bytes_stream()
            .map_err(|e| FutureError::new(futures::io::ErrorKind::Other, e))
            .into_async_read()
            .compat();
        let decoder = GzipDecoder::new(stream);
        let reader = BufReader::new(decoder);
        let mut lines = reader.lines();

        let mut records = vec![];
        while let Some(line) = lines.next_line().await? {
            if let Some(record) = parse_csv_line(&line) {
                records.push(record);
            }
        }

        Ok(Self(records))
    }

    pub fn instance() -> Box<dyn RecordDB> {
        Box::new(DB_INSTANCE.clone())
    }
}

#[async_trait]
impl RecordDB for DB {
    #[instrument(level = "TRACE", ret)]
    async fn lookup(&self, ip: &IpAddr) -> Option<Alpha2> {
        for record in &self.0 {
            if &record.start() <= ip && &record.end() >= ip {
                return Some(*record.alpha());
            }
        }
        None
    }

    #[instrument(level = "TRACE", ret)]
    async fn filtered(&self, alpha: &Alpha2) -> Vec<&Record> {
        self.0.iter().filter(|record| record.alpha() == alpha).collect::<Vec<&Record>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use keshvar::Alpha2::AU;
    use std::net::Ipv4Addr;

    #[test_log::test]
    fn test_parse_csv_line_valid() -> Result<()> {
        let line = "101.160.0.0,101.191.255.255,AU";
        let record = parse_csv_line(line).ok_or(anyhow::anyhow!("should parse valid line"))?;
        assert_eq!(record.start(), IpAddr::V4(Ipv4Addr::new(101, 160, 0, 0)));
        assert_eq!(record.end(), IpAddr::V4(Ipv4Addr::new(101, 191, 255, 255)));
        assert_eq!(*record.alpha(), AU);
        Ok(())
    }

    #[test_log::test]
    fn test_parse_csv_line_invalid_columns() {
        let line = "1.2.3.4,AU";
        assert!(parse_csv_line(line).is_none());
    }

    #[test_log::test]
    fn test_parse_csv_line_invalid_ip() {
        let line = "not-an-ip,10.0.0.1,AU";
        assert!(parse_csv_line(line).is_none());
    }

    #[test_log::test]
    fn test_parse_csv_line_invalid_country() {
        let line = "1.2.3.4,1.2.3.255,XYZ";
        assert!(parse_csv_line(line).is_none());
    }

    fn parse_fixture(lines: &[&str]) -> Vec<Record> {
        lines.iter().filter_map(|l| parse_csv_line(l)).collect()
    }

    #[test_log::test]
    fn test_lookup_from_fixture() {
        let records = parse_fixture(&["101.160.0.0,101.191.255.255,AU", "1.0.0.0,1.0.0.255,US"]);
        let db = DB(records);

        let rt = tokio::runtime::Runtime::new()
            .ok()
            .and_then(|r| Some(r.block_on(db.lookup(&IpAddr::V4(Ipv4Addr::new(101, 160, 0, 1))))));
        assert_eq!(rt, Some(Some(AU)));

        assert_eq!(
            block_lookup(&db, &IpAddr::V4(Ipv4Addr::new(1, 0, 0, 10))),
            Some(Alpha2::US)
        );
        assert_eq!(block_lookup(&db, &IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1))), None);
    }

    #[test_log::test]
    fn test_filtered_from_fixture() {
        let records =
            parse_fixture(&["101.160.0.0,101.191.255.255,AU", "1.0.0.0,1.0.0.255,US", "2.0.0.0,2.0.0.255,AU"]);
        let db = DB(records);

        let au_records = block_filtered(&db, &AU);
        assert_eq!(au_records.len(), 2);

        let us_records = block_filtered(&db, &Alpha2::US);
        assert_eq!(us_records.len(), 1);

        let missing_records = block_filtered(&db, &Alpha2::GB);
        assert_eq!(missing_records.len(), 0);
    }

    fn block_lookup(db: &DB, ip: &IpAddr) -> Option<Alpha2> {
        tokio::runtime::Runtime::new().ok().and_then(|rt| rt.block_on(db.lookup(ip)))
    }

    fn block_filtered<'a>(db: &'a DB, alpha: &'a Alpha2) -> Vec<&'a Record> {
        tokio::runtime::Runtime::new()
            .ok()
            .map(|rt| rt.block_on(db.filtered(alpha)))
            .unwrap_or_default()
    }
}
