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

pub mod field;
pub mod filler;

use crate::ui::buildable::field::BuildableField;
use crate::ui::buildable::filler::BuildableFiller;
use anyhow::Result;
use std::any::Any;
use std::fmt::Debug;
use std::marker::PhantomData;
use tracing::{debug, info};

pub trait Builder
where
    Self: Default + Sized + Debug,
{
    /// The name of the final struct that is being built.
    ///
    /// Example: `Test` for `TestBuilder`
    const NAME: &'static str;

    /// The final struct that is being built.
    type Final: Sized + Debug;

    type Field = BuildableField<Self::FieldType, Self>;

    type FieldType: Any + Sized + Debug = Box<dyn Any>;

    /// A constant of the fields that are in the struct.
    fn fields(&mut self) -> Vec<&mut Self::Field>;

    async fn fill<Filler>(&mut self, filler: &Filler) -> Result<Vec<&Self::Field>>
    where
        Filler: BuildableFiller,
    {
        let mut filled = Vec::new();
        for field in Self::fields(&mut self) {
            if field.value.is_some() {
                debug!("Field {} already has a value", field.name);
                continue;
            }

            match field.fillable.fill(filler).await {
                Ok(Some(value)) => {
                    debug!("Filled field {} with value {:#?}", field.name, value);
                    filled.push(&*field);
                }
                Ok(None) => {
                    debug!("Field {} was not filled", field.name);
                }
                Err(e) => {
                    info!("Field {} was not filled because of error: {:#?}", field.name, e);
                    return Err(e);
                }
            }
        }

        Ok(filled)
    }

    async fn build(self) -> Result<Self::Final>;

    // /// Attempts to set the field from a string.
    // ///
    // /// If this field cannot be parsed using [`std::str::FromStr`]
    // /// it will return an Err result of the parsing error.
    // ///
    // /// If there was an existing value on this field and parsing was successful,
    // /// the object which was the previous value will be returned in an [`Option`].
    // /// If the type of [`T`] is not the correct type of the requested field,
    // /// an Err result will be returned.
    // #[doc(hidden)]
    // fn set_field<T: Any + Sized>(&mut self, field: &str, value: &str) -> Result<Option<T>>;
    //
    // /// Returns the value of the field if it exists.
    // ///
    // /// If the field does not exist, this will return None.
    // /// If the field exists but is not the correct type, this will return None.
    // /// If the field exists, is the correct type and has a value, this will return Some(value).
    // #[doc(hidden)]
    // fn get_field<T: Any + Sized>(&self, field: &str) -> Option<&T>;

    // #[doc(hidden)]
    // fn display(&self, field: &str) -> String {
    //     match self.get_field::<Box<dyn Any>>(field) {
    //         Some(value) => format!("{value:?}"),
    //         None => "None".into(),
    //     }
    // }
}

// crate::builder_v2!(
//     Test {
//         test: String,
//         [opt]: String
//     }
// );

#[derive(Debug)]
pub struct Test {
    test: String,
    opt: Option<String>,
}
#[derive(Debug)]
pub struct TestBuilder {
    test: BuildableField<String, Self>,
    opt: BuildableField<String, Self>,
}

impl Builder for TestBuilder {
    const NAME: &'static str = "Test";
    type Final = Test;
    fn fields(&mut self) -> Vec<&mut Self::Field> {
        vec![&mut self.test as &mut Self::Field, &mut self.opt as &mut Self::Field]
    }

    async fn build(self) -> Result<Test> {
        Ok(Test {
            test: self.test.value.ok_or_else(|| {
                anyhow::Error::msg({
                    let res = format!("Field {0} is required but was not filled", "test",);
                    res
                })
            })?,
            opt: self.opt.value,
        })
    }
}
impl Default for TestBuilder {
    fn default() -> Self {
        Self {
            test: BuildableField {
                name: "test",
                required: true,
                value: None,
                fillable: (),
                buildable: PhantomData::<Self>,
            },
            opt: BuildableField {
                name: "opt",
                required: false,
                value: None,
                buildable: PhantomData::<Self>,
            },
        }
    }
}

