//! # Utilities for conditional rendering
//!
//! The [`html!`](crate::html) macro produces unique transient types, so you might run into compile errors when branching:
//!
//! ```compile_fail
//! # use kobold::prelude::*;
//! fn conditional(illuminatus: bool) -> impl Html {
//!     if illuminatus {
//!         html! { <p>"It was the year when they finally immanentized the Eschaton."</p> }
//!     } else {
//!         html! { <blockquote>"It was love at first sight."</blockquote> }
//!     }
//! }
//! ```
//!
//! Here Rust will inform you:
//!
//! ```text
//! /     if illuminatus {
//! |         html! { <p>"It was the year when they finally immanentized the Eschaton."</p> }
//! |         ------------------------------------------------------------------------------- expected because of this
//! |     } else {
//! |         html! { <blockquote>"It was love at first sight."</blockquote> }
//! |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected struct `conditional::Transient`, found a different struct `conditional::Transient`
//! |     }
//! |_____- `if` and `else` have incompatible types
//! ```
//!
//! While both types are _named_ `Transient`, they are in fact different types defined inline by the macro.
//!
//! To fix this, all you have to do is annotate the function with [`#[kobold::branching]`](macro@crate::branching):
//!
//! ```
//! # use kobold::prelude::*;
//! #[kobold::branching]
//! fn conditional(illuminatus: bool) -> impl Html {
//!     if illuminatus {
//!         html! { <p>"It was the year when they finally immanentized the Eschaton."</p> }
//!     } else {
//!         html! { <blockquote>"It was love at first sight."</blockquote> }
//!     }
//! }
//! ```
//!
//! This is still a somewhat experimental feature and **Kobold** doesn't (yet) perform control flow analysis here.
//! A safe, if more laborious, way is to manually use one of the enums from this module:
//!
//! ```
//! # use kobold::prelude::*;
//! use kobold::branching::Branch2;
//!
//! fn conditional(illuminatus: bool) -> impl Html {
//!     if illuminatus {
//!         Branch2::A(html! {
//!             <p>"It was the year when they finally immanentized the Eschaton."</p>
//!         })
//!     } else {
//!         Branch2::B(html! {
//!             <blockquote>"It was love at first sight."</blockquote>
//!         })
//!     }
//! }
//! ```
//!
//! This is in fact all that `#[kobold::branching]` does for you automatically.
//!
//! For simple optional renders you can always use standard the library `Option`:
//!
//! ```
//! # use kobold::prelude::*;
//! fn conditional(illuminatus: bool) -> impl Html {
//!     if illuminatus {
//!         Some(html! {
//!             <p>"It was the year when they finally immanentized the Eschaton."</p>
//!         })
//!     } else {
//!         None
//!     }
//! }
//! ```

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

pub struct Empty;

impl Mountable for EmptyNode {
    fn el(&self) -> &Element {
        &self.0
    }
}

impl Html for Empty {
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
            None => Branch2::B(Empty.build()),
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
