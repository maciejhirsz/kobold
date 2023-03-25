//! <img src="https://raw.githubusercontent.com/maciejhirsz/kobold/master/kobold.svg?sanitize=true" alt="Kobold logo" width="250" align="right" style="max-width: 40vw;">
//!
//! # Kobold
//!
//! **Kobold** uses macros to deliver familiar JSX-esque syntax for building declarative web interfaces,
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
//! value ([or pointer](crate::diff::StrExt::fast_diff)) and surgically update the DOM should they change. Changing a
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
//!         view! { <p>"It was the year when they finally immanentized the Eschaton."</p> }
//!     } else {
//!         view! { <blockquote>"It was love at first sight."</blockquote> }
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
//! fn IterateNumbers(count: u32) -> impl View {
//!     view! {
//!         <ul>
//!         {
//!             (1..=count)
//!                 .map(|n| view! { <li>"Item #"{n}</li> })
//!                 .list()
//!         }
//!         </ul>
//!     }
//! }
//! ```
//!
//! This wraps the iterator in the transparent [`List<_>`](list::List) type that implements [`View`](View).
//! On updates the iterator is consumed once and all items are diffed with the previous version.
//! No allocations are made by **Kobold** when updating such a list, unless the rendered list needs
//! to grow past its original capacity.
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
//!             names
//!                 .iter()
//!                 .map(|name| view! { <li>{ name }</li> })
//!                 .list()
//!         }
//!         </ul>
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
/// ### `#[component(auto_branch)]`
///
/// Automatically resolve all invocations of the [`view!`](view) macro inside `if` and `match` expressions
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

/// Macro for creating transient [`View`](View) types. See the [main documentation](crate) for details.
pub use kobold_macros::view;

use wasm_bindgen::{JsCast, JsValue};

pub mod attribute;
pub mod branching;
pub mod diff;
pub mod dom;
pub mod event;
pub mod list;
pub mod util;

mod value;

pub use value::Value;

#[cfg(feature = "stateful")]
pub mod stateful;

/// The prelude module with most commonly used types.
///
/// Intended use is:
/// ```
/// use kobold::prelude::*;
/// ```
pub mod prelude {
    pub use crate::event::{Event, KeyboardEvent, MouseEvent};
    pub use crate::list::ListIteratorExt as _;
    pub use crate::diff::StrExt as _;
    pub use crate::{bind, class};
    pub use crate::{component, view, View};

    #[cfg(feature = "stateful")]
    pub use crate::stateful::{stateful, Hook, IntoState, Signal, Then};
}

use dom::Element;

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
            html: self,
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
            html: self,
            handler,
        }
    }
}

pub struct OnMount<H, F> {
    html: H,
    handler: F,
}

impl<H, F> View for OnMount<H, F>
where
    H: View,
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

impl<H, F> View for OnRender<H, F>
where
    H: View,
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

/// Start the Kobold app by mounting given [`View`](View) in the document `body`.
pub fn start(html: impl View) {
    init_panic_hook();

    use std::mem::ManuallyDrop;

    let product = ManuallyDrop::new(html.build());

    util::append_body(product.js());
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
/// # fn throwaway(_: impl Fn(kobold::reexport::web_sys::Event)) {}
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
/// # fn throwaway(_: impl Fn(kobold::reexport::web_sys::Event)) {}
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
