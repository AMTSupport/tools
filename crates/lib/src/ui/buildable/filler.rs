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

use crate::named::Named;
use crate::ui::buildable::Builder;
use anyhow::Result;
use std::marker::PhantomData;
use std::str::FromStr;

type ImplBuilder<T>
where
    T: Builder,
= T;

type ImplFromStr<T>
where
    T: FromStr,
= T;

pub enum Fillable<T> {
    Str(PhantomData<ImplFromStr<T>>),
    Buildable(PhantomData<ImplBuilder<T>>),
}

pub trait BuildableFiller {
    async fn fill<T>(&self, _phantom: &PhantomData<ImplBuilder<T>>) -> Result<Option<T>>
    where
        T: Default + Builder;
}

impl<T> Fillable<T> {
    pub async fn fill<Filler>(&self, filler: Filler) -> Result<Option<T>>
    where
        Filler: BuildableFiller,
    {
        match self {
            Self::Str(str) => {
                let value = filler.fill(str).await?;
                Ok(Some(value))
            }
            Self::Buildable(buildable) => {
                let value = filler.fill(buildable).await?;
                Ok(value)
            }
        }
    }
}

auto trait IsFillable {}
impl<T> !IsFillable for T where T: FromStr + Builder {}

union FillableUnion<T>
where
    T: IsFillable,
{
    str: PhantomData<ImplFromStr<T>>,
    buildable: PhantomData<ImplBuilder<T>>,
}

impl<T> Into<Fillable<T>> for FillableUnion<T>
where
    T: IsFillable,
{
    fn into(self) -> Fillable<T> {
        unsafe {
            match self {
                Self { str } => Fillable::Str(str),
                Self { buildable } => Fillable::Buildable(buildable),
            }
        }
    }
}

// impl<T: Builder> Into<Fillable<T>> for PhantomData<T>
// where
//     T: IsFillable,
// {
//     fn into(self) -> Fillable<T> {
//         Fillable::Buildable(self)
//     }
// }
//
// impl<T: FromStr> Into<Fillable<T>> for PhantomData<T>
// where
//     T: IsFillable,
// {
//     fn into(self) -> Fillable<T> {
//         Fillable::Str(self)
//     }
// }

#[tracing::instrument(level = "DEBUG")]
fn fill_env<T>(_phantom: &PhantomData<T>) -> Result<T>
where
    T: FromStr + Named,
{
    use std::env::var;

    let env_name = format!("BUILDABLE_{}", <T as Named>::NAME);
    let raw_value = var(env_name)?;
    let value = T::from_str(&raw_value)?;

    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[derive(Debug, PartialEq, Eq)]
    struct Test {
        pub name: String,
    }

    impl FromStr for Test {
        type Err = anyhow::Error;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(Self { name: s.to_string() })
        }
    }

    impl Named for Test {
        const NAME: &'static str = "TEST";
    }

    #[test]
    fn test_fill_env() {
        env::set_var("BUILDABLE_TEST", "value");

        let test = fill_env::<Test>(&PhantomData).unwrap();
        assert_eq!(
            test,
            Test {
                name: "test".to_string()
            }
        );
    }
}
