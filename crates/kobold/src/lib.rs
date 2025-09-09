// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! <img src="https://raw.githubusercontent.com/maciejhirsz/kobold/master/kobold.svg?sanitize=true" alt="Kobold logo" width="250" align="right" style="max-width: 40vw;">
//!
//! # Kobold
//!
//! _Easy declarative web interfaces._
//!
//! Key features:
//!
//! * Declarative [`view!`](view) macro that uses HTML-esque syntax with optional closing tags.
//! * Functional [components](component) with optional parameters.
//! * State management and event handling.
//! * High performance and consistently the lowest Wasm footprint in the Rust ecosystem.
//!
//! ### Zero-Cost Static HTML
//!
//! The [`view!`](view) macro produces opaque [`impl View`](View) types that by default do no allocations.
//! All static [DOM](https://developer.mozilla.org/en-US/docs/Web/API/Document_Object_Model) elements compile to
//! inline JavaScript code that constructs them. Expressions are injected into the constructed DOM on first render.
//! Kobold keeps track of the DOM node references for these expressions.
//!
//! Since the exact types the expressions evaluate to are known to the Rust compiler, update calls can diff them by
//! value ([or reference](crate::keywords::ref)) and surgically update the DOM should they change. Changing a
//! string or an integer only updates the exact [`Text` node](https://developer.mozilla.org/en-US/docs/Web/API/Text)
//! that string or integer was rendered to.
//!
//! _If the [`view!`](view) macro invocation contains DOM elements with no expressions, the constructed [`View`]
//! type will be zero-sized, and its [`View::update`] method will be empty, making updates of static
//! HTML literally zero-cost._
//!
//! ### Example
//!
//! Components in **Kobold** are created by annotating a _render function_ with a [`#[component]`](component) attribute.
//!
//! ```no_run
//! use kobold::prelude::*;
//!
//! #[component]
//! fn app() -> impl View {
//!     let count = state!(0_i32);
//!
//!     view! {
//!         <p>
//!             <h3>"Counter is at "{ count }</h3>
//!             <button onclick={do *count -= 1}>"Decrement"</button>
//!             <button onclick={do *count += 1}>"Increment"</button>
//!     }
//! }
//!
//! kobold::start!(app);
//! # fn main() {}
//! ```
//!
//! The component function must return a type that implements the [`View`] trait. Since the [`view!`](view) macro
//! produces transient locally defined types the best approach here is to always use the opaque `impl View` return type.
//!
//! The [`state!`](macro.state.html) macro invocation creates a [`&Hook<i32>`](state::Hook) reference which can be freely read from.
//!
//! The [`do` keyword](keywords/macro.do.html) is a shorthand for the [`event!](macro.event.html) macro used to create a handler for a DOM event.
//!
//! ### Optional parameters
//!
//! Use `#[component(<param>?)]` syntax to set a component parameter as default:
//!
//! ```
//! # use kobold::prelude::*;
//! // `code` will default to `200` if omitted
//! #[component(code?: 200)]
//! fn status(code: u32) -> impl View {
//!     view! {
//!         <p> "Status code was "{ code }
//!     }
//! }
//!
//! kobold::start!(|| {
//!     view! {
//!         // Status code was 200
//!         <!status>
//!         // Status code was 404
//!         <!status code={404}>
//!     }
//! });
//! # fn main() {}
//! ```
//!
//! For more details visit the [`#[component]` macro documentation](component#optional-parameters-componentparam).
//!
//! ### Conditional Rendering
//!
//! Because the [`view!`](view) macro produces unique transient types, `if` and `match` expressions that invoke
//! the macro will naturally fail to compile.
//!
//! Using the [`auto_branch`](component#componentauto_branch) flag on the [`#[component]`](component) attribute
//! **Kobold** will scan the body of of your component render function, and make all [`view!`](view) macro invocations
//! inside an `if` or `match` expression, and wrap them in an enum making them the same type:
//!
//! ```
//! # use kobold::prelude::*;
//! #[component(auto_branch)]
//! fn conditional(illuminatus: bool) -> impl View {
//!     if illuminatus {
//!         view! { <p> "It was the year when they finally immanentized the Eschaton." }
//!     } else {
//!         view! { <blockquote> "It was love at first sight." }
//!     }
//! }
//! # fn main() {}
//! ```
//!
//! For more details visit the [`branching` module documentation](branching).
//!
//! ### Lists and Iterators
//!
//! To render an iterator use the [`for`](keywords::for) keyword:
//!
//! ```
//! use kobold::prelude::*;
//!
//! #[component]
//! fn iterate_numbers(count: u32) -> impl View {
//!     view! {
//!         <ul>
//!         {
//!             for (1..=count).map(|n| view! { <li> "Item #"{n} })
//!         }
//!     }
//! }
//! # fn main() {}
//! ```
//!
//! On updates the iterator is consumed once and all items are diffed with the previous version.
//! No allocations are made by **Kobold** when updating such a list, unless the rendered list needs
//! to grow past its original capacity.
//!
//! For more information about keywords visit the [`keywords` module documentation](keywords).
//!
//! ### Borrowed Values
//!
//! [`View`] types are truly transient and only need to live for the duration of the initial render,
//! or for the duration of the subsequent update. This means that you can easily and cheaply render borrowed
//! state without unnecessary clones:
//!
//! ```
//! # use kobold::prelude::*;
//! #[component]
//! fn users<'a>(names: &'a [&'a str]) -> impl View + 'a {
//!     view! {
//!         <ul>
//!         {
//!             for names.iter().map(|name| view! { <li> { name } })
//!         }
//!     }
//! }
//! # fn main() {}
//! ```
//!
//! ### Components with Children
//!
//! If you wish to capture children from parent [`view!`](view) invocation, simply change
//! `#[component]` to `#[component(children)]`:
//!
//! ```no_run
//! use kobold::prelude::*;
//!
//! #[component]
//! fn header(children: impl View) -> impl View {
//!     view! {
//!         <header><h1>{ children }</h1></header>
//!     }
//! }
//!
//! #[component]
//! fn body() -> impl View {
//!     view! {
//!         <!header>"Hello Kobold"</!header>
//!     }
//! }
//! # fn main() {}
//! ```
//!
//! You can change the name of the parameter used and even set it to a concrete:
//!
//! ```no_run
//! use kobold::prelude::*;
//!
//! // Capture children into the argument `n`
//! #[component(children: n)]
//! fn add_ten(n: i32) -> i32 {
//!     // integers implement `View` so they can be passed by value
//!     n + 10
//! }
//!
//! #[component]
//! fn body() -> impl View {
//!     view! {
//!         <p>
//!             "Meaning of life is "
//!             <!add_ten>{ 32 }</!add_ten>
//!         </p>
//!     }
//! }
//! # fn main() {}
//! ```
//!
//! ## More Examples
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
/// fn my_component() -> impl View {
///     view! {
///         <p>"Hello, world!"</p>
///     }
/// }
/// # fn main() {}
/// ```
///
/// ## Flags
///
/// The `#[component]` attribute accepts a few optional flags using syntax: `#[component(<flag>)]`.
/// Multiple comma-separated flags can be used at once.
///
/// ### Optional parameters: `#[component(<param>?)]`
///
/// Allows for parameters to have default values. Available syntax:
///
/// * `#[component(foo?)]`: mark the parameter `foo` as optional, use [`Default`] trait implementation if absent.
/// * `#[component(foo?: <expression>)]`: mark the parameter `foo` as optional, default to `<expression>`.
///
/// #### Examples
/// ```
/// # use kobold::prelude::*;
/// #[component(
///     // Make `name` an optional parameter, defaults to `"Kobold"`
///     name?: "Kobold",
///     // Make `age` an optional parameter, use the `Default` value
///     age?,
/// )]
/// fn greeter<'a>(name: &'a str, age: Option<u32>) -> impl View + 'a {
///     let age = age.map(|age| view!(", you are "{ age }" years old"));
///
///     view! {
///         <p> "Hello "{ name }{ age }
///     }
/// }
///
/// # fn main() { let _ =
/// view! {
///     // Hello Kobold
///     <!greeter>
///     // Hello Alice
///     <!greeter name="Alice">
///     // Hello Bob, you are 42 years old
///     <!greeter name="Bob" age={42}>
/// }
/// # ; }
/// ```
///
/// Optional parameters of any type `T` can be set using any type that implements
/// [`Maybe<T>`](crate::maybe::Maybe).
///
/// This allows you to set optional parameters using an [`Option`]:
/// ```
/// # use kobold::prelude::*;
/// #[component(code?: 200)]
/// fn status_code(code: u32) -> impl View {
///     view! {
///         <p> "Status code was "{ code }
///     }
/// }
///
/// # fn main() { let _ =
/// view! {
///     // Status code was 200
///     <!status_code>
///     // Status code was 404
///     <!status_code code={404}>
///
///     // Status code was 200
///     <!status_code code={None}>
///     // Status code was 500
///     <!status_code code={Some(500)}>
/// }
/// # ; }
/// ```
///
/// All values are lazy-evaluated:
///
/// ```
/// # use kobold::prelude::*;
/// // The owned `String` will only be created if the `name` is not set.
/// #[component(name?: "Kobold".to_string())]
/// fn greeter(name: String) -> impl View {
///     view! {
///         <p> "Hello "{ name }
///     }
/// }
/// # fn main() {}
/// ```
///
/// #### 💡 Note:
///
/// You can only mark types that implement the [`Default`] trait as optional, even if you provide
/// a concrete value using `param?: value`. This requirement might be relaxed in the future when trait
/// specialization is stabilized.
///
/// ### Enable auto-branching: `#[component(auto_branch)]`
///
/// Automatically resolve all invocations of the [`view!`](view) macro inside `if` and `match` expressions
/// to the same type.
///
/// For more details visit the [`branching` module documentation](branching).
///
/// ### Accept children: `#[component(children)]`
///
/// Turns the component into a component that accepts children. Available syntax:
///
/// * `#[component(children)]`: children will be captured by the `children` argument on the function.
/// * `#[component(children: my_name)]`: children will be captured by the `my_name` argument on the function.
pub use kobold_macros::component;

