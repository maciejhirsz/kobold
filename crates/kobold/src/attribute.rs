//! Utilities for dealing with DOM attributes

use crate::util;
use crate::{Element, Html, Mountable};

pub use crate::stateful::Callback;

pub struct Attribute<V> {
    name: &'static str,
    value: V,
}

impl<V> Attribute<V> {
    pub fn new(name: &'static str, value: V) -> Self {
        Attribute { name, value }
    }
}

impl Html for Attribute<&'static str> {
    type Product = AttributeProduct<&'static str>;

    fn build(self) -> Self::Product {
        let node = util::__kobold_create_attr(self.name, self.value);
        let el = Element::new(node);

        AttributeProduct {
            value: self.value.clone(),
            el,
        }
    }

    fn update(self, p: &mut Self::Product) {
        if self.value != p.value {
            util::__kobold_update_attr(&p.el.node, self.value);
            p.value = self.value;
        }
    }
}

impl Html for Attribute<String> {
    type Product = AttributeProduct<String>;

    fn build(self) -> Self::Product {
        let node = util::__kobold_create_attr(self.name, &self.value);
        let el = Element::new(node);

        AttributeProduct {
            value: self.value.clone(),
            el,
        }
    }

    fn update(self, p: &mut Self::Product) {
        if *self.value != p.value {
            util::__kobold_update_attr(&p.el.node, &self.value);
            p.value = self.value;
        }
    }
}

impl Html for Attribute<&String> {
    type Product = AttributeProduct<String>;

    fn build(self) -> Self::Product {
        let node = util::__kobold_create_attr(self.name, self.value);
        let el = Element::new(node);

        AttributeProduct {
            value: self.value.clone(),
            el,
        }
    }

    fn update(self, p: &mut Self::Product) {
        if *self.value != p.value {
            util::__kobold_update_attr(&p.el.node, self.value);
            p.value.clone_from(self.value);
        }
    }
}

impl Html for Attribute<bool> {
    type Product = AttributeProduct<bool>;

    fn build(self) -> Self::Product {
        let node = util::__kobold_create_attr(self.name, "TODO");
        let el = Element::new(node);

        AttributeProduct {
            value: self.value.clone(),
            el,
        }
    }

    fn update(self, p: &mut Self::Product) {
        if self.value != p.value {
            util::__kobold_update_attr(&p.el.node, "TODO");
            p.value = self.value;
        }
    }
}

macro_rules! create_named_attrs {
    ($($name:ident => $fun:ident,)*) => {$(
        pub struct $name<V>(pub V);

        impl Html for $name<&'static str> {
            type Product = AttributeProduct<&'static str>;

            fn build(self) -> Self::Product {
                let node = util::$fun(&self.0);
                let el = Element::new(node);

                AttributeProduct { value: self.0, el }
            }

            fn update(self, p: &mut Self::Product) {
                if self.0 != p.value {
                    util::__kobold_update_attr(&p.el.node, self.0);
                    p.value = self.0;
                }
            }
        }

        impl Html for $name<String> {
            type Product = AttributeProduct<String>;

            fn build(self) -> Self::Product {
                let node = util::$fun(&self.0);
                let el = Element::new(node);

                AttributeProduct { value: self.0, el }
            }

            fn update(self, p: &mut Self::Product) {
                if self.0 != p.value {
                    util::__kobold_update_attr(&p.el.node, &self.0);
                    p.value = self.0;
                }
            }
        }

        impl Html for $name<&String> {
            type Product = AttributeProduct<String>;

            fn build(self) -> Self::Product {
                let node = util::$fun(self.0);
                let el = Element::new(node);

                AttributeProduct { value: self.0.clone(), el }
            }

            fn update(self, p: &mut Self::Product) {
                if *self.0 != p.value {
                    util::__kobold_update_attr(&p.el.node, self.0);
                    p.value.clone_from(self.0);
                }
            }
        }
    )*};
}

create_named_attrs! {
    Class => __kobold_create_attr_class,
    Style => __kobold_create_attr_style,
}

pub struct AttributeProduct<V> {
    value: V,
    el: Element,
}

impl<V: 'static> Mountable for AttributeProduct<V> {
    fn el(&self) -> &Element {
        &self.el
    }
}
