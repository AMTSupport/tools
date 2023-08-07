/*
 * Copyright (C) 2023 James Draycott <me@racci.dev>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, version 3.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
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

#[derive(Debug, Clone)]
pub struct DB(Vec<Record>);

pub static DB_INSTANCE: LazyLock<DB> = LazyLock::new(|| {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async { DB::new().await }).unwrap()
});

impl DB {
    // TODO :: Get latest version from https://db-ip.com/db/download/ip-to-country-lite
    const URL: &'static str = "https://download.db-ip.com/free/dbip-country-lite-2023-08.csv.gz";

    #[instrument]
    async fn new() -> Result<Self> {
        let response = reqwest::get(Self::URL).await?;
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
            let split = line.split(',').map(|s| s.into()).collect::<Vec<String>>();
            if split.len() != 3 {
                error!("Invalid line {line}; skipping");
                continue;
            }

            let value = &*split[0];
            let start = match IpAddr::from_str(value) {
                Ok(start) => start,
                Err(e) => {
                    error!("Failed to parse start {value}; {e}");
                    continue;
                }
            };

            let value = &*split[1];
            let end = match IpAddr::from_str(value) {
                Ok(end) => end,
                Err(e) => {
                    error!("Failed to parse end {value}; {e}");
                    continue;
                }
            };

            let value = &*split[2];
            let alpha = match Alpha2::try_from(value) {
                Ok(alpha) => alpha,
                Err(e) => {
                    error!("Failed to parse country {value}; {e}");
                    continue;
                }
            };

            records.push(DBRecord { alpha, start, end })
        }

        Ok(Self(records))
    }

    pub fn instance() -> Box<dyn RecordDB> {
        Box::new(DB_INSTANCE.clone())
    }
}

#[async_trait]
impl RecordDB for DB {
    #[instrument]
    async fn lookup(&self, ip: &IpAddr) -> Option<Alpha2> {
        for record in &self.0 {
            if &record.start() <= ip && &record.end() >= ip {
                return Some(*record.alpha());
            }
        }
        None
    }

    #[instrument]
    async fn filtered(&self, alpha: &Alpha2) -> Vec<&Record> {
        self.0.iter().filter(|record| record.alpha() == alpha).collect::<Vec<&Record>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use keshvar::Alpha2::AU;
    use std::net::Ipv4Addr;

    #[test_log::test(tokio::test)]
    async fn test_download() -> Result<()> {
        let db = DB::new().await;
        assert!(db.is_ok());

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn test_db() -> Result<()> {
        let db = DB::new().await?;

        let record = DBRecord {
            start: IpAddr::V4(Ipv4Addr::new(101, 160, 0, 0)),
            end: IpAddr::V4(Ipv4Addr::new(101, 191, 255, 255)),
            alpha: AU,
        };
        let random = record.random();

        assert_eq!(db.lookup(&random).await, Some(AU));
        assert_eq!(db.lookup(&IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1))).await, None);

        Ok(())
    }
}
