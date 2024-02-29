/*
 * Copyright (c) 2023-2024. James Draycott <me@racci.dev>
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
 * You should have received a copy of the GNU General Public License along with this program.
 * If not, see <https://www.gnu.org/licenses/>.
 */

use crate::ui::builder::buildable::Buildable;
use crate::ui::builder::error::{BuildResult, FillResult};
use crate::ui::builder::filler::Filler;
use std::fmt::Debug;

pub mod buildable;
pub mod dummy;
pub mod error;
pub mod filler;

/// This trait is used to fill the builder with data and then build the final object.
///
/// The `Buildable` associated type is the type that the builder will build.
/// The `fill` method is used to fill the builder with data.
/// The `build` method is used to build the final object.
pub trait Builder: Default + Debug {
    type Buildable: Buildable;

    async fn fill<F: Filler>(&mut self, filler: &mut F) -> FillResult<()>;

    async fn build(self) -> BuildResult<Self::Buildable>;
}

#[derive(Debug, Clone)]
pub struct BuilderHolder<T>
where
    T: Sized + Clone + Debug,
{
    pub value: Option<T>,
    pub def: TypeWrapped<T>,
}

impl<T> BuilderHolder<T>
where
    T: Sized + Clone + Debug,
{
    pub fn new(def: FillableDefinition<T>) -> Self {
        Self {
            value: None,
            def: TypeWrapped::new(def),
        }
    }
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
    //#endregion

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

        paste::paste! { #[derive(Debug, Clone)] pub struct [< $name Builder >] {
            $($($required_ident: $crate::ui::builder::filler::BuilderHolder<$required_type>,)+)?
            $($($optional_ident: $crate::ui::builder::filler::BuilderHolder<$optional_type>,)+)?
        }}

        impl $crate::ui::builder::buildable::Buildable for $name {
            type Builder = paste::paste! { [< $name Builder >] };
        }

        paste::paste! { impl $crate::ui::builder::Builder for [< $name Builder >] {
            type Buildable = $name;

            async fn fill<F: $crate::ui::builder::filler::Filler>(&mut self, filler: &mut F) -> $crate::ui::builder::error::FillResult<()> {
                use $crate::ui::builder::filler::Filler as _Filler;
                use $crate::ui::builder::filler::FillableDefinition as _FillableDefinition;
                use $crate::ui::builder::Builder as _Builder;
                use $crate::ui::builder::error::FillError as _FillError;


                $($($crate::builder!(@try_fill [self, filler] $required_ident: $required_type);)+)?
                $($($crate::builder!(@try_fill [self, filler] $optional_ident: $optional_type);)+)?

                // $($($crate::builder!(@fill self $required_ident, filler);)+)?
                // $($($crate::builder!(@fill self $optional_ident, filler);)+)?

                Ok(())
            }

            async fn build(self) -> Result<$name, $crate::ui::builder::error::BuildError> {
                Ok($name {
                    $($($required_ident: $crate::builder!(@build_req self $required_ident),)+)?
                    $($($optional_ident: $crate::builder!(@build_opt self $optional_ident),)+)?
                })
            }
        }}

        paste::paste! { impl Default for [< $name Builder >] {
            fn default() -> Self {
                Self {
                    $($($required_ident: $crate::builder!(@default $required_ident: $required_type, true $(, $required_default)?),)+)?
                    $($($optional_ident: $crate::builder!(@default $optional_ident: $optional_type, false $(, $optional_default)?),)+)?
                }
            }
        }}
    };

    //#region Internal macros
    // The macro that fills the builder with the data from the filler.
    // (@fill $self:ident $field:ident, $filler:ident) => {
    //   if let Ok(value) = $crate::ui::builder::dummy::try_fill(&$self.$field.def, $filler).await {
    //     if let Some(old) = $self.$field.value.replace(value) {
    //         tracing::debug!("Replaced {} old value of {:?}", stringify!($field), old);
    //     }
    //   }
    // };

    // Build the final object from the builder with a required field.
    (@build_req $self:ident $field:ident) => {
        $self.$field.value.or_else(|| {
            let value = &$self.$field.def.value();
            value.default.map(|v| v())
        }).ok_or_else(|| lib::ui::builder::error::BuildError::MissingField { field: stringify!($field).to_string() })?
    };

    // Build the final object from the builder with an optional field.
    (@build_opt $self:ident $field:ident) => {
        $self.$field.value.or_else(|| {
            let value = &$self.$field.def.value();
            value.default.map(|v| v())
        })
    };

    // Create a default value for the builder struct.
    (@default $ident:ident: $ty:ty, $required:expr $(, $default:expr)?) => {
        $crate::ui::builder::filler::BuilderHolder::new($crate::ui::builder::filler::FillableDefinition::<$ty> {
            name: stringify!($ident),
            required: $required,
            default: $crate::builder!(@expr_or_none $(|| $default)?),

            _pd: std::marker::PhantomData,
        })
    };

    // If all fields have a default value create a default implementation.
    (@default $name:ident
        [$($required_ident:ident: $required_type:expr => $required_default:expr),+]
        [$($optional_ident:ident: $optional_type:expr => $optional_default:expr),+]
    ) => {
        impl Default for $name {
            fn default() -> Self {
                Self {
                    $($required_ident: $required_default,)+
                    $($optional_ident: $optional_default,)+
                }
            }
        }
    };

    // Utility to convert the default expression into an option or none.
    (@expr_or_none $expr:expr) => {
        Some($expr)
    };
    (@expr_or_none) => {
        None
    };

    (@maybe_continue $self:ident $ident:ident || $expr:expr) => {
        match $expr {
            Ok(value) => {
                if let Some(old) = $self.$ident.value.replace(value) {
                    tracing::debug!("Replaced {} old value of {:?}", stringify!($ident), old);
                }
            }
            Err($crate::ui::builder::error::FillError::InvalidFiller { field, filler }) => {
                tracing::error!("Filler {} failed to fill field {}", filler, field);
            }
        }
    };
    (@try_fill [$self:ident, $filler:ident] $ident:ident: $type:ty) => {
        let def = &$self.$ident.def.value();
        let pd = def._pd;

        trait Call {
            async fn call<F: Filler>(filler: &mut F, def: &FillableDefinition<Self>) -> FillResult<Self>;
        }

        impl Call for dyn FromStr {
            async fn call<F: _Filler>(filler: &mut F, def: &_FillableDefinition<Self>) -> _FillResult<Self> {
                $filler.fill_input(def).await.map(|v| {
                    unsafe { std::intrinsics::transmute_unchecked(v) }
                })
            }
        }

        impl Call for dyn Buildable {
            async fn call<F: _Filler>(filler: &mut F, def: &_FillableDefinition<Self>) -> _FillResult<Self> {
                let mut builder = <Self as _Builder>::Buildable::builder();
                builder.fill(filler).await?;
                Ok(unsafe { std::intrinsics::transmute_unchecked(builder) })
            }
        }

        $type::call($filler, def).await

        // if impls::impls!($type: FromStr) {
        //     $filler.fill_input(def).await.map(|v| {
        //         unsafe { std::intrinsics::transmute_unchecked(v) }
        //     })
        // } else if impls::impls!($type: Builder) {
        //     use $crate::ui::builder::Builder as _;
        //
        //     let mut builder = <$type as Buildable>::Builder::builder();
        //     builder.fill($filler).await?;
        //     Ok(unsafe { std::intrinsics::transmute_unchecked(builder) })
        // } else {
        //     Err($crate::ui::builder::error::FillError::NoFillers { field: stringify!($ident).to_string() })
        // }

        // $crate::builder!(@maybe_continue $self $ident || impls_utils!(async fn |pd, def: &_FillableDefinition<$type>, $filler: &mut impl _Filler| FromStr => $type => {
        //     $filler.fill_input(def).await.map(|v| {
        //         unsafe { std::intrinsics::transmute_unchecked(v) }
        //     })
        // }));

        // $crate::builder!(@maybe_continue $self $ident || impls_utils!(async fn |pd, def: &_FillableDefinition<$type>, $filler: &mut impl _Filler| Builder => $type => {
        //     use $crate::ui::builder::Builder as _;
        //
        //     let mut builder = T::Buildable::builder();
        //     builder.fill(filler).await?;
        //     Ok(unsafe { std::intrinsics::transmute_unchecked(builder) })
        // }));
    };
    //#endregion
}
