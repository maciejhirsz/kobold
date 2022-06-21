//! <img src="https://raw.githubusercontent.com/maciejhirsz/kobold/master/kobold.svg?sanitize=true" alt="Kobold logo" width="250" align="right">
//!
//! # Kobold
//!
//! _Easy web interfaces._
//!
//! **Kobold** uses macros to deliver familiar HTML-esque syntax for building web interfaces in rust,
//! while leveraging Rust's powerful type system for safety and performance.
//!
//! There is no need for a full [virtual DOM](https://en.wikipedia.org/wiki/Virtual_DOM), all static
//! elements are compiled into plain JavaScript functions that construct the [DOM](https://developer.mozilla.org/en-US/docs/Web/API/Document_Object_Model)
//! elements. All expressions wrapped in `{ ... }` braces are then injected into that DOM, and updated
//! if they change.
//!
//! Like in [React](https://reactjs.org/) or [Yew](https://yew.rs/) updates are done by calling a render
//! function/method, but unlike either the [`html!`][html] macro in Kobold produces transient static types that
//! implement the [`Html`](Html) trait. If you have a component that renders a whole bunch of HTML and one `i32`,
//! only that one `i32` is diffed between previous and current render and updated in DOM if necessary.
//!
//! ### Hello World
//!
//! Any struct that implements a `render` method can be used as a component:
//!
//! ```rust
//! use kobold::prelude::*;
//!
//! struct Hello {
//!     name: &'static str,
//! }
//!
//! impl Hello {
//!     fn render(self) -> impl Html {
//!         html! {
//!             <h1>"Hello "{ self.name }"!"</h1>
//!         }
//!     }
//! }
//!
//! fn main() {
//!     kobold::start(html! {
//!         <Hello name={"Kobold"} />
//!     });
//! }
//! ```

pub use kobold_macros::{html, branching};

use wasm_bindgen::JsValue;

mod render_fn;
mod util;
mod value;

pub mod attribute;
pub mod branch;
pub mod dom;
pub mod list;
pub mod stateful;

/// The prelude module with most commonly used types.
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

/// Trait that describes types that can be rendered in the DOM.
pub trait Html: Sized {
    /// HTML product of this type, this is effectively the strongly-typed
    /// virtual DOM equivalent for Kobold.
    type Product: Mountable;

    /// Build a product that can be mounted in the DOM from this type.
    fn build(self) -> Self::Product;

    /// Update the product and apply changes to the DOM if necessary.
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
