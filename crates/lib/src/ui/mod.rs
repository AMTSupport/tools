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

use anyhow::Result;
use std::any::Any;
use std::fmt::Debug;

#[cfg(feature = "ui-cli")]
pub mod cli;
#[cfg(feature = "ui-tui")]
pub mod tui;

pub trait Ui<R = Result<Self>>
where
    Self: UiBuidableFiller,
{
    /// The arguments that will be used to create the [`Ui`]
    ///
    /// This can be used to pass in configuration options, or other data.
    /// This is not required to be used, but is available if needed.
    type Args = ();

    /// Create a new instance of the [`Ui`]
    ///
    /// This is used to create a new instance of the [`Ui`], and can be used to
    /// parse arguments, or other data.
    ///
    /// Your logging guard should be set up here, as well as any other
    /// configuration that is needed.
    #[allow(clippy::new_ret_no_self)]
    fn new(args: Self::Args) -> R
    where
        Self: Sized;
}

impl<U: UiBuidableFiller + Default> Ui for U {
    fn new(_args: Self::Args) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(Self::default())
    }
}

pub trait UiBuidableFiller {
    async fn fill<B: UiBuildable<V>, V: From<B> + Debug>() -> Result<V>;

    async fn modify<B: UiBuildable<V>, V: From<B> + Debug>(builder: B) -> Result<V>;
}

pub trait UiBuildable<V>
where
    V: Sized + From<Self> + Debug,
    Self: Sized + Default + Debug,
{
    const REQUIRED_FIELDS: &'static [&'static str];
    const OPTIONAL_FIELDS: &'static [&'static str];

    fn env_fill(&mut self) -> Result<Vec<&'static str>>;

    /// Returns a list of fields which have been filled
    /// This is used to determine if the user has filled in all the required fields
    /// and to display a summary of the fields which will be used.
    fn filled_fields(&self) -> Vec<&&str>;

    fn set_field(&mut self, field: &str, value: &str) -> Result<()>;

    fn get_field<T: Any + Sized>(&self, field: &str) -> Option<&T>;

    fn display(&self, field: &str) -> String {
        match self.get_field::<Box<dyn Any>>(field) {
            Some(value) => format!("{value:?}"),
            None => "None".into(),
        }
    }

    fn build(self) -> Result<V>;
}

