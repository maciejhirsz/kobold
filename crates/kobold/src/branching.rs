// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! # Utilities for conditional rendering
//!
//! The [`view!`](crate::view) macro produces unique transient types, so you might run into compile errors when branching:
//!
//! ```compile_fail
//! # use kobold::prelude::*;
//! #[component]
//! fn Conditional(illuminatus: bool) -> impl View {
//!     if illuminatus {
//!         view! { <p>"It was the year when they finally immanentized the Eschaton."</p> }
//!     } else {
//!         view! { <blockquote>"It was love at first sight."</blockquote> }
//!     }
//! }
//! ```
//!
//! Here Rust will inform you that:
//!
//! ```text
//! /     if illuminatus {
//! |         view! { <p>"It was the year when they finally immanentized the Eschaton."</p> }
//! |         ------------------------------------------------------------------------------- expected because of this
//! |     } else {
//! |         view! { <blockquote>"It was love at first sight."</blockquote> }
//! |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected struct `Conditional::render::Transient`, found a different struct `Conditional::render::Transient`
//! |     }
//! |_____- `if` and `else` have incompatible types
//! ```
//!
//! While both types are _named_ `Transient`, they are in fact different types defined inline by the macro.
//!
//! In most cases all you have to do is annotate such component with [`#[component(auto_branch)]`](crate::component#componentauto_branch):
//!
//! ```
//! # use kobold::prelude::*;
//! #[component(auto_branch)]
//! fn Conditional(illuminatus: bool) -> impl View {
//!     if illuminatus {
//!         view! { <p>"It was the year when they finally immanentized the Eschaton."</p> }
//!     } else {
//!         view! { <blockquote>"It was love at first sight."</blockquote> }
//!     }
//! }
//! ```
//!
//! This flag is not enabled by default, yet, as there might be situations [`auto_branch`](crate::component#componentauto_branch)
//! doesn't handle correctly.
//!
//! ## Manual branching
//!
//! An always safe if more laborious way is to manually use one of the [`BranchN` enums](self#enums) from this module:
//!
//! ```
//! # use kobold::prelude::*;
//! use kobold::branching::Branch2;
//!
//! #[component]
//! fn Conditional(illuminatus: bool) -> impl View {
//!     if illuminatus {
//!         Branch2::A(view! {
//!             <p>"It was the year when they finally immanentized the Eschaton."</p>
//!         })
//!     } else {
//!         Branch2::B(view! {
//!             <blockquote>"It was love at first sight."</blockquote>
//!         })
//!     }
//! }
//! ```
//!
//! This is in fact all that the [`auto_branch`](crate::component#componentauto_branch) flag does for you automatically.
//!
//! For simple optional renders you can always use the standard library [`Option`](Option):
//!
//! ```
//! # use kobold::prelude::*;
//! #[component]
//! fn Conditional(illuminatus: bool) -> impl View {
//!     if illuminatus {
//!         Some(view! {
//!             <p>"It was the year when they finally immanentized the Eschaton."</p>
//!         })
//!     } else {
//!         None
//!     }
//! }
//! ```

use wasm_bindgen::JsValue;
use web_sys::Node;

use crate::dom::{self, Anchor, DynAnchor};
use crate::{Mountable, View};

macro_rules! branch {
    ($name:ident < $($var:ident),* >) => {
        pub enum $name<$($var),*> {
            $(
                $var($var),
            )*
        }

        impl<$($var),*> View for $name<$($var),*>
        where
            $(
                $var: View,
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

                        old.anchor().replace_with(new.js());

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
            type Js = Node;
            type Anchor = DynAnchor;

            fn anchor(&self) -> &DynAnchor {
                match self {
                    $(
                        $name::$var(p) => p.anchor().as_dyn(),
                    )*
                }
            }

            fn replace_with(&self, new: &JsValue) {
                match self {
                    $(
                        $name::$var(p) => p.replace_with(new),
                    )*
                }
            }

            fn unmount(&self) {
                match self {
                    $(
                        $name::$var(p) => p.unmount(),
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

pub struct EmptyNode(Node);

pub struct Empty;

impl Mountable for EmptyNode {
    type Js = Node;
    type Anchor = Node;

    fn anchor(&self) -> &Node {
        &self.0
    }
}

impl View for Empty {
    type Product = EmptyNode;

    fn build(self) -> Self::Product {
        EmptyNode(dom::empty_node())
    }

    fn update(self, _: &mut Self::Product) {}
}

impl<T: View> View for Option<T> {
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

                old.replace_with(new.js());

                *old = new;
            }
        }
    }
}
