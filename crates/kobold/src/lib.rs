// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! <img src="https://raw.githubusercontent.com/maciejhirsz/kobold/master/kobold.svg?sanitize=true" alt="Kobold logo" width="250" align="right" style="max-width: 40vw;">
//!
//! # Kobold
//!
//! **Kobold** uses macros to deliver familiar HTML-esque syntax for building declarative web interfaces,
//! while leveraging Rust's powerful type system for safety and performance.
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
//! _If the [`view!`](view) macro invocation contains DOM elements with no expressions, the constructed [`View`](View)
//! type will be zero-sized, and its [`View::update`](View::update) method will be empty, making updates of static
//! HTML literally zero-cost._
//!
//! ### Hello World!
//!
//! Components in **Kobold** are created by annotating a _render function_ with a [`#[component]`](component) attribute.
//!
//! ```no_run
//! use kobold::prelude::*;
//!
//! #[component]
//! fn Hello(name: &str) -> impl View + '_ {
//!     view! {
//!         <h1>"Hello "{ name }"!"</h1>
//!     }
//! }
//!
//! fn main() {
//!     kobold::start(view! {
//!         <Hello name="Kobold" />
//!     });
//! }
//! ```
//!
//! The component function must return a type that implements the [`View`](View) trait. Since the [`view!`](view) macro
//! produces transient locally defined types the best approach here is to always use the opaque `impl View` return type.
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
//! ### Stateful
//!
//! The [`stateful`](stateful::stateful) function can be used to create views that own and manipulate
//! their state:
//!
//! ```no_run
//! use kobold::prelude::*;
//!
//! #[component]
//! fn Counter(init: u32) -> impl View {
//!     stateful(init, |count| {
//!         bind! { count:
//!             // Create an event handler with access to `&mut u32`
//!             let onclick = move |_event| *count += 1;
//!         }
//!
//!         view! {
//!             <p>
//!                 "You clicked the "
//!                 // `{onclick}` here is shorthand for `onclick={onclick}`
//!                 <button {onclick}>"Button"</button>
//!                 " "{ count }" times."
//!             </p>
//!         }
//!     })
//! }
//!
//! fn main() {
//!     kobold::start(view! {
//!         <Counter init={0} />
//!     });
//! }
//! ```
//!
//! The [`stateful`](stateful::stateful) function takes two parameters:
//!
//! * State constructor that implements the [`IntoState`](stateful::IntoState) trait. **Kobold** comes with default
//!   implementations for most primitive types, so we can use `u32` here.
//! * The anonymous render closure that uses the constructed state, in our case its argument is `&Hook<u32>`.
//!
//! The [`Hook`](stateful::Hook) here is a smart pointer to the state itself that allows non-mutable access to the
//! state. The [`bind!`](bind) macro can be invoked for any `Hook` to create closures with `&mut` references to the
//! underlying state.
//!
//! For more details visit the [`stateful` module documentation](stateful).
//!
//! ### Optional parameters
//!
//! Use `#[component(<param>?)]` syntax to set a component parameter as default:
//!
//! ```
//! # use kobold::prelude::*;
//! // `code` will default to `200` if omitted
//! #[component(code?: 200)]
//! fn Status(code: u32) -> impl View {
//!     view! {
//!         <p> "Status code was "{ code }
//!     }
//! }
//!
//! # let _ =
//! view! {
//!     // Status code was 200
//!     <Status />
//!     // Status code was 404
//!     <Status code={404} />
//! }
//! # ;
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
//! fn Conditional(illuminatus: bool) -> impl View {
//!     if illuminatus {
//!         view! { <p> "It was the year when they finally immanentized the Eschaton." }
//!     } else {
//!         view! { <blockquote> "It was love at first sight." }
//!     }
//! }
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
//! fn IterateNumbers(count: u32) -> impl View {
//!     view! {
//!         <ul>
//!         {
//!             for (1..=count).map(|n| view! { <li> "Item #"{n} })
//!         }
//!     }
//! }
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
//! [`View`](View) types are truly transient and only need to live for the duration of the initial render,
//! or for the duration of the subsequent update. This means that you can easily and cheaply render borrowed
//! state without unnecessary clones:
//!
//! ```
//! # use kobold::prelude::*;
//! #[component]
//! fn Users<'a>(names: &'a [&'a str]) -> impl View + 'a {
//!     view! {
//!         <ul>
//!         {
//!             for names.iter().map(|name| view! { <li> { name } })
//!         }
//!     }
//! }
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
//! #[component(children)]
//! fn Header(children: impl View) -> impl View {
//!     view! {
//!         <header><h1>{ children }</h1></header>
//!     }
//! }
//!
//! fn main() {
//!     kobold::start(view! {
//!         <Header>"Hello Kobold"</Header>
//!     });
//! }
//! ```
//!
//! You can change the name of the parameter used and even set it to a concrete:
//!
//! ```no_run
//! use kobold::prelude::*;
//!
//! // Capture children into the argument `n`
//! #[component(children: n)]
//! fn AddTen(n: i32) -> i32 {
//!     // integers implement `View` so they can be passed by value
//!     n + 10
//! }
//!
//! fn main() {
//!     kobold::start(view! {
//!         <p>
//!             "Meaning of life is "
//!             <AddTen>{ 32 }</AddTen>
//!         </p>
//!     });
//! }
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
/// fn MyComponent() -> impl View {
///     view! {
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
/// ### Optional parameters: `#[component(<param>?)]`
///
/// Allows for parameters to have default values. Available syntax:
///
/// * `#[component(foo?)]`: mark the parameter `foo` as optional, use [`Default`](Default) trait implementation if absent.
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
/// fn Greeter<'a>(name: &'a str, age: Option<u32>) -> impl View + 'a {
///     let age = age.map(|age| view!(", you are "{ age }" years old"));
///
///     view! {
///         <p> "Hello "{ name }{ age }
///     }
/// }
///
/// # let _ =
/// view! {
///     // Hello Kobold
///     <Greeter />
///     // Hello Alice
///     <Greeter name="Alice" />
///     // Hello Bob, you are 42 years old
///     <Greeter name="Bob" age={42} />
/// }
/// # ;
/// ```
///
/// Optional parameters of any type `T` can be set using any type that implements
/// [`Maybe<T>`](crate::maybe::Maybe).
///
/// This allows you to set optional parameters using an [`Option`](Option):
/// ```
/// # use kobold::prelude::*;
/// #[component(code?: 200)]
/// fn StatusCode(code: u32) -> impl View {
///     view! {
///         <p> "Status code was "{ code }
///     }
/// }
///
/// # let _ =
/// view! {
///     // Status code was 200
///     <StatusCode />
///     // Status code was 404
///     <StatusCode code={404} />
///
///     // Status code was 200
///     <StatusCode code={None} />
///     // Status code was 500
///     <StatusCode code={Some(500)} />
/// }
/// # ;
/// ```
///
/// All values are lazy-evaluated:
///
/// ```
/// # use kobold::prelude::*;
/// // The owned `String` will only be created if the `name` is not set.
/// #[component(name?: "Kobold".to_string())]
/// fn Greeter(name: String) -> impl View {
///     view! {
///         <p> "Hello "{ name }
///     }
/// }
/// ```
///
/// #### 💡 Note:
///
/// You can only mark types that implement the [`Default`](Default) trait as optional, even if you provide
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

