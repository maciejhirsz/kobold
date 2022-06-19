// The `quote!` macro requires deep recursion.
#![recursion_limit = "196"]
#![warn(clippy::all, clippy::cast_possible_truncation, clippy::unused_self)]

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as QuoteTokens;
use quote::quote;

mod dom;
mod gen;
mod parser;

use gen::Generator;
use parser::Parser;

#[proc_macro]
pub fn html(body: TokenStream) -> TokenStream {
    let mut parser = Parser::new();

    let dom = match parser.parse(body) {
        Ok(dom) => dom,
        Err(err) => return err.tokenize(),
    };

    // panic!("{:#?}\n\n{:#?}", dom, parser.fields);

    let fields = &parser.fields;

    let generics = fields.iter().map(|field| &field.typ).collect::<Vec<_>>();
    let generics = &generics;

    let field_names = fields.iter().map(|field| &field.name).collect::<Vec<_>>();
    let field_names = &field_names;

    let field_defs = fields
        .iter()
        .map(|field| {
            let typ = &field.typ;
            let name = &field.name;

            quote! {
                #name: #typ,
            }
        })
        .collect::<QuoteTokens>();
    let field_defs = &field_defs;

    let field_declr = fields
        .iter()
        .map(|field| {
            let expr = &field.expr;
            let name = &field.name;

            quote! {
                #name: #expr,
            }
        })
        .collect::<QuoteTokens>();

    let mut generator = Generator::new(fields.iter());

    let root = generator.generate(&dom).unwrap();
    let (js_fn_name, render) = generator.render_js(&root);

    let tokens: TokenStream = (quote! {
        {
            use ::kobold::{Html, Node, Mountable, JsValue};
            use ::kobold::reexport::wasm_bindgen::{self, prelude::wasm_bindgen};

            #render

            struct Transient<#(#generics),*> {
                #field_defs
            }

            struct TransientProduct<#(#generics),*> {
                #field_defs
                node: Node,
            }

            impl<#(#generics: Html),*> Html for Transient<#(#generics),*> {
                type Product = TransientProduct<#(<#generics as Html>::Product),*>;

                fn build(self) -> Self::Product {
                    #(
                        let #field_names = self.#field_names.build();
                    )*
                    let node = #js_fn_name(#(#field_names.js()),*);

                    TransientProduct {
                        #(#field_names,)*
                        node,
                    }
                }

                fn update(self, p: &mut Self::Product) {
                    #(
                        self.#field_names.update(&mut p.#field_names);
                    )*
                }
            }

            impl<#(#generics),*> Mountable for TransientProduct<#(#generics),*>
            where
                Self: 'static,
            {
                fn js(&self) -> &JsValue {
                    &self.node
                }
            }

            Transient {
                #field_declr
            }
        }
    }).into();

    // panic!("{}", tokens);

    tokens
}
