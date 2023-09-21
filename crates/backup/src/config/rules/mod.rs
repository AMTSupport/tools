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

pub mod autoprune;
pub mod metadata;
pub mod rule;

use crate::config::rules::autoprune::AutoPrune;
use crate::config::rules::metadata::Metadata;
use crate::config::rules::rule::Rule;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::trace;

#[derive(Default, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Rules {
    /// The AutoPrune configuration.
    pub auto_prune: Option<AutoPrune>,
}

impl Rules {
    pub async fn would_survive(&self, existing_files: &[&Path], destination: &Path, metadata: Metadata) -> bool {
        let mut survive = true;
        let mut reason = None;

        if let Some(auto_prune) = &self.auto_prune {
            if !auto_prune.would_keep(existing_files, destination, &metadata).await {
                survive = false;
                reason = Some("AutoPrune");
            }
        }

        if survive {
            trace!("File {:?} would survive", destination);
        } else {
            trace!("File {:?} would not survive because of {:?}", destination, reason);
        }

        survive
    }
}
