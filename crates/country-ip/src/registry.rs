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
use std::io::BufRead;
use std::net::IpAddr;
use std::str::FromStr;
use std::sync::LazyLock;
use thiserror::Error;
use tokio::sync::Mutex;
use tracing::instrument;

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
}

#[async_trait]
impl RecordDB for RegistryRecords {
    #[instrument(level = "TRACE", ret)]
    async fn lookup(&self, ip: &IpAddr) -> Option<Alpha2> {
        self.inner
            .iter()
            .par_bridge()
            .find_first(|record| record.range().contains(ip))
            .map(|record| *record.alpha())
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

static DOWNLOADED: LazyLock<Mutex<HashMap<Registry, RegistryRecords>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

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

        Ok(Box::new(records))
    }

    #[instrument(level = "TRACE", ret)]
    async fn downloaded(&self) -> Result<RegistryRecords> {
        let map = &(DOWNLOADED.lock().await);
        map.get(self).ok_or(anyhow::anyhow!("Data not downloaded")).cloned()
    }

    #[instrument(level = "TRACE", ret, err)]
    async fn download(&self) -> Result<()> {
        let url = &self.url();
        let data = reqwest::get(url)
            .await
            .map_err(|e| RegistryErrors::DownloadFailed(*self, e.into()))?
            .bytes()
            .await
            .map_err(|e| RegistryErrors::DownloadFailed(*self, e.into()))?
            .to_vec();

        let parsed_lines = data
            .lines()
            .par_bridge()
            .flatten()
            .filter(|line| !line.starts_with('#')) // Remove comments
            .map(|line| line.split('|').map(|s| s.into()).collect::<Vec<String>>()) // Split on pipe
            .filter_map(|split| {
                let alpha = match &*split[1] {
                    str if str.len() != 2 => return None,
                    str => match Alpha2::try_from(str) {
                        Ok(alpha) => alpha,
                        Err(_) => return None,
                    },
                };

                let value = match IpAddr::from_str(&split[3]) {
                    Ok(value) => value,
                    Err(_) => return None,
                };

                let range = match u32::from_str(&split[4]) {
                    Ok(range) => range,
                    Err(_) => return None,
                };

                let date = split[5].to_string();

                let status = match &*split[6] {
                    "assigned" => Status::Assigned,
                    "allocated" => Status::Allocated,
                    "reserved" => Status::Reserved,
                    _ => unreachable!("Invalid status"),
                };

                Some(RegistryRecord {
                    registry: *self,
                    alpha,
                    value,
                    range,
                    date,
                    status,
                })
            })
            .collect::<Vec<_>>();

        let record = RegistryRecords {
            registry: *self,
            inner: parsed_lines,
        };

        let mut map = DOWNLOADED.lock().await;
        map.insert(*self, record);

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use keshvar::CountryIterator;
    use tracing::trace;

    #[test_log::test(tokio::test)]
    async fn test_downloads() -> Result<()> {
        for reg in Registry::get_variants() {
            trace!("Testing {reg:?} | url: {}", reg.url());

            reg.download().await?;
            let records = reg.downloaded().await;
            debug_assert!(records.is_ok(), "Failed to download {reg:?}; {records:?}");

            let records = records?;
            debug_assert!(!records.inner.is_empty(), "No records for {reg:?}");
        }

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn test_all_countries() -> Result<()> {
        trace!("Initialising registries");
        for reg in Registry::get_variants() {
            trace!("Initialising {reg:?}");
            reg.download().await?;
        }

        trace!("Testing all countries");
        for country in CountryIterator::new() {
            let alpha = country.alpha2();
            if DELEGATES.contains(&alpha) {
                trace!("Skipping {} as its delegated", country.iso_short_name());
                continue;
            }

            trace!("Testing {country:?}");

            let registry = Registry::get_for(&country);
            debug_assert!(registry.is_ok(), "Failed to get registry for {country:?}; {registry:?}");
            let registry = registry?;

            let records = registry.get().await?;
            let records = records.filtered(&country.alpha2()).await;
            debug_assert!(!records.is_empty(), "No records for {country:?}");
        }

        Ok(())
    }
}
