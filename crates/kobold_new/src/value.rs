use crate::traits::{Html, Mountable};
use crate::util;
use std::str;
use wasm_bindgen::JsValue;
use web_sys::Node;

pub struct ValueNode<T> {
    value: T,
    node: Node,
}

impl<T: 'static> Mountable for ValueNode<T> {
    fn js(&self) -> &JsValue {
        &self.node
    }
}

#[derive(PartialEq, Eq)]
pub struct StrRef {
    ptr: *const u8,
    len: usize,
}

impl StrRef {
    fn new(s: &str) -> Self {
        StrRef {
            ptr: s.as_ptr(),
            len: s.len(),
        }
    }
}

impl<'a> Html for &'a str {
	type Node = ValueNode<StrRef>;

	fn build(self) -> Self::Node {
		let node = util::__kobold_text_node(self);

		ValueNode {
			value: StrRef::new(self),
			node,
		}
	}

	fn update(self, built: &mut Self::Node) {
        let val = StrRef::new(self);

		if built.value != val {
			built.value = val;

			util::__kobold_update_text(&built.node, self);
		}
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
    type Node = ValueNode<bool>;

    fn build(self) -> Self::Node {
        let node = util::__kobold_text_node(bool_to_str(self));

        ValueNode { value: self, node }
    }

    fn update(self, built: &mut Self::Node) {
    	if built.value != self {
    		built.value = self;

    		util::__kobold_update_text(&built.node, bool_to_str(built.value));
    	}
    }
}

macro_rules! impl_int {
    ($($t:ty),*) => {
        $(
            impl Html for $t {
                type Node = ValueNode<$t>;

                fn build(self) -> Self::Node {
                    let mut buf = [0_u8; 20];

                    let n = itoa::write(&mut buf[..], self).unwrap_or_else(|_| 0);
                    let node = util::__kobold_text_node(unsafe {
                        str::from_utf8_unchecked(&buf[..n])
                    });

                    ValueNode {
                        value: self,
                        node,
                    }
                }

                fn update(self, built: &mut Self::Node) {
                    if built.value != self {
                        built.value = self;

                        let mut buf = [0_u8; 20];

                        let n = itoa::write(&mut buf[..], self).unwrap_or_else(|_| 0);

                        util::__kobold_update_text(&built.node, unsafe {
                            str::from_utf8_unchecked(&buf[..n])
                        });
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
                type Node = ValueNode<$t>;

                fn build(self) -> Self::Node {
                    let mut buf = [0_u8; 20];

                    let n = dtoa::write(&mut buf[..], self).unwrap_or_else(|_| 0);
                    let node = util::__kobold_text_node(unsafe {
                        str::from_utf8_unchecked(&buf[..n])
                    });

                    ValueNode {
                        value: self,
                        node,
                    }
                }

                fn update(self, built: &mut Self::Node) {
                    if (built.value - self).abs() > <$t>::EPSILON {
                        built.value = self;

                        let mut buf = [0_u8; 20];

                        let n = dtoa::write(&mut buf[..], self).unwrap_or_else(|_| 0);

                        util::__kobold_update_text(&built.node, unsafe {
                            str::from_utf8_unchecked(&buf[..n])
                        });
                    }
                }
            }
        )*
    };
}

macro_rules! impl_value {
    ($($t:ty),*) => {
        $(
            impl Html for $t {
                type Node = ValueNode<$t>;

                fn build(self) -> Self::Node {
                    let buf = self.to_string();
                    let node = util::__kobold_text_node(&buf);

                    ValueNode {
                        value: self,
                        node,
                    }
                }

                fn update(self, built: &mut Self::Node) {
                    if built.value != self {
                        built.value = self;

                        let buf = self.to_string();

                        util::__kobold_update_text(&built.node, &buf);
                    }
                }
            }
        )*
    };
}

impl_int!(u8, u16, u32, u64, i8, i16, i32, i64, usize, isize);
impl_float!(f32, f64);
impl_value!(u128, i128);
