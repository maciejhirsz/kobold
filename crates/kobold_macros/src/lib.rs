// The `quote!` macro requires deep recursion.
#![recursion_limit = "196"]
#![warn(clippy::all, clippy::cast_possible_truncation, clippy::unused_self)]

#[cfg(not(test))]
extern crate proc_macro;

#[cfg(test)]
extern crate proc_macro2 as proc_macro;

use proc_macro::{TokenStream, TokenTree};

mod branching;
mod component;
mod dom;
mod gen;
mod itertools;
mod parse;
mod syntax;
mod tokenize;

use parse::prelude::*;
use tokenize::prelude::*;

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

#[proc_macro_attribute]
pub fn component(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = unwrap_err!(component::args(args));

    unwrap_err!(component::component(args, input))
}

fn do_branching(input: TokenStream) -> TokenStream {
    let (input, count) = count_branches(input);

    let out = if count > 1 {
        let ident = format!("Branch{count}");
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
        if let TokenTree::Group(group) = &tt {
            let (_, subcount) = count_branches(group.stream());

            count += subcount;
        } else if tt.is("html") {
            out.write(tt);

            tt = match iter.next() {
                Some(tt) => {
                    if tt.is('!') {
                        count += 1;
                    }
                    tt
                }
                None => break,
            }
        }

        out.write(tt);
    }

    (out, count)
}

fn mark_branches(stream: TokenStream, branch_ty: &str, n: &mut u8) -> TokenStream {
    use proc_macro::Group;

    let mut out = TokenStream::new();
    let mut iter = stream.parse_stream();

    while let Some(tt) = iter.next() {
        if let TokenTree::Group(group) = tt {
            let stream = mark_branches(group.stream(), branch_ty, n);

            out.write(Group::new(group.delimiter(), stream));

            continue;
        } else if tt.is("html") {
            if let Some(bang) = iter.allow_consume('!') {
                let variant = [*n];
                let variant = std::str::from_utf8(&variant).unwrap();

                *n += 1;

                out.write(call(
                    format_args!("::kobold::branching::{branch_ty}::{variant}"),
                    (tt, bang, iter.next().unwrap()),
                ));

                continue;
            }
        }

        out.write(tt);
    }

    out
}

#[allow(clippy::let_and_return)]
#[proc_macro]
pub fn html(mut body: TokenStream) -> TokenStream {
    let mut iter = body.into_iter();

    let first = iter.next();

    body = TokenStream::new();
    body.extend(first.clone());
    body.extend(iter);

    if matches!(&first, Some(TokenTree::Ident(ident)) if ["match", "if"].contains(&&*ident.to_string()))
    {
        return do_branching(body);
    }

    // --

    let nodes = unwrap_err!(dom::parse(body.clone()));

    // panic!("{nodes:#?}");

    let transient = gen::generate(nodes);

    let out = transient.tokenize();

    // panic!("{out}");

    out
}
