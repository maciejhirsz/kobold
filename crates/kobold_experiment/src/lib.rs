use wasm_bindgen::JsValue;
use web_sys::Node;

mod render_fn;
mod util;
mod value;

pub mod attribute;
pub mod list;
pub mod stateful;

pub use stateful::Stateful;

pub mod prelude {
    pub use crate::list::ListExt;
    pub use crate::{Html, Stateful};
}

pub enum ShouldRender {
    No,
    Yes,
}

impl From<()> for ShouldRender {
    fn from(_: ()) -> ShouldRender {
        ShouldRender::Yes
    }
}

impl ShouldRender {
    fn should_render(self) -> bool {
        match self {
            ShouldRender::Yes => true,
            ShouldRender::No => false,
        }
    }
}

pub trait Html: Sized {
    type Product: Mountable;

    fn build(self) -> Self::Product;

    fn update(self, p: &mut Self::Product);
}

pub trait Mountable: 'static {
    fn js(&self) -> &JsValue;

    fn mount(&self, parent: &Node) {
        util::__kobold_mount(parent, self.js());
    }

    fn unmount(&self, parent: &Node) {
        util::__kobold_unmount(parent, self.js());
    }
}

#[derive(PartialEq, Eq)]
struct Counter {
    n: i32,
}

impl Counter {
    pub fn render(self) -> impl Html {
        self.stateful(|state, link| {
            let inc = link.callback(|state, _| state.n += 1);
            let dec = link.callback(|state, _| state.n -= 1);

            state.n
        })
    }
}
