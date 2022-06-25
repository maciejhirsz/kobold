//! <img src="https://raw.githubusercontent.com/maciejhirsz/kobold/master/kobold.svg?sanitize=true" alt="Kobold logo" width="250" align="right" style="max-width: 40vw;">
//!
//! # Kobold
//!
//! **Kobold** uses macros to deliver familiar HTML-esque syntax for building declarative web interfaces,
//! while leveraging Rust's powerful type system for safety and performance.
//!
//! ### Zero-Cost static HTML
//!
//! Like in [React](https://reactjs.org/) or [Yew](https://yew.rs/) updates are done by repeating calls
//! to a render function whenever the state changes. However, unlike either, **Kobold** does not produce a
//! full blown [virtual DOM](https://en.wikipedia.org/wiki/Virtual_DOM). Instead the [`html!`](html) macro compiles
//! all static HTML elements to a single JavaScript function that constructs the exact
//! [DOM](https://developer.mozilla.org/en-US/docs/Web/API/Document_Object_Model) for it.
//!
//! All expressions, which must implement the [`Html`](Html) trait, are injected into the constructed DOM on first
//! render. Kobold keeps track of the DOM node references for these expressions. Since the exact types the
//! expressions evaluate to are known to the Rust compiler, update calls can diff them by value and surgically
//! update the DOM should they change. Changing a string or an integer only updates the exact
//! [`Text` node](https://developer.mozilla.org/en-US/docs/Web/API/Text) that string or integer was rendered to.
//!
//! _If the [`html!`](html) macro invocation contains HTML elements with no expressions, the constructed [`Html`](Html)
//! type will be zero-sized, and its [`Html::update`](Html::update) method will be empty, making updates of static
//! HTML quite literally zero-cost._
//!
//! ### Hello World
//!
//! Any struct that implements a `render` method can be used as a component:
//!
//! ```no_run
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
//!         <Hello name="Kobold" />
//!     });
//! }
//! ```
//!
//! The `render` method here will return a transient type that contains _only_ the `&'static str` from
//! the `{ self.name }` expression. Kobold will create a text node for that string, and then send it to
//! a compiled JavaScript function that will build the `h1` element with the static text around it.
//!
//! Everything is statically typed and the macro doesn't delete any information when manipulating the
//! token stream, so the Rust compiler can tell you if you've made a mistake:
//!
//! ```text
//! error[E0560]: struct `Hello` has no field named `nam`
//!   --> examples/hello_world/src/main.rs:17:16
//!    |
//! 17 |         <Hello nam="Kobold" />
//!    |                ^^^ help: a field with a similar name exists: `name`
//! ```
//!
//! You can even use [rust-analyzer](https://rust-analyzer.github.io/) to refactor component or field names,
//! and it will change the invocations inside the macros for you.
//!
//! ### Stateful Components
//!
//! The [`Stateful`](stateful::Stateful) trait can be used to create components that own and manipulate
//! their state:
//!
//! ```no_run
//! use kobold::prelude::*;
//!
//! // To derive `Stateful` the component must also implement `PartialEq`.
//! #[derive(Stateful, PartialEq, Default)]
//! struct Counter {
//!     count: u32,
//! }
//!
//! impl Counter {
//!     fn render(self) -> impl Html {
//!         self.stateful(|state, link| {
//!             let onclick = link.callback(|state, _event| state.count += 1);
//!
//!             html! {
//!                 <p>
//!                     "You clicked on the "
//!                     // `{onclick}` here is shorthand for `onclick={onclick}`
//!                     <button {onclick}>"Button"</button>
//!                     " "{ state.count }" times."
//!                 </p>
//!             }
//!         })
//!     }
//! }
//!
//! fn main() {
//!     kobold::start(html! {
//!         // The `..` notation fills in the rest of the component with
//!         // values from the `Default` impl.
//!         <Counter ../>
//!     });
//! }
//! ```
//!
//! The [`stateful`](stateful::Stateful::stateful) method above accepts a non-capturing anonymous render function
//! matching the signature:
//!
//! ```text
//! fn(&State, Link<State>) -> impl Html
//! ```
//!
//! The [`State`](stateful::Stateful::State) here is an associated type which for all components that
//! use derived [`Stateful`](stateful::Stateful) implementation defaults to `Self`, so in the example above
//! it is the `Counter` itself.
//!
//! The [`Link`](stateful::Link) can be used to create event callbacks that take a `&mut` reference to the
//! state and a `&` reference to a DOM [`Event`](web_sys::Event) (ignored above). If the callback closure has no
//! return type (the return type is `()`) each invocation of it will update the component. If you would
//! rather perform a "silent" update, or if the callback does not always modify the state, return the provided
//! [`ShouldRender`](stateful::ShouldRender) enum instead.
//!
//! For more details visit the [`stateful` module documentation](stateful).
//!
//! ### Conditional Rendering
//!
//! Because the [`html!`](html) macro produces unique transient types, `if` and `match` expressions that invoke
//! the macro will naturally fail to compile. To fix this annotate a function with [`#[kobold::branching]`](macro@branching):
//!
//! ```
//! # use kobold::prelude::*;
//! #[kobold::branching]
//! fn conditional(illuminatus: bool) -> impl Html {
//!     if illuminatus {
//!         html! { <p>"It was the year when they finally immanentized the Eschaton."</p> }
//!     } else {
//!         html! { <blockquote>"It was love at first sight."</blockquote> }
//!     }
//! }
//! ```
//!
//! For more details visit the [`branching` module documentation](mod@branching).
//!
//! ### Lists and Iterators
//!
//! To render an iterator use the [`list`](list::ListIteratorExt::list) method from the
//! [`ListIteratorExt`](list::ListIteratorExt) extension trait:
//!
//! ```
//! // `ListIteratorExt` is included in the prelude
//! use kobold::prelude::*;
//!
//! fn make_list(count: u32) -> impl Html {
//!     html! {
//!         <ul>
//!         {
//!             (1..=count)
//!                 .map(|n| html! { <li>"Item #"{n}</li> })
//!                 .list()
//!         }
//!         </ul>
//!     }
//! }
//! ```
//!
//! This wraps the iterator in the transparent [`List<_>`](list::List) type that implements [`Html`](Html).
//! On updates the iterator is consumed once and all items are diffed with previous version.
//! No allocations are made by **Kobold** unless the rendered list needs to grow past its original capacity.
//!
//! ### Borrowed Values
//!
//! [`Html`](Html) types are truly transient and only need to live for the duration of the initial render,
//! or for the duration of the subsequent update. This means that you can easily and cheaply render borrowed
//! state without unnecessary clones:
//!
//! ```
//! # use kobold::prelude::*;
//! // Need to mark the return type with an elided lifetime
//! // to tell the compiler that we borrow from `names` here
//! fn render_names(names: &[String]) -> impl Html + '_ {
//!     html! {
//!         <ul>
//!         {
//!             names
//!                 .iter()
//!                 .map(|name| html! { <li>{ name }</li> })
//!                 .list()
//!         }
//!         </ul>
//!     }
//! }
//! ```
//!
//! ### Components with children
//!
//! If you wish to capture children from parent [`html!`](html) invocation, simply implement
//! a `render_with` method on the component:
//!
//! ```no_run
//! use kobold::prelude::*;
//!
//! struct Header;
//!
//! impl Header {
//!     fn render_with(self, children: impl Html) -> impl Html {
//!         html! {
//!             <header><h1>{ children }</h1></header>
//!         }
//!     }
//! }
//!
//! fn main() {
//!     kobold::start(html! {
//!         <Header>"Hello Kobold"</Header>
//!     });
//! }
//! ```
//!
//! If you know or expect children to be of a specific type, you can do that too:
//!
//! ```no_run
//! use kobold::prelude::*;
//!
//! struct AddTen;
//!
//! impl AddTen {
//!     // integers implement `Html` so they can be passed by value
//!     fn render_with(self, n: i32) -> i32 {
//!         n + 10
//!     }
//! }
//!
//! fn main() {
//!     kobold::start(html! {
//!         <p>
//!             "Meaning of life is "
//!             <AddTen>{ 32 }</AddTen>
//!         </p>
//!     });
//! }
//! ```
//!
//! A component can have both `render` and `render_with` methods if you want to
//! support both styles of invocation.
//!
//! ## More examples
//!
//! To run **Kobold** you'll need to install [`trunk`](https://trunkrs.dev/):
//! ```sh
//! cargo install --locked trunk
//! ```
//!
//! You might also need to add the Wasm target to Rust:
//! ```sh
//! rustup target add wasm32-unknown-unknown
//! ```
//!
//! Then just run an example:
//! ```sh
//! ## Go to an example
//! cd examples/counter
//!
//! ## Run with trunk
//! trunk serve
//! ```

