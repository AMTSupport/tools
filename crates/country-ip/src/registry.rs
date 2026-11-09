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

use crate::cache;
use crate::index::RangeIndex;
use crate::record::Record::RegistryRecord;
use crate::record::{Record, Status};
use crate::registry::RegistryErrors::RegistryFailed;
use crate::RecordDB;
use anyhow::{Context, Result};
use async_trait::async_trait;
use keshvar::{Alpha2, Country, Region, SubRegion};
use macros::EnumVariants;
use rayon::prelude::{ParallelBridge, ParallelIterator};
use std::collections::HashMap;
use std::net::IpAddr;
use std::path::PathBuf;

use std::sync::{Arc, LazyLock};
use std::time::Duration;
use thiserror::Error;
use tokio::sync::Mutex;
use tracing::{debug, error, instrument};

/// Parse a single line from a RIR delegated-extended stats file into a RegistryRecord.
/// Returns `None` if the line is a comment or malformed.
fn parse_registry_line(line: &str, registry: Registry) -> Option<Record> {
    if line.starts_with('#') || line.ends_with("|summary") {
        return None;
    }

    let mut fields = line.split('|');
    let (Some(_source), Some(alpha), Some(_kind), Some(value), Some(range), Some(_date), Some(status)) = (
        fields.next(),
        fields.next(),
        fields.next(),
        fields.next(),
        fields.next(),
        fields.next(),
        fields.next(),
    ) else {
        error!("Invalid line {line}; skipping");
        return None;
    };

    let alpha = match alpha {
        str if str.len() != 2 => return None,
        str => match Alpha2::try_from(str) {
            Ok(alpha) => alpha,
            Err(_) => return None,
        },
    };

    let value = match value.parse::<IpAddr>() {
        Ok(value) => value,
        Err(_) => return None,
    };

    let range = match range.parse::<u32>() {
        Ok(range) => range,
        Err(_) => return None,
    };

    let status = match status {
        "assigned" => Status::Assigned,
        "allocated" => Status::Allocated,
        "reserved" => Status::Reserved,
        _ => return None,
    };

    Some(RegistryRecord {
        registry,
        alpha,
        value,
        range,
        status,
    })
}

