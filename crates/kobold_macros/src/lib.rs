// The `quote!` macro requires deep recursion.
#![recursion_limit = "196"]
#![warn(clippy::all, clippy::cast_possible_truncation, clippy::unused_self)]

extern crate proc_macro;

use proc_macro::{Delimiter, Ident, TokenStream, TokenTree};
use proc_macro2::TokenStream as QuoteTokens;
use quote::quote;

mod dom;
mod dom2;
mod gen;
mod parse;
mod parser;
mod syntax;

use dom::FieldKind;
use gen::Generator;
use parse::prelude::*;
use parser::{into_quote, Parser};

macro_rules! unwrap_err {
    ($expr:expr) => {
        match $expr {
            Ok(dom) => dom,
            Err(err) => return err.tokenize(),
        }
    };
}

#[proc_macro_attribute]
pub fn branching(_: TokenStream, input: TokenStream) -> TokenStream {
    do_branching(input)
}

fn do_branching(input: TokenStream) -> TokenStream {
    use proc_macro2::{Ident, Span};

    let (input, count) = count_branches(input);

    let out = if count > 1 {
        let ident = Ident::new(&format!("Branch{count}"), Span::call_site());
        let mut variant = b'A';

        mark_branches(input, &ident, &mut variant)
    } else {
        input
    };

    out
}

fn count_branches(stream: TokenStream) -> (TokenStream, usize) {
    let mut out = TokenStream::new();
    let mut iter = stream.into_iter();
    let mut count = 0;

    while let Some(mut tt) = iter.next() {
        match &tt {
            TokenTree::Group(group) => {
                let (_, subcount) = count_branches(group.stream());

                count += subcount;
            }
            TokenTree::Ident(ident) if ident.to_string() == "html" => {
                out.extend([tt]);

                tt = match iter.next() {
                    Some(TokenTree::Punct(punct)) if punct.as_char() == '!' => {
                        count += 1;
                        punct.into()
                    }
                    Some(tt) => tt,
                    None => break,
                }
            }
            _ => (),
        }

        out.extend([tt]);
    }

    (out, count)
}

