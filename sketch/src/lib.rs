mod list;
mod text;
mod util;
mod value;

pub mod internals;
pub mod traits;

pub type ShouldRender = bool;

pub use list::{IterWrapper, RenderedList};
pub use text::RenderedText;
pub use traits::{Component, Html, Mountable, Update};

pub mod prelude {
    pub use super::{Component, Html, Mountable, ShouldRender, Update};
}

pub use sketch_macro::html;
pub use web_sys::Node;

pub mod reexport {
    pub use wasm_bindgen;
    pub use web_sys;
}

mod empty {
    use crate::prelude::*;
    use crate::util;
    use web_sys::Node;

    impl Html for () {
        type Rendered = EmptyNode;

        fn render(self) -> EmptyNode {
            EmptyNode(util::__sketch_empty_node())
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
}
