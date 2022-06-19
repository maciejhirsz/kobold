pub use kobold_macros::html;

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
    pub use crate::{html, Html, ShouldRender, Stateful};
}

/// Re-exports for the [`html!`](html) macro to use
pub mod reexport {
    pub use wasm_bindgen;
    pub use web_sys;
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

pub fn start(html: impl Html) {
    use std::cell::Cell;

    thread_local! {
        static INIT: Cell<bool> = Cell::new(false);
    }

    if !INIT.with(|init| init.get()) {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));

        INIT.with(|init| init.set(true));
    }

    use std::mem::ManuallyDrop;

    let built = ManuallyDrop::new(html.build());

    util::__kobold_start(built.js());
}
