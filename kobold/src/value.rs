use crate::traits::{Html, Mountable, Update};
use crate::util;
use std::str;
use wasm_bindgen::JsValue;
use web_sys::Node;

pub struct RenderedValue<T> {
    value: T,
    node: Node,
}

impl<T> Mountable for RenderedValue<T> {
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
    type Rendered = RenderedValue<bool>;

    fn render(self) -> Self::Rendered {
        let node = util::__kobold_text_node(bool_to_str(self));

        RenderedValue { value: self, node }
    }
}

impl Update<bool> for RenderedValue<bool> {
    fn update(&mut self, new: bool) {
        if self.value != new {
            self.value = new;

            util::__kobold_update_text(&self.node, bool_to_str(self.value));
        }
    }
}

macro_rules! impl_int {
    ($($t:ty),*) => {
        $(
            impl Html for $t {
                type Rendered = RenderedValue<$t>;

                fn render(self) -> Self::Rendered {
                    let mut buf = [0_u8; 20];

                    let n = itoa::write(&mut buf[..], self).unwrap_or_else(|_| 0);
                    let node = util::__kobold_text_node(unsafe {
                        str::from_utf8_unchecked(&buf[..n])
                    });

                    RenderedValue {
                        value: self,
                        node,
                    }
                }
            }

            impl Update<$t> for RenderedValue<$t> {
                fn update(&mut self, new: $t) {
                    if self.value != new {
                        self.value = new;

                        let mut buf = [0_u8; 20];

                        let n = itoa::write(&mut buf[..], new).unwrap_or_else(|_| 0);

                        util::__kobold_update_text(&self.node, unsafe {
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
                type Rendered = RenderedValue<$t>;

                fn render(self) -> Self::Rendered {
                    let mut buf = [0_u8; 20];

                    let n = dtoa::write(&mut buf[..], self).unwrap_or_else(|_| 0);
                    let node = util::__kobold_text_node(unsafe {
                        str::from_utf8_unchecked(&buf[..n])
                    });

                    RenderedValue {
                        value: self,
                        node,
                    }
                }
            }

            impl Update<$t> for RenderedValue<$t> {
                fn update(&mut self, new: $t) {
                    if (self.value - new).abs() > <$t>::EPSILON {
                        self.value = new;

                        let mut buf = [0_u8; 20];

                        let n = dtoa::write(&mut buf[..], new).unwrap_or_else(|_| 0);

                        util::__kobold_update_text(&self.node, unsafe {
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
                type Rendered = RenderedValue<$t>;

                fn render(self) -> Self::Rendered {
                    let buf = self.to_string();
                    let node = util::__kobold_text_node(&buf);

                    RenderedValue {
                        value: self,
                        node,
                    }
                }
            }

            impl Update<$t> for RenderedValue<$t> {
                fn update(&mut self, new: $t) {
                    if self.value != new {
                        self.value = new;

                        let buf = new.to_string();

                        util::__kobold_update_text(&self.node, &buf);
                    }
                }
            }
        )*
    };
}

impl_int!(u8, u16, u32, u64, i8, i16, i32, i64, usize, isize);
impl_float!(f32, f64);
impl_value!(u128, i128);