/// Macro for creating transient [`View`](View) types. See the [main documentation](crate) for details.
pub use kobold_macros::view;

use wasm_bindgen::JsCast;

pub mod attribute;
pub mod branching;
pub mod diff;
pub mod dom;
pub mod event;
pub mod internal;
pub mod keywords;
pub mod list;
pub mod maybe;

mod value;

#[cfg(feature = "stateful")]
pub mod stateful;

use internal::{In, Out};

/// The prelude module with most commonly used types.
///
/// Intended use is:
/// ```
/// use kobold::prelude::*;
/// ```
pub mod prelude {
    pub use crate::event::{Event, KeyboardEvent, MouseEvent};
    pub use crate::{bind, class};
    pub use crate::{component, view, View};

    #[cfg(feature = "stateful")]
    pub use crate::stateful::{stateful, Hook, IntoState, Signal, Then};
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
    fn build(self, p: In<Self::Product>) -> Out<Self::Product>;

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

    fn build(self, p: In<Self::Product>) -> Out<Self::Product> {
        let prod = self.view.build(p);

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

    fn build(self, p: In<Self::Product>) -> Out<Self::Product> {
        let prod = self.view.build(p);

        (self.handler)(prod.js().unchecked_ref());

        prod
    }

    fn update(self, p: &mut Self::Product) {
        self.view.update(p);

        (self.handler)(p.js().unchecked_ref());
    }
}

/// Start the Kobold app by mounting given [`View`](View) in the document `body`.
pub fn start(view: impl View) {
    init_panic_hook();

    use std::mem::MaybeUninit;
    use std::pin::pin;

    let product = pin!(MaybeUninit::uninit());
    let product = In::pinned(product, move |p| view.build(p));

    internal::append_body(product.js());
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
        ::kobold::attribute::OptionalClass::new($class, $on)
    };
    ($class:tt if $on:expr) => {
        ::kobold::attribute::OptionalClass::new($class, $on)
    };
}

/// Binds a closure to a given [`Hook`](stateful::Hook). In practice:
///
/// ```
/// # use kobold::{bind, stateful::Hook};
/// # fn test(count: &Hook<i32>) {
/// bind! { count:
///     let increment = move |_| *count += 1;
///     let decrement = move |_| *count -= 1;
/// }
/// # fn throwaway(_: impl kobold::event::Listener<kobold::reexport::web_sys::Event>) {}
/// # throwaway(increment);
/// # throwaway(decrement);
/// # }
/// ```
/// Desugars into:
///
/// ```
/// # use kobold::{bind, stateful::Hook};
/// # fn test(count: &Hook<i32>) {
/// let increment = count.bind(move |count, _| *count += 1);
/// let decrement = count.bind(move |count, _| *count -= 1);
/// # fn throwaway(_: impl kobold::event::Listener<kobold::reexport::web_sys::Event>) {}
/// # throwaway(increment);
/// # throwaway(decrement);
/// # }
/// ```
#[macro_export]
macro_rules! bind {
    ($hook:ident: $(let $v:ident = move |$e:tt $(: $e_ty:ty)?| $body:expr;)+) => {
        $(
            let $v = $hook.bind(move |$hook, $e $(: $e_ty)*| $body);
        )*
    };
}
