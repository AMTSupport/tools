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

use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::spanned::Spanned;
use syn::{Data, DataEnum, DeriveInput};

pub fn variants(input: DeriveInput) -> TokenStream {
    let name = &input.ident;
    let data = match &input.data {
        Data::Enum(ref data) => data,
        other => return crate::error_input::<DataEnum, _>(input.span(), other),
    };

    let variant_names = impl_names(data);
    quote! {
        impl #name {
            #[automatically_derived]
            pub fn get_variants() -> Vec<#name> {
                vec![#(#name::#variant_names),*]
            }
        }
    }
}

pub fn impl_names(data: &DataEnum) -> Vec<&Ident> {
    data.variants.iter().map(|v| &v.ident).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_variants() {
        let input = parse_quote! {
            enum Test {
                A,
                B,
                C,
            }
        };

        let expected = quote! {
            impl Test {
                #[automatically_derived]
                pub fn get_variants() -> Vec<Test> {
                    vec![Test::A, Test::B, Test::C]
                }
            }
        };

        let actual = variants(input);
        assert_eq!(expected.to_string(), actual.to_string());
    }
}
