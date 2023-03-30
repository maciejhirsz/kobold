// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::fmt::{self, Debug, Display, Write};

use arrayvec::ArrayString;
use proc_macro::{Ident, Literal, Span, TokenStream};

use crate::gen::element::{Attr, InlineAbi};
use crate::gen::Short;
use crate::itertools::IteratorExt;
use crate::tokenize::prelude::*;

// JS function name, capacity must fit a `Short`, a hash, and few underscores
pub type JsFnName = ArrayString<24>;

#[derive(Default, Debug)]
pub struct Transient {
    pub js: JsModule,
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
            && self.js.functions.len() == 1
            && !matches!(jsfn.anchor, Anchor::Fragment)
            && jsfn.args.is_empty()
    }

    fn tokenize_const(self, stream: &mut TokenStream) {
        let JsFunction { name, .. } = self.js.functions[0];

        block((
            "use ::kobold::reexport::wasm_bindgen;",
            self.js,
            format_args!("::kobold::internal::Precompiled({name})"),
        ))
        .tokenize_in(stream)
    }

    fn transient_signature(&self) -> TokenStream {
        let mut generics = String::new();
        let mut declare = String::new();

        for field in self.fields.iter() {
            let typ = field.make_type();

            field.declare(&mut declare);

            let _ = write!(generics, "{typ},");
        }

        let mut bounds = "where".tokenize();

        for field in self.fields.iter() {
            field.bounds(&mut bounds);
        }

        (
            format_args!("struct Transient <{generics}>"),
            bounds.clone(),
            block(declare.as_str()),
            format_args!("impl<{generics}> ::kobold::View for Transient<{generics}>"),
            bounds,
        )
            .tokenize()
    }
}