#[derive(Debug, Error)]
pub enum RegistryErrors {
    #[error("Failed to get data for {}", .0.name())]
    DownloadFailed(Registry, #[source] anyhow::Error),

    #[error("Failed to parse data for {}", .0.name())]
    ParseFailed(Registry, #[source] anyhow::Error),

    #[error("Failed to get internet registry for country {}", .0.iso_short_name())]
    RegistryFailed(Box<Country>, #[source] Option<anyhow::Error>),
}

#[derive(Debug, EnumVariants, Clone, Copy, Hash, Eq, PartialEq)]
pub enum Registry {
    Afrinic,
    Apnic,
    Arin,
    Lacnic,
    Ripencc,
}

/// For some reason there are a small number of countries that are not assigned correctly,
/// Such as Christmas Island (CX) being assigned to Australia (AU)
/// This list is used to know which countries are not assigned correctly
/// And we will then delegate them to the ```db_ip``` registry instead.
const DELEGATES: [Alpha2; 12] = [
    Alpha2::AQ, // Antarctica
    Alpha2::BV, // Bouvet Island
    Alpha2::CC, // Cocos (Keeling) Islands
    Alpha2::CX, // Christmas Island
    Alpha2::EH, // Western Sahara
    Alpha2::GS, // South Georgia and the South Sandwich Islands
    Alpha2::HM, // Heard Island and McDonald Islands
    Alpha2::PN, // Pitcairn
    Alpha2::SH, // Saint Helena, Ascension and Tristan da Cunha
    Alpha2::SJ, // Svalbard and Jan Mayen
    Alpha2::TF, // French Southern Territories
    Alpha2::UM, // United States Minor Outlying Islands
];

#[derive(Debug, Clone)]
pub struct RegistryRecords {
    pub registry: Registry,
    pub inner: Vec<Record>,
    index: RangeIndex,
}

impl RegistryRecords {
    pub fn new(registry: Registry, inner: Vec<Record>) -> Self {
        let index = RangeIndex::new(&inner);
        Self { registry, inner, index }
    }
}

#[async_trait]
impl RecordDB for RegistryRecords {
    #[instrument(level = "TRACE", ret)]
    async fn lookup(&self, ip: &IpAddr) -> Option<Alpha2> {
        self.index.lookup(ip)
    }

    #[instrument(level = "TRACE", ret)]
    async fn filtered(&self, country: &Alpha2) -> Vec<&Record> {
        self.inner
            .iter()
            .par_bridge()
            .filter(|record| record.alpha() == country)
            .filter(|record| match record {
                RegistryRecord { status, .. } => *status == Status::Allocated,
                _ => true,
            })
            .collect()
    }
}

#[async_trait]
impl RecordDB for Arc<RegistryRecords> {
    #[instrument(level = "TRACE", ret)]
    async fn lookup(&self, ip: &IpAddr) -> Option<Alpha2> {
        self.as_ref().lookup(ip).await
    }

    #[instrument(level = "TRACE", ret)]
    async fn filtered(&self, country: &Alpha2) -> Vec<&Record> {
        self.as_ref().filtered(country).await
    }
}

static DOWNLOADED: LazyLock<Mutex<HashMap<Registry, Arc<RegistryRecords>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Dumps are refreshed daily, so a day-old cache is as good as a fresh download.
const CACHE_MAX_AGE: Duration = Duration::from_secs(24 * 60 * 60);

impl Registry {
    #[instrument(level = "TRACE", ret, err, fields(country = %country.iso_short_name()))]
    pub fn get_for(country: &Country) -> Result<Registry> {
        use Registry::*;

        if DELEGATES.contains(&country.alpha2()) {
            return Err(RegistryFailed(Box::new(country.clone()), None).into());
        }

        let region = country.maybe_region();
        let registry = match region {
            Some(Region::Oceania) => Apnic,
            Some(Region::Europe) => Ripencc,
            Some(Region::Antarctica) => Arin,
            Some(Region::Africa) => match country.alpha2() {
                Alpha2::IO => Apnic,
                _ => Afrinic,
            },
            Some(Region::Americas) => {
                match country
                    .maybe_subregion()
                    .with_context(|| format!("Failed to get subregion for country {country:?}"))?
                {
                    SubRegion::CentralAmerica => Lacnic,
                    SubRegion::NorthernAmerica => match country.alpha2() {
                        Alpha2::GL => Ripencc,
                        _ => Arin,
                    },
                    SubRegion::SouthAmerica => match country.alpha2() {
                        Alpha2::FK => Ripencc,
                        _ => Lacnic,
                    },
                    SubRegion::Caribbean => match country.alpha2() {
                        Alpha2::AW | Alpha2::BQ | Alpha2::CU | Alpha2::CW | Alpha2::HT | Alpha2::SX | Alpha2::TT => {
                            Lacnic
                        }
                        _ => Arin,
                    },
                    _ => return Err(RegistryFailed(Box::new(country.clone()), None).into()),
                }
            }
            Some(Region::Asia) => {
                match country
                    .maybe_subregion()
                    .with_context(|| format!("Failed to get subregion for country {country:?}"))?
                {
                    SubRegion::CentralAsia | SubRegion::WesternAsia => Ripencc,
                    SubRegion::EasternAsia | SubRegion::SouthEasternAsia => Apnic,
                    SubRegion::SouthernAsia => match country.alpha2() {
                        Alpha2::IR => Ripencc,
                        _ => Apnic,
                    },
                    _ => return Err(RegistryFailed(Box::new(country.clone()), None).into()),
                }
            }
            None => return Err(RegistryFailed(Box::new(country.clone()), None).into()),
        };

        Ok(registry)
    }

    pub const fn name(&self) -> &'static str {
        match self {
            Registry::Afrinic => "afrinic",
            Registry::Apnic => "apnic",
            Registry::Arin => "arin",
            Registry::Lacnic => "lacnic",
            Registry::Ripencc => "ripe-ncc",
        }
    }

    pub const fn target_name(&self) -> &'static str {
        match self {
            Registry::Ripencc => "ripencc",
            _ => self.name(),
        }
    }

    pub const fn suffix(&self) -> &'static str {
        "extended-latest"
    }

    pub fn url(&self) -> String {
        let name = self.name();
        let target_name = self.target_name();
        let suffix = self.suffix();

        format!("https://ftp.apnic.net/stats/{name}/delegated-{target_name}-{suffix}")
    }

    #[instrument(level = "TRACE", err, ret)]
    pub async fn get(&self) -> Result<Box<dyn RecordDB>> {
        let records = match self.downloaded().await {
            Err(_) => {
                self.download().await?;
                self.downloaded().await
            }
            value => value,
        }?;

        Ok(Box::new(records.clone()))
    }

    #[instrument(level = "TRACE", ret)]
    async fn downloaded(&self) -> Result<Arc<RegistryRecords>> {
        let map = &(DOWNLOADED.lock().await);
        map.get(self).ok_or(anyhow::anyhow!("Data not downloaded")).cloned()
    }

    #[instrument(level = "TRACE", ret, err)]
    async fn download(&self) -> Result<()> {
        let cache_path = self.cache_path();
        let data = match cache_path.as_deref().and_then(|path| cache::read(path, CACHE_MAX_AGE)) {
            Some(data) => {
                debug!("Using cached dump for {}", self.name());
                data
            }
            None => {
                let data = self.fetch().await?;

                if let Some(path) = cache_path {
                    cache::write(&path, &data);
                }

                data
            }
        };

        let parsed_lines = data
            .lines()
            .par_bridge()
            .filter_map(|line| parse_registry_line(line, *self))
            .collect::<Vec<_>>();

        let record = RegistryRecords::new(*self, parsed_lines);

        let mut map = DOWNLOADED.lock().await;
        map.insert(*self, Arc::new(record));

        Ok(())
    }

