use std::fmt::{self, Debug, Display, Write};
use std::ops::Deref;

use arrayvec::ArrayString;
use proc_macro::{Literal, TokenStream};

use crate::gen::Short;
use crate::itertools::IteratorExt;
use crate::tokenize::prelude::*;

// JS function name, capacity must fit a `Short`, a hash, and few underscores
pub type JsFnName = ArrayString<24>;

#[derive(Default, Debug)]
pub struct Transient {
    pub js: JsModule,
    pub js_type: Option<&'static str>,
    pub fields: Vec<Field>,
    pub els: Vec<Short>,
}

impl Transient {
    fn is_const(&self) -> bool {
        let jsfn = match self.js.functions.get(0) {
            Some(fun) => fun,
            None => return false,
        };

        self.fields.is_empty()
            && self.els.len() == 1
            && self.js.attr_constructors.len() == 0
            && self.js.functions.len() == 1
            && jsfn.constructor == "Element::new"
            && jsfn.args.is_empty()
    }

    fn tokenize_const(self, stream: &mut TokenStream) {
        let JsFunction { name, .. } = self.js.functions[0];

        block((
            "use ::kobold::reexport::wasm_bindgen;",
            self.js,
            format_args!("::kobold::util::Static({name})"),
        ))
        .tokenize_in(stream)
    }
}

impl Tokenize for Transient {
    fn tokenize_in(mut self, stream: &mut TokenStream) {
        if self.is_const() {
            self.tokenize_const(stream);
            return;
        }

        if self.els.is_empty() {
            return match self.fields.remove(0) {
                Field::View { value, .. } | Field::Attribute { value, .. } => {
                    value.tokenize_in(stream)
                }
            };
        }

        let abi_lifetime = if self.fields.iter().any(Field::borrows) {
            "'abi,"
        } else {
            ""
        };
        let js_type = self.js_type.unwrap_or("Node");

        let mut generics = String::new();
        let mut generics_product = String::new();
        let mut bounds = String::new();
        let mut build = String::new();
        let mut update = String::new();
        let mut declare = String::new();
        let mut vars = String::new();

        for field in self.fields.iter() {
            let typ = field.make_type();

            let _ = write!(generics, "{typ},");
            let _ = write!(generics_product, "{typ}::Product,");

            field.bounds(&mut bounds);
            field.build(&mut build);
            field.update(&mut update);
            field.declare(&mut declare);
            field.var(&mut vars);
        }

        let mut declare_els = String::new();

        for (jsfn, el) in self.js.functions.iter().zip(self.els) {
            let JsFunction {
                name,
                constructor,
                args,
            } = jsfn;

            let _ = write!(declare_els, "{el}: ::kobold::dom::Element,");

            let args = args
                .iter()
                .map(|a| {
                    let mut temp = ArrayString::<8>::new();
                    let _ = write!(temp, "{}.js()", a.name);
                    temp
                })
                .join(",");

            let _ = write!(
                build,
                "let {el} = ::kobold::dom::{constructor}({name}({args}));"
            );
            let _ = write!(vars, "{el},");
        }

        block((
            "\
                use ::kobold::{Mountable as _};\
                use ::kobold::reexport::wasm_bindgen;\
                ",
            self.js,
            format_args!(
                "\
                    struct TransientProduct <{generics}> {{\
                        {declare}\
                        {declare_els}\
                    }}\
                    \
                    impl<{generics}> ::kobold::Mountable for TransientProduct<{generics}>\
                    where \
                        Self: 'static,\
                    {{\
                        type Js = ::kobold::reexport::web_sys::{js_type};\
                        \
                        fn el(&self) -> &::kobold::dom::Element {{\
                            &self.e0\
                        }}\
                    }}\
                    \
                    struct Transient <{generics}> {{\
                        {declare}\
                    }}\
                    \
                    impl<{abi_lifetime}{generics}> ::kobold::View for Transient<{generics}>\
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
                    Transient\
                    "
            ),
            block(each(self.fields.iter().map(Field::invoke))),
        ))
        .tokenize_in(stream)
    }
}

