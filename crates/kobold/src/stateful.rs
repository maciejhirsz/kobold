//! # Utilities for building stateful components
//!
//! **Kobold** uses _functional components_ that are _transient_, meaning they can include
//! borrowed values and are discarded on each render call. **Kobold** doesn't
//! allocate any memory on the heap for its simple components, and there is no way to update them
//! short of the parent component re-rendering them.
//!
//! However an app built entirely from such components wouldn't be very useful, as all it
//! could ever do is render itself once. To get around this the [`stateful`](stateful) function can
//! be used to give a component ownership over some arbitrary mutable state.
//!

// TODO: missing docs!
//
// ### The `IntoState` trait
//
// The [`Stateful`](Stateful) trait allows you to define an associated `State` type which can be
// different from the component itself. That state is put in a heap allocation so that it can be
// referenced from callbacks. It also allows you to define how and if the state should be updated
// when the parent component updates:
//
// ```no_run
// use kobold::prelude::*;
//
// // This is our component struct, note that it can take arbitrary lifetimes.
// struct Borrowing<'a> {
//     name: &'a str,
// }
//
// // This is our owned state, it must live for a `'static` lifetime, and may
// // contain different fields than those on the component.
// struct OwnedState {
//     name: String,
// }
//
// impl Stateful for Borrowing<'_> {
//     // We define that `OwnedState` is the state for this component
//     type State = OwnedState;
//
//     // Create `OwnedState` from this component
//     fn init(self) -> OwnedState {
//         OwnedState {
//             name: self.name.into(),
//         }
//     }
//
//     // Update the pre-existing state
//     fn update(self, state: &mut Self::State) -> ShouldRender {
//         if self.name != state.name {
//             // `state.name = self.name.into()` would have been fine too,
//             // but this saves an allocation if the original `String` has
//             // enough capacity
//             state.name.replace_range(.., self.name);
//
//             ShouldRender::Yes
//         } else {
//             // If the name hasn't change there is no need to do anything
//             ShouldRender::No
//         }
//     }
// }
//
// impl<'a> Borrowing<'a> {
//     fn render(self) -> impl Html + 'a {
//         // Types here are:
//         // state: &OwnedState,
//         // ctx: Context<OwnedState>,
//         self.stateful(|state, ctx| {
//             // Since we work with a state that owns a `String`,
//             // callbacks can mutate it at will.
//             let exclaim = ctx.bind(|state, _| state.name.push('!'));
//
//             // Repeatedly clicking the Alice button does not have to do anything.
//             let alice = ctx.bind(|state, _| {
//                 if state.name != "Alice" {
//                     state.name.replace_range(.., "Alice");
//
//                     ShouldRender::Yes
//                 } else {
//                     ShouldRender::No
//                 }
//             });
//
//             html! {
//                 <div>
//                     // Render can borrow `name` from state, no need for clones
//                     <h1>"Hello: "{ &state.name }</h1>
//                     <button onclick={alice}>"Alice"</button>
//                     <button onclick={exclaim}>"!"</button>
//                 </div>
//             }
//         })
//     }
// }
//
// kobold::start(html! {
//     // Constructing the component only requires a `&str` slice.
//     <Borrowing name="Bob" />
// });
// ```

use std::cell::{RefCell, UnsafeCell};
use std::marker::PhantomData;
use std::rc::Rc;

use crate::render_fn::RenderFn;
use crate::{Element, Html, Mountable};

mod hook;
mod owned_hook;

pub use hook::{Callback, Hook};
pub use owned_hook::OwnedHook;

/// Describes whether or not a component should be rendered after state changes.
/// For uses see:
///
/// * [`Hook::bind`](Hook::bind)
/// * [`IntoState::update`](IntoState::update)
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
pub trait IntoState: Sized {
    type State: 'static;

    fn init(self) -> Self::State;

    fn update(self, state: &mut Self::State) -> ShouldRender;
}

impl<F, S> IntoState for F
where
    S: 'static,
    F: FnOnce() -> S,
{
    type State = S;

    fn init(self) -> Self::State {
        (self)()
    }

    fn update(self, _: &mut Self::State) -> ShouldRender {
        ShouldRender::No
    }
}

pub fn stateful<'a, S, H>(init: S, render: fn(&'a Hook<S::State>) -> H) -> Stateful<S, H>
where
    S: IntoState,
    H: Html + 'a,
{
    Stateful {
        stateful: init,
        render: RenderFn::new(render),
        _marker: PhantomData,
    }
}

pub struct Stateful<S: IntoState, H: Html> {
    stateful: S,
    render: RenderFn<S::State, H::Product>,
    _marker: PhantomData<H>,
}

struct Inner<S, P> {
    hook: RefCell<Hook<S>>,
    product: UnsafeCell<P>,
    render: RenderFn<S, P>,
    update: fn(RenderFn<S, P>, &Hook<S>),
}

impl<S: 'static, P: 'static> Inner<S, P> {
    fn rerender(&self, hook: &Hook<S>) {
        (self.update)(self.render, hook)
    }
}

pub struct StatefulProduct<S: 'static, P> {
    inner: Rc<Inner<S, P>>,
    el: Element,
}

impl<S, H> Html for Stateful<S, H>
where
    S: IntoState,
    H: Html,
{
    type Product = StatefulProduct<S::State, H::Product>;

    fn build(self) -> Self::Product {
        let inner = Rc::new_cyclic(move |inner| {
            let state = self.stateful.init();
            let hook = Hook::new(state, inner.as_ptr());

            // Safety: this is safe as long as `S` and `H` are the same types that
            // were used to create this `RenderFn` instance.
            let render_fn = unsafe { self.render.cast::<H>() };
            let product = (render_fn)(&hook).build();

            Inner {
                hook: RefCell::new(hook),
                product: UnsafeCell::new(product),
                render: self.render,
                update: |render, hook| {
                    // Safety: this is safe as long as `S` and `H` are the same types that
                    // were used to create this `RenderFn` instance.
                    let render = unsafe { render.cast::<H>() };
                    let inner = unsafe { &*hook.inner() };

                    (render)(hook).update(unsafe { &mut *inner.product.get() });
                },
            }
        });

        let el = unsafe { &*inner.product.get() }.el().clone();

        StatefulProduct { inner, el }
    }

    fn update(self, p: &mut Self::Product) {
        let mut hook = p.inner.hook.borrow_mut();

        if self.stateful.update(&mut hook.state).should_render() {
            p.inner.rerender(&hook);
        }
    }
}

impl<S, P> Mountable for StatefulProduct<S, P>
where
    S: 'static,
    P: Mountable,
{
    type Js = P::Js;

    fn el(&self) -> &Element {
        &self.el
    }
}

impl<S, H> Stateful<S, H>
where
    S: IntoState,
    H: Html,
{
    pub fn once<F>(self, handler: F) -> Once<S, H, F>
    where
        F: FnOnce(OwnedHook<S::State>),
    {
        Once {
            with_state: self,
            handler,
        }
    }
}

pub struct Once<S: IntoState, H: Html, F> {
    with_state: Stateful<S, H>,
    handler: F,
}

impl<S, H, F> Html for Once<S, H, F>
where
    S: IntoState,
    H: Html,
    F: FnOnce(OwnedHook<S::State>),
{
    type Product = StatefulProduct<S::State, H::Product>;

    fn build(self) -> Self::Product {
        let product = self.with_state.build();

        (self.handler)(OwnedHook::new::<H>(&product.inner));

        product
    }

    fn update(self, p: &mut Self::Product) {
        self.with_state.update(p);
    }
}