/// This macro is used to generate the builder pattern for the UI
/// It generates a builder struct and a builder struct for each field
/// The builder struct is used to build the UI struct
///
/// Surrounding the ident of a field with [] will make it optional
/// All optional fields must be at the start of the macro.
///
/// Example:
/// ```
/// use lib::builder;
///
/// builder!(Test = [[opt_field] => String, field => String]);
/// builder!(Test2 = [field => String]);
/// ```
#[macro_export]
macro_rules! builder {
    (@count_inputs) => { 0 };
    (@count_inputs $expr:tt) => { 1 };
    (@count_inputs $($expr:tt),*) => { $($crate::builder!(@count_inputs $expr) +)* 0 };

    (@sort
        $name:ident
        {$($($(#[$opt_meta:meta])* $opt_field:ident => $opt_type:ty),+)? }
        ($($($(#[$meta:meta])* $field:ident => $type:ty $(= $default:expr)?),+)? )
        [$($(#,[$item_meta:meta]),+ ,)? $item_field:ident , => , $item_type:ty $(, = , $item_default:expr)? ,,, $($remaining:tt),+]) => {

        $crate::builder!(@sort
            $name
            {$($($(#[$opt_meta])* $opt_field => $opt_type),+)? }
            ($($($(#[$meta])* $field => $type $(= $default)?),+ ,)? $($(#[$item_meta])+)? $item_field => $item_type $(= $item_default)?)
            [$($remaining),+]
        );
    };
    (@sort
        $name:ident
        {$($($(#[$opt_meta:meta])* $opt_field:ident => $opt_type:ty),+)? }
        ($($($(#[$meta:meta])* $field:ident => $type:ty $(= $default:expr)?),+)? )
        [$($(#,[$item_meta:meta]),+,)? $item_field:ident , => , $item_type:ty $(, = , $item_default:expr)?]) => {

        $crate::builder!(@sort
            $name
            {$($($(#[$opt_meta])* $opt_field => $opt_type),+)? }
            ($($($(#[$meta])* $field => $type $(= $default)?),+ ,)? $($(#[$item_meta])+)? $item_field => $item_type $(= $item_default)?)
            []
        );
    };

    (@sort
        $name:ident
        {$($($(#[$opt_meta:meta])* $opt_field:ident => $opt_type:ty),+)? }
        ($($($(#[$meta:meta])* $field:ident => $type:ty $(= $default:expr)?),+)? )
        [$($(#,[$item_meta:meta]),+,)? [$item_field:ident], => , $item_type:ty,,, $($remaining:tt),+]) => {

        $crate::builder!(@sort
            $name
            {$($($(#[$opt_meta])* $opt_field => $opt_type),+ ,)? $($(#[$item_meta])+)? [$item_field] => $item_type }
            ($($($(#[$meta])* $field => $type $(= $default)?),+)?)
            [$($remaining),+]
        );
    };
    (@sort
        $name:ident
        {$($($(#[$opt_meta:meta])* $opt_field:ident => $opt_type:ty),+)? }
        ($($($(#[$meta:meta])* $field:ident => $type:ty $(= $default:expr)?),+)? )
        [$($(#,[$item_meta:meta]),+,)? [$item_field:ident] , => , $item_type:ty,,]) => {

        $crate::builder!(@sort
            $name
            {$($($(#[$opt_meta])* $opt_field => $opt_type),+ ,)? $($(#[$item_meta])+)? $item_field => $item_type }
            ($($($(#[$meta])* $field => $type $(= $default)?),+)?)
            []
        );
    };

    (@sort
        $name:ident
        {$($($(#[$opt_meta:meta])* $opt_field:ident => $opt_type:ty),+)? }
        ($($($(#[$meta:meta])* $field:ident => $type:ty $(= $default:expr)?),+)? )
        []
    ) => {
        $crate::builder!(@impl $name = [ $({$($(#[$meta])* $field => $type $(= $default)?),+ ,})? $([$($(#[$opt_meta])* [$opt_field] => $opt_type),+ ,])?]);
    };

    ($name:ident = [ $($fields:tt)+ ]) => {
        $crate::builder!(@sort $name {} () [$($fields),+]);
    };

    (@impl $name:ident = [
        $({$(,)? $(
            $(#[$meta:meta])*
            $field:ident => $type:ty $(= $default:expr)?
        ),+,})?
        $([$(,)? $(
            $(#[$opt_meta:meta])*
            [$opt_field:ident] => $opt_type:ty
        ),+,])?
    ]) => {
        #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
        pub struct $name {
            $($(
                $(#[$opt_meta])*
                $opt_field: Option<$opt_type>
            ),+,)?
            $($(
                $(#[$meta])*
                $field: $type
            ),+,)?
        }

        paste::paste! {
            #[derive(Debug, Clone)]
            pub struct [<$name Builder>] {
                $($($opt_field: Option<$opt_type>),+,)?
                $($($field: Option<$type>),+,)?
            }

            impl $name {
                $($(pub fn [<set_ $field>](&self, $field: $type) -> Self {
                    let mut builder = self.clone();
                    builder.$field = $field;
                    builder
                })+)?
                $($(pub fn [<set_ $opt_field>](&self, $opt_field: $opt_type) -> Self {
                    let mut builder = self.clone();
                    builder.$opt_field = Some($opt_field);
                    builder
                })+)?

                $($(pub fn [<get_ $field>](&self) -> &$type {
                    &self.$field
                })+)?

                $($(pub fn [<get_ $opt_field>](&self) -> Option<&$opt_type> {
                    self.$opt_field.as_ref()
                })+)?
            }

            impl [<$name Builder>] {
                $($(pub fn [<set_ $field>](&self, $field: $type) -> Self {
                    let mut builder = self.clone();
                    builder.$field = Some($field);
                    builder
                })+)?
                $($(pub fn [<set_ $opt_field>](&self, $opt_field: $opt_type) -> Self {
                    let mut builder = self.clone();
                    builder.$opt_field = Some($opt_field);
                    builder
                })+)?

                $($(pub fn [<get_ $field>](&self) -> Option<&$type> {
                    self.$field.as_ref()
                })+)?

                $($(pub fn [<get_ $opt_field>](&self) -> Option<&$opt_type> {
                    self.$opt_field.as_ref()
                })+)?
            }

            use $crate::ui::UiBuildable as _UiBuildable;
            impl _UiBuildable<$name> for [<$name Builder>] {
                const REQUIRED_FIELDS: &'static [&'static str] = &[$($(stringify!($field)),+)?];
                const OPTIONAL_FIELDS: &'static [&'static str] = &[$($(stringify!($opt_field)),+)?];

                fn env_fill(&mut self) -> anyhow::Result<Vec<&'static str>> {
                    let mut filled = vec![];
                    let mut env_fill_inner = |field| {
                        if let Ok(value) = std::env::var(field) {
                            match self.set_field(field, &*value) {
                                Ok(_) => filled.push(field),
                                Err(_) => tracing::error!("Failed to set field {} from env", field)
                            };
                        }
                    };

                    $($(env_fill_inner(stringify!($field));)+)?
                    $($(env_fill_inner(stringify!($opt_field));)+)?

                    Ok(filled)
                }

                fn filled_fields(&self) -> Vec<&&str> {
                    Self::REQUIRED_FIELDS
                        .into_iter()
                        .chain(Self::OPTIONAL_FIELDS.into_iter())
                        .filter(|field| self.get_field::<Box<dyn std::any::Any>>(field).is_some())
                        .collect()
                }

                fn set_field(&mut self, field: &str, value: &str) -> anyhow::Result<()> {
                    match field {
                        $($(stringify!($field) => self.[<set_ $field>](value.parse()?),)+)?
                        $($(stringify!($opt_field) => self.[<set_ $opt_field>](value.parse()?),)+)?
                        _ => return Err(anyhow::anyhow!("Unknown field {}", field))
                    };

                    Ok(())
                }

                fn get_field<T: std::any::Any + Sized>(&self, field: &str) -> Option<&T> {
                    // Downcast and box the value into an any type so we can return it
                    // without knowing the type, don't use as to cast as not all values are primitives
                    let boxed = match field {
                        $($(stringify!($field) => self.[<get_ $field>]().map(|value| Box::<&dyn std::any::Any>::new(value)),)*)?
                        $($(stringify!($opt_field) => self.[<get_ $opt_field>]().map(|value| Box::<&dyn std::any::Any>::new(value)),)*)?
                        _ => None
                    };

                    // convert to the correct type while checking if it is the correct type
                    boxed.and_then(|value| value.downcast_ref::<T>())
                }

                fn build(self) -> anyhow::Result<$name> {

                    $($(
                        $(self.$field.get_or_insert_with(|| $default);)?

                        if self.$field.is_none() {
                            return Err(anyhow::anyhow!("Missing required field {}", stringify!($field)));
                        }
                    )+)?

                    Ok($name {
                        $($($opt_field: self.$opt_field),+,)?
                        $($($field: self.$field.unwrap()),+,)?
                    })
                }
            }

            impl Default for [<$name Builder>] {
                fn default() -> Self {
                    let mut instance = Self {
                        $($($opt_field: None),+,)?
                        $($($field: None),+,)?
                    };

                    instance.env_fill().unwrap();
                    instance
                }
            }

            impl From<[<$name Builder>]> for $name {
                fn from(builder: [<$name Builder>]) -> Self {

                    builder.build().unwrap()
                }
            }
        }
    };
}
