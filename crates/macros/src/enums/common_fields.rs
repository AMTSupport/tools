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

use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::spanned::Spanned;
use syn::{Data, DataEnum, DeriveInput, Fields, Type};

pub fn common_fields(input: DeriveInput) -> TokenStream {
    let enum_name = &input.ident;

    let variants = match input.data {
        Data::Enum(ref item_enum) => &item_enum.variants,
        ref other => return crate::error_input::<DataEnum, _>(input.span(), other),
    };

    // Extract the common field names and types from the enum variants
    if variants.is_empty() {
        return crate::error(input.span(), "Enum has no variants");
    }

    let mut global_fields = get_fields_from_variant(&variants[0]);
    if global_fields.is_empty() {
        return crate::error(variants[0].span(), "Enum variant has no fields.");
    }

    for variant in variants.iter().skip(1) {
        let fields = get_fields_from_variant(variant);
        if fields.is_empty() {
            return crate::error(variant.span(), "Enum variant has no fields.");
        }

        // Will remove all fields that are not in the current variant
        global_fields.retain(|(field_name, field_type)| {
            fields.iter().any(|(other_field_name, other_field_type)| {
                field_name == other_field_name && field_type == other_field_type
            })
        });
    }

    let common_fields = global_fields;

    // Generate the conditional fields
    let functions = common_fields.iter().map(|(field_name, field_type)| {
        let mut string = field_name.to_string();
        if string.starts_with("__self_") {
            string.retain(|c| c.is_numeric());
        };

        let (function_name, field_index) = match string.parse::<usize>() {
            Ok(index) => (Ident::new(&format!("field_{index}"), field_name.span()), Some(index)),
            Err(_) => (field_name.clone(), None),
        };

        let enum_branches = variants.iter().map(|variant| {
            let variant_name = &variant.ident;
            match field_index {
                Some(index) => {
                    let varargs = (0..index).map(|_| quote! { _ }).collect::<Vec<_>>();
                    quote! { #enum_name::#variant_name(#(#varargs,)* field, ..) => field }
                }
                None => quote! { #enum_name::#variant_name { #field_name, .. } => #field_name },
            }
        });

        quote! {
            #[automatically_derived]
            pub const fn #function_name(&self) -> &#field_type {
                match self {
                    #(#enum_branches,)*
                }
            }
        }
    });

    let output = quote! {
        #[automatically_derived]
        impl #enum_name {
            #(#functions)*
        }
    };

    // Convert the generated code back into tokens and return them
    output
}

// Helper function get get all fields from a enum variant
fn get_fields_from_variant(variant: &syn::Variant) -> Vec<(Ident, &Type)> {
    match &variant.fields {
        Fields::Unit => vec![],
        Fields::Named(fields) => fields
            .named
            .iter()
            .map(|field| {
                let field_name = field.ident.as_ref().unwrap().clone();
                let field_type = &field.ty;
                (field_name, field_type)
            })
            .collect::<Vec<(Ident, &Type)>>(),
        Fields::Unnamed(fields) => fields
            .unnamed
            .iter()
            .enumerate()
            .map(|(index, field)| {
                let field_name = Ident::new(&format!("__self_{}", index), field.span());
                let field_type = &field.ty;
                (field_name, field_type)
            })
            .collect::<Vec<(Ident, &Type)>>(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    fn is_compile_error(input: DeriveInput) -> bool {
        common_fields(input).to_string().contains("compile_error!")
    }

    #[test]
    fn named_fields() {
        let input: DeriveInput = parse_quote! {
            enum Test {
                A { a: u8, b: u8 },
                B { a: u8, b: u8 },
                C { a: u8, b: u8 },
            }
        };

        let expected = quote! {
            #[automatically_derived]
            impl Test {
                #[automatically_derived]
                pub const fn a(&self) -> &u8 {
                    match self {
                        Test::A { a, .. } => a,
                        Test::B { a, .. } => a,
                        Test::C { a, .. } => a,
                    }
                }
                #[automatically_derived]
                pub const fn b(&self) -> &u8 {
                    match self {
                        Test::A { b, .. } => b,
                        Test::B { b, .. } => b,
                        Test::C { b, .. } => b,
                    }
                }
            }
        };

        assert!(!is_compile_error(input.clone()));
        assert_eq!(common_fields(input).to_string(), expected.to_string());
    }

    #[test]
    fn unnamed_fields() {
        let input: DeriveInput = parse_quote! {
            enum Test {
                A(u8, u8),
                B(u8, u8),
                C(u8, u8),
            }
        };

        let expected = quote! {
            #[automatically_derived]
            impl Test {
                #[automatically_derived]
                pub const fn field_0(&self) -> &u8 {
                    match self {
                        Test::A(field, ..) => field,
                        Test::B(field, ..) => field,
                        Test::C(field, ..) => field,
                    }
                }
                #[automatically_derived]
                pub const fn field_1(&self) -> &u8 {
                    match self {
                        Test::A(_, field, ..) => field,
                        Test::B(_, field, ..) => field,
                        Test::C(_, field, ..) => field,
                    }
                }
            }
        };

        assert!(!is_compile_error(input.clone()));
        assert_eq!(common_fields(input).to_string(), expected.to_string());
    }

    #[test]
    fn mixed_fields() {
        let input: DeriveInput = parse_quote! {
            enum Test {
                A { a: u8, b: u8, c: u8 },
                B { a: u8, c: u8 },
                C { a: u8, b: u8 },
            }
        };

        let expected = quote! {
            #[automatically_derived]
            impl Test {
                #[automatically_derived]
                pub const fn a(&self) -> &u8 {
                    match self {
                        Test::A { a, .. } => a,
                        Test::B { a, .. } => a,
                        Test::C { a, .. } => a,
                    }
                }
            }
        };

        assert!(!is_compile_error(input.clone()));
        assert_eq!(common_fields(input).to_string(), expected.to_string());
    }
}
