//! Utilities for dealing with DOM attributes

use wasm_bindgen::convert::IntoWasmAbi;
use wasm_bindgen::JsValue;

use crate::util;
use crate::value::Stringify;
use crate::{Element, Html, Mountable};

pub use crate::stateful::Callback;

pub trait Attribute {
    type Product: AttributeProduct;

    fn build(self) -> Self::Product;

    fn update(self, p: &mut Self::Product, el: &JsValue);
}

pub trait AttributeProduct: 'static {
    type Abi: IntoWasmAbi;

    fn js(&self) -> Self::Abi;
}

impl<T> AttributeProduct for T
where
    T: IntoWasmAbi + Copy + 'static,
{
    type Abi = Self;

    fn js(&self) -> Self::Abi {
        *self
    }
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

impl Html for AttributeNode<String> {
    type Product = AttributeNodeProduct<String>;

    fn build(self) -> Self::Product {
        let node = util::__kobold_attr(self.name, &self.value);
        let el = Element::new(node);

        AttributeNodeProduct {
            value: self.value,
            el,
        }
    }

    fn update(self, p: &mut Self::Product) {
        if *self.value != p.value {
            util::__kobold_attr_update(&p.el.node, &self.value);
            p.value = self.value;
        }
    }
}

impl Html for AttributeNode<&String> {
    type Product = AttributeNodeProduct<String>;

    fn build(self) -> Self::Product {
        let node = util::__kobold_attr(self.name, self.value);
        let el = Element::new(node);

        AttributeNodeProduct {
            value: self.value.clone(),
            el,
        }
    }

    fn update(self, p: &mut Self::Product) {
        if *self.value != p.value {
            util::__kobold_attr_update(&p.el.node, self.value);
            p.value.clone_from(self.value)
        }
    }
}

impl<S> Html for AttributeNode<S>
where
    S: Stringify + Eq + Copy + 'static,
{
    type Product = AttributeNodeProduct<S>;

    fn build(self) -> Self::Product {
        let node = self.value.stringify(|s| util::__kobold_attr(self.name, s));
        let el = Element::new(node);

        AttributeNodeProduct {
            value: self.value,
            el,
        }
    }

    fn update(self, p: &mut Self::Product) {
        if self.value != p.value {
            self.value
                .stringify(|s| util::__kobold_attr_update(&p.el.node, s));
            p.value = self.value;
        }
    }
}

/// A class that interacts with `classList` property of an element
///
/// <https://developer.mozilla.org/en-US/docs/Web/API/Element/classList>
pub struct Class(&'static str);

impl From<&'static str> for Class {
    fn from(class: &'static str) -> Self {
        debug_assert!(!class.chars().any(|c| c == ' '), "Class name cannot contain spaces, offending class: \"{class}\"");

        Class(class)
    }
}

impl From<Option<&'static str>> for Class {
    fn from(class: Option<&'static str>) -> Self {
        Class::from(class.unwrap_or_default())
    }
}

impl Attribute for Class {
    type Product = &'static str;

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

/// A single `className` attribute, spaces are permitted
///
/// <https://developer.mozilla.org/en-US/docs/Web/API/Element/className>
pub struct ClassName(&'static str);

impl From<&'static str> for ClassName {
    fn from(class: &'static str) -> Self {
        ClassName(class)
    }
}

impl From<Option<&'static str>> for ClassName {
    fn from(class: Option<&'static str>) -> Self {
        ClassName(class.unwrap_or_default())
    }
}

impl Attribute for ClassName {
    type Product = &'static str;

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

/// The `checked` attribute for `<input>` elements
pub struct Checked(pub bool);

impl Attribute for Checked {
    type Product = bool;

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
    fn el(&self) -> &Element {
        &self.el
    }
}
