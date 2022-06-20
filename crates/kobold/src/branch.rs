use crate::{Element, Html, Mountable};

macro_rules! branch {
    ($name:ident < $($var:ident),* >) => {
        pub enum $name<$($var),*> {
            $(
                $var($var),
            )*
        }

        impl<$($var),*> Html for $name<$($var),*>
        where
            $(
                $var: Html,
            )*
        {
            type Product = $name<$($var::Product),*>;

            fn build(self) -> Self::Product {
                match self {
                    $(
                        $name::$var(html) => $name::$var(html.build()),
                    )*
                }
            }

            fn update(self, p: &mut Self::Product) {
                match (self, p) {
                    $(
                        ($name::$var(html), $name::$var(p)) => html.update(p),
                    )*

                    (html, old) => {
                        let new = html.build();

                        old.el().replace_with(new.js());

                        *old = new;
                    }
                }
            }
        }

        impl<$($var),*> Mountable for $name<$($var),*>
        where
            $(
                $var: Mountable,
            )*
        {
            fn el(&self) -> &Element {
                match self {
                    $(
                        $name::$var(p) => p.el(),
                    )*
                }
            }
        }

    };
}

branch!(Branch2<A, B>);
branch!(Branch3<A, B, C>);
branch!(Branch4<A, B, C, D>);
branch!(Branch5<A, B, C, D, E>);
branch!(Branch6<A, B, C, D, E, F>);
branch!(Branch7<A, B, C, D, E, F, G>);
branch!(Branch8<A, B, C, D, E, F, G, H>);
branch!(Branch9<A, B, C, D, E, F, G, H, I>);

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