fn mark_branches(stream: TokenStream, branch_ty: &proc_macro2::Ident, n: &mut u8) -> TokenStream {
    use proc_macro::Group;
    use proc_macro2::{Ident, Span};

    let mut out = TokenStream::new();
    let mut iter = stream.into_iter().peekable();

    while let Some(mut tt) = iter.next() {
        match tt {
            TokenTree::Group(group) => {
                let delimiter = group.delimiter();
                let stream = mark_branches(group.stream(), branch_ty, n);

                tt = Group::new(delimiter, stream).into();
            }
            TokenTree::Ident(ident) if ident.to_string() == "html" => {
                tt = ident.into();

                match iter.peek() {
                    Some(TokenTree::Punct(punct)) if punct.as_char() == '!' => {
                        let mut branch = TokenStream::new();

                        branch.extend([tt, iter.next().unwrap(), iter.next().unwrap()]);

                        let variant = [*n];
                        let variant = std::str::from_utf8(&variant).unwrap();
                        let variant = Ident::new(variant, Span::call_site());

                        *n += 1;

                        out.extend::<TokenStream>(
                            quote!(::kobold::branching::#branch_ty::#variant).into(),
                        );

                        tt = Group::new(Delimiter::Parenthesis, branch).into();
                    }
                    _ => (),
                }
            }
            _ => (),
        }

        out.extend([tt]);
    }

    out
}

#[proc_macro]
pub fn html(mut body: TokenStream) -> TokenStream {
    // let shallow = dom2::shallow_parse(body.clone());

    // panic!("{shallow:#?}");

    let mut iter = body.into_iter();

    let first = iter.next();

    body = TokenStream::new();
    body.extend(first.clone());
    body.extend(iter);

    if matches!(&first, Some(TokenTree::Ident(ident)) if ["match", "if"].contains(&&*ident.to_string()))
    {
        return do_branching(body);
    }

    let mut parser = Parser::new();

    let dom = unwrap_err!(parser.parse(body));

    if dom.is_expression() && parser.fields.len() == 1 {
        let expr = parser.fields.pop().unwrap().expr;

        return expr.into();
    }

    // panic!("{:#?}\n\n{:#?}", dom, parser.fields);

    let fields = &parser.fields;

    let mut generics = Vec::new();
    let mut generics_with_bounds = Vec::new();
    let mut update_calls = Vec::new();

    for field in fields.iter() {
        let typ = &field.typ;
        let name = &field.name;

        generics.push(typ);

        match &field.kind {
            FieldKind::AttrHoisted(abi) => {
                generics_with_bounds.push(quote! {
                    #typ: ::kobold::attribute::Attribute,
                    #typ::Product: ::kobold::attribute::AttributeProduct<Abi = #abi>
                });

                update_calls.push(quote! {
                    self.#name.update(&mut p.#name, &p.el);
                })
            }
            _ => {
                generics_with_bounds.push(quote! { #typ: ::kobold::Html });

                update_calls.push(quote! {
                    self.#name.update(&mut p.#name);
                })
            }
        }
    }

    let generics = &generics[..];

    let field_names = fields.iter().map(|field| &field.name).collect::<Vec<_>>();
    let field_names = &field_names[..];

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

    let el = if dom.is_fragment() {
        quote! { unsafe { ::kobold::dom::Element::new_fragment_raw(#js_fn_name(#(#field_names.js()),*)) } }
    } else {
        quote! { ::kobold::dom::Element::new(#js_fn_name(#(#field_names.js()),*)) }
    };

    let tokens: TokenStream = (quote! {
        {
            use ::kobold::{Mountable as _};
            use ::kobold::attribute::{AttributeProduct as _};
            use ::kobold::reexport::wasm_bindgen;

            #render

            struct Transient<#(#generics),*> {
                #field_defs
            }

            struct TransientProduct<#(#generics),*> {
                #field_defs
                el: ::kobold::dom::Element,
            }

            impl<#(#generics),*> ::kobold::Html for Transient<#(#generics),*>
            where
                #(#generics_with_bounds),*
            {
                type Product = TransientProduct<#(#generics::Product),*>;

                fn build(self) -> Self::Product {
                    #(
                        let #field_names = self.#field_names.build();
                    )*
                    let el = #el;

                    TransientProduct {
                        #(#field_names,)*
                        el,
                    }
                }

                fn update(self, p: &mut Self::Product) {
                    #(#update_calls)*
                }
            }

            impl<#(#generics),*> ::kobold::Mountable for TransientProduct<#(#generics),*>
            where
                Self: 'static,
            {
                fn el(&self) -> &::kobold::dom::Element {
                    &self.el
                }
            }

            Transient {
                #field_declr
            }
        }
    })
    .into();

    tokens
}

#[proc_macro_derive(Stateful)]
pub fn stateful(tokens: TokenStream) -> TokenStream {
    unwrap_err!(do_stateful(tokens))
}

fn do_stateful(tokens: TokenStream) -> Result<TokenStream, ParseError> {
    let mut parser = tokens.into_iter().peekable();

    let _: Ident = parser.parse()?;
    let name: Ident = parser.parse()?;
    let name = into_quote(name);

    let tokens = quote! {
        impl ::kobold::stateful::Stateful for #name
        where
            Self: PartialEq,
        {
            type State = Self;

            fn init(self) -> Self::State {
                self
            }

            fn update(self, state: &mut Self::State) -> ::kobold::stateful::ShouldRender {
                if self != *state {
                    *state = self;
                    ::kobold::stateful::ShouldRender::Yes
                } else {
                    ::kobold::stateful::ShouldRender::No
                }
            }
        }
    }
    .into();

    Ok(tokens)
}
