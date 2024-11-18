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

use crate::ui::builder::buildable::Buildable;
use crate::ui::builder::error::{BuildResult, FillResult};
use crate::ui::builder::filler::Filler;
use std::fmt::Debug;

pub mod buildable;
pub mod error;
pub mod filler;

/// This trait is used to fill the builder with data and then build the final object.
///
/// The `Buildable` associated type is the type that the builder will build.
/// The `fill` method is used to fill the builder with data.
/// The `build` method is used to build the final object.
pub trait Builder: Default + Debug {
    type Buildable: Buildable<Builder = Self>;

    async fn fill<F: Filler>(&mut self, filler: &F) -> FillResult<()>;

    async fn build(self) -> BuildResult<Self::Buildable>;
}

#[macro_export]
macro_rules! builder {
    // The base matcher, which will parse the input and eventually call the `impl` macro.
    ($(#[$attr:meta])* $name:ident { $($($fields:tt)+)? }) => {
        $crate::builder!(@parse $(#[$attr])* $name ([] []) $($($fields)+)?);
    };

    //#region Field parsers
    // Utility macro to count the number of fields in a builder.
    (@count $current:expr; $next:ident $(, $($rest:ident,)+)?) => { $crate::builder!(@count $current + 1 $(; $($rest),+)?); };
    (@count $final:expr) => { $final };

    // Parser that defines the current variable as an optional.
    // This matches variables with their ident surrounded by [].
    (@parse
        $(#[$attr:meta])*
        $struct_name:ident
        // The currently parsed fields
        (
            [$($($(#[$req_attr:meta])* $req_i:ident: $req_t:ty $(=> $req_default:expr)?),+)?]
            [$($($(#[$opt_attr:meta])* $opt_i:ident: $opt_t:ty $(=> $opt_default:expr)?),+)?]
        )
        // The current field being parsed
        $(#[$field_attr:meta])* [$name:ident]: $type:ty $(=> $default:expr)?
        // The remaining fields to be parsed
        $(, $($rest:tt)+)?
    ) => {
        $crate::builder!(@parse
            $(#[$attr])*
            $struct_name
            (
                [$($($(#[$req_attr])* $req_i: $req_t $(=> $req_default)?),+)?]
                [$($($(#[$opt_attr])* $opt_i: $opt_t $(=> $opt_default)?),+,)? $(#[$field_attr])* $name: $type $(=> $default)?]
            )
            $($($rest)+)?
        );
    };

    // Parser that defines the current variable as a required field.
    (@parse
        $(#[$attr:meta])*
        $struct_name:ident
        // The currently parsed fields
        (
            [$($($(#[$req_attr:meta])* $req_i:ident: $req_t:ty $(=> $req_default:expr)?),+)?]
            [$($($(#[$opt_attr:meta])* $opt_i:ident: $opt_t:ty $(=> $opt_default:expr)?),+)?]
        )
        // The current field being parsed
        $(#[$field_attr:meta])* $name:ident: $type:ty $(=> $default:expr)?
        // The remaining fields to be parsed
        $(, $($rest:tt)+)?
    ) => {
        $crate::builder!(@parse
            $(#[$attr])*
            $struct_name
            (
                [$($($(#[$req_attr])* $req_i: $req_t $(=> $req_default)?),+,)? $(#[$field_attr])* $name: $type $(=> $default)?]
                [$($($(#[$opt_attr])* $opt_i: $opt_t $(=> $opt_default)?),+)?]
            )
            $($($rest)+)?
        );
    };
    (@parse
        $(#[$attr:meta])*
        $struct_name:ident
        // All the parsed fields
        (
            [$($($(#[$req_attr:meta])* $req_i:ident: $req_t:ty $(=> $req_default:expr)?),+)?]
            [$($($(#[$opt_attr:meta])* $opt_i:ident: $opt_t:ty $(=> $opt_default:expr)?),+)?]
        )
    ) => {
        $crate::builder!(impl
            $(#[$attr])*
            $struct_name
            [$($($(#[$req_attr])* $req_i: $req_t $(=> $req_default)?),+)?]
            [$($($(#[$opt_attr])* $opt_i: $opt_t $(=> $opt_default)?),+)?]
        );
    };
    //endregion

    (impl
        $(#[$attr:meta])*
        $name:ident
        [$($($(#[$required_attr:meta])* $required_ident:ident: $required_type:ty $(=> $required_default:expr)?),+)?]
        [$($($(#[$optional_attr:meta])* $optional_ident:ident: $optional_type:ty $(=> $optional_default:expr)?),+)?]
    ) => {
        $(#[$attr])*
        #[derive(Debug, Clone)]
        pub struct $name {
            $($($(#[$required_attr])* $required_ident: $required_type,)+)?
            $($($(#[$optional_attr])* $optional_ident: Option<$optional_type>,)+)?
        }

        impl $name {
            $($($crate::builder!(@get $required_ident $required_type);)+)?
            $($($crate::builder!(@get $optional_ident Option<$optional_type>);)+)?
        }

        $crate::builder!(@default_impl $name
            [$($($required_ident: $required_type $(=> $required_default)?),+)?]
            [$($($optional_ident: $optional_type $(=> $optional_default)?),+)?]
        );

        impl $crate::ui::builder::buildable::Buildable for $name {
            type Builder = $crate::_paste::paste! { [< $name Builder >] };
        }

        $crate::_paste::paste! {
            #[derive(Debug, Clone)]
            pub struct [< $name Builder >] {
                $($($required_ident: $crate::ui::builder::filler::FillableDefinition<$required_type>,)+)?
                $($($optional_ident: $crate::ui::builder::filler::FillableDefinition<$optional_type>,)+)?
            }

            impl [< $name Builder >] {
                $($($crate::builder!(@get_set $required_ident $required_type);)+)?
                $($($crate::builder!(@get_set $optional_ident $optional_type);)+)?

                fn set_field(&mut self, field: &str, value: &str) -> anyhow::Result<()> {
                    match field {
                        $($(stringify!($required_ident) => { $crate::builder!(@set_field_match value => [self, $required_ident: $required_type]) },)+)?
                        $($(stringify!($optional_ident) => { $crate::builder!(@set_field_match value => [self, $optional_ident: $optional_type]) },)+)?
                        _ => return Err(anyhow::anyhow!("Invalid field")),
                    };

                    Ok(())
                }

            }

            impl $crate::ui::builder::Builder for [< $name Builder >] {
                type Buildable = $name;

                async fn fill<F: $crate::ui::builder::filler::Filler>(&mut self, filler: &F) -> $crate::ui::builder::error::FillResult<()> {
                    use $crate::ui::builder::Builder as _Builder;

                    $($($crate::builder!(@try_fill [self, filler: F] $required_ident: $required_type);)+)?
                    $($($crate::builder!(@try_fill [self, filler: F] $optional_ident: $optional_type);)+)?

                    Ok(())
                }

                async fn build(self) -> Result<$name, $crate::ui::builder::error::BuildError> {
                    Ok($name {
                        $($($required_ident: $crate::builder!(@build_req self $required_ident),)+)?
                        $($($optional_ident: $crate::builder!(@build_opt self $optional_ident),)+)?
                    })
                }
            }
        }

        $crate::_paste::paste! { impl Default for [< $name Builder >] {
            fn default() -> Self {
                Self {
                    $($($required_ident: $crate::builder!(@default $(#[$required_attr])* $required_ident: $required_type, true $(, $required_default)?),)+)?
                    $($($optional_ident: $crate::builder!(@default $(#[$optional_attr])* $optional_ident: $optional_type, false $(, $optional_default)?),)+)?
                }
            }
        }}
    };

    // #region Internal macros
    (@get $field:ident $type:ty) => { $crate::_paste::paste! {
        pub fn [< get_ $field >](&self) -> &$type {
            &self.$field
        }
    }};

    // Create a getter and setter for the field.
    (@get_set $field:ident $type:ty) => { $crate::_paste::paste! {
        pub fn [< get_ $field >](&self) -> Option<&$type> {
            self.$field.value.as_ref()
        }

        pub fn [< set_ $field >](&mut self, value: $type) {
            if let Some(old) = self.$field.value.replace(value) {
                tracing::debug!("Replaced {} old value of {:?}", stringify!($field), old);
            }
        }
    }};

    // set_field match arm
    (@set_field_match $value:ident => [$self:ident, $field:ident: $type:ty]) => { $crate::_paste::paste! {
        $self.[<set_ $field>] ({
            use std::str::FromStr as _FromStr;

            $crate::conditional_call!(impl $type where T: Sized + Clone | T: _FromStr + Clone {
                fn call(value: &str) -> anyhow::Result<T> {
                    value.parse::<T>().map_err(|_| anyhow::anyhow!("Failed to parse value"))
                } else {
                    Err(anyhow::anyhow!("No impl for FromStr"))
                }
            });

            $crate::conditional_call!(call::<$type>($value))?
        })
    }};

    // Build the final object from the builder with a required field.
    (@build_req $self:ident $field:ident) => {
        $self.$field.value.or($self.$field.default).ok_or_else(|| lib::ui::builder::error::BuildError::MissingField { field: stringify!($field).to_string() })?
    };

    // Build the final object from the builder with an optional field.
    (@build_opt $self:ident $field:ident) => {
        $self.$field.value.or($self.$field.default)
    };

    // Create a default value for the builder struct.
    (@default $(#[$attr:meta])* $ident:ident: $ty:ty, $required:expr $(, $default:expr)?) => {{
        let default = $crate::builder!(@expr_or_none $($default)?);
        $crate::ui::builder::filler::FillableDefinition::<$ty> {
            name: stringify!($ident),
            required: $required,
            value: default.clone(),
            default,
            description: $crate::builder!(@doc $(#[$attr])*),

            _pd: std::marker::PhantomData,
        }
    }};

    // If all fields have a default value create a default implementation.
    (@default_impl $name:ident
        [$($required_ident:ident: $required_type:ty => $required_default:expr),*]
        [$($optional_ident:ident: $optional_type:ty => $optional_default:expr),*]
    ) => {
        impl Default for $name {
            fn default() -> Self {
                Self {
                    $($required_ident: $required_default,)*
                    $($optional_ident: $optional_default,)*
                }
            }
        }
    };
    // Otherwise, do nothing.
    (@default_impl $name:ident
        [$($required_ident:ident: $required_type:ty $(=> $required_default:expr)?),*]
        [$($optional_ident:ident: $optional_type:ty $(=> $optional_default:expr)?),*]
    ) => {};

    // Utility to compute the default expression into an option or none.
    (@expr_or_none $expr:expr) => {
        Some($expr)
    };
    (@expr_or_none) => {
        None
    };

    // Extract the doc attribute from the list of attributes.
    (@doc #[doc = $doc:literal] $(#[$attr:meta])*) => { Some($doc) };
    (@doc #[$first_attr:meta] $(#[$attr:meta])*) => { $crate::builder!(@doc $(#[$attr])* ) };
    (@doc) => { None };

    (@try_fill [$self:ident, $filler:ident: $filler_generic:ident] $ident:ident: $type:ty) => {{
        use core::fmt::Debug as _Debug;
        use $crate::ui::builder::error::FillResult as _FillResult;
        use $crate::ui::builder::filler::FillableDefinition as _FillableDefinition;
        use $crate::ui::builder::filler::Filler as _Filler;

        let mut filled = {{
            use std::str::FromStr as _FromStr;

            $crate::conditional_call!(impl $type where T: Sized + Clone, F: _Filler | T: _FromStr + Clone + _Debug, F: _Filler {
                async fn call<'a>(
                    filler: &F,
                    def: &'a mut _FillableDefinition<T>
                ) -> _FillResult<&'a T> {
                    let value = filler.fill_input(def).await?;
                    def.value.replace(value);
                    def.value.as_ref().ok_or($crate::ui::builder::error::FillError::InvalidFiller {
                        field: stringify!($ident).to_string(),
                        filler: "FromStr".to_string(),
                    })
                } else {
                    Err($crate::ui::builder::error::FillError::InvalidDefinition)
                }
            });

            $crate::conditional_call!(call::<$type, $filler_generic>($filler, &mut $self.$ident)).await
        }};

        if let Err($crate::ui::builder::error::FillError::InvalidDefinition) = filled {
            filled = {
                use lib::ui::builder::buildable::Buildable as _Buildable;

                $crate::conditional_call!(impl $type where T: Sized + Clone, F: _Filler | T: _Buildable + Clone + _Debug, F: _Filler {
                    async fn call<'a>(
                        filler: &F,
                        def: &'a mut _FillableDefinition<T>
                    ) -> _FillResult<&'a T> {
                        let mut builder = <T as _Buildable>::builder();
                        builder.fill(filler).await?;
                        match builder.build().await {
                            Err(err) => Err($crate::ui::builder::error::FillError::Nested(err)),
                            Ok(value) => {
                                def.value.replace(value);
                                def.value.as_ref().ok_or($crate::ui::builder::error::FillError::InvalidFiller {
                                    field: stringify!($ident).to_string(),
                                    filler: "Buildable".to_string(),
                                })
                            }
                        }
                    } else {
                        Err($crate::ui::builder::error::FillError::InvalidDefinition)
                    }
                });

                $crate::conditional_call!(call::<$type, $filler_generic>($filler, &mut $self.$ident)).await
            };
        }

        filled?;
    }};
    // #endregion
}
