use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::rc::Rc;

use wasm_bindgen::JsValue;
use web_sys::Node;

use crate::stateful::{IntoState, ShouldRender, WithCell};
use crate::{Mountable, View};

mod hook;

pub use hook::Hook;

#[repr(C)]
struct Inner<S, P: ?Sized> {
    state: WithCell<S>,
    prod: UnsafeCell<P>,
}

impl<S, P> Inner<S, MaybeUninit<P>> {
    unsafe fn as_init(&self) -> &Inner<S, P> {
        &*(self as *const _ as *const Inner<S, P>)
    }

    unsafe fn into_init(self: Rc<Self>) -> Rc<Inner<S, P>> {
        std::mem::transmute(self)
    }
}

impl<S> Inner<S, dyn Product<S>> {
    fn update(&self) {
        let hook = Hook::new(self);

        unsafe { (*self.prod.get()).update(hook) }
    }
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

pub struct Stateful<S, F> {
    state: S,
    render: F,
}

pub struct StatefulProduct<S> {
    inner: Rc<Inner<S, dyn Product<S>>>,
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
