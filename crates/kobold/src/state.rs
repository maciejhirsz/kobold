use std::cell::RefCell;
use std::rc::{Rc, Weak};

use web_sys::Node;

use crate::{dom::Element, Html, Mountable};

pub struct Inner<S> {
    hook: RefCell<Hook<S>>,
    el: Element,
    updater: Box<dyn FnMut(&Hook<S>)>,
}

pub struct Hook<S> {
    state: S,
    weak: Weak<Inner<S>>,
}

pub struct Stateful<S, F> {
    state: S,
    render: F,
}

pub struct StatefulProduct<S> {
    inner: Rc<Inner<S>>,
}

pub fn stateful<S, F>(state: S, render: F) -> Stateful<S, F> {
    Stateful { state, render }
}

impl<S, F, H> Html for Stateful<S, F>
where
    S: 'static,
    F: Fn(&Hook<S>) -> H + 'static,
    H: Html,
{
    type Product = StatefulProduct<S>;

    fn build(self) -> Self::Product {
        let inner = Rc::new_cyclic(move |weak| {
            let hook = Hook { state: self.state, weak: weak.clone() };

            let mut product = (self.render)(&hook).build();

            Inner {
                hook: RefCell::new(hook),
                el: product.el().clone(),
                updater: Box::new(move |hook| {
                    (self.render)(hook).update(&mut product);
                })
            }
        });

        StatefulProduct { inner }
    }

    fn update(self, p: &mut Self::Product) {}
}

impl<S: 'static> Mountable for StatefulProduct<S> {
    type Js = Node;

    fn el(&self) -> &Element {
        &self.inner.el
    }
}
