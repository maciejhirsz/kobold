use std::ops::Deref;
use std::rc::Weak;

use crate::stateful::{Inner, ShouldRender, WeakRef};
use crate::util::WithCell;
use crate::View;

/// A hook to some state `S`. A reference to `Hook` is obtained by using the [`stateful`](crate::stateful::stateful)
/// function.
pub struct Hook<S> {
    pub(super) state: S,
    pub(super) inner: WeakRef<WithCell<Inner<S>>>,
}

pub struct Signal<S> {
    pub(super) weak: Weak<WithCell<Inner<S>>>,
}

impl<S> Signal<S> {
    pub fn update<F, O>(&self, mutator: F)
    where
        F: FnOnce(&mut S) -> O,
        O: ShouldRender,
    {
        if let Some(inner) = self.weak.upgrade() {
            inner.with(move |inner| {
                if mutator(&mut inner.hook.state).should_render() {
                    inner.update()
                }
            });
        }
    }

    pub fn update_silent<F>(&self, mutator: F)
    where
        F: FnOnce(&mut S),
    {
        if let Some(inner) = self.weak.upgrade() {
            inner.with(move |inner| mutator(&mut inner.hook.state));
        }
    }

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
    pub fn signal(&self) -> Signal<S> {
        let weak = self.inner.weak();

        Signal {
            weak: (*weak).clone(),
        }
    }

    pub fn bind<E, F, O>(&self, callback: F) -> impl Fn(E) + 'static
    where
        S: 'static,
        F: Fn(&mut S, E) -> O + 'static,
        O: ShouldRender,
    {
        let inner = self.inner;

        move |e| {
            if let Some(inner) = inner.weak().upgrade() {
                inner.with(|inner| {
                    if callback(&mut inner.hook.state, e).should_render() {
                        inner.update()
                    }
                });
            }
        }
    }

    pub fn get(&self) -> S
    where
        S: Copy,
    {
        self.state
    }
}

impl<S> Deref for Hook<S> {
    type Target = S;

    fn deref(&self) -> &S {
        &self.state
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
