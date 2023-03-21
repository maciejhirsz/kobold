use std::rc::Weak;
use std::ops::Deref;

use crate::util::WithCell;
use crate::stateful::{Inner, ShouldRender};

pub struct Hook<S> {
    pub(super) state: S,
    pub(super) weak: Weak<WithCell<Inner<S>>>,
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
        Signal {
            weak: self.weak.clone(),
        }
    }

    pub fn bind<E, F, O>(&self, callback: F) -> impl Fn(E) + 'static
    where
        S: 'static,
        F: Fn(&mut S, E) -> O + 'static,
        O: ShouldRender,
    {
        let signal = self.signal();
        // let signal = self.weak.as_ptr();
        move |e| {
            signal.update(|s| callback(s, e));

            // unsafe { &* signal }.with(|inner| {
            //     let s = &mut inner.hook.state;
            //     if callback(s, e).should_render() {
            //         inner.update();
            //     }
            // })
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
