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

#![feature(proc_macro_diagnostic)]
#![feature(result_option_inspect)]
#![feature(downcast_unchecked)]
#![feature(type_name_of_val)]
#![feature(cfg_eval)]
#![feature(cfg_match)]

mod builder;
mod enums;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use std::any::{type_name, type_name_of_val};
use syn::parse::Parser;
use syn::spanned::Spanned;
use syn::{parse_macro_input, Data, DeriveInput, Ident, Path, TypePath};

pub(crate) fn error(span: proc_macro2::Span, message: &str) -> TokenStream2 {
    // span.unwrap().error(message).emit();
    syn::Error::new(span, message).into_compile_error()
}

fn error_input<E, O>(span: proc_macro2::Span, other: O) -> TokenStream2 {
    error(
        span,
        &format!(
            "Derive macro can only be applied to {}, got {}",
            type_name::<E>(),
            type_name_of_val(&other),
        ),
    )
}

#[proc_macro_derive(EnumVariants)]
pub fn enum_variants(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    enums::variants::variants(input).into()
}

#[proc_macro_derive(EnumNames)]
pub fn enum_names(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    enums::names::names(input).into()
}

#[proc_macro_derive(EnumRegex)]
pub fn enum_regex(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    enums::regex::regex(input).into()
}

#[proc_macro_derive(Delegation, attributes(delegate))]
pub fn delegate_trait(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // Check if the input is an enum
    let item_enum = match &input.data {
        Data::Enum(item_enum) => item_enum,
        _ => return error(input.span(), "Delegate can only be derived for enums").into(),
    };

    // Retrieve the name of the enum
    let enum_name = &input.ident;

    let delegate_attr = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("delegate"))
        .unwrap_or_else(|| panic!("Missing `delegate` attribute on enum root: {enum_name}"));

    let mut delegate_type = None;
    delegate_attr
        .parse_nested_meta(|meta| {
            if meta.path.is_ident("trait") {
                let value = meta.value()?;
                let path: TypePath = value.parse()?;
                delegate_type = Some(path);
                Ok(())
            } else {
                Err(meta.error("Expected `trait = crate::path::to::class`"))
            }
        })
        .unwrap();

    // Generate the output code for each enum variant
    let mut consts = Vec::new();
    let mut delegation_arms = Vec::new();
    for variant in &item_enum.variants {
        let ident = &variant.ident;
        let attr = match variant.attrs.iter().find(|a| a.path().is_ident("delegate")) {
            Some(attr) => attr,
            None => return error(variant.span(), "Missing `delegate` attribute on variant").into(),
        };

        let nested = attr.parse_nested_meta(|meta| {
            if !meta.path.is_ident("path") {
                return Err(meta.error("Expected `delegate(path = crate::path::to::class)`"));
            }

            let path = meta.value()?.parse::<Path>()?;
            let const_ident = Ident::new(&format!("{}_INSTANCE", ident), ident.span());
            let r#const = quote! { #[allow(non_upper_case_globals)] static #const_ident: _LazyLock<#enum_name::Delegate> = _LazyLock::new(|| Box::new(#path::new())); };
            let arm = quote! { #enum_name::#ident => &*#const_ident };
            consts.push(r#const);
            delegation_arms.push(arm);
            Ok(())
        });

        if let Err(err) = nested {
            return err.into_compile_error().into();
        }
    }

    // Generate the output code
    let expanded = quote! {
        #[automatically_derived]
        impl #enum_name {
            #[automatically_derived]
            pub type Delegate = Box<(dyn #delegate_type)>;
        }

        const _: () = {
            use std::sync::LazyLock as _LazyLock;
            use std::ops::Deref as _Deref;

            #(#consts)*

            #[automatically_derived]
            impl std::ops::Deref for #enum_name {
                type Target = #enum_name::Delegate;

                #[automatically_derived]
                fn deref(&self) -> &Self::Target {
                    match self {
                        #(#delegation_arms),*
                    }
                }
            }
        };

    };

    // Return the generated code as a TokenStream
    TokenStream::from(expanded)
}

#[proc_macro_derive(CommonFields)]
pub fn conditional_fields_macro(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    enums::common_fields::common_fields(input).into()
}

#[proc_macro_derive(Builder, attributes(builder))]
pub fn builder_macro(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    builder::builder(input)
}

// #[proc_macro_derive(Generics)]
// pub fn generics_macro(input: TokenStream) -> TokenStream {
//     // Parse the input tokens into a syntax tree
//     let input = parse_macro_input!(input as DeriveInput);
//
//     // Get the name of the struct
//     let struct_name = &input.ident;
//
//     // Extract generics information from the first field (you can modify this based on your needs)
//     let generics_from_field = if let Data::Struct(data) = &input.data {
//         if let Some(field) = data.fields.iter().next() {
//             let ty = &field.ty;
//             let is_bool = quote! { impls::impls!(#ty) }
//
//             quote! { #ty }
//         } else {
//             quote! { compile_error!("Struct must have at least one field") }
//         }
//     } else {
//         quote! { compile_error!("Only structs are supported") }
//     };
//
//     // Convert the generated code into a TokenStream and return it
//     output.into()
// }
