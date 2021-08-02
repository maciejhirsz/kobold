// The `quote!` macro requires deep recursion.
#![recursion_limit = "196"]

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;

mod dom;
mod gen;

use gen::Generator;

#[proc_macro]
pub fn html(body: TokenStream) -> TokenStream {
    let mut parser = dom::Parser::new();

    let dom = match parser.parse(body) {
        Ok(dom) => dom,
        Err(err) => return err.tokenize(),
    };

    // panic!("{:#?}", dom);

    let generics = parser.fields.iter().map(|field| &field.typ).collect::<Vec<_>>();
    let generics = &generics;

    let generic_bounds = parser.fields.iter().map(|field| {
        let typ = &field.typ;

        if field.iterator {
            quote! { #typ: IntoIterator, <#typ as IntoIterator>::Item: Html, Vec<<#typ as IntoIterator>::Item>: Html, }
        } else {
            quote! { #typ: Html, }
        }
    });

    let update_bounds = parser.fields.iter().map(|field| {
        let typ = &field.typ;

        if field.iterator {
            quote! { #typ: IntoIterator, <#typ as IntoIterator>::Item: Html, }
        } else {
            quote! { #typ: Html, }
        }
    });

    let rendered_types = parser.fields.iter().map(|field| {
        let typ = &field.typ;

        if field.iterator {
            quote! { <Vec<<#typ as IntoIterator>::Item> as Html>::Rendered }
        } else {
            quote! { <#typ as Html>::Rendered }
        }
    }).collect::<Vec<_>>();
    let rendered_types = &rendered_types;

    let field_names = parser.fields.iter().map(|field| &field.name).collect::<Vec<_>>();
    let field_names = &field_names;

    let field_defs = parser.fields.iter().map(|field| {
        let typ = &field.typ;
        let name = &field.name;

        quote! {
            #name: #typ,
        }
    }).collect::<Vec<_>>();
    let field_defs = &field_defs;

    let field_declr = parser.fields.iter().map(|field| {
        let expr = &field.expr;
        let name = &field.name;

        quote! {
            #name: #expr,
        }
    });

    let field_renders = parser.fields.iter().map(|field| {
        let name = &field.name;

        if field.iterator {
            quote! {
                let #name = self.#name.into_iter().collect::<Vec<_>>().render();
            }
        } else {
            quote! {
                let #name = self.#name.render();
            }
        }
    });

    let mut generator = Generator::new();

    let root = generator.generate(&dom).unwrap();
    let (js_fn_name, render) = generator.render_js(&root);

    let tokens: TokenStream = (quote! {
        {
            use ::sketch::{Html, Update, Mountable, Node};
            use ::sketch::reexport::wasm_bindgen::{self, prelude::wasm_bindgen};

            #render

            struct TransientHtml<#(#generics),*> {
                #(#field_defs)*
            }

            struct TransientRendered<#(#generics),*> {
                #(#field_defs)*
                node: Node,
            }

            impl<#(#generics),*> Html for TransientHtml<#(#generics),*>
            where
                #(#generic_bounds)*
            {
                type Rendered = TransientRendered<#(#rendered_types),*>;

                fn render(self) -> Self::Rendered {
                    #(#field_renders)*
                    let node = unsafe {
                        #js_fn_name(#(
                            #field_names.node()
                        ),*)
                    };

                    TransientRendered {
                        #(#field_names,)*
                        node,
                    }
                }
            }

            impl<#(#generics),*> Mountable for TransientRendered<#(#generics),*> {
                fn node(&self) -> &Node {
                    &self.node
                }
            }

            impl<#(#generics),*> Update<TransientHtml<#(#generics),*>> for TransientRendered<#(#rendered_types),*>
            where
                #(#update_bounds)*
            {
                fn update(&mut self, new: TransientHtml<#(#generics),*>) {
                    #(
                        self.#field_names.update(new.#field_names);
                    )*
                }
            }

            TransientHtml {
                #(#field_declr)*
            }
        }
    }).into();

    // panic!("{}", tokens);

    tokens
}
