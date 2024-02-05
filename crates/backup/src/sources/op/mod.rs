/*
 * Copyright (c) 2024. James Draycott <me@racci.dev>
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

pub mod account;
pub mod cli;
pub mod core;
pub mod identifier;
pub mod one_pux;
pub mod v2;

/// Generates an object which can be converted to and from,
/// to structs, the first struct is in the 1Pux format,
/// the second is how the data is returned from the OnePassword CLI.
///
/// The macro should be like creating a struct but with a few extra features,
/// each field can be defined with the following syntax:
/// vis field_name {
///     type: Type,
///     required: bool,
///     renamed: "new_name",
/// } = default_value;
///
/// Example:
/// ```rust
/// use backup::generate_object;
///
/// generate_object!(Account, {
///    pub username {
///      type: String,
///      required: true,
///    };
/// });
/// ```
///
#[macro_export]
macro_rules! generate_object {
    ($object_name:ident {
        $(
            $field:ident > $one_pux_field:ident => $cli_type:ty > $one_pux_type:ty $transform_block:block
        ),* $(,)?

        $(
            [ $($multi_field:ident: $multi_type:ty),+ $(,)? ] for $multi_one_pux_field:ident: $multi_one_pux_type:ty => $multi_transform_block:block
        ),* $(,)?
    }) => { $crate::generate_object!(impl $object_name
        [ $($field, $one_pux_field, $cli_type, $one_pux_type, $transform_block),* ]
        [ $([ $($multi_field: $multi_type),+ ] for $multi_one_pux_field: $multi_one_pux_type => $multi_transform_block),* ]
    ); };

    (impl $object_name:ident
        [ $($cli_field:ident, $one_pux_field:ident, $cli_type:ty, $one_pux_type:ty, $transform_block:block),* ]
        [ $([ $($multi_field:ident: $multi_type:ty),+ ] for $multi_one_pux_field:ident: $multi_one_pux_type:ty => $multi_transform_block:block),* ]
    ) => {paste::paste! {
        /// The first layer struct which is gotten from the OnePassword CLI.
        pub struct [< $object_name Cli >] {
            pub $($cli_field: $cli_type),*
            $($(pub $multi_field: $multi_type),+),*
        }

        /// The second layer struct which is compatible with the 1Pux format.
        pub struct $object_name {
            pub $($one_pux_field: $one_pux_type),*
            $(pub $multi_one_pux_field: $multi_one_pux_type),*
        }

        impl From<[< $object_name Cli >]> for $object_name {
            fn from(cli: [< $object_name Cli >]) -> Self {
                Self {
                    $($one_pux_field: $transform_block,)*
                    $($multi_one_pux_field: $multi_transform_block,)*
                }
            }
        }
    }};
}