#[macro_export]
macro_rules! builder_v2 {
    ($($(#[$attr:meta])+)? $name:ident { $($($fields:tt)+)? }) => {
        $crate::builder_v2!(@parse $($(#[$attr])+)? $name, ([] []) $($($fields)+)?);
    };

    (@count $current:expr; $next:ident $(, $($rest:ident,)+)?) => { $crate::builder_v2!(@count $current + 1 $(; $($rest),+)?); };
    (@count $final:expr) => { $final };

    (@parse
        $($(#[$attr:meta])+)?
        $struct_name:ident,
        (
            [$($($req_i:ident: $req_t:expr),+)?]
            [$($($opt_i:ident: $opt_t:expr),+)?]
        )
        [$name:ident]: $type:expr
        $(, $($rest:tt)+)?
    ) => {
        $crate::builder_v2!(@parse
            $($(#[$attr])+)?
            $struct_name,
            (
                [$($($req_i: $req_t),+)?]
                [$($($opt_i: $opt_t),+)? $name: $type]
            )
            $($($rest)+)?
        );
    };
    (@parse
        $($(#[$attr:meta])+)?
        $struct_name:ident,
        (
            [$($($req_i:ident: $req_t:expr),+)?]
            [$($($opt_i:ident: $opt_t:expr),+)?]
        )
        $name:ident: $type:expr
        $(, $($rest:tt)+)?
    ) => {
        $crate::builder_v2!(@parse
            $($(#[$attr])+)?
            $struct_name,
            (
                [$($($req_i: $req_t),+)? $name: $type]
                [$($($opt_i: $opt_t),+)?]
            )
            $($($rest)+)?
        );
    };
    (@parse
        $($(#[$attr:meta])+)?
        $struct_name:ident,
        (
            [$($($req_i:ident: $req_t:expr),+)?]
            [$($($opt_i:ident: $opt_t:expr),+)?]
        )
    ) => {
        $crate::builder_v2!(impl
            $($(#[$attr])+)?
            $struct_name,
            [$($($req_i: $req_t),+)?]
            [$($($opt_i: $opt_t),+)?]
        );
    };

    (impl
        $($(#[$attr:meta])+)?
        $name:ident,
        [$($($required_ident:ident: $required_type:expr),+)?]
        [$($($optional_ident:ident: $optional_type:expr),+)?]
    ) => {
        $($(#[$attr])+)?
        #[derive(Debug)]
        pub struct $name {
            $($($required_ident: $required_type,)+)?
            $($($optional_ident: Option<$optional_type>,)+)?
        }

        paste::paste! { #[derive(Debug)] pub struct [< $name Builder >]<'b> {
            $($($required_ident: BuildableField<'b, $required_type, Self>,)+)?
            $($($optional_ident: BuildableField<'b, $optional_type, Self>,)+)?
        }}

        paste::paste! { impl $crate::ui::buildable::Buildable for [< $name Builder >]<'_> {
            const NAME: &'static str = stringify!([<$name>]);
            type Final = $name;

            fn fields(&self) -> Vec<&mut Self::Field> {
                &[
                    $($(&mut self.$required_ident,)+)?
                    $($(&mut self.$optional_ident,)+)?
                ]
            }

            async fn build(self) -> Result<$name> {
                Ok($name {
                    $($($required_ident: self.$required_ident.value.ok_or_else(|| anyhow::anyhow!("Field {} is required but was not filled", stringify!($required_ident)))?,)+)?
                    $($($optional_ident: self.$optional_ident.value,)+)?
                })
            }
        }}

        paste::paste! { impl Default for [< $name Builder >]<'_> {
            fn default() -> Self {
                Self {
                    $($($required_ident: $crate::ui::buildable::field::BuildableField {
                        name: stringify!($required_ident),
                        required: true,
                        value: None,
                        buildable: &Self,
                    },)+)?
                    $($($optional_ident: $crate::ui::buildable::field::BuildableField {
                        name: stringify!($optional_ident),
                        required: false,
                        value: None,
                        buildable: &Self,
                    },)+)?
                }
            }
        }}
    };
}
