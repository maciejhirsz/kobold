mod callback;
mod list;
mod scope;
mod text;
mod util;
mod value;

pub mod attribute;
pub mod internals;
pub mod traits;

pub type ShouldRender = bool;

pub use internals::Link;
pub use list::{BuiltList, IterWrapper};
pub use text::BuiltText;
pub use traits::{Component, Html, Mountable, Update};

pub mod prelude {
    pub use super::{html, Component, Html, Link, Mountable, ShouldRender, Update};
}

pub use kobold_macros::html;
pub use wasm_bindgen::JsValue;
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
        type Built = EmptyNode;

        fn build(self) -> EmptyNode {
            EmptyNode(util::__kobold_empty_node())
        }
    }

    pub struct EmptyNode(Node);

    impl Mountable for EmptyNode {
        fn js(&self) -> &wasm_bindgen::JsValue {
            &self.0
        }
    }

    impl Update<()> for EmptyNode {
        fn update(&mut self, _: ()) {}
    }
}
