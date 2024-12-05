/*
 * Copyright (C) 2023-2024. James Draycott me@racci.dev
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

use crate::enums::variants;
use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{Data, DataEnum, DeriveInput};

pub fn regex(input: DeriveInput) -> TokenStream {
    let name = &input.ident;
    let data = match &input.data {
        Data::Enum(ref data) => data,
        other => return crate::error_input::<DataEnum, _>(input.span(), other),
    };

    let variant_names = variants::impl_names(data);
    let concatted = variant_names.iter().map(|i| i.to_string()).collect::<Vec<String>>().join("|");
    let len = variant_names.len();

    quote! {
        #[automatically_derived]
        impl #name {
            /// The regex that matches any single variant of this enum
            /// This is case insensitive.
            #[automatically_derived]
            pub const REGEX: &'static str = concat!(
                r"((?i)(?x)",
                #concatted,
                r")",
            );

            /// The regex that matches having multiple variants of this enum in a row (e.g. `hourly-daily`)
            /// These are separated by a `-`, and will be in the order they are defined in the enum.
            #[automatically_derived]
            pub const MULTI_REGEX: &'static str = concat!(
                r"((?i)(?x)",
                #concatted,
                r")-(",
                #concatted,
                r"){0,",
                #len,
                r"}",
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_regex() {
        let input = parse_quote! {
            enum Test {
                A,
                B,
                C,
            }
        };

        let expected = quote! {
            #[automatically_derived]
            impl Test {
                /// The regex that matches any single variant of this enum
                /// This is case insensitive.
                #[automatically_derived]
                pub const REGEX: &'static str = concat!(
                    r"((?i)(?x)",
                    "A|B|C",
                    r")",
                );

                /// The regex that matches having multiple variants of this enum in a row (e.g. `hourly-daily`)
                /// These are separated by a `-`, and will be in the order they are defined in the enum.
                #[automatically_derived]
                pub const MULTI_REGEX: &'static str = concat!(
                    r"((?i)(?x)",
                    "A|B|C",
                    r")-(",
                    "A|B|C",
                    r"){0,",
                    3usize,
                    r"}",
                );
            }
        };

        let actual = regex(input);
        assert_eq!(actual.to_string(), expected.to_string());
    }
}
