use crate::util;
use crate::{Html, Mountable};
use wasm_bindgen::JsValue;
use web_sys::Node;

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

impl<V> Html for Attribute<V>
where
    V: AsRef<str> + PartialEq + 'static,
{
    type Product = AttributeProduct<V>;

    fn build(self) -> Self::Product {
        let node = util::__kobold_create_attr(self.name, self.value.as_ref());

        AttributeProduct {
            value: self.value,
            node,
        }
    }

    fn update(self, p: &mut Self::Product) {
        if p.value != self.value {
            p.value = self.value;
            util::__kobold_update_attr(&p.node, p.value.as_ref());
        }
    }
}

macro_rules! create_named_attrs {
    ($($name:ident => $fun:ident,)*) => {$(
        pub struct $name<V>(pub V);

        impl<V> Html for $name<V>
        where
            V: AsRef<str> + PartialEq + 'static,
        {
            type Product = AttributeProduct<V>;

            fn build(self) -> Self::Product {
                let node = util::$fun(self.0.as_ref());

                AttributeProduct {
                    value: self.0,
                    node,
                }
            }

            fn update(self, p: &mut Self::Product) {
                if p.value != self.0 {
                    p.value = self.0;
                    util::__kobold_update_attr(&p.node, p.value.as_ref());
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
    node: Node,
}

impl<V: 'static> Mountable for AttributeProduct<V> {
    fn js(&self) -> &JsValue {
        &self.node
    }
}
