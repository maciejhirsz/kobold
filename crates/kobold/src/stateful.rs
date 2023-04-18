// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! # Utilities for building stateful views
//!
//! **Kobold** doesn't allocate any memory on the heap for its simple components, and there
//! is no way to update them short of the parent view re-rendering them.
//!
//! However a fully functional app like that wouldn't be very useful, as all it
//! could ever do is render itself once. To get around this the [`stateful`](stateful) function can
//! be used to create views that have ownership over some arbitrary mutable state.
//!
use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
use std::rc::Rc;

use wasm_bindgen::JsValue;
use web_sys::Node;

use crate::dom::Anchor;
use crate::internal::{Mut, Pre};
use crate::{init, Mountable, View};

mod cell;
mod hook;
mod into_state;
mod product;
mod should_render;

use cell::WithCell;
use product::{Product, ProductHandler};

pub use hook::{Hook, Signal};
pub use into_state::IntoState;
pub use should_render::{ShouldRender, Then};

#[repr(C)]
struct Inner<S, P: ?Sized = dyn Product<S>> {
    state: WithCell<S>,
    prod: UnsafeCell<P>,
}

pub struct Stateful<S, F> {
    state: S,
    render: F,
}

pub struct StatefulProduct<S> {
    inner: Rc<Inner<S>>,
}

/// Create a stateful [`View`](crate::View) over some mutable state. The state
/// needs to be created using the [`IntoState`](IntoState) trait.
///
/// ```
/// # use::kobold::prelude::*;
/// // `IntoState` is implemented for primitive values
/// let int_view = stateful(0, |count: &Hook<i32>| { "TODO" });
///
/// // Another easy way to create arbitrary state is using a closure...
/// let string_view = stateful(|| String::from("foo"), |text: &Hook<String>| { "TODO" });
///
/// // ...or a function with no parameters
/// let vec_view = stateful(Vec::new, |counts: &Hook<Vec<i32>>| { "TODO" });
/// ```
pub fn stateful<'a, S, F, V>(
    state: S,
    render: F,
) -> Stateful<S, impl Fn(*const Hook<S::State>) -> V + 'static>
where
    S: IntoState,
    F: Fn(&'a Hook<S::State>) -> V + 'static,
    V: View + 'a,
{
    // There is no safe way to represent a generic closure with generic return type
    // that borrows from that closure's arguments, without also slapping a lifetime.
    //
    // The `stateful` function ensures that correct lifetimes are used before we
    // erase them for the use in the `Stateful` struct.
    let render = move |hook: *const Hook<S::State>| render(unsafe { &*hook });
    Stateful { state, render }
}

impl<S, P> Inner<S, MaybeUninit<P>> {
    unsafe fn as_init(&self) -> &Inner<S, P> {
        &*(self as *const _ as *const Inner<S, P>)
    }

    unsafe fn into_init(self: Rc<Self>) -> Rc<Inner<S, P>> {
        std::mem::transmute(self)
    }
}

impl<S> Inner<S> {
    fn update(&self) {
        // ⚠️ Safety:
        // ==========
        //
        // `prod` is an implementation detail and it's never mut borrowed
        // unless `state` is borrowed first, which is guarded by `WithCell`
        // or otherwise guaranteed to be safe.
        //
        // Ideally whole `Inner` would be wrapped in `WithCell`, but we
        // can't do that until `CoerceUnsized` is stabilized.
        //
        // <https://github.com/rust-lang/rust/issues/18598>
        unsafe { (*self.prod.get()).update(Hook::new(self)) }
    }
}

impl<S, F, V> View for Stateful<S, F>
where
    S: IntoState,
    F: Fn(*const Hook<S::State>) -> V + 'static,
    V: View,
    S::State: Unpin,
{
    type Product = StatefulProduct<S::State>;

    fn build(self, p: Pre<Self::Product>) -> Mut<Self::Product> {
        let inner = Rc::new(Inner {
            state: WithCell::new(self.state.init()),
            prod: UnsafeCell::new(MaybeUninit::uninit()),
        });

        // ⚠️ Safety:
        // ==========
        //
        // Initial render can only access the `state` from the hook, the `prod` is
        // not touched until an event is fired, which happens after this method
        // completes and initializes the `prod`.
        let view = (self.render)(Hook::new(unsafe { inner.as_init() })); //.build();

        // ⚠️ Safety:
        // ==========
        //
        // This looks scary, but it just initializes the `prod`. We need to use the
        // closure syntax with a raw pointer to get around lifetime restrictions.
        unsafe {
            let _ = Pre::in_raw((*inner.prod.get()).as_mut_ptr(), |prod| {
                ProductHandler::new(
                    move |hook, product: *mut V::Product| (self.render)(hook).update(&mut *product),
                    view,
                    prod,
                )
            });
        }

        // ⚠️ Safety:
        // ==========
        //
        // At this point `Inner` is fully initialized.
        p.put(StatefulProduct {
            inner: unsafe { inner.into_init() },
        })
    }

    fn update(self, p: &mut Self::Product) {
        p.inner.state.with(|state| {
            if self.state.update(state).should_render() {
                p.inner.update();
            }
        })
    }
}

impl<S> Mountable for StatefulProduct<S>
where
    S: 'static,
{
    type Js = Node;

    fn js(&self) -> &JsValue {
        unsafe { (*self.inner.prod.get()).js() }
    }

    fn unmount(&self) {
        unsafe { (*self.inner.prod.get()).unmount() }
    }

    fn replace_with(&self, new: &JsValue) {
        unsafe { (*self.inner.prod.get()).replace_with(new) }
    }
}

impl<S, R> Stateful<S, R>
where
    S: IntoState,
{
    pub fn once<F, P>(self, handler: F) -> Once<S, R, F>
    where
        F: FnOnce(Signal<S::State>) -> P,
    {
        Once {
            with_state: self,
            handler,
        }
    }
}

pub struct Once<S, R, F> {
    with_state: Stateful<S, R>,
    handler: F,
}

pub struct OnceProduct<S, P> {
    product: StatefulProduct<S>,
    // hold onto the return value of the `handler`, so it can
    // be safely dropped along with the `StatefulProduct`
    _no_drop: P,
}

impl<S, P> Anchor for OnceProduct<S, P>
where
    StatefulProduct<S>: Mountable,
    S: Unpin,
    P: Unpin,
{
    type Js = <StatefulProduct<S> as Mountable>::Js;
    type Target = StatefulProduct<S>;

    fn anchor(&self) -> &Self::Target {
        &self.product
    }
}

impl<S, R, F, P> View for Once<S, R, F>
where
    S: IntoState,
    F: FnOnce(Signal<S::State>) -> P,
    P: Unpin + 'static,
    S::State: Unpin,
    Stateful<S, R>: View<Product = StatefulProduct<S::State>>,
{
    type Product = OnceProduct<S::State, P>;

    fn build(self, p: Pre<Self::Product>) -> Mut<Self::Product> {
        let p = p.into_raw();

        unsafe {
            let product = init!(p.product @ self.with_state.build(p));
            let signal = Signal {
                weak: Rc::downgrade(&product.inner),
            };

            init!(p._no_drop = (self.handler)(signal));

            Mut::from_raw(p)
        }
    }

    fn update(self, p: &mut Self::Product) {
        self.with_state.update(&mut p.product);
    }
}
