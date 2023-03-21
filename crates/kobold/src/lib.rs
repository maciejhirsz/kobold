//! <img src="https://raw.githubusercontent.com/maciejhirsz/kobold/master/kobold.svg?sanitize=true" alt="Kobold logo" width="250" align="right" style="max-width: 40vw;">
//!
//! # Kobold
//!
//! **Kobold** uses macros to deliver familiar HTML-esque syntax for building declarative web interfaces,
//! while leveraging Rust's powerful type system for safety and performance.
//!
//! ### Zero-cost static HTML
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
//! ### Hello World!
//!
//! Components in **Kobold** are created by annotating a _render function_ with a [`#[component]`](component) attribute.
//!
//! ```no_run
//! use kobold::prelude::*;
//!
//! #[component]
//! fn Hello(name: &str) -> impl Html + '_ {
//!     html! {
//!         <h1>"Hello "{ name }"!"</h1>
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
//! The _render function_ must return a type that implements the [`Html`](Html) trait. Since the [`html!`](html) macro
//! produces _transient types_, or [_Voldemort types_](https://wiki.dlang.org/Voldemort_types), the best approach
//! here is to always use the `impl Html` return type.
//!
//! Everything here is statically typed and the macro doesn't delete any information when manipulating the
//! token stream, so the Rust compiler can tell you when you've made a mistake:
//!
//! ```text
//! error[E0560]: struct `Hello` has no field named `nam`
//!   --> examples/hello_world/src/main.rs:12:16
//!    |
//! 12 |         <Hello nam="Kobold" />
//!    |                ^^^ help: a field with a similar name exists: `name`
//! ```
//!
//! You can even use [rust-analyzer](https://rust-analyzer.github.io/) to refactor component or field names,
//! and it will change the invocations inside the macros for you.
//!
//! ### Stateful components
//!
//! The [`stateful`](stateful::stateful) function can be used to create components that own and manipulate
//! their state:
//!
//! ```no_run
//! use kobold::prelude::*;
//!
//! #[component]
//! fn Counter() -> impl Html {
//!     stateful(0_u32, |count| {
//!         let onclick = count.bind(|count, _event| *count += 1);
//!
//!         html! {
//!             <p>
//!                 "You clicked on the "
//!                 // `{onclick}` here is shorthand for `onclick={onclick}`
//!                 <button {onclick}>"Button"</button>
//!                 " "{ count }" times."
//!             </p>
//!         }
//!     })
//! }
//!
//! fn main() {
//!     kobold::start(html! {
//!         <Counter />
//!     });
//! }
//! ```
//!
//! The [`stateful`](stateful::stateful) function takes two parameters:
//!
//! * State constructor that implements the [`IntoState`](stateful::IntoState) trait. **Kobold** comes with default
//!   implementations for most primitive types, so we can use `u32` here.
//! * The anonymous render function that uses the constructed state, in our case `fn(&Hook<u32>) -> impl Html`.
//!
//! The [`Hook`](stateful::Hook) here is a smart pointer to the state itself that allows non-mutable access to the
//! state, as well as the [`bind`](stateful::Hook::bind) method for creating event callbacks. These take a `&mut`
//! reference to the state and a `&` reference to a DOM [`Event`](event::Event) (ignored above).
//!
//! For more details visit the [`stateful` module documentation](stateful).
//!
//! ### Conditional rendering
//!
//! Because the [`html!`](html) macro produces unique transient types, `if` and `match` expressions that invoke
//! the macro will naturally fail to compile.
//!
//! Using the [`auto_branch`](component#componentauto_branch) flag on the [`#[component]`](component) attribute
//! **Kobold** will scan the body of of your component render function, and make all [`html!`](html) macro invocations
//! inside an `if` or `match` expression, and wrap them in an enum making them the same type:
//!
//!
//! ```
//! # use kobold::prelude::*;
//! #[component(auto_branch)]
//! fn Conditional(illuminatus: bool) -> impl Html {
//!     if illuminatus {
//!         html! { <p>"It was the year when they finally immanentized the Eschaton."</p> }
//!     } else {
//!         html! { <blockquote>"It was love at first sight."</blockquote> }
//!     }
//! }
//! ```
//!
//! For more details visit the [`branching` module documentation](branching).
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
//! #[component]
//! fn IterateNumbers(count: u32) -> impl Html {
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
//! On updates the iterator is consumed once and all items are diffed with the previous version.
//! No allocations are made by **Kobold** when updating such a list, unless the rendered list needs
//! to grow past its original capacity.
//!
//! ### Borrowed values
//!
//! [`Html`](Html) types are truly transient and only need to live for the duration of the initial render,
//! or for the duration of the subsequent update. This means that you can easily and cheaply render borrowed
//! state without unnecessary clones:
//!
//! ```
//! # use kobold::prelude::*;
//! #[component]
//! fn Users<'a>(names: &'a [&'a str]) -> impl Html + 'a {
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
//! If you wish to capture children from parent [`html!`](html) invocation, simply change
//! `#[component]` to `#[component(children)]`:
//!
//! ```no_run
//! use kobold::prelude::*;
//!
//! #[component(children)]
//! fn Header(children: impl Html) -> impl Html {
//!     html! {
//!         <header><h1>{ children }</h1></header>
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
//! You can change the name of the function argument used, or even set a concrete type:
//!
//! ```no_run
//! use kobold::prelude::*;
//!
//! // Capture children into the argument `n`
//! #[component(children: n)]
//! fn AddTen(n: i32) -> i32 {
//!     // integers implement `Html` so they can be passed by value
//!     n + 10
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
//! cd examples/todomvc
//!
//! ## Run with trunk
//! trunk serve
//! ```

