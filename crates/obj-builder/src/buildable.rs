/*
 * Copyright (C) 2024. James Draycott me@racci.dev
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

use crate::Builder;
use std::fmt::Debug;

use super::{error::FillError, filler::Filler};

pub trait Buildable: Debug {
    type Builder: Builder<Buildable = Self>;

    fn builder() -> Self::Builder
    where
        Self: Sized,
    {
        Default::default()
    }

    async fn from<F: Filler>(filler: &F) -> crate::error::FillResult<Self>
    where
        Self: Sized,
    {
        let mut builder = Self::builder();
        builder.fill(filler).await?;
        builder.build().await.map_err(|err| FillError::Nested(err))
    }
}
