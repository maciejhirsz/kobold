use std::mem::ManuallyDrop;
use std::ops::Deref;
use std::rc::Weak;

use crate::stateful::{Inner, ShouldRender};
use crate::util::WithCell;
use crate::Html;

/// A hook to some state `S`. A reference to `Hook` is obtained by using the [`stateful`](crate::stateful::stateful)
/// function.
pub struct Hook<S> {
    pub(super) state: S,
    pub(super) inner: *const WithCell<Inner<S>>,
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
        if self.weak.strong_count() == 0 {
            return;
        }

        let inner = unsafe { &*self.weak.as_ptr() };

        inner.with(move |inner| {
            if mutator(&mut inner.hook.state).should_render() {
                inner.update()
            }
        });
    }

    pub fn update_silent<F>(&self, mutator: F)
    where
        F: FnOnce(&mut S),
    {
        if self.weak.strong_count() == 0 {
            return;
        }

        let inner = unsafe { &*self.weak.as_ptr() };

        inner.with(move |inner| mutator(&mut inner.hook.state));
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
        let weak = ManuallyDrop::new(unsafe { Weak::from_raw(self.inner) });

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
            let weak = ManuallyDrop::new(unsafe { Weak::from_raw(inner) });

            if weak.strong_count() == 0 {
                return;
            }

            let inner = unsafe { &*inner };

            inner.with(|inner| {
                if callback(&mut inner.hook.state, e).should_render() {
                    inner.update()
                }
            });
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

impl<'a, H> Html for &'a Hook<H>
where
    &'a H: Html + 'a,
{
    type Product = <&'a H as Html>::Product;

    fn build(self) -> Self::Product {
        (**self).build()
    }

    fn update(self, p: &mut Self::Product) {
        (**self).update(p)
    }
}
