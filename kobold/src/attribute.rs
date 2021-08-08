use crate::traits::{Html, Mountable, Update};
use crate::util;
use wasm_bindgen::JsValue;
use web_sys::Node;

pub use crate::callback::Callback;

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
    type Built = BuiltAttribute<V>;

    fn build(self) -> Self::Built {
        let node = util::__kobold_create_attr(self.name, self.value.as_ref());

        BuiltAttribute {
            value: self.value,
            node,
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
            type Built = BuiltAttribute<V>;

            fn build(self) -> Self::Built {
                let node = util::$fun(self.0.as_ref());

                BuiltAttribute {
                    value: self.0,
                    node,
                }
            }
        }

        impl<V> Update<$name<V>> for BuiltAttribute<V>
        where
            V: AsRef<str> + PartialEq,
        {
            fn update(&mut self, new: $name<V>) {
                if self.value != new.0 {
                    self.value = new.0;
                    util::__kobold_update_attr(&self.node, self.value.as_ref());
                }
            }
        }
    )*};
}

create_named_attrs! {
    Class => __kobold_create_attr_class,
    Style => __kobold_create_attr_style,
}

pub struct BuiltAttribute<V> {
    value: V,
    node: Node,
}

impl<V> Mountable for BuiltAttribute<V> {
    fn js(&self) -> &JsValue {
        &self.node
    }
}

impl<V> Update<Attribute<V>> for BuiltAttribute<V>
where
    V: AsRef<str> + PartialEq,
{
    fn update(&mut self, new: Attribute<V>) {
        if self.value != new.value {
            self.value = new.value;
            util::__kobold_update_attr(&self.node, self.value.as_ref());
        }
    }
}
