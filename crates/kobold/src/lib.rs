//! <img src="https://raw.githubusercontent.com/maciejhirsz/kobold/master/kobold.svg?sanitize=true" alt="Kobold logo" width="250" align="right">
//!
//! # Kobold

pub use kobold_macros::html;

use wasm_bindgen::JsValue;

mod render_fn;
mod util;
mod value;

pub mod attribute;
pub mod branch;
pub mod dom;
pub mod list;
pub mod stateful;

pub mod prelude {
    pub use crate::{html, Html, IntoHtml};
    pub use crate::stateful::{Stateful, Link, ShouldRender};
}

use dom::Element;

/// Re-exports for the [`html!`](html) macro to use
pub mod reexport {
    pub use wasm_bindgen;
    pub use web_sys;
}

pub trait Html: Sized {
    type Product: Mountable;

    fn build(self) -> Self::Product;

    fn update(self, p: &mut Self::Product);

    /// This is a no-op method that returns self, you souldn't override the default
    /// implementation. For details see [`IntoHtml`](IntoHtml).
    #[inline]
    fn into_html(self) -> Self {
        self
    }
}

/// Types that cannot implement [`Html`](Html) can instead implement `IntoHtml` and
/// still be usable within the `html!` macro.
///
/// This works as a trait specialization of sorts, allowing for `IntoHtml` to be
/// implemented for iterators without running into potential future conflict with
/// `std` foreign types like `&str`.
pub trait IntoHtml {
    type Html: Html;

    fn into_html(self) -> Self::Html;
}

pub trait Mountable: 'static {
    fn el(&self) -> &Element;

    fn js(&self) -> &JsValue {
        self.el().anchor()
    }
}

pub fn start(html: impl Html) {
    init_panic_hook();

    use std::mem::ManuallyDrop;

    let built = ManuallyDrop::new(html.build());

    util::__kobold_start(built.js());
}

fn init_panic_hook() {
    // Only enable console hook on debug builds
    #[cfg(debug_assertions)]
    {
        use std::cell::Cell;

        thread_local! {
            static INIT: Cell<bool> = Cell::new(false);
        }
        if !INIT.with(|init| init.get()) {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));

            INIT.with(|init| init.set(true));
        }
    }
}
