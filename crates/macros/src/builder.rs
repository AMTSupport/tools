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

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_quote, Block, Data, DeriveInput, Expr, Fields, Ident, Type};

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
            impl lib::ui::Buildable for #struct_name {
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

    let quoted_fields = fields.iter().map(|(name, ty, _, _)| {
        quote! {
            #name: Option<#ty>
        }
    });

    quote! {
        #[derive(Default)]
        pub struct #builder_name {
            #(#quoted_fields,)*
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

    let field_fillers = fields.iter().map(|(name, ty, _, default)| {
        macro_rules! create_impls {
            ($trait:path) => {
                quote! {{
                    trait DoesNotImpl<T> {
                        const IMPLS: bool = false;

                        // async fn fill(name: &str, filler: &lib::ui::Filler, default: Option<T>) -> ! {
                        //     panic!("Field {} does not implement {}", name, stringify!($trait));
                        // }
                    }
                    impl<T: ?Sized> DoesNotImpl for T {}

                    struct Wrapper<T: ?Sized>(std::marker::PhantomData<T>);

                    #[allow(dead_code)]
                    impl<T: ?Sized + $trait> Wrapper<T> {
                        const IMPLS: bool = true;
                    }

                    use #ty as _type;
                    <Wrapper<_type>>::IMPLS
                }}
            };
        }

        let is_bool = ty.to_token_stream().to_string() == "bool";
        let is_buildable = create_impls!(lib::ui::Buildable);
        let is_from_str = create_impls!(std::str::FromStr);

        let buildable_ident = Ident::new(&format!("{}_BUILDABLE", name), name.span());
        let from_str_ident = Ident::new(&format!("{}_FROM_STR", name), name.span());

        let default_expr = if default.is_none() {
            quote! { None }
        } else {
            quote! { Some(#default) }
        };

        let filler = quote! {
            <Wrapper<#ty>>::fill(stringify!(#name), filler, #default_expr).await?
        };

        filler.into_token_stream()

        // let filler = if is_bool {
        //     quote! { filler.fill_bool(stringify!(#name), #default_expr).await?; }
        // } else {
        //     quote! {
        //         const #buildable_ident: bool = #is_buildable;
        //         const #from_str_ident: bool = #is_from_str;
        //
        //         // if buildable_ident is true force transmute type to Buildable
        //         if #buildable_ident {
        //             use lib::ui::Buildable as _;
        //
        //             transmute::<_, &mut dyn lib::ui::Buildable>(filler).fill_input(stringify!(#name), #default_expr).await?;
        //             let builder = <#ty as lib::ui::Buildable>::builder();
        //             builder.fill(&mut filler).await?;
        //         } else if #from_str_ident {
        //             filler.fill_input(stringify!(#name), Some(#default_expr)).await?;
        //         } else {
        //             panic!("Field is not a bool or a builder");
        //         }
        //     }
        // };
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
                #name: self.#name.take().ok_or_else(|| anyhow::anyhow!("Field {} is required", stringify!(#name)))?
            }
        }
    });

    quote! {
        impl lib::ui::Builder for #builder_name {
            type Buildable = #struct_name;

            async fn fill<F: lib::ui::builder::Filler>(&mut self, filler: &mut F) -> anyhow::Result<()> {
                #(#field_fillers)*

                Ok(())
            }

            async fn build(self) -> anyhow::Result<#struct_name> {
                Ok(#struct_name {
                    #(#field_builders,)*
                })
            }
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
                let ty = &field.ty;
                let ident = match &field.ident {
                    Some(ident) => ident,
                    None => {
                        return Err(
                            quote! { compile_error!("Builder macro can only be used on structs with named fields"); },
                        )
                    }
                };

                let mut optional = false;
                let mut default = None;
                for attr in &field.attrs {
                    if !attr.path().is_ident("builder") {
                        continue;
                    }

                    attr.parse_nested_meta(|meta| {
                        if meta.path.is_ident("optional") {
                            optional = true;
                            return Ok(());
                        }

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
                                    )
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
