//! Utilities for dealing with DOM attributes
use wasm_bindgen::convert::IntoWasmAbi;
use wasm_bindgen::JsValue;

use crate::util;
use crate::value::{FastDiff, NoDiff, Stringify};
use crate::{Element, Mountable, View};

pub trait Attribute {
    type Abi: IntoWasmAbi;
    type Product: 'static;

    fn js(&self) -> Self::Abi;

    fn build(self) -> Self::Product;

    fn update(self, p: &mut Self::Product, el: &JsValue);
}

#[inline]
fn update(el: &Element, value: &str) {
    util::__kobold_attr_update(&el.node, value);
}

#[inline]
fn create(name: &str, value: &str) -> Element {
    Element::new(util::__kobold_attr(name, value))
}

pub struct AttributeNode<V> {
    name: &'static str,
    value: V,
}

impl<V> AttributeNode<V> {
    pub fn new(name: &'static str, value: V) -> Self {
        AttributeNode { name, value }
    }
}

impl View for AttributeNode<String> {
    type Product = AttributeNodeProduct<String>;

    fn build(self) -> Self::Product {
        let el = create(self.name, &self.value);

        AttributeNodeProduct {
            value: self.value,
            el,
        }
    }

    fn update(self, p: &mut Self::Product) {
        if *self.value != p.value {
            update(&p.el, &self.value);
            p.value = self.value;
        }
    }
}

impl View for AttributeNode<&String> {
    type Product = AttributeNodeProduct<String>;

    fn build(self) -> Self::Product {
        let el = create(self.name, self.value);

        AttributeNodeProduct {
            value: self.value.clone(),
            el,
        }
    }

    fn update(self, p: &mut Self::Product) {
        if *self.value != p.value {
            update(&p.el, self.value);
            p.value.clone_from(self.value)
        }
    }
}

impl<S> View for AttributeNode<S>
where
    S: Stringify + Eq + Copy + 'static,
{
    type Product = AttributeNodeProduct<S>;

    fn build(self) -> Self::Product {
        let el = self.value.stringify(|s| create(self.name, s));

        AttributeNodeProduct {
            value: self.value,
            el,
        }
    }

    fn update(self, p: &mut Self::Product) {
        if self.value != p.value {
            self.value.stringify(|s| update(&p.el, s));
            p.value = self.value;
        }
    }
}

impl<S> View for AttributeNode<NoDiff<S>>
where
    S: Stringify,
{
    type Product = Element;

    fn build(self) -> Self::Product {
        self.value.stringify(|s| create(self.name, s))
    }

    fn update(self, _: &mut Self::Product) {}
}

impl View for AttributeNode<FastDiff<'_>> {
    type Product = AttributeNodeProduct<usize>;

    fn build(self) -> Self::Product {
        let el = create(self.name, &self.value);

        AttributeNodeProduct {
            value: self.value.as_ptr() as usize,
            el,
        }
    }

    fn update(self, p: &mut Self::Product) {
        if p.value != self.value.as_ptr() as usize {
            update(&p.el, &self.value);
            p.value = self.value.as_ptr() as usize;
        }
    }
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
            (new, "") => util::__kobold_class_add(js, new),
            ("", old) => util::__kobold_class_remove(js, old),
            (new, old) => util::__kobold_class_replace(js, old, new),
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
            (true, false) => util::__kobold_class_add(js, self.0.class),
            (false, true) => util::__kobold_class_remove(js, self.0.class),
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
            util::__kobold_class_set(js, self.0);
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
            util::__kobold_class_set(js, self.0.get());
            *p = self.0.on;
        }
    }
}

/// The `checked` attribute for `<input>` elements
pub struct Checked(pub bool);

impl Attribute for Checked {
    type Abi = bool;
    type Product = bool;

    fn js(&self) -> Self::Abi {
        self.0
    }

    fn build(self) -> Self::Product {
        self.0
    }

    fn update(self, _: &mut Self::Product, js: &JsValue) {
        // Checkboxes are weird because a `click` or `change` event
        // can affect the state without reflecting it on the product.
        //
        // Best to do the diff in DOM directly.
        util::__kobold_attr_checked_set(js, self.0);
    }
}

pub struct AttributeNodeProduct<V> {
    value: V,
    el: Element,
}

impl<V: 'static> Mountable for AttributeNodeProduct<V> {
    type Js = JsValue;

    fn el(&self) -> &Element {
        &self.el
    }
}
