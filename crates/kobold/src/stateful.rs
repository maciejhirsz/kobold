//! # Utilities for building stateful components
//!
//! **Kobold** has a very simple notion of a component: any struct that has a `render` method
//! is a component. These components are by default transient, meaning they can include borrowed
//! and are discarded on each render call. They are also stateless, meaning **Kobold** doesn't
//! allocate any memory on the heap for them, and there is no way to update them short of
//! the parent component re-creating them on its state update.
//!
//! If you're familiar with [React](https://reactjs.org/) or [Yew](https://yew.rs/) a good way to
//! think about it is that the component structs in **Kobold** are more like _property lists_ or
//! _pure functional components_. They are meant to be fast to write, run, and understand.
//!
//! However an app built entirely from such components wouldn't be very useful, as all it
//! could ever do is render itself once. To get around this the [`Stateful`](Stateful) trait can be
//! implemented on any type either manually, or with `#[derive(Stateful)]` as in the simple example
//! in the [main documentation](crate#stateful-components).
//!
//! ### When to manually implement `Stateful`?
//!
//! The derived version of a stateful component will be good enough for many cases, however
//! to use the derive the component needs to fulfill few main criteria:
//!
//! 1. It must also implement [`PartialEq<Self>`](PartialEq) so that it can be compared to itself.
//! 2. It must live for a `'static` lifetime.
//! 3. All the fields of the state must be constructed every time parent component performs an update.
//!
//! While the first criterion isn't so bad, the second and third can be a real performance killer in
//! case you'd want to use any heap allocated containers such as a [`String`](String), [`Vec`](Vec),
//! or [`HashMap`](std::collections::HashMap).
//!
//! ### Implementing `Stateful`
//!
//! The [`Stateful`](Stateful) trait allows you to define an associated `State` type which can be
//! different from the component itself. That state is put in a heap allocation so that it can be
//! referenced from callbacks. It also allows you to define how and if the state should be updated
//! when the parent component updates:
//!
//! ```no_run
//! use kobold::prelude::*;
//!
//! // This is our component struct, note that it can take arbitrary lifetimes.
//! struct Borrowing<'a> {
//!     name: &'a str,
//! }
//!
//! // This is our owned state, it must live for a `'static` lifetime, and may
//! // contain different fields than those on the component.
//! struct OwnedState {
//!     name: String,
//! }
//!
//! impl Stateful for Borrowing<'_> {
//!     // We define that `OwnedState` is the state for this component
//!     type State = OwnedState;
//!
//!     // Create `OwnedState` from this component
//!     fn init(self) -> OwnedState {
//!         OwnedState {
//!             name: self.name.into(),
//!         }
//!     }
//!
//!     // Update the pre-existing state
//!     fn update(self, state: &mut Self::State) -> ShouldRender {
//!         if self.name != state.name {
//!             // `state.name = self.name.into()` would have been fine too,
//!             // but this saves an allocation if the original `String` has
//!             // enough capacity
//!             state.name.replace_range(.., self.name);
//!
//!             ShouldRender::Yes
//!         } else {
//!             // If the name hasn't change there is no need to do anything
//!             ShouldRender::No
//!         }
//!     }
//! }
//!
//! impl<'a> Borrowing<'a> {
//!     fn render(self) -> impl Html + 'a {
//!         // Types here are:
//!         // state: &OwnedState,
//!         // ctx: Context<OwnedState>,
//!         self.stateful(|state, ctx| {
//!             // Since we work with a state that owns a `String`,
//!             // callbacks can mutate it at will.
//!             let exclaim = ctx.bind(|state, _| state.name.push('!'));
//!
//!             // Repeatedly clicking the Alice button does not have to do anything.
//!             let alice = ctx.bind(|state, _| {
//!                 if state.name != "Alice" {
//!                     state.name.replace_range(.., "Alice");
//!
//!                     ShouldRender::Yes
//!                 } else {
//!                     ShouldRender::No
//!                 }
//!             });
//!
//!             html! {
//!                 <div>
//!                     // Render can borrow `name` from state, no need for clones
//!                     <h1>"Hello: "{ &state.name }</h1>
//!                     <button onclick={alice}>"Alice"</button>
//!                     <button onclick={exclaim}>"!"</button>
//!                 </div>
//!             }
//!         })
//!     }
//! }
//!
//! fn main() {
//!     kobold::start(html! {
//!         // Constructing the component only requires a `&str` slice.
//!         <Borrowing name="Bob" />
//!     });
//! }
//! ```

