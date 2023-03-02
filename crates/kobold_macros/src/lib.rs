// The `quote!` macro requires deep recursion.
#![recursion_limit = "196"]
#![warn(clippy::all, clippy::cast_possible_truncation, clippy::unused_self)]

#[cfg(not(test))]
extern crate proc_macro;

#[cfg(test)]
extern crate proc_macro2 as proc_macro;

use proc_macro::TokenStream;

mod branching;
mod dom;
mod fn_component;
mod gen;
mod itertools;
mod parse;
mod syntax;
mod tokenize;

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
pub fn component(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = unwrap_err!(fn_component::args(args));

    let out = unwrap_err!(fn_component::component(args, input));

    // panic!("{out}");

    out
}

#[allow(clippy::let_and_return)]
#[proc_macro]
pub fn html(body: TokenStream) -> TokenStream {
    let nodes = unwrap_err!(dom::parse(body));

    // panic!("{nodes:#?}");

    let transient = gen::generate(nodes);

    let out = transient.tokenize();

    // panic!("{out}");

    out
}
