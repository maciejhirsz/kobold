// The `quote!` macro requires deep recursion.
#![recursion_limit = "196"]
#![warn(clippy::all, clippy::cast_possible_truncation, clippy::unused_self)]

extern crate proc_macro;

use std::cell::Cell;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as QuoteTokens;
use quote::quote;

mod dom;
mod gen;
mod parser;

use gen::Generator;
use parser::Parser;

thread_local! {
    static COUNT: Cell<usize> = Cell::new(0);
}

#[proc_macro]
pub fn html(body: TokenStream) -> TokenStream {
    COUNT.with(|count| count.set(count.get() + 1));

    let mut parser = Parser::new();

    let dom = match parser.parse(body) {
        Ok(dom) => dom,
        Err(err) => return err.tokenize(),
    };

    if dom.is_expression() && parser.fields.len() == 1 {
        let expr = parser.fields.pop().unwrap().expr;

        return quote! {
            {
                use ::kobold::{IntoHtml as _};

                (#expr).into_html()
            }
        }
        .into();
    }

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
                #name: (#expr).into_html(),
            }
        })
        .collect::<QuoteTokens>();

    let mut generator = Generator::new(fields.iter());

    let root = generator.generate(&dom).unwrap();
    let (js_fn_name, render) = generator.render_js(&root);

    let tokens: TokenStream = (quote! {
        {
            use ::kobold::{Html, Mountable, IntoHtml as _};
            use ::kobold::reexport::web_sys::Node;
            use ::kobold::reexport::wasm_bindgen::{self, JsValue, prelude::wasm_bindgen};

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
    })
    .into();

    // let count = COUNT.with(|c| c.get());

    // if count == 4 {
    //     panic!("{count}: {}", tokens);
    // }

    tokens
}
