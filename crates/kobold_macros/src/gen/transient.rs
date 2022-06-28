use std::fmt::{self, Debug, Display, Write};

use arrayvec::ArrayString;
use proc_macro::{Literal, TokenStream};

use crate::gen::Short;
use crate::tokenize::prelude::*;

// JS function name, capacity must fit a `Short`, a hash, and few underscores
pub type JsFnName = ArrayString<24>;

#[derive(Default, Debug)]
pub struct Transient {
    pub js: JsModule,
    pub fields: Vec<Field>,
    pub els: Vec<Short>,
}

impl Tokenize for Transient {
    fn tokenize(mut self) -> TokenStream {
        if self.els.is_empty() {
            return match self.fields.remove(0) {
                Field::Html { value, .. } | Field::Attribute { value, .. } => value,
            };
        }

        let mut generics = String::new();
        let mut generics_product = String::new();
        let mut bounds = String::new();
        let mut build = String::new();
        let mut update = String::new();
        let mut declare = String::new();
        let mut vars = String::new();

        for field in self.fields.iter() {
            let (name, _) = field.name_value();
            let typ = field.make_type();

            let _ = write!(generics, "{typ},");
            let _ = write!(generics_product, "{typ}::Product,");

            field.bounds(&mut bounds);
            field.build(&mut build);
            field.update(&mut update);
            field.declare(&mut declare);

            let _ = write!(vars, "{name},");
        }

        let mut declare_els = String::new();

        for (jsfn, el) in self.js.functions.iter().zip(self.els) {
            let _ = write!(declare_els, "{el}: ::kobold::dom::Element,");
            let _ = write!(build, "let {el} = ::kobold::dom::Element::new({}(", jsfn.name);

            for arg in jsfn.args.iter() {
                let _ = write!(build, "{}.js(),", arg.name);
            }
            build.push_str("));");

            let _ = write!(vars, "{el},");
        }

        group(
            '{',
            (
                "\
                use ::kobold::{Mountable as _};\
                use ::kobold::attribute::{AttributeProduct as _};\
                use ::kobold::reexport::wasm_bindgen;\
                ",
                self.js,
                format_args!(
                    "\
                    struct Transient <{generics}> {{\
                        {declare}\
                    }}\
                    \
                    struct TransientProduct <{generics}> {{\
                        {declare}\
                        {declare_els}\
                    }}\
                    \
                    impl<{generics}> ::kobold::Html for Transient<{generics}>\
                    where \
                        {bounds}\
                    {{\
                        type Product = TransientProduct<{generics_product}>;\
                        \
                        fn build(self) -> Self::Product {{\
                            {build}\
                            \
                            TransientProduct {{\
                                {vars}\
                            }}\
                        }}\
                        \
                        fn update(self, p: &mut Self::Product) {{\
                            {update}\
                        }}\
                    }}\
                    \
                    impl<{generics}> ::kobold::Mountable for TransientProduct<{generics}>\
                    where \
                        Self: 'static,\
                    {{\
                        fn el(&self) -> &::kobold::dom::Element {{\
                            &self.e0\
                        }}\
                    }}\
                    \
                    Transient\
                    "
                ),
                group('{', each(self.fields.iter().map(Field::invoke))),
            ),
        )
        .tokenize()
    }
}

#[derive(Default)]
pub struct JsModule {
    pub functions: Vec<JsFunction>,
    pub code: String,
}

impl Tokenize for JsModule {
    fn tokenize(self) -> TokenStream {
        if self.functions.is_empty() {
            return TokenStream::new();
        }

        (
            '#',
            group(
                '[',
                (
                    "wasm_bindgen::prelude::wasm_bindgen",
                    group('(', ("inline_js = ", string(&self.code))),
                ),
            ),
            "extern \"C\"",
            group('{', each(self.functions)),
        )
            .tokenize()
    }
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
    pub args: Vec<JsArgument>,
}

impl Tokenize for JsFunction {
    fn tokenize(self) -> TokenStream {
        let name = self.name;

        (
            format_args!("fn {name}"),
            group('(', each(self.args)),
            "-> ::kobold::reexport::web_sys::Node;",
        )
            .tokenize()
    }
}

#[derive(Debug)]
pub struct JsArgument {
    pub name: Short,
    pub abi: &'static str,
}

impl JsArgument {
    pub fn new(name: Short) -> Self {
        JsArgument {
            name,
            abi: "&wasm_bindgen::JsValue",
        }
    }

    pub fn with_abi(name: Short, abi: &'static str) -> Self {
        JsArgument { name, abi }
    }
}

impl Tokenize for JsArgument {
    fn tokenize(self) -> TokenStream {
        (ident(&self.name), ':', self.abi, ',').tokenize()
    }

    fn tokenize_in(self, stream: &mut TokenStream) {
        (ident(&self.name), ':', self.abi, ',').tokenize_in(stream)
    }
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
    fn name_value(&self) -> (&Short, &TokenStream) {
        match self {
            Field::Html { name, value } | Field::Attribute { name, value, .. } => (name, value),
        }
    }

    fn make_type(&self) -> Short {
        let (name, _) = self.name_value();

        let mut typ = *name;
        typ.make_ascii_uppercase();
        typ
    }

    fn bounds(&self, buf: &mut String) {
        match self {
            Field::Html { name, .. } => {
                let mut typ = *name;
                typ.make_ascii_uppercase();

                let _ = write!(buf, "{typ}: ::kobold::Html,");
            }
            Field::Attribute { name, abi, .. } => {
                let mut typ = *name;
                typ.make_ascii_uppercase();

                let _ = write!(
                    buf,
                    "{typ}: ::kobold::attribute::Attribute,\
                    {typ}::Product: ::kobold::attribute::AttributeProduct<Abi = {abi}>,"
                );
            }
        }
    }

    fn declare(&self, buf: &mut String) {
        let (name, _) = self.name_value();
        let typ = self.make_type();

        let _ = write!(buf, "{name}: {typ},");
    }

    fn build(&self, buf: &mut String) {
        let (name, _) = self.name_value();

        let _ = write!(buf, "let {name} = self.{name}.build();");
    }

    fn update(&self, buf: &mut String) {
        match self {
            Field::Html { name, .. } => {
                let _ = write!(buf, "self.{name}.update(&mut p.{name});");
            }
            Field::Attribute { name, el, .. } => {
                let _ = write!(buf, "self.{name}.update(&mut p.{name}, &p.{el});");
            }
        }
    }

    fn invoke(&self) -> impl Tokenize {
        let (name, value) = self.name_value();

        (ident(name), ':', value.clone(), ',')
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
