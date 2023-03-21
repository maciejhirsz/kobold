use std::mem::MaybeUninit;
use std::rc::Rc;

use web_sys::Node;

use crate::util::WithCell;
use crate::{dom::Element, Html, Mountable};

mod hook;
mod should_render;

pub use hook::{Hook, Signal};
pub use should_render::{ShouldRender, Then};

pub struct Inner<S> {
    hook: Hook<S>,
    updater: Box<dyn FnMut(&Hook<S>)>,
}

impl<S> Inner<S> {
    fn update(&mut self) {
        (self.updater)(&self.hook)
    }
}

/// Trait used to create stateful components, see the [module documentation](crate::stateful) for details.
pub trait IntoState: Sized {
    type State: 'static;

    fn init(self) -> Self::State;

    fn update(self, state: &mut Self::State) -> Then;
}

impl<F, S> IntoState for F
where
    S: 'static,
    F: FnOnce() -> S,
{
    type State = S;

    fn init(self) -> Self::State {
        (self)()
    }

    fn update(self, _: &mut Self::State) -> Then {
        Then::Stop
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

pub fn stateful<'a, S, F, H>(state: S, render: F) -> Stateful<S, impl Fn(*const Hook<S::State>) -> H + 'static>
where
    S: IntoState,
    F: Fn(&'a Hook<S::State>) -> H + 'static,
    H: Html + 'a,
{
    let render = move |hook: *const Hook<S::State>| render(unsafe { &*hook });
    Stateful { state, render }
}

impl<S, F, H> Html for Stateful<S, F>
where
    S: IntoState,
    F: Fn(*const Hook<S::State>) -> H + 'static,
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