#[derive(Default)]
pub struct JsModule {
    pub attr_constructors: Vec<JsAttrConstructor>,
    pub functions: Vec<JsFunction>,
    pub code: String,
}

impl Tokenize for JsModule {
    fn tokenize_in(self, stream: &mut TokenStream) {
        if self.functions.is_empty() {
            return;
        }

        stream.write((
            '#',
            group(
                '[',
                (
                    "wasm_bindgen::prelude::wasm_bindgen",
                    group('(', ("inline_js = ", string(&self.code))),
                ),
            ),
            "extern \"C\"",
            block((each(self.attr_constructors), each(self.functions))),
        ))
    }
}

impl Debug for JsModule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(&self.functions, f)?;

        write!(f, "\ncode: ({})", self.code)
    }
}

pub struct JsAttrConstructor(pub JsFnName);

impl Tokenize for JsAttrConstructor {
    fn tokenize_in(self, stream: &mut TokenStream) {
        let name = self.0;
        stream.write(format_args!(
            "fn {name}() -> ::kobold::reexport::web_sys::Node;"
        ));
    }
}

#[derive(Debug)]
pub struct JsFunction {
    pub name: JsFnName,
    pub constructor: &'static str,
    pub args: Vec<JsArgument>,
}

impl Tokenize for JsFunction {
    fn tokenize_in(self, stream: &mut TokenStream) {
        let name = self.name;

        stream.write((
            call(format_args!("fn {name}"), each(self.args)),
            "-> ::kobold::reexport::web_sys::Node;",
        ));
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
    View {
        name: Short,
        value: TokenStream,
    },
    Attribute {
        name: Short,
        el: Short,
        abi: Abi,
        value: TokenStream,
    },
}

pub enum Abi {
    Owned(&'static str),
    Borrowed(&'static str),
}

impl Deref for Abi {
    type Target = str;

    fn deref(&self) -> &str {
        match self {
            Abi::Owned(abi) => abi,
            Abi::Borrowed(abi) => abi,
        }
    }
}

impl Debug for Field {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Field::View { name, value } => {
                write!(f, "{name} <View>: {value}")
            }
            Field::Attribute {
                name,
                el,
                abi,
                value,
            } => {
                let abi = abi.deref();

                write!(f, "{name} <Attribute({abi} -> {el})>: {value}")
            }
        }
    }
}

impl Field {
    fn borrows(&self) -> bool {
        matches!(
            self,
            Field::Attribute {
                abi: Abi::Borrowed(_),
                ..
            }
        )
    }

    fn name_value(&self) -> (&Short, &TokenStream) {
        match self {
            Field::View { name, value } | Field::Attribute { name, value, .. } => (name, value),
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
            Field::View { name, .. } => {
                let mut typ = *name;
                typ.make_ascii_uppercase();

                let _ = write!(buf, "{typ}: ::kobold::View,");
            }
            Field::Attribute { name, abi, .. } => {
                let mut typ = *name;
                typ.make_ascii_uppercase();

                let abi = abi.deref();

                let _ = write!(buf, "{typ}: ::kobold::attribute::Attribute<Abi = {abi}>,");
            }
        }
    }

    fn declare(&self, buf: &mut String) {
        let (name, _) = self.name_value();
        let typ = self.make_type();

        let _ = write!(buf, "{name}: {typ},");
    }

    fn build(&self, buf: &mut String) {
        match self {
            Field::View { name, .. } => {
                let _ = write!(buf, "let {name} = self.{name}.build();");
            }
            Field::Attribute { name, .. } => {
                let _ = write!(buf, "let {name} = self.{name};");
            }
        }
    }

    fn var(&self, buf: &mut String) {
        match self {
            Field::View { name, .. } => {
                let _ = write!(buf, "{name},");
            }
            Field::Attribute { name, .. } => {
                let _ = write!(buf, "{name}: {name}.build(),");
            }
        }
    }

    fn update(&self, buf: &mut String) {
        match self {
            Field::View { name, .. } => {
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
