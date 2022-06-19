use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::{Rc, Weak};

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsValue;
use web_sys::Event;

use crate::render_fn::RenderFn;
use crate::{Html, Mountable, ShouldRender};

pub trait Stateful: Sized {
    type State: 'static;

    fn init(self) -> Self::State;

    fn update(self, state: &mut Self::State) -> ShouldRender;

    fn stateful<'a, H: Html + 'a>(
        self,
        render: fn(&'a Self::State, &'a Link<Self::State, H::Product>) -> H,
    ) -> WithState<Self, H> {
        WithState {
            props: self,
            render: RenderFn::new(render),
            _marker: PhantomData,
        }
    }
}

impl<T: Copy + Eq + 'static> Stateful for T {
    type State = Self;

    fn init(self) -> Self::State {
        self
    }

    fn update(self, state: &mut Self::State) -> ShouldRender {
        if self != *state {
            *state = self;
            ShouldRender::Yes
        } else {
            ShouldRender::No
        }
    }
}

pub struct WithState<S: Stateful, H: Html> {
    props: S,
    render: RenderFn<S::State, H::Product>,
    _marker: PhantomData<H>,
}

struct Inner<S, P> {
    state: RefCell<S>,
    product: RefCell<P>,
    render: RenderFn<S, P>,
    link: Link<S, P>,
    update: fn(RenderFn<S, P>, &Link<S, P>),
}

impl<S, P> Inner<S, P> {
    fn update(&self) {
        (self.update)(self.render, &self.link)
    }
}

pub struct WithStateProduct<S, P> {
    inner: Rc<Inner<S, P>>,
    js: JsValue,
}

pub struct Link<S, P> {
    inner: Weak<Inner<S, P>>,
}

pub struct Callback<F, L> {
    cb: F,
    link: L,
}

// I should not need to write this, but lifetime checking
// was going really off the rails with inlined boxing
#[inline]
fn make_closure<F>(fun: F) -> Box<dyn FnMut(&Event)>
where
    F: FnMut(&Event) + 'static,
{
    Box::new(fun)
}

pub struct CallbackProduct {
    closure: Closure<dyn FnMut(&Event)>,
}

impl<F, A, S, P> Html for Callback<F, &Link<S, P>>
where
    F: Fn(&mut S, &Event) -> A + 'static,
    A: Into<ShouldRender>,
    S: 'static,
    P: 'static,
{
    type Product = CallbackProduct;

    fn build(self) -> Self::Product {
        let link = self.link.clone();
        let cb = self.cb;

        let closure = make_closure(move |event| {
            if let Some(rc) = link.inner.upgrade() {
                if cb(&mut rc.state.borrow_mut(), event).into().should_render() {
                    rc.update();
                }
            }
        });
        let closure = Closure::wrap(closure);

        CallbackProduct { closure }
    }

    fn update(self, _: &mut Self::Product) {}
}

impl Mountable for CallbackProduct {
    fn js(&self) -> &JsValue {
        self.closure.as_ref()
    }
}

impl<S, P> Link<S, P>
where
    S: 'static,
    P: 'static,
{
    pub fn bind<F, A>(&self, cb: F) -> Callback<F, &Self>
    where
        F: Fn(&mut S, &Event) -> A + 'static,
        A: Into<ShouldRender>,
    {
        Callback { cb, link: self }
    }
}

impl<S, P> Clone for Link<S, P> {
    fn clone(&self) -> Self {
        Link {
            inner: self.inner.clone(),
        }
    }
}

impl<S, H> Html for WithState<S, H>
where
    S: Stateful,
    H: Html,
{
    type Product = WithStateProduct<S::State, H::Product>;

    fn build(self) -> Self::Product {
        let state = self.props.init();

        let inner = Rc::new_cyclic(move |inner| {
            let link = Link {
                inner: inner.clone(),
            };

            // Safety: this is safe as long as `S` and `H` are the same types that
            // were used to create this `RenderFn` instance.
            let render_fn = unsafe { self.render.cast::<H>() };
            let product = (render_fn)(&state, &link).build();

            Inner {
                state: RefCell::new(state),
                product: RefCell::new(product),
                render: self.render,
                link,
                update: |render, link| {
                    // Safety: this is safe as long as `S` and `H` are the same types that
                    // were used to create this `RenderFn` instance.
                    let render = unsafe { render.cast::<H>() };

                    if let Some(inner) = link.inner.upgrade() {
                        (render)(&inner.state.borrow(), &link)
                            .update(&mut inner.product.borrow_mut());
                    }
                },
            }
        });

        let js = inner.product.borrow().js().clone();

        WithStateProduct { inner, js }
    }

    fn update(self, p: &mut Self::Product) {
        if self
            .props
            .update(&mut p.inner.state.borrow_mut())
            .should_render()
        {
            p.inner.update();
        }
    }
}

impl<S, P> Mountable for WithStateProduct<S, P>
where
    S: 'static,
    P: Mountable,
{
    fn js(&self) -> &JsValue {
        &self.js
    }
}
