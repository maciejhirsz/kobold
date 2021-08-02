mod traits;
mod text;
mod util;
mod list;
mod value;

pub use traits::{Html, Mountable, Update};
pub use text::{RenderedText};
pub use list::{RenderedList, IterWrapper};

pub use web_sys::Node;
pub use sketch_macro::html;

pub mod reexport {
    pub use web_sys;
    pub use wasm_bindgen;
}

impl Html for () {
    type Rendered = EmptyNode;

    fn render(self) -> EmptyNode {
        EmptyNode(util::__sketch_text_node(""))
    }
}

pub struct EmptyNode(Node);

impl Mountable for EmptyNode {
    fn node(&self) -> &Node {
        &self.0
    }
}

impl Update<()> for EmptyNode {
    fn update(&mut self, _: ()) {}
}
