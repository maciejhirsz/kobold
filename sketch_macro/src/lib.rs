// The `quote!` macro requires deep recursion.
#![recursion_limit = "196"]

extern crate proc_macro;

use proc_macro::{TokenStream, TokenTree, Delimiter};
use proc_macro2::TokenStream as QuoteTokens;
use quote::quote;

mod dom;
mod gen;

use gen::{Generator, Generate};

struct Parser {
    tokens: std::iter::Peekable<proc_macro::token_stream::IntoIter>,
}

impl Parser {
    fn new(tokens: TokenStream) -> Self {
        Self {
            tokens: tokens.into_iter().peekable()
        }
    }

    fn ident(&mut self) -> syn::Ident {
        syn::parse(self.next("identifier").into()).unwrap()
    }

    fn group(&mut self, expected: Delimiter) -> TokenStream {
        match self.next("(") {
            TokenTree::Group(group) if group.delimiter() == expected => {
                group.stream()
            },
            tt => panic!("Expected (, got {}", tt)
        }
    }

    fn next<D: std::fmt::Display>(&mut self, expected: D) -> TokenTree {
        match self.tokens.next() {
            Some(tt) => tt,
            None => panic!("Unexpected end of token stream, expected {}", expected),
        }
    }
}

#[proc_macro]
pub fn sketch(input: TokenStream) -> TokenStream {
    let mut parser = Parser::new(input);

    let name = parser.ident();
    let args = QuoteTokens::from(parser.group(Delimiter::Parenthesis));
    let body = parser.group(Delimiter::Brace);

    let dom = match dom::parse(body) {
        Ok(dom) => dom,
        Err(err) => return err.tokenize(),
    };

    // panic!("{:#?}", dom);

    let mut generator = Generator::new();

    let root = dom.generate(&mut generator);
    let (prerender, render, update) = generator.tokens();
    let fields = generator.fields();

    let tokens: TokenStream = (quote! {
        struct #name {
            root: Node,
            #(
                #fields: Node,
            )*
        }

        impl Rendered for #name {
            fn root(&self) -> &Node {
                &self.root
            }
        }

        impl #name {
            pub fn render(#args) -> Self {
                #prerender

                #name {
                    root: to_node(stdweb::js! {
                        #render

                        return #root;
                    }),
                    #(
                        #fields,
                    )*
                }
            }

            pub fn update(&self, #args) {
                #update
            }
        }
    }).into();

    // panic!("{}", tokens);

    tokens
}
