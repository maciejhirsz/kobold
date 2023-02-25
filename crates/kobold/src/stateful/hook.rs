use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::ops::Deref;

use wasm_bindgen::prelude::*;

use crate::event::Event;
use crate::stateful::{Inner, ShouldRender};
use crate::{Element, Html, Mountable};

type UnsafeCallback<S> = *const UnsafeCell<dyn CallbackFn<S, web_sys::Event>>;

pub struct Hook<S> {
    pub(super) state: S,
    inner: *const (),
    make_closure: fn(*const (), cb: UnsafeCallback<S>) -> Box<dyn Fn(&web_sys::Event)>,
}

impl<S> Deref for Hook<S> {
    type Target = S;

    fn deref(&self) -> &S {
        &self.state
    }
}

impl<S> Hook<S>
where
    S: 'static,
{
    pub(super) fn new<P: 'static>(state: S, inner: *const Inner<S, P>) -> Self {
        Hook {
            state,
            inner: inner as *const _,
            make_closure: |inner, callback| {
                Box::new(move |event| {
                    let callback = unsafe { &*(*callback).get() };
                    let inner = unsafe { &*(inner as *const Inner<S, P>) };

                    let mut state = inner.hook.borrow_mut();

                    if callback.call(&mut state.state, event).should_render() {
                        inner.rerender(&state);
                    }
                })
            },
        }
    }

    pub(super) fn inner<P: 'static>(&self) -> *const Inner<S, P> {
        self.inner as *const Inner<S, P>
    }

    pub fn bind<E, T, F, A>(&self, cb: F) -> Callback<E, T, F, S>
    where
        F: Fn(&mut S, &Event<E, T>) -> A + 'static,
        A: Into<ShouldRender>,
    {
        Callback {
            cb,
            hook: self,
            _target: PhantomData,
        }
    }
}

impl<S: Copy> Hook<S> {
    pub fn get(&self) -> S {
        self.state
    }
}

pub struct Callback<'state, E, T, F, S> {
    cb: F,
    hook: &'state Hook<S>,
    _target: PhantomData<(E, T)>,
}

pub struct CallbackProduct<F> {
    closure: Closure<dyn Fn(&web_sys::Event)>,
    cb: Box<UnsafeCell<F>>,
}

trait CallbackFn<S, E> {
    fn call(&self, state: &mut S, event: &E) -> ShouldRender;
}

impl<F, A, E, S> CallbackFn<S, E> for F
where
    F: Fn(&mut S, &E) -> A + 'static,
    A: Into<ShouldRender>,
    S: 'static,
{
    fn call(&self, state: &mut S, event: &E) -> ShouldRender {
        (self)(state, event).into()
    }
}

impl<E, T, F, A, S> Html for Callback<'_, E, T, F, S>
where
    F: Fn(&mut S, &Event<E, T>) -> A + 'static,
    A: Into<ShouldRender>,
    S: 'static,
{
    type Product = CallbackProduct<F>;

    fn build(self) -> Self::Product {
        let Self { hook, cb, .. } = self;

        let cb = Box::new(UnsafeCell::new(cb));

        let closure = Closure::wrap((hook.make_closure)(hook.inner, {
            let cb: *const UnsafeCell<dyn CallbackFn<S, Event<E, T>>> = &*cb;

            // Casting `*const UnsafeCell<dyn CallbackFn<S, Event<E, T>>>`
            // to `UnsafeCallback<S>`, which is safe since `Event<E, T>`
            // is a `#[repr(transparent)]` wrapper for `web_sys::Event`.
            cb as UnsafeCallback<S>
        }));

        CallbackProduct { closure, cb }
    }

    fn update(self, p: &mut Self::Product) {
        // Technically we could just write to this box, but since
        // this is a shared pointer I felt some prudence with `UnsafeCell`
        // is warranted.
        unsafe { *p.cb.get() = self.cb }
    }
}

impl<F: 'static> Mountable for CallbackProduct<F> {
    fn el(&self) -> &Element {
        panic!("Callback is not an element");
    }

    fn js(&self) -> &JsValue {
        self.closure.as_ref()
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
