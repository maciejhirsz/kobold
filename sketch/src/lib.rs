mod list;
mod text;
mod traits;
mod util;
mod value;

pub use list::{IterWrapper, RenderedList};
pub use text::RenderedText;
pub use traits::{Html, Mountable, Update};

pub use sketch_macro::html;
pub use web_sys::Node;

pub mod reexport {
    pub use wasm_bindgen;
    pub use web_sys;
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