/// Macro for creating transient [`View`] types. See the [main documentation](crate) for details.
pub use kobold_macros::{class, view};

use wasm_bindgen::JsCast;

#[cfg(all(
    target_arch = "wasm32",
    feature = "rlsf",
    not(target_feature = "atomics")
))]
#[global_allocator]
static A: rlsf::SmallGlobalTlsf = rlsf::SmallGlobalTlsf::new();

pub mod attribute;
pub mod branching;
pub mod diff;
pub mod dom;
pub mod event;
pub mod internal;
pub mod keywords;
pub mod list;
pub mod maybe;
pub mod path;
pub mod runtime;

mod value;

pub mod state;

/// The prelude module with most commonly used types.
///
/// Intended use is:
/// ```
/// use kobold::prelude::*;
/// ```
pub mod prelude {
    pub use crate::event::{Event, KeyboardEvent, MouseEvent};
    pub use crate::runtime::Then;
    pub use crate::state::{Hook, IntoState, Signal, stateful};
    pub use crate::{View, component, view};
    pub use crate::{class, event, state};
}

use dom::Mountable;

/// Crate re-exports for the [`view!`](view) macro internals
pub mod reexport {
    pub use wasm_bindgen;
    pub use web_sys;
}

/// Trait that describes types that can be rendered in the DOM.
pub trait View {
    /// The product should contain a DOM reference to this View and
    /// any data it needs to update itself.
    type Product: Mountable;

