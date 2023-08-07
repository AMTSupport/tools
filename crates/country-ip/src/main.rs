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
#![feature(slice_take)]

use country_ip::config::cli::CliAction;
use country_ip::config::runtime::Runtime;
use country_ip::db_ip;
use country_ip::registry::Registry;
use futures::FutureExt;
use keshvar::{Alpha2, Alpha3, Country, CountryIterator};
use lib::runtime::runtime::Runtime as _;
use rand::thread_rng;
use std::net::IpAddr;
use std::ops::Deref;
use std::sync::LazyLock;
use thiserror::Error;
use tracing::{debug, info, instrument};

#[derive(Debug, Error)]
pub enum Errors {
    #[error("Invalid Country Code: {0}")]
    InvalidCountryCode(String),
}

static RUNTIME: LazyLock<Runtime> = LazyLock::new(|| Runtime::new().unwrap());

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let runtime = &*RUNTIME;

    match &runtime.cli.action {
        CliAction::Get { country, ipv6 } => get(runtime, country, ipv6).await,
        CliAction::Lookup { addr, .. } => lookup(runtime, addr).await,
    }?;

    return Ok(());
}

#[instrument]
async fn get(_runtime: &'static Runtime, alpha: &Option<String>, use_ipv6: &bool) -> anyhow::Result<()> {
    use rand::prelude::{thread_rng, IteratorRandom};

    let country = alpha
        .as_ref()
        .filter(|alpha| (2..=3).contains(&alpha.len()))
        .map(|alpha| alpha.to_uppercase())
        .map(|alpha| match alpha.len() {
            2 => Alpha2::try_from(&*alpha).map(Country::from).map_err(|_| Errors::InvalidCountryCode(alpha)),
            3 => Alpha3::try_from(&*alpha).map(Country::from).map_err(|_| Errors::InvalidCountryCode(alpha)),
            _ => unreachable!(),
        })
        .unwrap_or_else(|| {
            info!("No country specified, generating random country");
            CountryIterator::new().choose(&mut thread_rng()).ok_or_else(|| unreachable!())
        })?;
    let alpha = country.alpha2();

    debug!("Selected Country: {country:?}");

    let registry = match Registry::get_for(&country) {
        Ok(registry) => {
            info!("Using registry {} for {}", registry.name(), country.iso_short_name());
            registry.get().await?
        }
        Err(_) => {
            info!(
                "No registry found for {}, falling back to DB-IP",
                country.iso_short_name()
            );
            db_ip::DB::instance()
        }
    };

    let random = match use_ipv6 {
        true => registry.random_ipv6(&alpha),
        false => registry.random_ipv4(&alpha),
    }
    .await
    .ok_or_else(|| anyhow::anyhow!("No IP addresses found for {}", country.iso_short_name()))?;

    info!(
        "Generated random IP address for {} => {random}",
        country.iso_short_name()
    );

    Ok(())
}

async fn lookup(runtime: &'static Runtime, addr: &IpAddr) -> anyhow::Result<()> {
    unimplemented!()
}
