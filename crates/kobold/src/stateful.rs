// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! # Utilities for building stateful views
//!
//! **Kobold** doesn't allocate any memory on the heap for its simple components, and there
//! is no way to update them short of the parent view re-rendering them.
//!
//! However a fully functional app like that wouldn't be very useful, as all it
//! could ever do is render itself once. To get around this the [`stateful`] function can
//! be used to create views that have ownership over some arbitrary mutable state.
//!
use std::any::TypeId;
use std::cell::Cell;
use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
use std::rc::Rc;

use wasm_bindgen::JsValue;
use web_sys::Node;

use crate::dom::Anchor;
use crate::internal::{In, Out};
use crate::{init, Mountable, View};

mod cell;
mod hook;
mod into_state;
mod product;
mod should_render;

use cell::WithCell;
use product::{Product, ProductHandler};

pub use hook::{Bound, Hook, Signal};
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

/// Create a stateful [`View`] over some mutable state. The state
/// needs to be created using the [`IntoState`] trait.
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
    let render = move |hook: *const Hook<S::State>| render(set_global_hook(unsafe { &*hook }));
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

thread_local! {
    static HOOK: Cell<StaticHook> = const { Cell::new(VOID) };
}

#[derive(Clone, Copy)]
struct StaticHook {
    typ: u64,
    ptr: (usize, usize),
}

union StaticHookPtr {
    ptr: *const Hook<()>,
    raw: (usize, usize),
}

const VOID: StaticHook = StaticHook {
    typ: 0,
    ptr: (0, 0),
};

impl StaticHook {
    fn new<S: 'static>(hook: &Hook<S>) -> Self {
        StaticHook {
            typ: u64type_id::<S>(),
            ptr: unsafe { StaticHookPtr { ptr: hook as *const _ as *const Hook<()> }.raw },
        }
    }
}

fn u64type_id<S: 'static>() -> u64 {
    let typ: (u64, u64) = unsafe { std::mem::transmute(TypeId::of::<S>()) };

    typ.0 ^ typ.1
}

pub fn hook<S, F, V>(f: F) -> V
where
    S: 'static,
    F: FnOnce(&Hook<S>) -> V,
{
    let hook = HOOK.get();

    if hook.typ == u64type_id::<S>() {
        let hook = unsafe { StaticHookPtr { raw: hook.ptr }.ptr };

        return f(unsafe { &*(hook as *const Hook<S>) });
    }

    panic!();
}

unsafe fn get_hook_unchecked<S>() -> *const Hook<S>
where
    S: 'static,
{
    let hook = HOOK.get();

    debug_assert!(hook.typ == u64type_id::<S>(), "get_hook_unchecked");

    StaticHookPtr { raw: hook.ptr }.ptr as _
}

fn set_global_hook<S>(hook: &Hook<S>) -> &Hook<S>
where
    S: 'static,
{
    HOOK.set(StaticHook::new(hook));
    hook
}

fn unset_global_hook() {
    HOOK.set(VOID);
}

impl<S, F, V> View for Stateful<S, F>
where
    S: IntoState,
    F: Fn(*const Hook<S::State>) -> V + 'static,
    V: View,
{
    type Product = StatefulProduct<S::State>;

    fn build(self, p: In<Self::Product>) -> Out<Self::Product> {
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
        let view = (self.render)(Hook::new(unsafe { inner.as_init() }));

        // ⚠️ Safety:
        // ==========
        //
        // This looks scary, but it just initializes the `prod`. We need to use the
        // closure syntax with a raw pointer to get around lifetime restrictions.
        unsafe {
            In::raw((*inner.prod.get()).as_mut_ptr(), |prod| {
                ProductHandler::build(
                    move |hook, product: *mut V::Product| {
                        (self.render)(hook).update(&mut *product);
                        unset_global_hook();
                    },
                    view,
                    prod,
                )
            });
        }

        unset_global_hook();

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
    P: 'static,
    Stateful<S, R>: View<Product = StatefulProduct<S::State>>,
{
    type Product = OnceProduct<S::State, P>;

    fn build(self, p: In<Self::Product>) -> Out<Self::Product> {
        p.in_place(|p| unsafe {
            let product = init!(p.product @ self.with_state.build(p));
            let signal = Signal {
                weak: Rc::downgrade(&product.inner),
            };

            init!(p._no_drop = (self.handler)(signal));

            Out::from_raw(p)
        })
    }

    fn update(self, p: &mut Self::Product) {
        self.with_state.update(&mut p.product);
    }
}
