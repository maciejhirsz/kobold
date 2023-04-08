// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::mem::ManuallyDrop;
use std::ops::Deref;
use std::rc::{Rc, Weak};

use crate::event::{EventCast, Listener};
use crate::stateful::{Inner, ShouldRender};
use crate::View;

/// A hook into some state `S`. A reference to `Hook` is obtained by using the [`stateful`](crate::stateful::stateful)
/// function.
///
/// Hook can be read from though its `Deref` implementation, and it allows for mutations either by [`bind`ing](Hook::bind)
/// closures to it, or the creation of [`signal`s](Hook::signal).
#[repr(transparent)]
pub struct Hook<S> {
    inner: Inner<S>,
}

#[repr(transparent)]
pub struct Signal<S> {
    pub(super) weak: Weak<Inner<S>>,
}

impl<S> Signal<S> {
    /// Update the state behind this `Signal`.
    ///
    /// ```
    /// # use kobold::prelude::*;
    /// fn example(count: Signal<i32>) {
    ///     // increment count and trigger a render
    ///     count.update(|count| *count += 1);
    ///
    ///     // increment count if less than 10, only render on change
    ///     count.update(|count| {
    ///         if *count < 10 {
    ///             *count += 1;
    ///             Then::Render
    ///         } else {
    ///             Then::Stop
    ///         }
    ///     })
    /// }
    /// ```
    pub fn update<F, O>(&self, mutator: F)
    where
        F: FnOnce(&mut S) -> O,
        O: ShouldRender,
    {
        if let Some(inner) = self.weak.upgrade() {
            inner.state.with(|state| {
                if mutator(state).should_render() {
                    inner.update()
                }
            });
        }
    }

    /// Same as [`update`](Signal::update), but it never renders updates.
    pub fn update_silent<F>(&self, mutator: F)
    where
        F: FnOnce(&mut S),
    {
        if let Some(inner) = self.weak.upgrade() {
            inner.state.with(move |state| mutator(state));
        }
    }

    /// Replace the entire state with a new value and trigger an update.
    pub fn set(&self, val: S) {
        self.update(move |s| *s = val);
    }
}

impl<S> Clone for Signal<S> {
    fn clone(&self) -> Self {
        Signal {
            weak: self.weak.clone(),
        }
    }
}

impl<S> Hook<S> {
    pub(super) fn new(inner: &Inner<S>) -> &Self {
        unsafe { &*(inner as *const _ as *const Hook<S>) }
    }

    /// Binds a closure to a mutable reference of the state. While this method is public
    /// it's recommended to use the [`bind!`](crate::bind) macro instead.
    pub fn bind<E, F, O>(&self, callback: F) -> impl Listener<E>
    where
        S: 'static,
        E: EventCast,
        F: Fn(&mut S, E) -> O + 'static,
        O: ShouldRender,
    {
        let inner = &self.inner as *const Inner<S>;

        move |e| {
            // ⚠️ Safety:
            // ==========
            //
            // This is fired only as event listener from the DOM, which guarantees that
            // state is not currently borrowed, as events cannot interrupt normal
            // control flow, and `Signal`s cannot borrow state across .await points.
            let inner = unsafe { &*inner };
            let state = unsafe { inner.state.mut_unchecked() };

            if callback(state, e).should_render() {
                inner.update();
            }
        }
    }

    pub fn bind_signal<E, F>(&self, callback: F) -> impl Listener<E>
    where
        S: 'static,
        E: EventCast,
        F: Fn(Signal<S>, E) + 'static,
    {
        let inner = &self.inner as *const Inner<S>;

        move |e| {
            // ⚠️ Safety:
            // ==========
            //
            // This is fired only as event listener from the DOM, which guarantees that
            // state is not currently borrowed, as events cannot interrupt normal
            // control flow, and `Signal`s cannot borrow state across .await points.
            //
            // This temporary `Rc` will not mess with the `strong_count` value, we only
            // need it to construct a `Weak` reference to `Inner`.
            let rc = ManuallyDrop::new(unsafe { Rc::from_raw(inner) });

            let signal = Signal {
                weak: Rc::downgrade(&*rc),
            };

            callback(signal, e);
        }
    }

    /// Get the value of state if state implements `Copy`. This is equivalent to writing
    /// `**hook` but conveys intent better.
    pub fn get(&self) -> S
    where
        S: Copy,
    {
        **self
    }
}

impl<S> Deref for Hook<S> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        // ⚠️ Safety:
        // ==========
        //
        // Hook only lives inside the inner closure of `stateful`, and no mutable
        // references to `Inner` are present while it's around.
        unsafe { self.inner.state.ref_unchecked() }
    }
}

impl<'a, V> View for &'a Hook<V>
where
    &'a V: View + 'a,
{
    type Product = <&'a V as View>::Product;

    fn build(self) -> Self::Product {
        (**self).build()
    }

    fn update(self, p: &mut Self::Product) {
        (**self).update(p)
    }
}
