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

#![feature(lazy_cell)]
#![feature(result_option_inspect)]
#![feature(ip_bits)]
#![feature(async_closure)]
#![feature(unboxed_closures)]
#![feature(async_fn_in_trait)]

use crate::record::Record;
use crate::registry::Registry;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use futures::StreamExt;
use keshvar::{Alpha2, Alpha3, Country, CountryIterator};
use rand::prelude::IteratorRandom;
use rand::thread_rng;
use std::fmt::Debug;
use std::net::IpAddr;
use thiserror::Error;
use tracing::{debug, error, instrument, warn};

pub mod db_ip;
pub mod record;
pub mod registry;
pub mod ui;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid Country Code: {0}")]
    InvalidCountryCode(String),
}

#[instrument(level = "TRACE", ret, err, fields(country = %country.iso_short_name()))]
pub async fn get_record_db(country: &Country) -> Result<Box<dyn RecordDB>> {
    match Registry::get_for(country) {
        Ok(registry) => {
            debug!("Using registry {} for {}", registry.name(), country.iso_short_name());
            registry.get().await
        }
        Err(err) => {
            error!("{err}");
            debug!("Falling back to DB-IP for {}", country.iso_short_name());
            Ok(db_ip::DB::instance())
        }
    }
}

#[async_trait]
pub trait RecordDB: Send + Sync + Debug + Unpin {
    async fn lookup(&self, ip: &IpAddr) -> Option<Alpha2>;

    async fn filtered(&self, alpha: &Alpha2) -> Vec<&Record>;

    async fn random_ipv4(&self, alpha: &Alpha2) -> Option<IpAddr> {
        self.filtered(alpha)
            .await
            .into_iter()
            .filter(|record| record.start().is_ipv4())
            .choose(&mut thread_rng())
            .map(|record| record.random())
    }

    async fn random_ipv6(&self, alpha: &Alpha2) -> Option<IpAddr> {
        self.filtered(alpha)
            .await
            .into_iter()
            .filter(|record| record.start().is_ipv6())
            .choose(&mut thread_rng())
            .map(|record| record.random())
    }
}

#[instrument(level = "TRACE", ret, err)]
fn get_country(alpha: &Option<String>) -> std::result::Result<Country, Error> {
    alpha
        .as_ref()
        .filter(|alpha| (2..=3).contains(&alpha.len()))
        .map(|alpha| alpha.to_uppercase())
        .map(|alpha| match alpha.len() {
            2 => Alpha2::try_from(&*alpha).map(Country::from).map_err(|_| Error::InvalidCountryCode(alpha)),
            3 => Alpha3::try_from(&*alpha).map(Country::from).map_err(|_| Error::InvalidCountryCode(alpha)),
            _ => unreachable!(),
        })
        .unwrap_or_else(|| {
            warn!("No country specified, generating random country");
            CountryIterator::new().choose(&mut thread_rng()).ok_or_else(|| unreachable!())
        })
}

#[instrument(level = "TRACE", ret, err, fields(country = %country.iso_short_name()))]
async fn get(country: &Country, use_ipv6: &bool) -> Result<IpAddr> {
    let alpha = country.alpha2();
    let record_db = get_record_db(country).await?;
    match use_ipv6 {
        true => record_db.random_ipv6(&alpha),
        false => record_db.random_ipv4(&alpha),
    }
    .await
    .ok_or_else(|| anyhow::anyhow!("No IP addresses found for {}", country.iso_short_name()))
}

#[instrument(level = "TRACE", ret, err)]
async fn lookup(addr: &IpAddr) -> Result<Country> {
    let variants: Vec<Registry> = Registry::get_variants();
    let mut stream = futures::stream::iter(variants.iter()).map(|reg| reg.get()).buffer_unordered(variants.len());

    while let Some(Ok(db)) = stream.next().await {
        if let Some(alpha) = db.lookup(addr).await {
            return Ok(alpha.to_country());
        }
    }

    Err(anyhow!("No country found for IP address"))
}