#![doc(html_logo_url = "https://maciej.codes/kosz/kobold.png")]

/// The `#[component]` attribute macro that transforms functions into proper components.
///
/// ## Example
/// ```
/// # use kobold::prelude::*;
/// #[component]
/// fn MyComponent() -> impl Html {
///     html! {
///         <p>"Hello, world!"</p>
///     }
/// }
/// ```
///
/// ## Flags
///
/// The `#[component]` attribute accepts a few optional flags using syntax: `#[component(<flag>)]`.
/// Multiple comma-separated flags can be used at once.
///
/// ### `#[component(auto_branch)]`
///
/// Automatically resolve all invocations of the [`html!`](html) macro inside `if` and `match` expressions
/// to the same type.
///
/// For more details visit the [`branching` module documentation](branching).
///
/// ### `#[component(children)]`
///
/// Turns the component into a component that accepts children. Available syntax:
///
/// * `#[component(children)]`: children will be captured by the `children` argument on the function.
/// * `#[component(children: my_name)]`: children will be captured by the `my_name` argument on the function.
pub use kobold_macros::component;

/// Macro for creating transient [`Html`](Html) types. See the [main documentation](crate) for details.
pub use kobold_macros::html;

use wasm_bindgen::{JsCast, JsValue};

mod value;

pub mod attribute;
pub mod branching;
pub mod dom;
pub mod event;
pub mod list;
pub mod stateful;
pub mod util;

/// The prelude module with most commonly used types
pub mod prelude {
    pub use crate::event::{Event, KeyboardEvent, MouseEvent};
    pub use crate::list::ListIteratorExt as _;
    pub use crate::stateful::{stateful, Hook, IntoState, Signal, Then};
    pub use crate::{bind, class};
    // pub use crate::stateful::{ShouldRender, WeakHook};
    // pub use crate::stateful::{stateful, Hook, IntoState, ShouldRender, WeakHook};
    pub use crate::value::{StrExt as _, Stringify as _};
    pub use crate::{component, html, Html};
}

use dom::Element;

/// Crate re-exports for the [`html!`](html) macro internals
pub mod reexport {
    pub use wasm_bindgen;
    pub use web_sys;
}

/// Trait that describes types that can be rendered in the DOM.
pub trait Html {
    /// HTML product of this type, this is effectively the strongly-typed
    /// virtual DOM equivalent for Kobold.
    type Product: Mountable;

    /// Build a product that can be mounted in the DOM from this type.
    fn build(self) -> Self::Product;

    /// Update the product and apply changes to the DOM if necessary.
    fn update(self, p: &mut Self::Product);

    fn on_mount<F>(self, handler: F) -> OnMount<Self, F>
    where
        F: FnOnce(&<Self::Product as Mountable>::Js),
        Self: Sized,
    {
        OnMount {
            html: self,
            handler,
        }
    }

    fn on_render<F>(self, handler: F) -> OnRender<Self, F>
    where
        F: FnOnce(&<Self::Product as Mountable>::Js),
        Self: Sized,
    {
        OnRender {
            html: self,
            handler,
        }
    }
}

pub struct OnMount<H, F> {
    html: H,
    handler: F,
}

impl<H, F> Html for OnMount<H, F>
where
    H: Html,
    F: FnOnce(&<H::Product as Mountable>::Js),
{
    type Product = H::Product;

    fn build(self) -> Self::Product {
        let prod = self.html.build();

        (self.handler)(prod.el().unchecked_ref());

        prod
    }

    fn update(self, p: &mut Self::Product) {
        self.html.update(p);
    }
}

pub struct OnRender<H, F> {
    html: H,
    handler: F,
}

impl<H, F> Html for OnRender<H, F>
where
    H: Html,
    F: FnOnce(&<H::Product as Mountable>::Js),
{
    type Product = H::Product;

    fn build(self) -> Self::Product {
        let prod = self.html.build();

        (self.handler)(prod.el().unchecked_ref());

        prod
    }

    fn update(self, p: &mut Self::Product) {
        self.html.update(p);

        (self.handler)(p.el().unchecked_ref());
    }
}

/// A type that can be mounted in the DOM
pub trait Mountable: 'static {
    type Js: JsCast;

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

#[macro_export]
macro_rules! class {
    ($class:literal if $on:expr) => {
        ::kobold::attribute::OptionalClass::new($class, $on).no_diff()
    };
    ($class:literal) => {
        $class.no_diff()
    };
    ($class:tt if $on:expr) => {
        ::kobold::attribute::OptionalClass::new($class, $on)
    };
    ($class:expr) => {
        $class
    };
}

#[macro_export]
macro_rules! bind {
    ($hook:ident: $(let $v:ident = move |$e:tt $(: $e_ty:ty)?| $body:expr;)+) => {
        $(
            #[allow(unused_variables)]
            let $v = $hook.bind(move |$hook, $e $(: $e_ty)*| $body);
        )*
    };
}
