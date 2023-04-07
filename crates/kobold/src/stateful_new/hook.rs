use std::ops::Deref;

use crate::stateful::ShouldRender;
use crate::View;

use super::{Inner, Product};

#[repr(transparent)]
pub struct Hook<S> {
    inner: Inner<S, dyn Product<S>>,
}

impl<S> Hook<S> {
    pub(super) fn new(inner: &Inner<S, dyn Product<S>>) -> &Self {
        unsafe { &*(inner as *const _ as *const Hook<S>) }
    }
}

impl<S> Hook<S> {
    // /// Create an owned `Signal` to the state. This is effectively a weak reference
    // /// that allows for remote updates, particularly useful in async code.
    // pub fn signal(&self) -> Signal<S> {
    //     let weak = self.inner.weak();

    //     Signal {
    //         weak: (*weak).clone(),
    //     }
    // }

    /// Binds a closure to a mutable reference of the state. While this method is public
    /// it's recommended to use the [`bind!`](crate::bind) macro instead.
    pub fn bind<E, F, O>(&self, callback: F) -> impl Fn(E) + 'static
    where
        S: 'static,
        F: Fn(&mut S, E) -> O + 'static,
        O: ShouldRender,
    {
        let inner = &self.inner as *const Inner<S, dyn Product<S>>;

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