use std::cell::{RefCell, UnsafeCell};
use std::marker::PhantomData;
use std::rc::Rc;

/// Derive macro for the [`Stateful`](Stateful) trait, see the [module documentation](crate::stateful) for details.
pub use kobold_macros::Stateful;

use crate::render_fn::RenderFn;
use crate::{Element, Html, Mountable};

mod ctx;
mod hook;

pub use ctx::{Callback, Context};
pub use hook::Hook;

/// Describes whether or not a component should be rendered after state changes.
/// For uses see:
///
/// * [Context::bind](Context::bind)
/// * [Stateful::update](Stateful::update)
pub enum ShouldRender {
    /// This is a silent update
    No,

    /// Yes, re-render the component after this update
    Yes,
}

/// Closures without a return type (those that return `()`)
/// are considered to return [`ShouldRender::Yes`](ShouldRender::Yes).
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

/// Trait used to create stateful components, see the [module documentation](crate::stateful) for details.
pub trait Stateful: Sized {
    type State: 'static;

    fn init(self) -> Self::State;

    fn update(self, state: &mut Self::State) -> ShouldRender;

    fn stateful<'a, H: Html + 'a>(
        self,
        render: fn(&'a Self::State, Context<'a, Self::State>) -> H,
    ) -> WithState<Self, H> {
        WithState {
            stateful: self,
            render: RenderFn::new(render),
            _marker: PhantomData,
        }
    }
}

pub struct WithState<S: Stateful, H: Html> {
    stateful: S,
    render: RenderFn<S::State, H::Product>,
    _marker: PhantomData<H>,
}

struct Inner<S, P> {
    state: RefCell<S>,
    product: UnsafeCell<P>,
    render: RenderFn<S, P>,
    update: fn(RenderFn<S, P>, &S, Context<S>),
}

impl<S: 'static, P: 'static> Inner<S, P> {
    fn rerender(&self, state: &S) {
        (self.update)(self.render, state, Context::new(self))
    }
}

pub struct WithStateProduct<S: 'static, P> {
    inner: Rc<Inner<S, P>>,
    el: Element,
}

impl<S, H> Html for WithState<S, H>
where
    S: Stateful,
    H: Html,
{
    type Product = WithStateProduct<S::State, H::Product>;

    fn build(self) -> Self::Product {
        let inner = Rc::new_cyclic(move |inner| {
            let state = self.stateful.init();
            let ctx = Context::new(inner.as_ptr());

            // Safety: this is safe as long as `S` and `H` are the same types that
            // were used to create this `RenderFn` instance.
            let render_fn = unsafe { self.render.cast::<H>() };
            let product = (render_fn)(&state, ctx).build();

            Inner {
                state: RefCell::new(state),
                product: UnsafeCell::new(product),
                render: self.render,
                update: |render, state, ctx| {
                    // Safety: this is safe as long as `S` and `H` are the same types that
                    // were used to create this `RenderFn` instance.
                    let render = unsafe { render.cast::<H>() };
                    let inner = unsafe { &*ctx.inner() };

                    (render)(state, ctx).update(unsafe { &mut *inner.product.get() });
                },
            }
        });

        let el = unsafe { &*inner.product.get() }.el().clone();

        WithStateProduct { inner, el }
    }

    fn update(self, p: &mut Self::Product) {
        let mut state = p.inner.state.borrow_mut();

        if self.stateful.update(&mut state).should_render() {
            p.inner.rerender(&state);
        }
    }
}

impl<S, P> Mountable for WithStateProduct<S, P>
where
    S: 'static,
    P: Mountable,
{
    fn el(&self) -> &Element {
        &self.el
    }
}

impl<S, H> WithState<S, H>
where
    S: Stateful,
    H: Html,
{
    pub fn then<F>(self, handler: F) -> WithHook<S, H, F>
    where
        F: FnOnce(Hook<S::State>),
    {
        WithHook {
            with_state: self,
            handler,
        }
    }
}

pub struct WithHook<S: Stateful, H: Html, F> {
    with_state: WithState<S, H>,
    handler: F,
}

impl<S, H, F> Html for WithHook<S, H, F>
where
    S: Stateful,
    H: Html,
    F: FnOnce(Hook<S::State>),
{
    type Product = WithStateProduct<S::State, H::Product>;

    fn build(self) -> Self::Product {
        let product = self.with_state.build();

        (self.handler)(Hook::new::<H>(&product.inner));

        product
    }

    fn update(self, p: &mut Self::Product) {
        self.with_state.update(p);
    }
}
