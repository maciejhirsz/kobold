use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::rc::{Rc, Weak};

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsValue;

use crate::event::{Event, WithEventTarget};
use crate::stateful::{Inner, ShouldRender};
use crate::{Element, Html, Mountable};

pub struct Link<'state, S> {
    inner: *const (),
    make_closure: fn(
        *const (),
        cb: Weak<UnsafeCell<dyn CallbackFn<S, web_sys::Event>>>,
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
            make_closure: |inner, weak_cb| {
                make_closure(move |event| {
                    if let Some(cb) = weak_cb.upgrade() {
                        let inner = unsafe { &*(inner as *const Inner<S, P>) };
                        let cb = unsafe { &*cb.get() };

                        if cb
                            .call(&mut inner.state.borrow_mut(), event)
                            .should_render()
                        {
                            inner.update();
                        }
                    }
                })
            },
            _marker: PhantomData,
        }
    }

    pub(super) fn inner<P: 'static>(&self) -> *const Inner<S, P> {
        self.inner as *const Inner<S, P>
    }

    pub fn callback<F, T, A>(self, cb: F) -> Callback<F, T, Self>
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

pub struct Callback<F, T, L> {
    cb: F,
    link: L,
    _target: PhantomData<T>,
}

impl<F, T, L> WithEventTarget<T> for Callback<F, T, L> {}

// I should not need to write this, but lifetime checking
// was going really off the rails with inlined boxing
#[inline]
fn make_closure<F>(fun: F) -> Box<dyn FnMut(&web_sys::Event)>
where
    F: FnMut(&web_sys::Event) + 'static,
{
    Box::new(fun)
}

pub struct CallbackProduct<F> {
    closure: Closure<dyn FnMut(&web_sys::Event)>,
    cb: Rc<UnsafeCell<F>>,
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

impl<F, T, A, S> Html for Callback<F, T, Link<'_, S>>
where
    F: Fn(&mut S, &Event<T>) -> A + 'static,
    A: Into<ShouldRender>,
    S: 'static,
{
    type Product = CallbackProduct<F>;

    fn build(self) -> Self::Product {
        let Self { link, cb, .. } = self;

        let cb = Rc::new(UnsafeCell::new(cb));
        let weak = Rc::downgrade(&cb);

        let closure = (link.make_closure)(link.inner, unsafe {
            let weak: Weak<UnsafeCell<dyn CallbackFn<S, Event<T>>>> = weak;

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
