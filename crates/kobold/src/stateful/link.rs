use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::rc::Weak;

use wasm_bindgen::prelude::*;

use crate::event::Event;
use crate::stateful::{Inner, ShouldRender};
use crate::{Element, Html, Mountable};

pub struct Link<'state, S> {
    inner: *const (),
    make_closure: fn(
        *const (),
        cb: *const UnsafeCell<dyn CallbackFn<S, web_sys::Event>>,
    ) -> Box<dyn FnMut(&web_sys::Event)>,
    _marker: PhantomData<&'state S>,
}

impl<S> Clone for Link<'_, S> {
    fn clone(&self) -> Self {
        Link {
            inner: self.inner,
            make_closure: self.make_closure,
            _marker: PhantomData,
        }
    }
}

impl<S> Copy for Link<'_, S> {}

impl<'state, S> Link<'state, S>
where
    S: 'static,
{
    pub(super) fn new<P: 'static>(inner: &'state Inner<S, P>) -> Self {
        Self::new_raw(inner)
    }

    pub(super) fn from_weak<P: 'static>(weak: &'state Weak<Inner<S, P>>) -> Self {
        Self::new_raw(weak.as_ptr())
    }

    fn new_raw<P: 'static>(inner: *const Inner<S, P>) -> Self {
        Link {
            inner: inner as *const _,
            make_closure: |inner, callback| {
                Box::new(move |event| {
                    let callback = unsafe { &*(*callback).get() };
                    let inner = unsafe { &*(inner as *const Inner<S, P>) };

                    if callback
                        .call(&mut inner.state.borrow_mut(), event)
                        .should_render()
                    {
                        inner.update();
                    }
                })
            },
            _marker: PhantomData,
        }
    }

    pub(super) fn inner<P: 'static>(&self) -> *const Inner<S, P> {
        self.inner as *const Inner<S, P>
    }

    pub fn callback<T, F, A>(self, cb: F) -> Callback<'state, T, F, S>
    where
        F: Fn(&mut S, &Event<T>) -> A + 'static,
        A: Into<ShouldRender>,
    {
        Callback {
            cb,
            link: self,
            _target: PhantomData,
        }
    }
}

pub struct Callback<'state, T, F, S> {
    cb: F,
    link: Link<'state, S>,
    _target: PhantomData<T>,
}

pub struct CallbackProduct<F> {
    closure: Closure<dyn FnMut(&web_sys::Event)>,
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

impl<T, F, A, S> Html for Callback<'_, T, F, S>
where
    F: Fn(&mut S, &Event<T>) -> A + 'static,
    A: Into<ShouldRender>,
    S: 'static,
{
    type Product = CallbackProduct<F>;

    fn build(self) -> Self::Product {
        let Self { link, cb, .. } = self;

        let cb = Box::new(UnsafeCell::new(cb));

        let closure = (link.make_closure)(link.inner, unsafe {
            let weak: &UnsafeCell<dyn CallbackFn<S, Event<T>>> = &*cb;

            // Safety: This is casting `dyn CallbackFn<S, Event<T>>` to `dyn CallbackFn<S, web_sys::Event>`
            //         which is safe as `Event<T>` is a transparent wrapper for `web_sys::Event`.
            std::mem::transmute(weak)
        });
        let closure = Closure::wrap(closure);

        CallbackProduct { closure, cb }
    }

    fn update(self, p: &mut Self::Product) {
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
