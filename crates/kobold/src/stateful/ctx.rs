use std::cell::UnsafeCell;
use std::marker::PhantomData;

use wasm_bindgen::prelude::*;
use web_sys::Event as RawEvent;

use crate::event::Event;
use crate::stateful::{Inner, ShouldRender};
use crate::{Element, Html, Mountable};

type UnsafeCallback<S> = *const UnsafeCell<dyn CallbackFn<S, RawEvent>>;

pub struct Context<'state, S> {
    inner: *const (),
    make_closure: fn(*const (), cb: UnsafeCallback<S>) -> Box<dyn Fn(&RawEvent)>,
    _marker: PhantomData<&'state S>,
}

impl<S> Clone for Context<'_, S> {
    fn clone(&self) -> Self {
        Context {
            inner: self.inner,
            make_closure: self.make_closure,
            _marker: PhantomData,
        }
    }
}

impl<S> Copy for Context<'_, S> {}

impl<'state, S> Context<'state, S>
where
    S: 'static,
{
    pub(super) fn new<P: 'static>(inner: *const Inner<S, P>) -> Self {
        Context {
            inner: inner as *const _,
            make_closure: |inner, callback| {
                Box::new(move |event| {
                    let callback = unsafe { &*(*callback).get() };
                    let inner = unsafe { &*(inner as *const Inner<S, P>) };

                    let mut state = inner.state.borrow_mut();

                    if callback.call(&mut state, event).should_render() {
                        inner.rerender(&state);
                    }
                })
            },
            _marker: PhantomData,
        }
    }

    pub(super) fn inner<P: 'static>(&self) -> *const Inner<S, P> {
        self.inner as *const Inner<S, P>
    }

    pub fn bind<E, T, F, A>(self, cb: F) -> Callback<'state, E, T, F, S>
    where
        F: Fn(&mut S, &Event<E, T>) -> A + 'static,
        A: Into<ShouldRender>,
    {
        Callback {
            cb,
            ctx: self,
            _target: PhantomData,
        }
    }
}

pub struct Callback<'state, E, T, F, S> {
    cb: F,
    ctx: Context<'state, S>,
    _target: PhantomData<(E, T)>,
}

pub struct CallbackProduct<F> {
    closure: Closure<dyn Fn(&RawEvent)>,
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
        let Self { ctx, cb, .. } = self;

        let cb = Box::new(UnsafeCell::new(cb));

        let closure = Closure::wrap((ctx.make_closure)(ctx.inner, {
            let cb: *const UnsafeCell<dyn CallbackFn<S, Event<E, T>>> = &*cb;

            // Casting `*const UnsafeCell<dyn CallbackFn<S, UntypedEvent<E, T>>>`
            // to `UnsafeCallback<S>`, which is safe since `UntypedEvent<E, T>`
            // is `#[repr(transparent)]` wrapper for `RawEvent`.
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
