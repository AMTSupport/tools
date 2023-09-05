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

use crate::enums::variants;
use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{Data, DataEnum, DeriveInput};

pub fn names(input: DeriveInput) -> TokenStream {
    let name = &input.ident;
    let data = match &input.data {
        Data::Enum(ref data) => data,
        other => return crate::error_input::<DataEnum, _>(input.span(), other),
    };

    let variant_names = variants::impl_names(data);
    let lowercase_names = variant_names.iter().map(|v| v.to_string().to_lowercase()).collect::<Vec<String>>();
    quote! {
        #[automatically_derived]
        impl #name {
            pub const NAMES: &'static [&'static str] = &[#(stringify!(#variant_names)),*];

            #[automatically_derived]
            pub const fn name(&self) -> &'static str {
                match self {
                    #(#name::#variant_names => stringify!(#variant_names)),*
                }
            }
        }

        #[automatically_derived]
        impl std::fmt::Display for #name {
            #[automatically_derived]
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.name())
            }
        }

        #[automatically_derived]
        impl std::str::FromStr for #name {
            type Err = anyhow::Error;

            #[automatically_derived]
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s.to_lowercase().as_str() {
                    #(#lowercase_names => Ok(#name::#variant_names)),*,
                    _ => Err(anyhow::anyhow!("Unknown variant name: {}", s)),
                }
            }
        }
    }
}
