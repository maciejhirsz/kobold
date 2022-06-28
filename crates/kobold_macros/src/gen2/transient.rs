use std::fmt::{self, Debug, Display};

use arrayvec::ArrayString;
use proc_macro::{Literal, TokenStream};

use crate::gen2::Short;
use crate::tokenize::prelude::*;

// JS function name, capacity must fit a `Short`, a hash, and few underscores
pub type JsFnName = ArrayString<24>;

#[derive(Default, Debug)]
pub struct Transient {
    pub js: JsModule,
    pub fields: Vec<Field>,
    pub els: Vec<Short>,
}

#[derive(Default)]
pub struct JsModule {
    pub functions: Vec<JsFunction>,
    pub code: String,
}

impl Debug for JsModule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(&self.functions, f)?;

        write!(f, "\ncode: ({})", self.code)
    }
}

#[derive(Debug)]
pub struct JsFunction {
    pub name: JsFnName,
    pub args: Vec<Short>,
}

pub enum Field {
    Html {
        name: Short,
        value: TokenStream,
    },
    Attribute {
        name: Short,
        el: Short,
        abi: &'static str,
        value: TokenStream,
    },
}

impl Debug for Field {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Field::Html { name, value } => {
                write!(f, "{name} <Html>: {value}")
            }
            Field::Attribute {
                name,
                el,
                abi,
                value,
            } => {
                write!(f, "{name} <Attribute({abi} -> {el})>: {value}")
            }
        }
    }
}

impl Field {
    fn to_bounds(&self) -> TokenStream {
        match self {
            Field::Html { name, .. } => {
                let mut typ = *name;
                typ.make_ascii_uppercase();

                format_args!("{typ}: ::kobold:Html,").tokenize()
            }
            Field::Attribute { name, abi, .. } => {
                let mut typ = *name;
                typ.make_ascii_uppercase();

                format_args!(
                    "{typ}: ::kobold:Html,\
                    {typ}::Product: ::kobold::attribute::AttributeProduct<Abi = {abi}>,"
                )
                .tokenize()
            }
        }
    }

    fn build(&self) -> TokenStream {
        let name = match self {
            Field::Html { name, .. } | Field::Attribute { name, .. } => name,
        };

        format_args!("let {name} = self.{name}.build();").tokenize()
    }

    fn update(&self) -> TokenStream {
        match self {
            Field::Html { name, .. } => {
                format_args!("self.{name}.update(&mut p.{name});").tokenize()
            }
            Field::Attribute { name, el, .. } => {
                format_args!("self.{name}.update(&mut p.{name}, &p.{el});").tokenize()
            }
        }
    }
}

pub struct JsString(pub Literal);

impl Display for JsString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let stringified = self.0.to_string();

        match stringified.chars().next() {
            // Take the string verbatim
            Some('"' | '\'') => f.write_str(&stringified),
            // Add quotes
            _ => write!(f, "\"{stringified}\""),
        }
    }
}
