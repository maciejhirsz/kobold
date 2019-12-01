use proc_macro2::TokenStream as QuoteTokens;
use quote::quote;
use crate::gen::{Generator, Generate};

mod parser;

pub use parser::parse;

#[derive(Debug)]
pub enum Node {
    Element(Element),
    Text(QuoteTokens),
    Expression(QuoteTokens),
    Fragment(Vec<Node>),
}

#[derive(Debug)]
pub struct Element {
    tag: String,
    props: Vec<(String, QuoteTokens)>,
    children: Vec<Node>,
}

impl Generate for Node {
    fn generate(&self, gen: &mut Generator) -> QuoteTokens {
        match self {
            Node::Element(element) => element.generate(gen),
            Node::Text(text) => {
                quote! {
                    document.createTextNode(#text)
                }
            },
            Node::Expression(expr) => {
                gen.add_field(expr)
            },
            Node::Fragment(nodes) => {
                let el = gen.var();

                gen.extend(quote! {
                    let #el = document.createDocumentFragment();
                });

                append(gen, el, nodes)
            },
        }
    }
}

impl Generate for Element {
    fn generate(&self, gen: &mut Generator) -> QuoteTokens {
        let tag = &self.tag;
        let el = gen.var();

        gen.extend(quote! {
            let #el = document.createElement(#tag);
        });

        for (key, value) in self.props.iter() {
            gen.extend(match key.as_str() {
                "class" => quote! {
                    #el.className = #value;
                },
                "style" => quote! {
                    #el.style = #value;
                },
                key if key.starts_with("on") => {
                    let event = &key[2..];

                    quote!{
                        #el.addEventListener(#event, #value);
                    }
                },
                key => quote! {
                    #el.setAttribute(#key, #value);
                },
            });
        }

        append(gen, el, &self.children)
    }
}

fn append(gen: &mut Generator, el: QuoteTokens, children: &[Node]) -> QuoteTokens {
    let children = children.iter().map(|child| {
        match child {
            Node::Text(text) => text.clone(),
            node => gen.add(node),
        }
    });

    let tokens = quote! {
        #el.append(#(#children),*);
    };

    gen.extend(tokens);

    el
}
