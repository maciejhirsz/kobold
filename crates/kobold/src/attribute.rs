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

pub struct Checked(pub bool);

impl Attribute for Checked {
    type Product = bool;

    fn build(self) -> Self::Product {
        self.0
    }

    fn update(self, p: &mut Self::Product, js: &JsValue) {
        if self.0 != *p {
            util::__kobold_attr_checked_set(js, self.0);
            *p = self.0;
        }
    }
}

macro_rules! create_named_attrs {
    ($($name:ident => $fun:ident,)*) => {$(
        pub struct $name<V>(pub V);

        impl Html for $name<String> {
            type Product = AttributeNodeProduct<String>;

            fn build(self) -> Self::Product {
                let node = util::$fun(&self.0);
                let el = Element::new(node);

                AttributeNodeProduct { value: self.0, el }
            }

            fn update(self, p: &mut Self::Product) {
                if self.0 != p.value {
                    util::__kobold_attr_update(&p.el.node, &self.0);
                    p.value = self.0;
                }
            }
        }

        impl Html for $name<&String> {
            type Product = AttributeNodeProduct<String>;

            fn build(self) -> Self::Product {
                let node = util::$fun(self.0);
                let el = Element::new(node);

                AttributeNodeProduct { value: self.0.clone(), el }
            }

            fn update(self, p: &mut Self::Product) {
                if *self.0 != p.value {
                    util::__kobold_attr_update(&p.el.node, self.0);
                    p.value.clone_from(self.0);
                }
            }
        }

        impl<S> Html for $name<S>
        where
            S: Stringify + Eq + Copy + 'static,
        {
            type Product = AttributeNodeProduct<S>;

            fn build(self) -> Self::Product {
                let node = self.0.stringify(util::$fun);
                let el = Element::new(node);

                AttributeNodeProduct { value: self.0, el }
            }

            fn update(self, p: &mut Self::Product) {
                if self.0 != p.value {
                    self.0.stringify(|s| util::__kobold_attr_update(&p.el.node, s));
                    p.value = self.0;
                }
            }
        }
    )*};
}

create_named_attrs! {
    Class => __kobold_attr_class,
    Style => __kobold_attr_style,
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
