use crate::{Html, Mountable};
use crate::util;
use std::str;
use wasm_bindgen::JsValue;
use web_sys::Node;

pub struct ValueProduct<T> {
    value: T,
    node: Node,
}

impl<T: 'static> Mountable for ValueProduct<T> {
    fn js(&self) -> &JsValue {
        &self.node
    }
}

fn bool_to_str(b: bool) -> &'static str {
    if b {
        "true"
    } else {
        "false"
    }
}

impl Html for bool {
    type Product = ValueProduct<bool>;

    fn build(self) -> Self::Product {
        let node = util::__kobold_text_node(bool_to_str(self));

        ValueProduct { value: self, node }
    }

    fn update(self, p: &mut Self::Product) {
        if p.value != self {
            p.value = self;

            util::__kobold_update_text(&p.node, bool_to_str(p.value));
        }
    }
}

macro_rules! impl_int {
    ($($t:ty),*) => {
        $(
            impl Html for $t {
                type Product = ValueProduct<$t>;

                fn build(self) -> Self::Product {
                    let mut buf = itoa::Buffer::new();

                    let node = util::__kobold_text_node(buf.format(self));

                    ValueProduct {
                        value: self,
                        node,
                    }
                }

                fn update(self, p: &mut Self::Product) {
                    if p.value != self {
                        p.value = self;

                        let mut buf = itoa::Buffer::new();

                        util::__kobold_update_text(&p.node, buf.format(self));
                    }
                }
            }
        )*
    };
}

macro_rules! impl_float {
    ($($t:ty),*) => {
        $(
            impl Html for $t {
                type Product = ValueProduct<$t>;

                fn build(self) -> Self::Product {
                    let mut buf = ryu::Buffer::new();

                    let node = util::__kobold_text_node(buf.format(self));

                    ValueProduct {
                        value: self,
                        node,
                    }
                }

                fn update(self, p: &mut Self::Product) {
                    if (p.value - self).abs() > <$t>::EPSILON {
                        p.value = self;

                        let mut buf = ryu::Buffer::new();

                        util::__kobold_update_text(&p.node, buf.format(self));
                    }
                }
            }
        )*
    };
}

impl_int!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize, isize);
impl_float!(f32, f64);
