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
use std::mem::{ManuallyDrop, MaybeUninit};
use std::rc::{Rc, Weak};

use web_sys::Node;

use crate::{dom::Element, Mountable, View};

mod hook;
mod should_render;

pub use hook::{Hook, Signal};
pub use should_render::{ShouldRender, Then};

pub struct Inner<S> {
    hook: Hook<S>,
    updater: Box<dyn FnMut(&Hook<S>)>,
}

impl<S> Inner<S> {
    fn update(&mut self) {
        (self.updater)(&self.hook)
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
    inner: Rc<WithCell<Inner<S>>>,
    el: Element,
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

#[repr(transparent)]
struct WeakRef<T>(*const T);

impl<T> Clone for WeakRef<T> {
    fn clone(&self) -> WeakRef<T> {
        WeakRef(self.0)
    }
}

impl<T> Copy for WeakRef<T> {}

impl<T> WeakRef<T> {
    pub fn weak(self) -> ManuallyDrop<Weak<T>> {
        ManuallyDrop::new(unsafe { Weak::from_raw(self.0) })
    }
}

impl<S, F, H> View for Stateful<S, F>
where
    S: IntoState,
    F: Fn(*const Hook<S::State>) -> H + 'static,
    H: View,
{
    type Product = StatefulProduct<S::State>;

    fn build(self) -> Self::Product {
        let mut el = MaybeUninit::uninit();
        let el_ref = &mut el;

        let inner = Rc::new_cyclic(move |weak| {
            let hook = Hook {
                state: self.state.init(),
                inner: WeakRef(weak.as_ptr()),
            };

            let mut product = (self.render)(&hook).build();

            el_ref.write(product.el().clone());

            WithCell::new(Inner {
                hook,
                updater: Box::new(move |hook| {
                    (self.render)(hook).update(&mut product);
                }),
            })
        });

        StatefulProduct {
            inner,
            el: unsafe { el.assume_init() },
        }
    }

    fn update(self, p: &mut Self::Product) {
        p.inner.with(|inner| {
            if self.state.update(&mut inner.hook.state).should_render() {
                inner.update();
            }
        });
    }
}

impl<S: 'static> Mountable for StatefulProduct<S> {
    type Js = Node;

    fn el(&self) -> &Element {
        &self.el
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

        product.inner.with(move |inner| {
            let signal = inner.hook.signal();

            (self.handler)(signal);
        });

        product
    }

    fn update(self, p: &mut Self::Product) {
        self.with_state.update(p);
    }
}

struct WithCell<T> {
    borrowed: Cell<bool>,
    data: UnsafeCell<T>,
}

impl<T> WithCell<T> {
    pub fn new(data: T) -> Self {
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
}
