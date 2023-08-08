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

use crate::record::Record;
use crate::registry::Registry;
use async_trait::async_trait;
use keshvar::{Alpha2, Country};
use rand::prelude::IteratorRandom;
use std::fmt::Debug;
use std::net::IpAddr;
use tracing::{info, instrument};

pub mod config;
pub mod db_ip;
#[cfg(feature = "gui")]
pub mod gui;
pub mod record;
pub mod registry;

#[instrument]
pub async fn get_record_db(country: &Country) -> anyhow::Result<Box<dyn RecordDB>> {
    match Registry::get_for(&country) {
        Ok(registry) => {
            info!("Using registry {} for {}", registry.name(), country.iso_short_name());
            registry.get().await
        }
        Err(_) => {
            info!(
                "No registry found for {}, falling back to DB-IP",
                country.iso_short_name()
            );
            Ok(db_ip::DB::instance())
        }
    }
}

#[async_trait]
pub trait RecordDB: Send + Sync + Debug {
    async fn lookup(&self, ip: &IpAddr) -> Option<Alpha2>;

    async fn filtered(&self, alpha: &Alpha2) -> Vec<&Record>;

    async fn random_ipv4(&self, alpha: &Alpha2) -> Option<IpAddr> {
        self.filtered(alpha)
            .await
            .into_iter()
            .filter(|record| record.start().is_ipv4())
            .choose(&mut rand::thread_rng())
            .map(|record| record.random())
    }

    async fn random_ipv6(&self, alpha: &Alpha2) -> Option<IpAddr> {
        self.filtered(alpha)
            .await
            .into_iter()
            .filter(|record| record.start().is_ipv6())
            .choose(&mut rand::thread_rng())
            .map(|record| record.random())
    }
}

