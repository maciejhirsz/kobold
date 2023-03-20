use std::mem::MaybeUninit;
use std::ops::Deref;
use std::rc::{Rc, Weak};

use wasm_bindgen::closure::Closure as WasmClosure;
use wasm_bindgen::JsValue;
use web_sys::Node;

use crate::stateful::IntoState;
use crate::util::WithCell;
use crate::{dom::Element, Html, Mountable};

pub struct Inner<S> {
    hook: Hook<S>,
    updater: Box<dyn FnMut(&Hook<S>)>,
}

impl<S> Inner<S> {
    fn update(&mut self) {
        (self.updater)(&self.hook)
    }
}

pub struct Hook<S> {
    state: S,
    weak: Weak<WithCell<Inner<S>>>,
}

pub struct Signal<S> {
    weak: Weak<WithCell<Inner<S>>>,
}

pub trait ShouldRender {
    fn should_render(self) -> bool;
}

impl ShouldRender for () {
    fn should_render(self) -> bool {
        true
    }
}

pub enum Then {
    Stop,
    Render,
}

impl ShouldRender for Then {
    fn should_render(self) -> bool {
        match self {
            Then::Stop => false,
            Then::Render => true,
        }
    }
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

impl<F> Html for Closure<F>
where
    F: Fn(web_sys::Event) + 'static,
{
    type Product = ClosureProduct<F>;

    fn build(self) -> Self::Product {
        ClosureProduct::make(self.0)
    }

    fn update(self, p: &mut Self::Product) {
        p.update(self.0);
    }
}

pub struct Closure<F>(F);

impl<F> Mountable for ClosureProduct<F>
where
    F: 'static,
{
    type Js = JsValue;

    fn el(&self) -> &Element {
        panic!()
    }

    fn js(&self) -> &JsValue {
        &self.js
    }
}

pub struct ClosureProduct<F> {
    js: JsValue,
    boxed: Box<F>,
}

impl<F> ClosureProduct<F>
where
    F: FnMut(web_sys::Event) + 'static,
{
    fn make(f: F) -> Self {
        let raw = Box::into_raw(Box::new(f));

        let js = WasmClosure::wrap(unsafe { Box::from_raw(raw) } as Box<dyn FnMut(web_sys::Event)>)
            .into_js_value();

        // `into_js_value` will _forget_ the previous Box, so we can safely reconstruct it
        let boxed = unsafe { Box::from_raw(raw) };

        ClosureProduct { js, boxed }
    }

    fn update(&mut self, f: F) {
        *self.boxed = f;
    }
}

impl<S> Deref for Hook<S> {
    type Target = S;

    fn deref(&self) -> &S {
        &self.state
    }
}

pub struct Stateful<S, F> {
    state: S,
    render: F,
}

pub struct StatefulProduct<S> {
    inner: Rc<WithCell<Inner<S>>>,
    el: Element,
}

pub fn stateful<S, F, H>(state: S, render: F) -> Stateful<S, F>
where
    S: IntoState,
    F: Fn(&Hook<S::State>) -> H + 'static,
    H: Html,
{
    Stateful { state, render }
}

impl<S, F, H> Html for Stateful<S, F>
where
    S: IntoState,
    F: Fn(&Hook<S::State>) -> H + 'static,
    H: Html,
{
    type Product = StatefulProduct<S::State>;

    fn build(self) -> Self::Product {
        let mut el = MaybeUninit::uninit();
        let el_ref = &mut el;

        let inner = Rc::new_cyclic(move |weak| {
            let hook = Hook {
                state: self.state.init(),
                weak: weak.clone(),
            };

            let mut product = (self.render)(&hook).build();

            el_ref.write(product.el().clone());

            WithCell::new(Inner {
                hook,
                updater: Box::new(move |hook| {
                    (self.render)(hook).update(&mut product);
                }),
            })
        });

        StatefulProduct {
            inner,
            el: unsafe { el.assume_init() },
        }
    }

    fn update(self, p: &mut Self::Product) {
        p.inner.with(|inner| {
            if self.state.update(&mut inner.hook.state).should_render() {
                inner.update();
            }
        });
    }
}

impl<S: 'static> Mountable for StatefulProduct<S> {
    type Js = Node;

    fn el(&self) -> &Element {
        &self.el
    }
}
