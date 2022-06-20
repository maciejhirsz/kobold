use crate::{Element, Html, Mountable};

pub enum Dead {}

impl Mountable for Dead {
    fn el(&self) -> &Element {
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

                old.el().replace_with(new.js());

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
    fn el(&self) -> &Element {
        match self {
            Branch::A(p) => p.el(),
            Branch::B(p) => p.el(),
            Branch::C(p) => p.el(),
            Branch::D(p) => p.el(),
            Branch::E(p) => p.el(),
            Branch::F(p) => p.el(),
        }
    }
}

pub struct EmptyNode(Element);

impl Mountable for EmptyNode {
    fn el(&self) -> &Element {
        &self.0
    }
}

impl Html for () {
    type Product = EmptyNode;

    fn build(self) -> Self::Product {
        EmptyNode(Element::new_empty())
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

                old.el().replace_with(new.js());

                *old = new;
            }
        }
    }
}
