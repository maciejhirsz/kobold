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
use std::cell::{Cell, UnsafeCell};
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::rc::Rc;

use wasm_bindgen::JsValue;
use web_sys::Node;

use crate::diff::Diff;
use crate::{Mountable, View};

mod hook;
mod should_render;

pub use hook::{Hook, Signal};
pub use should_render::{ShouldRender, Then};

#[repr(C)]
struct Inner<S, P: ?Sized = dyn Product<S>> {
    state: WithCell<S>,
    prod: UnsafeCell<P>,
}

trait Product<S> {
    fn update(&mut self, hook: &Hook<S>);

    fn js(&self) -> &JsValue;

    fn unmount(&self);

    fn replace_with(&self, new: &JsValue);
}

struct ProductHandler<S, P, F> {
    updater: F,
    product: P,
    _state: PhantomData<S>,
}

impl<S, P, F> Product<S> for ProductHandler<S, P, F>
where
    S: 'static,
    P: Mountable,
    F: FnMut(*const Hook<S>, *mut P),
{
    fn update(&mut self, hook: &Hook<S>) {
        (self.updater)(hook, &mut self.product);
    }

    fn js(&self) -> &JsValue {
        self.product.js()
    }

    fn unmount(&self) {
        self.product.unmount()
    }

    fn replace_with(&self, new: &JsValue) {
        self.product.replace_with(new)
    }
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
        let hook = Hook::new(self);

        unsafe { (*self.prod.get()).update(hook) }
    }
}

/// Trait used to create stateful components, see [`stateful`](crate::stateful::stateful) for details.
pub trait IntoState: Sized {
    type State: 'static;

    fn init(self) -> Self::State;

    fn update(self, state: &mut Self::State) -> Then;
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

    fn update(self, _: &mut Self::State) -> Then {
        Then::Stop
    }
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
pub fn stateful<'a, S, F, H>(
    state: S,
    render: F,
) -> Stateful<S, impl Fn(*const Hook<S::State>) -> H + 'static>
where
    S: IntoState,
    F: Fn(&'a Hook<S::State>) -> H + 'static,
    H: View + 'a,
{
    // There is no safe way to represent a generic closure with generic return type
    // that borrows from that closure's arguments, without also slapping a lifetime.
    //
    // The `stateful` function ensures that correct lifetimes are used before we
    // erase them for the use in the `Stateful` struct.
    let render = move |hook: *const Hook<S::State>| render(unsafe { &*hook });
    Stateful { state, render }
}

impl<S, F, V> View for Stateful<S, F>
where
    S: IntoState,
    F: Fn(*const Hook<S::State>) -> V + 'static,
    V: View,
{
    type Product = StatefulProduct<S::State>;

    fn build(self) -> Self::Product {
        let inner = Rc::new(Inner {
            state: WithCell::new(self.state.init()),
            prod: UnsafeCell::new(MaybeUninit::uninit()),
        });

        let product = (self.render)(Hook::new(unsafe { inner.as_init() })).build();

        unsafe { &mut *inner.prod.get() }.write(ProductHandler {
            updater: move |hook, product: *mut V::Product| {
                (self.render)(hook).update(unsafe { &mut *product })
            },
            product,
            _state: PhantomData,
        });

        StatefulProduct {
            inner: unsafe { inner.into_init() },
        }
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
    pub fn once<F>(self, handler: F) -> Once<S, R, F>
    where
        F: FnOnce(Signal<S::State>),
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

impl<S, R, F> View for Once<S, R, F>
where
    S: IntoState,
    F: FnOnce(Signal<S::State>),
    Stateful<S, R>: View<Product = StatefulProduct<S::State>>,
{
    type Product = StatefulProduct<S::State>;

    fn build(self) -> Self::Product {
        let product = self.with_state.build();

        let signal = Signal {
            weak: Rc::downgrade(&product.inner),
        };

        (self.handler)(signal);

        product
    }

    fn update(self, p: &mut Self::Product) {
        self.with_state.update(p);
    }
}

pub(crate) struct WithCell<T> {
    borrowed: Cell<bool>,
    data: UnsafeCell<T>,
}

impl<T> WithCell<T> {
    pub const fn new(data: T) -> Self {
        WithCell {
            borrowed: Cell::new(false),
            data: UnsafeCell::new(data),
        }
    }

    pub fn with<F>(&self, mutator: F)
    where
        F: FnOnce(&mut T),
    {
        if self.borrowed.get() {
            return;
        }

        self.borrowed.set(true);
        mutator(unsafe { &mut *self.data.get() });
        self.borrowed.set(false);
    }

    pub unsafe fn borrow_unchecked(&self) -> &T {
        &*self.data.get()
    }
}

macro_rules! impl_into_state {
    ($($ty:ty),*) => {
        $(
            impl IntoState for $ty {
                type State = <Self as Diff>::Memo;

                fn init(self) -> Self::State {
                    self.into_memo()
                }

                fn update(self, state: &mut Self::State) -> Then {
                    match self.diff(state) {
                        false => Then::Stop,
                        true => Then::Render,
                    }
                }
            }
        )*
    };
}

impl_into_state!(
    &str, &String, bool, u8, u16, u32, u64, u128, usize, isize, i8, i16, i32, i64, i128, f32, f64
);
