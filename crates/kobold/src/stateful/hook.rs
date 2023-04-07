// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::mem::ManuallyDrop;
use std::ops::Deref;
use std::rc::{Rc, Weak};

use crate::stateful::{Inner, ShouldRender};
use crate::View;

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

    /// Create an owned `Signal` to the state. This is effectively a weak reference
    /// that allows for remote updates, particularly useful in async code.
    pub fn signal(&self) -> Signal<S> {
        let rc = ManuallyDrop::new(unsafe { Rc::from_raw(&self.inner) });

        Signal {
            weak: Rc::downgrade(&*rc),
        }
    }

    /// Binds a closure to a mutable reference of the state. While this method is public
    /// it's recommended to use the [`bind!`](crate::bind) macro instead.
    pub fn bind<E, F, O>(&self, callback: F) -> impl Fn(E) + 'static
    where
        S: 'static,
        F: Fn(&mut S, E) -> O + 'static,
        O: ShouldRender,
    {
        let inner = &self.inner as *const Inner<S>;

        move |e| {
            let inner = unsafe { &*inner };

            inner.state.with(|state| {
                if callback(state, e).should_render() {
                    inner.update();
                }
            });
        }
    }

    /// Get the value of state if state implements `Copy`. This is equivalent to writing
    /// `**hook` but conveys intent better.
    pub fn get(&self) -> S
    where
        S: Copy,
    {
        unsafe { *self.inner.state.borrow_unchecked() }
    }
}

impl<S> Deref for Hook<S> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        unsafe { self.inner.state.borrow_unchecked() }
    }
}

impl<'a, H> View for &'a Hook<H>
where
    &'a H: View + 'a,
{
    type Product = <&'a H as View>::Product;

    fn build(self) -> Self::Product {
        (**self).build()
    }

    fn update(self, p: &mut Self::Product) {
        (**self).update(p)
    }
}
