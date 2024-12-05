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

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{Data, DeriveInput, Expr, Fields, GenericArgument, Ident, Type};

pub fn builder(input: DeriveInput) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let struct_name = &input.ident;
    let builder_name = Ident::new(&format!("{}Builder", struct_name), struct_name.span());

    // Check if the input is a struct
    if let Data::Struct(ref s) = input.data {
        // Generate the builder struct
        let builder_struct = generate_builder_struct(&builder_name, s);

        // Generate the implementation of the builder
        let builder_impl = generate_builder_impl(struct_name, &builder_name, s);

        // Combine the generated code
        let generated_code = quote! {
            impl lib::ui::builder::buildable::Buildable for #struct_name {
                type Builder = #builder_name;
            }

            #builder_struct
            #builder_impl
        };

        // Return the generated code as a TokenStream
        TokenStream::from(generated_code)
    } else {
        (quote! { compile_error!("Builder macro can only be used on structs"); }).into()
    }
}

// Generate the builder struct
fn generate_builder_struct(builder_name: &Ident, data: &syn::DataStruct) -> proc_macro2::TokenStream {
    let fields = match get_fields(data) {
        Ok(fields) => fields,
        Err(err) => return err,
    };

    let quoted_fields = fields.iter().map(|(name, ty, _, _)| quote!(#name: Option<#ty>));

    quote! {
        #[derive(Default)]
        pub struct #builder_name<'b> {
            #(#quoted_fields,)*

            _phantom: std::marker::PhantomData<&'b ()>,
        }
    }
}

// Generate the implementation of the builder
fn generate_builder_impl(
    struct_name: &Ident,
    builder_name: &Ident,
    data: &syn::DataStruct,
) -> proc_macro2::TokenStream {
    let fields = match get_fields(data) {
        Ok(fields) => fields,
        Err(err) => return err,
    };

    let field_fillers = fields.iter().map(|(name, ty, _, _)| {
        let filler = quote! {
            self.#name = Some(lib::ui::builder::dummy::try_fill::<_, #ty>(
                unsafe { std::mem::transmute_copy(&#builder_name::DEFINITIONS[stringify!(#name)]) },
                filler
            ).await?);
        };

        filler.into_token_stream()
    });

    let field_builders = fields.iter().map(|(name, _, optional, default)| {
        if *optional {
            if let Some(default) = default {
                quote! {
                    #name: self.#name.take().unwrap_or(Some(#default))
                }
            } else {
                quote! {
                    #name: self.#name.take()
                }
            }
        } else {
            quote! {
                #name: self.#name.take().ok_or_else(|| lib::ui::builder::error::BuildError::MissingField { field: stringify!(#name).to_string() })?
            }
        }
    });

    let insert_definitions = fields.iter().map(|(name, ty, _, default)| {
        let default = if default.is_none() {
            quote! { None }
        } else {
            quote! { Some(|| #default) }
        };

        let init = match ty.to_token_stream().to_string().as_str() {
            "bool" => {
                println!("Found bool");
                quote! {
                    lib::ui::builder::filler::TypeWrapped::Bool(PhantomData::<bool>, lib::ui::builder::filler::FillableDefinition {
                        name: stringify!(#name),
                        default: #default,
                    })
                }
            }
            "String" => {
                println!("Found string");
                quote! {
                    lib::ui::builder::filler::TypeWrapped::String(PhantomData::<String>, lib::ui::builder::filler::FillableDefinition {
                        name: stringify!(#name),
                        default: #default,
                    })
                }
            }
            _ => {
                println!("Found unknown struct, looking for impls");
                quote! {
                    if impls::impls!(#ty: lib::ui::builder::buildable::Buildable) {
                        lib::ui::builder::filler::TypeWrapped::Buildable(PhantomData::<#ty>, lib::ui::builder::filler::FillableDefinition {
                            name: stringify!(#name),
                            default: #default,
                        })
                    } else if impls::impls!(#ty: std::str::FromStr) {
                        lib::ui::builder::filler::TypeWrapped::String(PhantomData::<#ty>, lib::ui::builder::filler::FillableDefinition {
                            name: stringify!(#name),
                            default: #default,
                        })
                    } else {
                        panic!("Unknown type");
                    }
                }
            }
        };

        quote! {
            map.insert(stringify!(#name), #init)
        }
    });

    quote! {
        impl<'b> lib::ui::builder::Builder for #builder_name<'b> {
            type Buildable = #struct_name;

            async fn fill<F: lib::ui::builder::filler::Filler>(mut self, filler: &mut F) -> lib::ui::builder::error::FillResult<Self> {
                #(#field_fillers)*

                Ok(self)
            }

            async fn build(mut self) -> lib::ui::builder::error::BuildResult<Self::Buildable> {
                Ok(Self::Buildable {
                    #(#field_builders,)*
                })
            }
        }

        impl<'b> #builder_name<'b> {
            // type FillableImpl = impl ?core::marker::Sized + std::any::Any;
            const DEFINITIONS: std::collections::HashMap<&'b str, lib::ui::buildable::filler::TypeWrapped> = {
                use std::collections::HashMap;

                let mut map = HashMap::new();
                #(#insert_definitions)*
                map
            };
        }
    }
}

// Get the fields of the struct
fn get_fields(
    data: &syn::DataStruct,
) -> Result<Vec<(Ident, Type, bool, Option<proc_macro2::TokenStream>)>, proc_macro2::TokenStream> {
    match &data.fields {
        Fields::Named(fields) => {
            let mut vec = Vec::new();
            for field in fields.named.iter() {
                let mut ty = &field.ty;
                let ident = match &field.ident {
                    Some(ident) => ident,
                    None => {
                        return Err(
                            quote! { compile_error!("Builder macro can only be used on structs with named fields"); },
                        );
                    }
                };

                let mut optional = false;
                let mut default = None;

                if let Type::Path(path) = ty {
                    let segment = &path.path.segments[0];
                    if segment.ident == "Option" {
                        if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                            if args.args.len() != 1 {
                                return Err(quote! {
                                    compile_error!("Option type argument must have exactly one type argument");
                                });
                            }

                            let arg = &args.args[0];
                            if let GenericArgument::Type(inner_type) = arg {
                                ty = inner_type;
                                optional = true;
                            } else {
                                return Err(quote! {
                                    compile_error!("Option type argument must have exactly one type argument");
                                });
                            };
                        }
                    }
                }

                for attr in &field.attrs {
                    if !attr.path().is_ident("builder") {
                        continue;
                    }

                    attr.parse_nested_meta(|meta| {
                        if meta.path.is_ident("default") {
                            // get the default expr
                            match meta.value()?.parse()? {
                                Expr::Lit(literal) => {
                                    default = Some(literal.into_token_stream());
                                }
                                Expr::Block(block) => {
                                    default = Some(block.into_token_stream());
                                }
                                Expr::Call(call) => {
                                    default = Some(call.into_token_stream());
                                }
                                Expr::Macro(mac) => {
                                    default = Some(mac.into_token_stream());
                                }
                                Expr::Struct(struc) => {
                                    default = Some(struc.into_token_stream());
                                }
                                expr => {
                                    return Err(
                                        meta.error(format!("expected literal, block, call or macro, found {expr:?}"))
                                    );
                                }
                            }

                            return Ok(());
                        }

                        Err(meta.error(format!(
                            "unrecognized builder argument {}",
                            meta.path.get_ident().expect("expected ident")
                        )))
                    })
                    .expect("failed to parse builder attribute");
                }

                vec.push((ident.clone(), ty.clone(), optional, default));
            }

            Ok(vec)
        }
        _ => Err(quote! { compile_error!("Builder macro can only be used on structs with named fields"); }),
    }
}
