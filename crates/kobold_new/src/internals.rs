use std::marker::PhantomData;

use wasm_bindgen::{JsStatic, JsValue};
use web_sys::Node;
use crate::traits::{Html, Component, Mountable};
use crate::ptr::Prime;


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
pub struct ComponentInit<'a, C: 'a> {
    component: C,
    _marker: PhantomData<&'a ()>,
}

impl<'a, C: 'a> ComponentInit<'a, C> {
    pub fn new(component: C) -> Self {
        ComponentInit {
            component,
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

impl<'a, C> Html for ComponentInit<'a, C>
where
    C: Component + 'a,
{
    type Node = ComponentNode<C::State, <C::Rendered<'a> as Html>::Node>;

    fn build(self) -> Self::Node {
        let mut inner = Prime::new_uninit();

        let state = self.component.init();

        let node = C::render(&state).build();
        // let node = C::render(unsafe {
        //     // We are expanding the borrow scope from scope to
        //     // the generic 'a. It is possible to define the
        //     // generics in such a way that makes the lifetime
        //     // of the ref taken by the generic closure bound
        //     // to the generic Html<'a> output of said closure,
        //     // but doing so makes the code miscompile since
        //     // lifetime ellision only kicks in at later stage.
        //     //
        //     // Regardless of the lifetime, this borrow should
        //     // always be ephemeral and dropped immediately when
        //     // the `build` is called, which produces an owned
        //     // value with 'static lifetime.
        //     &*(&state as *const C::State)
        // }).build();
        // let node = (self.render)(unsafe {
        //     // We are expanding the borrow scope from scope to
        //     // the generic 'a. It is possible to define the
        //     // generics in such a way that makes the lifetime
        //     // of the ref taken by the generic closure bound
        //     // to the generic Html<'a> output of said closure,
        //     // but doing so makes the code miscompile since
        //     // lifetime ellision only kicks in at later stage.
        //     //
        //     // Regardless of the lifetime, this borrow should
        //     // always be ephemeral and dropped immediately when
        //     // the `build` is called, which produces an owned
        //     // value with 'static lifetime.
        //     &*(&state as *const C::State)
        // }).build();
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
            C::render(&inner.state).update(&mut inner.node);
            // C::render(unsafe {
            //     // See comment above.
            //     &*(&inner.state as *const C::State)
            // }).update(&mut inner.node);
            // (self.render)(unsafe {
            //     // See comment above.
            //     &*(&inner.state as *const C::State)
            // }).update(&mut inner.node);
        }
    }
}
