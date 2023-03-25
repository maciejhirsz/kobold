//! Utilities for dealing with DOM attributes
use wasm_bindgen::convert::IntoWasmAbi;
use wasm_bindgen::JsValue;
use web_sys::Node;

use crate::diff::Diff;
use crate::dom::{NoDiff, Text, Property};
use crate::util;
use crate::{Element, Mountable, View};

pub struct Attr(pub &'static str);
pub struct Checked;

impl Property<&str> for Attr {
    fn set(self, this: &JsValue, value: &str) {
        util::set_attr(this, self.0, value)
    }
}

impl Property<f64> for Attr {
    fn set(self, this: &JsValue, value: f64) {
        util::set_attr_num(this, self.0, value)
    }
}

impl Property<bool> for Attr {
    fn set(self, this: &JsValue, value: bool) {
        util::set_attr_bool(this, self.0, value)
    }
}

impl Property<bool> for Checked {
    fn set(self, this: &JsValue, value: bool) {
        util::set_checked(this, value)
    }
}

pub struct AttributeNode<A, V> {
    constructor: A,
    value: V,
}

macro_rules! def_attr {
    ($($name:ident,)*) => {
        $(
            #[doc = concat!("The `", stringify!($name) ,"` attribute constructor")]
            pub fn $name<V>(value: V) -> AttributeNode<impl Fn() -> Node, V> {
                AttributeNode::new(util::$name, value)
            }
        )*
    };
}

def_attr! {
    href,
    style,
    value,
}

impl<A, T> AttributeNode<A, T> {
    pub const fn new(constructor: A, value: T) -> Self {
        AttributeNode { constructor, value }
    }
}

impl<A, T> View for AttributeNode<A, T>
where
    A: Fn() -> Node,
    T: Text + Diff + Copy,
{
    type Product = AttributeNodeProduct<T::State>;

    fn build(self) -> Self::Product {
        let el = Element::new((self.constructor)());

        self.value.set_attr(&el);
        let state = self.value.init();

        AttributeNodeProduct { state, el }
    }

    fn update(self, p: &mut Self::Product) {
        if self.value.update(&mut p.state) {
            self.value.set_attr(&p.el);
        }
    }
}

pub trait Attribute {
    type Abi: IntoWasmAbi;
    type Product: 'static;

    fn js(&self) -> Self::Abi;

    fn build(self) -> Self::Product;

    fn update(self, p: &mut Self::Product, el: &JsValue);
}

/// A class that interacts with `classList` property of an element
///
/// <https://developer.mozilla.org/en-US/docs/Web/API/Element/classList>
#[repr(transparent)]
pub struct Class<T>(T);

impl From<&'static str> for Class<&'static str> {
    fn from(class: &'static str) -> Self {
        debug_assert!(
            !class.chars().any(|c| c == ' '),
            "Class name cannot contain spaces, offending class: \"{class}\""
        );

        Class(class)
    }
}

impl From<Option<&'static str>> for Class<&'static str> {
    fn from(class: Option<&'static str>) -> Self {
        Class::from(class.unwrap_or_default())
    }
}

#[derive(Clone, Copy)]
pub struct OptionalClass<'a> {
    class: &'a str,
    on: bool,
}

impl<'a> OptionalClass<'a> {
    pub const fn new(class: &'a str, on: bool) -> Self {
        OptionalClass { class, on }
    }

    pub const fn no_diff(self) -> NoDiff<Self> {
        NoDiff(self)
    }

    fn get(&self) -> &'a str {
        if self.on {
            self.class
        } else {
            ""
        }
    }
}

impl<'a> From<OptionalClass<'a>> for Class<OptionalClass<'a>> {
    fn from(class: OptionalClass<'a>) -> Self {
        Class(class)
    }
}

impl<'a> From<NoDiff<OptionalClass<'a>>> for Class<NoDiff<OptionalClass<'a>>> {
    fn from(class: NoDiff<OptionalClass<'a>>) -> Self {
        Class(class)
    }
}

impl Attribute for Class<&'static str> {
    type Abi = &'static str;
    type Product = &'static str;

    fn js(&self) -> Self::Abi {
        self.0
    }

    fn build(self) -> Self::Product {
        self.0
    }

    fn update(self, p: &mut Self::Product, js: &JsValue) {
        match (self.0, *p) {
            (new, old) if new == old => return,
            (new, "") => util::add_class(js, new),
            ("", old) => util::remove_class(js, old),
            (new, old) => util::replace_class(js, old, new),
        }
        *p = self.0;
    }
}

impl<'a> Attribute for Class<NoDiff<OptionalClass<'a>>> {
    type Abi = &'a str;
    type Product = bool;

    fn js(&self) -> Self::Abi {
        self.0.get()
    }

    fn build(self) -> Self::Product {
        self.0.on
    }

    fn update(self, p: &mut Self::Product, js: &JsValue) {
        match (self.0.on, *p) {
            (true, true) | (false, false) => return,
            (true, false) => util::add_class(js, self.0.class),
            (false, true) => util::remove_class(js, self.0.class),
        }
        *p = self.0.on;
    }
}

/// A single `className` attribute, spaces are permitted
///
/// <https://developer.mozilla.org/en-US/docs/Web/API/Element/className>
pub struct ClassName<T>(T);

impl From<&'static str> for ClassName<&'static str> {
    fn from(class: &'static str) -> Self {
        ClassName(class)
    }
}

impl From<Option<&'static str>> for ClassName<&'static str> {
    fn from(class: Option<&'static str>) -> Self {
        ClassName(class.unwrap_or_default())
    }
}

impl<'a> From<NoDiff<OptionalClass<'a>>> for ClassName<NoDiff<OptionalClass<'a>>> {
    fn from(class: NoDiff<OptionalClass<'a>>) -> Self {
        ClassName(class)
    }
}

impl Attribute for ClassName<&'static str> {
    type Abi = &'static str;
    type Product = &'static str;

    fn js(&self) -> Self::Abi {
        self.0
    }

    fn build(self) -> Self::Product {
        self.0
    }

    fn update(self, p: &mut Self::Product, js: &JsValue) {
        if self.0 != *p {
            util::set_class_name(js, self.0);
            *p = self.0;
        }
    }
}

impl<'a> Attribute for ClassName<NoDiff<OptionalClass<'a>>> {
    type Abi = &'a str;
    type Product = bool;

    fn js(&self) -> Self::Abi {
        self.0.get()
    }

    fn build(self) -> Self::Product {
        self.0.on
    }

    fn update(self, p: &mut Self::Product, js: &JsValue) {
        if self.0.on != *p {
            util::set_class_name(js, self.0.get());
            *p = self.0.on;
        }
    }
}

// /// The `checked` attribute for `<input>` elements
// pub struct Checked(pub bool);

// impl Attribute for Checked {
//     type Abi = bool;
//     type Product = ();

//     fn js(&self) -> Self::Abi {
//         self.0
//     }

//     fn build(self) -> Self::Product {}

//     fn update(self, _: &mut Self::Product, js: &JsValue) {
//         // Checkboxes are weird because a `click` or `change` event
//         // can affect the state without reflecting it on the product.
//         //
//         // Best to do the diff in the DOM directly.
//         util::set_checked(js, self.0);
//     }
// }

pub struct AttributeNodeProduct<V> {
    state: V,
    el: Element,
}

impl<V: 'static> Mountable for AttributeNodeProduct<V> {
    type Js = JsValue;

    fn el(&self) -> &Element {
        &self.el
    }
}
