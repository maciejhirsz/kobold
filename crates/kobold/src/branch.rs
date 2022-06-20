use crate::{util, Html, JsValue, Mountable, Node};

pub enum Dead {}

impl Mountable for Dead {
    fn js(&self) -> &JsValue {
        unreachable!()
    }
}

impl Html for Dead {
    type Product = Dead;

    fn build(self) -> Self::Product {
        self
    }

    fn update(self, _: &mut Self::Product) {}
}

pub type Branch2<A, B> = Branch<A, B, Dead, Dead, Dead, Dead>;
pub type Branch3<A, B, C> = Branch<A, B, C, Dead, Dead, Dead>;
pub type Branch4<A, B, C, D> = Branch<A, B, C, D, Dead, Dead>;
pub type Branch5<A, B, C, D, E> = Branch<A, B, C, D, E, Dead>;
pub type Branch6<A, B, C, D, E, F> = Branch<A, B, C, D, E, F>;

pub enum Branch<A, B, C, D, E, F> {
    A(A),
    B(B),
    C(C),
    D(D),
    E(E),
    F(F),
}

impl<A, B, C, D, E, F> Html for Branch<A, B, C, D, E, F>
where
    A: Html,
    B: Html,
    C: Html,
    D: Html,
    E: Html,
    F: Html,
{
    type Product = Branch<A::Product, B::Product, C::Product, D::Product, E::Product, F::Product>;

    fn build(self) -> Self::Product {
        match self {
            Branch::A(html) => Branch::A(html.build()),
            Branch::B(html) => Branch::B(html.build()),
            Branch::C(html) => Branch::C(html.build()),
            Branch::D(html) => Branch::D(html.build()),
            Branch::E(html) => Branch::E(html.build()),
            Branch::F(html) => Branch::F(html.build()),
        }
    }

    fn update(self, p: &mut Self::Product) {
        match (self, p) {
            (Branch::A(html), Branch::A(p)) => html.update(p),
            (Branch::B(html), Branch::B(p)) => html.update(p),
            (Branch::C(html), Branch::C(p)) => html.update(p),
            (Branch::D(html), Branch::D(p)) => html.update(p),
            (Branch::E(html), Branch::E(p)) => html.update(p),
            (Branch::F(html), Branch::F(p)) => html.update(p),

            (html, old) => {
                let new = html.build();

                new.mount_replace(old);

                *old = new;
            }
        }
    }
}

impl<A, B, C, D, E, F> Mountable for Branch<A, B, C, D, E, F>
where
    A: Mountable,
    B: Mountable,
    C: Mountable,
    D: Mountable,
    E: Mountable,
    F: Mountable,
{
    fn js(&self) -> &JsValue {
        match self {
            Branch::A(p) => p.js(),
            Branch::B(p) => p.js(),
            Branch::C(p) => p.js(),
            Branch::D(p) => p.js(),
            Branch::E(p) => p.js(),
            Branch::F(p) => p.js(),
        }
    }
}

pub struct EmptyNode(Node);

impl Mountable for EmptyNode {
    fn js(&self) -> &JsValue {
        self.0.as_ref()
    }
}

impl Html for () {
    type Product = EmptyNode;

    fn build(self) -> Self::Product {
        EmptyNode(util::__kobold_empty_node())
    }

    fn update(self, _: &mut Self::Product) {}
}

impl<T: Html> Html for Option<T> {
    type Product = Branch2<T::Product, EmptyNode>;

    fn build(self) -> Self::Product {
        match self {
            Some(html) => Branch2::A(html.build()),
            None => Branch2::B(().build()),
        }
    }

    fn update(self, p: &mut Self::Product) {
        match (self, p) {
            (Some(html), Branch2::A(p)) => html.update(p),
            (None, Branch2::B(_)) => (),

            (html, old) => {
                let new = html.build();

                new.mount_replace(old);

                *old = new;
            }
        }
    }
}
