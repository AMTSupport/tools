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

use crate::cache;
use crate::index::RangeIndex;
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
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use tokio::io::{AsyncReadExt, BufReader};
use tokio::sync::OnceCell;
use tokio_util::compat::FuturesAsyncReadCompatExt;
use tracing::{debug, error, instrument};

/// Parse a single CSV line into a DBRecord.
/// Returns `None` if the line is malformed.
fn parse_csv_line(line: &str) -> Option<Record> {
    let mut fields = line.split(',');
    let (Some(start), Some(end), Some(alpha), None) = (fields.next(), fields.next(), fields.next(), fields.next())
    else {
        error!("Invalid line {line}; skipping");
        return None;
    };

    let start = start
        .parse::<IpAddr>()
        .map_err(|e| error!("Failed to parse start {start}; {e}"))
        .ok()?;
    let end = end
        .parse::<IpAddr>()
        .map_err(|e| error!("Failed to parse end {end}; {e}"))
        .ok()?;
    let alpha = Alpha2::try_from(alpha)
        .map_err(|e| error!("Failed to parse country {alpha}; {e}"))
        .ok()?;

    Some(DBRecord { alpha, start, end })
}

#[derive(Debug, Clone)]
pub struct DB {
    records: Vec<Record>,
    index: RangeIndex,
}

impl DB {
    fn from_records(records: Vec<Record>) -> Self {
        let index = RangeIndex::new(&records);
        Self { records, index }
    }
}

static DB_INSTANCE: OnceCell<Arc<DB>> = OnceCell::const_new();

/// DB-IP free dumps are published monthly; refresh local cache daily.
const CACHE_MAX_AGE: Duration = Duration::from_secs(24 * 60 * 60);

impl DB {
    fn dated_parts() -> (i32, u32) {
        use chrono::Datelike;

        (chrono::Utc::now().year(), chrono::Utc::now().month())
    }

    /// Get the URL to download the current csv file from.
    fn get_dated_url() -> String {
        let (year, month) = Self::dated_parts();
        format!(
            "https://download.db-ip.com/free/dbip-country-lite-{}-{:02}.csv.gz",
            year, month
        )
    }

    fn cache_path() -> Option<PathBuf> {
        let (year, month) = Self::dated_parts();
        let name = format!("dbip-country-lite-{}-{:02}.csv", year, month);
        cache::dir().map(|dir| dir.join(name))
    }

    async fn download_csv() -> Result<String> {
        let response = reqwest::get(Self::get_dated_url()).await?.error_for_status()?;
        let stream = response
            .bytes_stream()
            .map_err(|e| FutureError::new(futures::io::ErrorKind::Other, e))
            .into_async_read()
            .compat();
        let decoder = GzipDecoder::new(stream);
        let mut reader = BufReader::new(decoder);

        let mut csv = String::new();
        reader.read_to_string(&mut csv).await?;
        Ok(csv)
    }

    #[instrument(level = "TRACE", ret, err)]
    async fn new() -> Result<Self> {
        let cache_path = Self::cache_path();
        let csv = match cache_path.as_ref() {
            Some(path) => match cache::read_async(path.clone(), CACHE_MAX_AGE).await {
                Some(csv) => {
                    debug!("Using cached DB-IP CSV");
                    csv
                }
                None => {
                    let csv = Self::download_csv().await?;
                    cache::write(path, &csv);
                    csv
                }
            },
            None => Self::download_csv().await?,
        };

        let mut records = vec![];
        for line in csv.lines() {
            if let Some(record) = parse_csv_line(line) {
                records.push(record);
            }
        }

        Ok(Self::from_records(records))
    }

    pub async fn instance() -> Result<Box<dyn RecordDB>> {
        let db = DB_INSTANCE.get_or_try_init(|| async { DB::new().await.map(Arc::new) }).await?;
        Ok(Box::new(db.clone()))
    }
}

#[async_trait]
impl RecordDB for DB {
    #[instrument(level = "TRACE", ret)]
    async fn lookup(&self, ip: &IpAddr) -> Option<Alpha2> {
        self.index.lookup(ip)
    }

    #[instrument(level = "TRACE", ret)]
    async fn filtered(&self, alpha: &Alpha2) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.alpha() == alpha)
            .collect::<Vec<&Record>>()
    }
}

#[async_trait]
impl RecordDB for Arc<DB> {
    #[instrument(level = "TRACE", ret)]
    async fn lookup(&self, ip: &IpAddr) -> Option<Alpha2> {
        self.as_ref().lookup(ip).await
    }

    #[instrument(level = "TRACE", ret)]
    async fn filtered(&self, alpha: &Alpha2) -> Vec<&Record> {
        self.as_ref().filtered(alpha).await
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
        assert_eq!(record.end(), Some(IpAddr::V4(Ipv4Addr::new(101, 191, 255, 255))));
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
        let db = DB::from_records(records);

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
        let db = DB::from_records(records);

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
