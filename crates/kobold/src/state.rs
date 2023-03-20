use std::cell::RefCell;
use std::rc::Rc;

use web_sys::Node;

use crate::{dom::Element, Html, Mountable};

pub struct Hook<S> {
    state: RefCell<S>,
    updater: Option<Box<dyn FnMut(&Hook<S>)>>,
}

pub struct Stateful<S, F> {
    state: S,
    render: F,
}

pub struct StatefulProduct<S> {
    hook: Rc<Hook<S>>,
    el: Element,
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
        let mut hook = Rc::new(Hook {
            state: RefCell::new(self.state),
            updater: None,
        });

        let el = {
            let hook = Rc::get_mut(&mut hook).unwrap();
            let mut product = (self.render)(hook).build();
            let el = product.el().clone();

            hook.updater = Some(Box::new(move |hook| {
                (self.render)(hook).update(&mut product);
            }));

            el
        };

        StatefulProduct {
        	hook,
        	el,
        }
    }

    fn update(self, p: &mut Self::Product) {}
}

impl<S: 'static> Mountable for StatefulProduct<S> {
    type Js = Node;

    fn el(&self) -> &Element {
        &self.el
    }
}
