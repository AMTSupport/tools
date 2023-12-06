/*
 * Copyright (c) 2023. James Draycott <me@racci.dev>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, version 3.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along with this program.
 * If not, see <https://www.gnu.org/licenses/>.
 */

use super::Result;
use crate::ui::builder::buildable::Buildable;
use crate::ui::builder::filler::Filler;
use crate::ui::builder::Builder;
use std::fmt::Display;
use std::marker::PhantomData;
use std::str::FromStr;
use tracing::warn;

trait DummyImpl<T> {
    async fn fill<F: Filler>(name: &str, _filler: &mut F, _default: Option<T>) -> ! {
        panic!("Field {0} does not implement {1}", name, "$trait");
    }
}

impl<T> DummyImpl<T> for T {}

struct WrapperFromStr<T>(PhantomData<T>);
struct WrapperBuildable<T>(PhantomData<T>);

#[allow(dead_code)]
impl<T: FromStr + Display> WrapperFromStr<T> {
    #[tracing::instrument(level = "TRACE", skip(filler, default))]
    async fn fill<F>(name: &str, filler: &mut F, default: Option<T>) -> anyhow::Result<T>
    where
        F: Filler,
    {
        let value = filler.fill_input(name, default).await?;

        Ok(value)
    }
}

#[allow(dead_code)]
impl<T, B> WrapperBuildable<T>
where
    T: Buildable<Builder = B>,
    B: Builder<Buildable = T>,
{
    #[tracing::instrument(level = "TRACE", skip(filler, default))]
    async fn fill<F: Filler>(name: &str, filler: &mut F, default: Option<T>) -> Result<T, T> {
        if default.is_some() {
            warn!("default value for {name} is ignored");
        }

        let builder = <T as Buildable>::builder();
        let builder = builder.fill(filler).await?;
        let built = builder.build().await?;

        Ok(built)
    }
}

// #[allow(dead_code)]
// impl Wrapper<bool> {
//     async fn fill<F: Filler>(name: &str, filler: &mut F, default: Option<bool>) -> Result<bool> {
//         let value = filler.fill_bool(name, default).await?;
//         Ok(value)
//     }
// }