/// Macro for resolving branching issues with the [`html!`](html) macro. See the [`branching` module documentation](mod@branching) for details.
pub use kobold_macros::branching;
/// Macro for creating transient [`Html`](Html) types. See the [main documentation](crate) for details.
pub use kobold_macros::html;

use wasm_bindgen::JsValue;

mod render_fn;
mod util;
mod value;

pub mod attribute;
pub mod branching;
pub mod dom;
pub mod event;
pub mod list;
pub mod stateful;

/// The prelude module with most commonly used types
pub mod prelude {
    pub use crate::event::Event;
    pub use crate::list::ListIteratorExt;
    pub use crate::stateful::{Link, ShouldRender, Stateful};
    pub use crate::{html, Html};
}

use dom::Element;

/// Crate re-exports for the [`html!`](html) macro internals
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
}

/// A type that can be mounted in the DOM
pub trait Mountable: 'static {
    fn el(&self) -> &Element;

    fn js(&self) -> &JsValue {
        self.el().anchor()
    }
}

/// Start the Kobold app by mounting given [`Html`](Html) in the document `body`.
pub fn start(html: impl Html) {
    init_panic_hook();

    use std::mem::ManuallyDrop;

    let product = ManuallyDrop::new(html.build());

    util::__kobold_start(product.js());
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
