use wasm_bindgen::JsValue;
use web_sys::Node;
use crate::traits::{Html, Component, Mountable};

pub struct ComponentInit<C, R> {
    pub component: C,
    pub render: R,
}

pub struct ComponentNode<State, Node> {
    state: State,
    node: Node,
}

impl<S, N> Mountable for ComponentNode<S, N>
where
    N: Mountable,
    S: 'static,
{
    fn js(&self) -> &JsValue {
        self.node.js()
    }

    fn mount(&self, parent: &Node) {
        self.node.mount(parent);
    }

    fn unmount(&self, parent: &Node) {
        self.node.unmount(parent);
    }
}

pub trait Render<'a, In>: Sized {
    type Node: Mountable + 'static;

    fn build(self, input: &'a In) -> Self::Node;

    fn update(self, input: &'a In, node: &mut Self::Node);
}

impl<'a, In, Out, F> Render<'a, In> for F
where
    In: 'a,
    F: Fn(&'a In) -> Out,
    Out: Html<'a>,
    Out::Node: 'static,
{
    type Node = <Out as Html<'a>>::Node;

    fn build(self, input: &'a In) -> Self::Node {
        (self)(input).build()
    }

    fn update(self, input: &'a In, node: &mut Self::Node) {
        (self)(input).update(node);
    }
}

impl<'a, C, R> Html<'a> for ComponentInit<C, R>
where
    C: 'static,
    R: Render<'a, C>,
{
    type Node = ComponentNode<C, R::Node>;

    fn build(self) -> Self::Node {
        // let node = self.render.build(&self.component);
        panic!();
    }

    fn update(self, node: &mut Self::Node) {
        panic!();
    }
}

// impl<'a, C, R> Html<'a> for ComponentInit<C, R>
// where
//     R: Render<'a, C>,
// {
//     type Node = ComponentNode<c, <R::Out as Html>::Node>,
// }
// impl<'a, C, R> Html<'a> for ComponentInit<C, R> {
//     type Node = ComponentNode<C, N>;

//     fn build(self) ->
// }


// impl<C, N, B, U> Html<'_> for ComponentInit<C, B, U>
// where
//     // C: Component,
//     B: Fn(&C::State) -> N,
//     U: Fn(&mut C::State, &mut N),
//     N: Mountable,
// {
//     type Node = ComponentNode<C::State, N>;

//     fn build(self) -> Self::Node {
//         let state = self.component.init();
//         let node = (self.build)(&state);

//         ComponentNode { state, node }
//     }

//     fn update(self, node: &mut Self::Node) {
//         (self.update)(&mut node.state, &mut node.node)
//     }
// }
