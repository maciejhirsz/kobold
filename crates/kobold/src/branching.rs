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
//! fn conditional(illuminatus: bool) -> impl View {
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
//! fn conditional(illuminatus: bool) -> impl View {
//!     if illuminatus {
//!         view! { <p>"It was the year when they finally immanentized the Eschaton."</p> }
//!     } else {
//!         view! { <blockquote>"It was love at first sight."</blockquote> }
//!     }
//! }
//! # fn main() {}
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
//! fn conditional(illuminatus: bool) -> impl View {
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
//! # fn main() {}
//! ```
//!
//! This is in fact all that the [`auto_branch`](crate::component#componentauto_branch) flag does for you automatically.
//!
//! For simple optional renders you can always use the standard library [`Option`]:
//!
//! ```
//! # use kobold::prelude::*;
//! #[component]
//! fn conditional(illuminatus: bool) -> impl View {
//!     if illuminatus {
//!         Some(view! {
//!             <p>"It was the year when they finally immanentized the Eschaton."</p>
//!         })
//!     } else {
//!         None
//!     }
//! }
//! # fn main() {}
//! ```

use std::mem::replace;

use wasm_bindgen::JsValue;
use web_sys::Node;

use crate::dom::Anchor;
use crate::internal::empty_node;
use crate::runtime::{EventContext, Then, Trigger, UsedEvents};
use crate::{Mountable, View};

macro_rules! branch {
    ($name:ident < $($var:ident),* >) => {
        #[repr(C)]
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
                        $name::$var(view) => $name::$var(view.build()),
                    )*
                }
            }

            fn update(self, p: &mut Self::Product) {
                match (self, p) {
                    $(
                        ($name::$var(view), $name::$var(p)) => view.update(p),
                    )*

                    (view, p) => {
                        let old = replace(p, view.build());

                        old.replace_with(p.js());
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
            const EVENTS: UsedEvents = UsedEvents::empty()
            $(
                .combine($var::EVENTS)
            )*;

            type Js = Node;

            fn js(&self) -> &JsValue {
                match self {
                    $(
                        $name::$var(p) => p.js(),
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

            fn replace_with(&self, new: &JsValue) {
                match self {
                    $(
                        $name::$var(p) => p.replace_with(new),
                    )*
                }
            }
        }

        impl<$($var),*> Trigger for $name<$($var),*>
        where
            $(
                $var: Trigger,
            )*
        {
            fn trigger<Ctx: EventContext>(&mut self, ctx: &mut Ctx) -> Option<Then> {
                match self {
                    $(
                        $name::$var(p) => p.trigger(ctx),
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

impl Anchor for EmptyNode {
    type Js = Node;
    type Target = Node;

    fn anchor(&self) -> &Node {
        &self.0
    }
}

impl Trigger for EmptyNode {}

impl View for Empty {
    type Product = EmptyNode;

    fn build(self) -> EmptyNode {
        EmptyNode(empty_node())
    }

    fn update(self, _: &mut EmptyNode) {}
}

impl<T: View> View for Option<T> {
    type Product = Branch2<T::Product, EmptyNode>;

    fn build(self) -> Self::Product {
        match self {
            Some(view) => Branch2::A(view.build()),
            None => Branch2::B(EmptyNode(empty_node())),
        }
    }

    fn update(self, p: &mut Self::Product) {
        match (self, p) {
            (Some(view), Branch2::A(p)) => view.update(p),
            (None, Branch2::B(_)) => (),

            (view, p) => {
                let old = replace(p, view.build());

                old.replace_with(p.js());
            }
        }
    }
}
