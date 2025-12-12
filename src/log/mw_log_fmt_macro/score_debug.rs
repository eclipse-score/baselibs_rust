//
// Copyright (c) 2025 Contributors to the Eclipse Foundation
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache License Version 2.0 which is available at
// <https://www.apache.org/licenses/LICENSE-2.0>
//
// SPDX-License-Identifier: Apache-2.0
//

use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Error, Index};

pub(crate) fn expand(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let DeriveInput {
        attrs: _,
        vis: _,
        ident,
        generics,
        data,
    } = parse_macro_input!(input as DeriveInput);

    // Split generics.
    let (impl_generics, ty_generics, _) = generics.split_for_impl();

    match data {
        Data::Struct(data_struct) => match data_struct.fields {
            // Regular struct - contains named fields.
            syn::Fields::Named(fields) => {
                // Generate `.field` method calls based on struct fields.
                let mut field_methods = Vec::new();
                for field in fields.named.into_iter() {
                    let ident = match field.ident {
                        Some(ident) => ident,
                        None => return Error::new_spanned(field, "identifier not found").to_compile_error().into(),
                    };
                    let name = ident.to_string();
                    field_methods.push(quote! { .field(#name, &self.#ident) });
                }

                // Generate `ScoreDebug` implementation for provided struct.
                let struct_name = ident.to_string();
                quote! {
                    #[automatically_derived]
                    impl #impl_generics mw_log::fmt::ScoreDebug for #ident #ty_generics {
                        fn fmt(&self, f: mw_log::fmt::Writer, spec: &mw_log::fmt::FormatSpec) -> mw_log::fmt::Result {
                            mw_log::fmt::DebugStruct::new(f, spec, #struct_name)
                                #(#field_methods)*
                                .finish()
                        }
                    }
                }
            },

            // Tuple struct - contains unnamed fields.
            syn::Fields::Unnamed(fields) => {
                // Generate `.field` method calls based on struct fields.
                let mut field_methods = Vec::new();
                for index in 0..fields.unnamed.len() {
                    let syn_index = Index::from(index);
                    field_methods.push(quote! { .field(&self.#syn_index) });
                }

                // Generate `ScoreDebug` implementation for provided struct.
                let struct_name = ident.to_string();
                quote! {
                    #[automatically_derived]
                    impl #impl_generics mw_log::fmt::ScoreDebug for #ident #ty_generics {
                        fn fmt(&self, f: mw_log::fmt::Writer, spec: &mw_log::fmt::FormatSpec) -> mw_log::fmt::Result {
                            mw_log::fmt::DebugTuple::new(f, spec, #struct_name)
                                #(#field_methods)*
                                .finish()
                        }
                    }
                }
            },

            // Unit struct - no fields.
            syn::Fields::Unit => {
                // Generate `ScoreDebug` implementation for provided struct.
                let struct_name = ident.to_string();
                quote! {
                    #[automatically_derived]
                    impl mw_log::fmt::ScoreDebug for #ident {
                        fn fmt(&self, f: mw_log::fmt::Writer, spec: &mw_log::fmt::FormatSpec) -> mw_log::fmt::Result {
                            mw_log::fmt::DebugStruct::new(f, spec, #struct_name).finish()
                        }
                    }
                }
            },
        },

        Data::Enum(data_enum) => {
            // Generate matches for each variant.
            let mut matches = Vec::new();
            for variant in data_enum.variants.into_iter() {
                let variant_ident = variant.ident;
                let variant_str = variant_ident.to_string();
                matches.push(quote! { Self::#variant_ident => #variant_str, });
            }

            // Generate `ScoreDebug` implementation for provided enum.
            quote! {
                #[automatically_derived]
                impl #impl_generics mw_log::fmt::ScoreDebug for #ident #ty_generics {
                    fn fmt(&self, f: mw_log::fmt::Writer, spec: &mw_log::fmt::FormatSpec) -> mw_log::fmt::Result {
                        let v = match self {
                            #(#matches)*
                        };
                        f.write_str(v, spec)
                    }
                }
            }
        },

        Data::Union(_) => Error::new(proc_macro2::Span::call_site(), "`#[derive(ScoreDebug)] does not support unions`").into_compile_error(),
    }
    .into()
}
