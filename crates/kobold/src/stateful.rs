use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

use crate::render_fn::RenderFn;
use crate::{Element, Html, Mountable};

mod link;

pub use link::{Callback, Link};

/// Describes whether or not a component should be rendered after state changes.
/// For uses see:
///
/// * [Link::callback](Link::callback)
/// * [Stateful::update](Stateful::update)
pub enum ShouldRender {
    /// This is a silent update
    No,

    /// Yes, re-render the component after this update
    Yes,
}

/// Closures without a return type (those that return `()`)
/// are considered to return [`ShouldRender::Yes`](ShouldRender::Yes).
impl From<()> for ShouldRender {
    fn from(_: ()) -> ShouldRender {
        ShouldRender::Yes
    }
}

impl ShouldRender {
    fn should_render(self) -> bool {
        match self {
            ShouldRender::Yes => true,
            ShouldRender::No => false,
        }
    }
}


pub trait Stateful: Sized {
    type State: 'static;

    fn init(self) -> Self::State;

    fn update(self, state: &mut Self::State) -> ShouldRender {
        *state = self.init();

        ShouldRender::Yes
    }

    fn stateful<'a, H: Html + 'a>(
        self,
        render: fn(&'a Self::State, Link<'a, Self::State>) -> H,
    ) -> WithState<Self, H> {
        WithState {
            stateful: self,
            render: RenderFn::new(render),
            _marker: PhantomData,
        }
    }
}

impl<T: Eq + 'static> Stateful for T {
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
    stateful: S,
    render: RenderFn<S::State, H::Product>,
    _marker: PhantomData<H>,
}

struct Inner<S, P> {
    state: RefCell<S>,
    product: RefCell<P>,
    render: RenderFn<S, P>,
    update: fn(RenderFn<S, P>, Link<S>),
}

impl<S: 'static, P: 'static> Inner<S, P> {
    fn update(&self) {
        (self.update)(self.render, Link::new(self))
    }
}

pub struct WithStateProduct<S: 'static, P> {
    inner: Rc<Inner<S, P>>,
    el: Element,
}

impl<S, H> Html for WithState<S, H>
where
    S: Stateful,
    H: Html,
{
    type Product = WithStateProduct<S::State, H::Product>;

    fn build(self) -> Self::Product {
        let state = self.stateful.init();

        let inner = Rc::new_cyclic(move |inner| {
            let link = Link::from_weak(inner);

            // Safety: this is safe as long as `S` and `H` are the same types that
            // were used to create this `RenderFn` instance.
            let render_fn = unsafe { self.render.cast::<H>() };
            let product = (render_fn)(&state, link).build();

            Inner {
                state: RefCell::new(state),
                product: RefCell::new(product),
                render: self.render,
                update: |render, link| {
                    // Safety: this is safe as long as `S` and `H` are the same types that
                    // were used to create this `RenderFn` instance.
                    let render = unsafe { render.cast::<H>() };
                    let inner = unsafe { &*link.inner() };

                    (render)(&inner.state.borrow(), link)
                        .update(&mut inner.product.borrow_mut());
                },
            }
        });

        let el = inner.product.borrow().el().clone();

        WithStateProduct { inner, el }
    }

    fn update(self, p: &mut Self::Product) {
        if self
            .stateful
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
    fn el(&self) -> &Element {
        &self.el
    }
}