    /// Cache location of this registry's dump, if a cache directory is available.
    fn cache_path(&self) -> Option<PathBuf> {
        let name = format!("delegated-{}-{}.txt", self.target_name(), self.suffix());
        cache::dir().map(|dir| dir.join(name))
    }

    #[instrument(level = "TRACE", err)]
    async fn fetch(&self) -> Result<String> {
        let url = self.url();
        let response = reqwest::get(&url)
            .await
            .map_err(|e| RegistryErrors::DownloadFailed(*self, e.into()))?;
        let response = response
            .error_for_status()
            .map_err(|e| RegistryErrors::DownloadFailed(*self, e.into()))?;
        let bytes = response
            .bytes()
            .await
            .map_err(|e| RegistryErrors::DownloadFailed(*self, e.into()))?;

        let data = String::from_utf8(bytes.to_vec()).map_err(|e| RegistryErrors::ParseFailed(*self, e.into()))?;

        Ok(data)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use keshvar::Country;
    use std::net::Ipv4Addr;

    #[test_log::test]
    fn test_parse_registry_line_valid() -> Result<()> {
        let line = "apnic|AU|ipv4|101.160.0.0|4194304|20250101|allocated";
        let record = parse_registry_line(line, Registry::Apnic).ok_or(anyhow::anyhow!("should parse valid line"))?;
        assert_eq!(*record.alpha(), Alpha2::AU);
        assert_eq!(record.start(), IpAddr::V4(Ipv4Addr::new(101, 160, 0, 0)));
        Ok(())
    }

    #[test_log::test]
    fn test_parse_registry_line_comment() {
        let line = "# This is a comment";
        assert!(parse_registry_line(line, Registry::Apnic).is_none());
    }

    #[test_log::test]
    fn test_parse_registry_line_summary() {
        let line = "apnic|*|ipv6|*|114553|summary";
        assert!(parse_registry_line(line, Registry::Apnic).is_none());
    }

    #[test_log::test]
    fn test_parse_registry_line_invalid_alpha() {
        let line = "apnic|XXX|ipv4|1.2.3.0|256|20250101|allocated";
        assert!(parse_registry_line(line, Registry::Apnic).is_none());
    }

    #[test_log::test]
    fn test_parse_registry_line_invalid_status() {
        let line = "apnic|AU|ipv4|1.2.3.0|256|20250101|unknown";
        assert!(parse_registry_line(line, Registry::Apnic).is_none());
    }

    #[test_log::test]
    fn test_get_for_au_returns_apnic() -> Result<()> {
        let country = Country::try_from(Alpha2::AU)?;
        assert_eq!(Registry::get_for(&country)?, Registry::Apnic);
        Ok(())
    }

    #[test_log::test]
    fn test_get_for_de_returns_ripencc() -> Result<()> {
        let country = Country::try_from(Alpha2::DE)?;
        assert_eq!(Registry::get_for(&country)?, Registry::Ripencc);
        Ok(())
    }

    #[test_log::test]
    fn test_get_for_us_returns_arin() -> Result<()> {
        let country = Country::try_from(Alpha2::US)?;
        assert_eq!(Registry::get_for(&country)?, Registry::Arin);
        Ok(())
    }

    #[test_log::test]
    fn test_get_for_br_returns_lacnic() -> Result<()> {
        let country = Country::try_from(Alpha2::BR)?;
        assert_eq!(Registry::get_for(&country)?, Registry::Lacnic);
        Ok(())
    }

    #[test_log::test]
    fn test_get_for_za_returns_afrinic() -> Result<()> {
        let country = Country::try_from(Alpha2::ZA)?;
        assert_eq!(Registry::get_for(&country)?, Registry::Afrinic);
        Ok(())
    }

    #[test_log::test]
    fn test_get_for_cx_returns_err_delegated() -> Result<()> {
        let country = Country::try_from(Alpha2::CX)?;
        assert!(Registry::get_for(&country).is_err());
        Ok(())
    }

    #[test_log::test]
    fn test_get_for_aq_returns_err_delegated() -> Result<()> {
        let country = Country::try_from(Alpha2::AQ)?;
        assert!(Registry::get_for(&country).is_err());
        Ok(())
    }

    #[test_log::test]
    fn test_registry_url_not_empty() {
        for reg in Registry::get_variants() {
            let url = reg.url();
            assert!(!url.is_empty(), "URL for {reg:?} should not be empty");
            assert!(url.starts_with("https://"), "URL for {reg:?} should start with https");
        }
    }
}
