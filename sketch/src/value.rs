use crate::traits::{Html, Mountable, Update};
use crate::util;
use web_sys::Node;

fn bool_to_str(b: bool) -> &'static str {
    if b { "true" } else { "false" }
}

pub struct RenderedValue<T> {
	value: T,
	buf: String,
	node: Node,
}

impl<T> Mountable for RenderedValue<T> {
	fn node(&self) -> &Node {
		&self.node
	}
}

impl Html for bool {
    type Rendered = RenderedValue<bool>;

    fn render(self) -> Self::Rendered {
        let node = util::__sketch_text_node(bool_to_str(self));

        RenderedValue {
        	value: self,
        	buf: String::new(),
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
                    	buf,
                    	node,
                    }
                }
            }

            impl Update<$t> for RenderedValue<$t> {
                fn update(&mut self, new: $t) {
                	use std::fmt::Write;

                	if self.value != new {
                		self.value = new;

                		self.buf.clear();

                		// Writing to String is infallible
                		let _ = write!(&mut self.buf, "{}", new);

                		util::__sketch_update_text(&self.node, &self.buf);
                	}
                }
            }
        )*
    };
}

impl_render_value!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64);