impl Tokenize for Transient {
    fn tokenize_in(mut self, stream: &mut TokenStream) {
        if self.is_const() {
            self.tokenize_const(stream);
            return;
        }

        let transient_signature = self.transient_signature();

        if self.els.is_empty() {
            return self.fields.remove(0).value.tokenize_in(stream);
        }

        let mut generics = String::new();

        let mut build = String::new();
        let mut update = String::new();
        let mut declare = String::new();
        let mut vars = String::new();

        let mut product_declare = String::new();
        let mut product_generics = String::new();
        let mut product_generics_binds = String::new();

        for field in self.fields.iter() {
            let typ = field.make_type();

            let _ = write!(generics, "{typ},");

            field.build(&mut build);
            field.update(&mut update);
            field.declare(&mut declare);
            field.var(&mut vars);

            match field.kind {
                FieldKind::StaticView => (),
                FieldKind::View | FieldKind::Attribute { .. } => {
                    let _ = write!(product_generics, "{typ},");
                    let _ = write!(product_generics_binds, "{typ}::Product,");
                    field.declare(&mut product_declare);
                }
                FieldKind::Event { .. } => {
                    let _ = write!(product_generics, "{typ},");
                    let _ = write!(
                        product_generics_binds,
                        "::kobold::event::ListenerProduct<{typ}>,"
                    );
                    field.declare(&mut product_declare);
                }
            }
        }

        let mut declare_els = String::new();

        for (jsfn, el) in self.js.functions.iter().zip(self.els) {
            let JsFunction { name, anchor, args } = jsfn;

            let anchor_typ = anchor.as_type();

            let _ = write!(declare_els, "{el}: {anchor_typ},");

            let args = args
                .iter()
                .map(|a| {
                    let mut temp = ArrayString::<24>::new();
                    let name = a.name;
                    let _ = match a.abi.map(InlineAbi::method) {
                        Some(method) => write!(temp, "self.{name}{method}"),
                        None => write!(temp, "{name}.js()"),
                    };
                    temp
                })
                .join(",");

            let _ = write!(build, "let {el}: {anchor_typ} = {name}({args}).into();");
            let _ = write!(vars, "{el},");
        }
        let anchor = &self.js.functions.last().unwrap().anchor;

        let anchor_type = anchor.as_type();
        let anchor_js_type = anchor.as_js_type();

        block((
            (
                "\
                use ::kobold::dom::{Mountable as _};\
                use ::kobold::reexport::wasm_bindgen;\
                ",
                self.js,
                format_args!(
                    "\
                    struct TransientProduct <{product_generics}> {{\
                        {product_declare}\
                        {declare_els}\
                    }}\
                    \
                    impl<{product_generics}> ::kobold::dom::Anchor for TransientProduct<{product_generics}>\
                    where \
                        Self: 'static,\
                    ",
                ),
            ),
            block((
                "type Js = ::kobold::reexport::web_sys::",
                anchor_js_type,
                ";",
                format_args!("\
                    type Target = {anchor_type};

                    fn anchor(&self) -> &Self::Target {{\
                        &self.e0\
                    }}\
                "),
            )),
            transient_signature,
            format_args!(
                "\
                {{\
                    type Product = TransientProduct<{product_generics_binds}>;\
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
            block(each(self.functions)),
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
    pub anchor: Anchor,
    pub args: Vec<JsArgument>,
}

#[derive(Debug, Clone)]
pub enum Anchor {
    Element(Ident),
    Node,
    Fragment,
}

impl Anchor {
    fn as_js_type(&self) -> Ident {
        match self {
            Anchor::Element(typ) => typ.clone(),
            Anchor::Node | Anchor::Fragment => ident("Node"),
        }
    }

    fn as_type(&self) -> &'static str {
        match self {
            Anchor::Element(_) | Anchor::Node => "::kobold::reexport::web_sys::Node",
            Anchor::Fragment => "::kobold::dom::Fragment",
        }
    }
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
    pub abi: Option<InlineAbi>,
}

impl JsArgument {
    pub fn new(name: Short) -> Self {
        JsArgument { name, abi: None }
    }

    pub fn with_abi(name: Short, abi: InlineAbi) -> Self {
        JsArgument {
            name,
            abi: Some(abi),
        }
    }
}

impl Tokenize for JsArgument {
    fn tokenize_in(self, stream: &mut TokenStream) {
        let abi = self
            .abi
            .map(InlineAbi::abi)
            .unwrap_or("&wasm_bindgen::JsValue");

        (ident(&self.name), ':', abi, ',').tokenize_in(stream)
    }
}

pub struct Field {
    pub name: Short,
    pub value: TokenStream,
    pub kind: FieldKind,
}

pub enum FieldKind {
    StaticView,
    View,
    Event {
        event: &'static str,
        target: Ident,
    },
    Attribute {
        el: Short,
        attr: Attr,
        span: Span,
        prop: TokenStream,
    },
}

impl Debug for Field {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Field { name, value, kind } = self;

        match kind {
            FieldKind::StaticView => {
                write!(f, "{name} <StaticView>: {value}")
            }
            FieldKind::View => {
                write!(f, "{name} <View>: {value}")
            }
            FieldKind::Event { event, target } => {
                write!(f, "{name} <Listener<{event}<{target}>>>: {value}")
            }
            FieldKind::Attribute { attr, .. } => {
                write!(f, "{name} <AttributeView<{}>>: {value}", attr.name)
            }
        }
    }
}

impl Field {
    pub fn new(name: Short, value: TokenStream) -> Self {
        Field {
            name,
            value,
            kind: FieldKind::View,
        }
    }

    pub fn event(&mut self, event: &'static str, target: Ident) -> &mut Self {
        self.kind = FieldKind::Event { event, target };
        self
    }

    pub fn attr(&mut self, el: Short, span: Span, attr: Attr, prop: TokenStream) -> &mut Self {
        self.kind = FieldKind::Attribute {
            el,
            span,
            attr,
            prop,
        };
        self
    }

    fn name_value(&self) -> (&Short, &TokenStream) {
        (&self.name, &self.value)
    }

    fn make_type(&self) -> Short {
        let (name, _) = self.name_value();

        let mut typ = *name;
        typ.make_ascii_uppercase();
        typ
    }

    fn bounds(&self, buf: &mut TokenStream) {
        let Field { name, kind, .. } = self;

        let mut typ = *name;
        typ.make_ascii_uppercase();

        match kind {
            FieldKind::View | FieldKind::StaticView => {
                buf.write((typ.as_str(), ": ::kobold::View,"));
            }
            FieldKind::Event { event, target } => {
                buf.write(format_args!(
                    "{typ}: ::kobold::event::Listener<\
                        ::kobold::event::{event}<\
                            ::kobold::reexport::web_sys::{target}\
                        >
                    >,"
                ));
            }
            FieldKind::Attribute { attr, span, .. } => {
                buf.write((
                    ident(typ.as_str()),
                    ": ::kobold::attribute::AttributeView<::kobold::attribute::",
                    Ident::new(attr.name, *span),
                    '>',
                    attr.abi.map(InlineAbi::bound),
                    ',',
                ));
            }
        }
    }

    fn declare(&self, buf: &mut String) {
        let (name, _) = self.name_value();
        let typ = self.make_type();

        let _ = write!(buf, "{name}: {typ},");
    }

    fn build(&self, buf: &mut String) {
        let Field { name, kind, .. } = self;

        match kind {
            FieldKind::View | FieldKind::StaticView | FieldKind::Event { .. } => {
                let _ = write!(buf, "let {name} = self.{name}.build();");
            }
            _ => (),
        }
    }

    fn var(&self, buf: &mut String) {
        let Field { name, kind, .. } = self;

        match kind {
            FieldKind::StaticView => (),
            FieldKind::View => {
                let _ = write!(buf, "{name},");
            }
            FieldKind::Event { .. } => {
                let _ = write!(buf, "{name},");
            }
            FieldKind::Attribute { attr, .. } if attr.abi.is_some() => {
                let _ = write!(buf, "{name}: self.{name}.build(),");
            }
            FieldKind::Attribute { el, prop, .. } => {
                let _ = write!(buf, "{name}: self.{name}.build_in({prop}, &{el}),");
            }
        }
    }

    fn update(&self, buf: &mut String) {
        let Field { name, kind, .. } = self;

        match kind {
            FieldKind::StaticView => (),
            FieldKind::View | FieldKind::Event { .. } => {
                let _ = write!(buf, "self.{name}.update(&mut p.{name});");
            }
            FieldKind::Attribute { el, prop, .. } => {
                let _ = write!(
                    buf,
                    "self.{name}.update_in({prop}, &p.{el}, &mut p.{name});"
                );
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
