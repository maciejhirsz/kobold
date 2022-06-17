use std::marker::PhantomData;

use wasm_bindgen::{JsStatic, JsValue};
use web_sys::Node;
use crate::traits::{Html, Component, Mountable, Link};
use crate::ptr::{Prime, Weak};


/// This is a wrapper for component initialization, we desugar:
///
/// ```ignore
/// <SomeComponent prop={value} />
/// ```
///
/// Into:
///
/// ```ignore
/// ComponentInit::new(SomeComponent { prop: value }, SomeComponent::render)
/// ```
pub struct ComponentInit<C, R, H> {
    component: C,
    render: R,
    _marker: PhantomData<H>,
}

impl<'a, C, R, H> ComponentInit<C, R, H>
where
    C: Component,
    R: Clone + Copy + Fn(&'a C::State, LinkedComponent<C, R, H>) -> H,
    H: Html + 'a,
{
    pub fn new(component: C, render: R) -> Self {
        ComponentInit {
            component,
            render,
            _marker: PhantomData,
        }
    }
}

pub struct ComponentNodeInner<State, Node> {
    state: State,
    node: Node,
}

pub struct ComponentNode<State, Node> {
    inner: Prime<ComponentNodeInner<State, Node>>,
    js: JsValue,
}

impl<S, N> Mountable for ComponentNode<S, N>
where
    N: Mountable,
    S: 'static,
{
    fn js(&self) -> &JsValue {
        &self.js
    }
}

trait ComponentWithNode: Component {
    type Node;
}

pub struct LinkedComponent<C, R, H>
where
    C: Component,
    H: Html,
{
    inner: Weak<ComponentNodeInner<C::State, H::Node>>,
    render: R,
    _marker: PhantomData<H>,
}

impl<'a, C, R, H> Link<C> for LinkedComponent<C, R, H>
where
    C: Component,
    R: Fn(&'a C::State, &dyn Link<C>) -> H,
    H: Html + 'a,
{}

impl<'a, C, R, H> Html for ComponentInit<C, R, H>
where
    C: Component,
    R: Clone + Copy + Fn(&'a C::State, &dyn Link<C>) -> H,
    H: Html + 'a,
{
    type Node = ComponentNode<C::State, H::Node>;

    fn build(self) -> Self::Node {
        let mut inner = Prime::new_uninit();

        let state = self.component.init();

        let node = unsafe {
            // We are expanding the borrow scope from scope to
            // the generic 'a. It is possible to define the
            // generics in such a way that makes the lifetime
            // of the ref taken by the generic closure bound
            // to the generic Html<'a> output of said closure,
            // but doing so makes the code miscompile since
            // lifetime ellision only kicks in at later stage.
            //
            // Regardless of the lifetime, this borrow should
            // always be ephemeral and dropped immediately when
            // the `build` is called, which produces an owned
            // value with 'static lifetime.
            let state = &*(&state as *const C::State);
            let link = LinkedComponent {
                inner: inner.new_weak(),
                render: self.render,
                _marker: PhantomData,
            };

            (self.render)(state, &link).build()
        };
        let js = node.js().clone();

        // let inner = Rc::new(RefCell::new(ComponentNodeInner {
        inner.init(ComponentNodeInner {
            state,
            node,
        });

        ComponentNode {
            inner,
            js,
        }
    }

    fn update(self, built: &mut Self::Node) {
        let mut inner = built
            .inner
            .borrow()
            .expect("Component is currently borrowed by a Weak reference!");

        if self.component.update(&mut inner.state) {
            unsafe {
                // See comment above.
                let state = &*(&inner.state as *const _);
                let link = LinkedComponent {
                    inner: built.inner.new_weak(),
                    render: self.render,
                    _marker: PhantomData,
                };
                (self.render)(state, &link).update(&mut inner.node);
            }
        }
    }
}
