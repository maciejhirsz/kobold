use crate::traits::{Html, Mountable, Update};
use crate::util;
use web_sys::Node;
use std::str;

pub struct RenderedValue<T> {
    value: T,
    node: Node,
}

impl<T> Mountable for RenderedValue<T> {
    fn node(&self) -> &Node {
        &self.node
    }
}

fn bool_to_str(b: bool) -> &'static str {
    if b { "true" } else { "false" }
}

impl Html for bool {
    type Rendered = RenderedValue<bool>;

    fn render(self) -> Self::Rendered {
        let node = util::__sketch_text_node(bool_to_str(self));

        RenderedValue {
            value: self,
            node,
        }
    }
}

impl Update<bool> for RenderedValue<bool> {
    fn update(&mut self, new: bool) {
        if self.value != new {
            self.value = new;

            util::__sketch_update_text(&self.node, bool_to_str(self.value));
        }
    }
}

macro_rules! impl_render_num {
    ($lib:ident, $($t:ty),*) => {
        $(
            impl Html for $t {
                type Rendered = RenderedValue<$t>;

                fn render(self) -> Self::Rendered {
                    let mut buf = [0_u8; 20];

                    let n = $lib::write(&mut buf[..], self).unwrap_or_else(|_| 0);
                    let node = util::__sketch_text_node(unsafe {
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

                        let n = $lib::write(&mut buf[..], new).unwrap_or_else(|_| 0);

                        util::__sketch_update_text(&self.node, unsafe {
                            str::from_utf8_unchecked(&buf[..n])
                        });
                    }
                }
            }
        )*
    };
}

macro_rules! impl_render_value {
    ($($t:ty),*) => {
        $(
            impl Html for $t {
                type Rendered = RenderedValue<$t>;

                fn render(self) -> Self::Rendered {
                    let buf = self.to_string();
                    let node = util::__sketch_text_node(&buf);

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

                        util::__sketch_update_text(&self.node, &buf);
                    }
                }
            }
        )*
    };
}

impl_render_num!(itoa, u8, u16, u32, u64, i8, i16, i32, i64, usize, isize);
impl_render_num!(dtoa, f32, f64);
impl_render_value!(u128, i128);