    /// Build a product that can be mounted in the DOM from this type.
    fn build(self) -> Self::Product;

    /// Update the product and apply changes to the DOM if necessary.
    fn update(self, p: &mut Self::Product);

    /// Once this view is built, do something once.
    fn on_mount<F>(self, handler: F) -> OnMount<Self, F>
    where
        F: FnOnce(&<Self::Product as Mountable>::Js),
        Self: Sized,
    {
        OnMount {
            view: self,
            handler,
        }
    }

    /// Similar to [`on_mount`](View::on_mount) but triggers on every
    /// update, not just initial render.
    fn on_render<F>(self, handler: F) -> OnRender<Self, F>
    where
        F: FnOnce(&<Self::Product as Mountable>::Js),
        Self: Sized,
    {
        OnRender {
            view: self,
            handler,
        }
    }
}

pub struct OnMount<V, F> {
    view: V,
    handler: F,
}

impl<V, F> View for OnMount<V, F>
where
    V: View,
    F: FnOnce(&<V::Product as Mountable>::Js),
{
    type Product = V::Product;

    fn build(self) -> Self::Product {
        let prod = self.view.build();

        (self.handler)(prod.js().unchecked_ref());

        prod
    }

    fn update(self, p: &mut Self::Product) {
        self.view.update(p);
    }
}

pub struct OnRender<V, F> {
    view: V,
    handler: F,
}

impl<V, F> View for OnRender<V, F>
where
    V: View,
    F: FnOnce(&<V::Product as Mountable>::Js),
{
    type Product = V::Product;

    fn build(self) -> Self::Product {
        let prod = self.view.build();

        (self.handler)(prod.js().unchecked_ref());

        prod
    }

    fn update(self, p: &mut Self::Product) {
        self.view.update(p);

        (self.handler)(p.js().unchecked_ref());
    }
}

// TODO: docs!
#[macro_export]
macro_rules! state {
    ($($dat:tt)*) => {
        compile_error!(concat!(
            "`let _ = state!(",
            stringify!($($dat)*),
            ");` statement MUST be at the top of a #[component]"
        ));
    };
}

// TODO: docs!
#[macro_export]
macro_rules! start {
    ($root:expr) => {
        use wasm_bindgen::prelude::wasm_bindgen;
        use $crate::reexport::wasm_bindgen;

        #[wasm_bindgen(start)]
        fn kobold_main() {
            $crate::runtime::start($root);
        }
    };
}

// TODO: docs!
#[macro_export]
macro_rules! event {
    (move |$state:ident| $body:expr) => {
        $state.bind(move |$state, _| $body)
    };

    (move |$state:ident, $e:tt $(: $e_ty:ty)?| $body:expr) => {
        $state.bind(move |$state, $e $(: $e_ty)*| $body)
    };

    (|$state:ident| $body:expr) => {
        $state.bind(|$state, _| $body)
    };

    (|$state:ident, $e:tt $(: $e_ty:ty)?| $body:expr) => {
        $state.bind(|$state, $e $(: $e_ty)*| $body)
    };

    (*$state:ident $($body:tt)+) => {
        $state.bind(move |$state, _| *$state $($body)*)
    };

    ($state:ident $($body:tt)+) => {
        $state.bind(move |$state, _| $state $($body)*)
    };
}
