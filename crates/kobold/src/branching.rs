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

use std::mem::MaybeUninit;

use wasm_bindgen::JsValue;
use web_sys::Node;

use crate::dom::{self, Anchor};
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

                        old.replace_with(new.js());

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

            fn js(&self) -> &JsValue {
                match self {
                    $(
                        $name::$var(p) => p.js(),
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

// branch!(Branch2<A, B>);
branch!(Branch3<A, B, C>);
branch!(Branch4<A, B, C, D>);
branch!(Branch5<A, B, C, D, E>);
branch!(Branch6<A, B, C, D, E, F>);
branch!(Branch7<A, B, C, D, E, F, G>);
branch!(Branch8<A, B, C, D, E, F, G, H>);
branch!(Branch9<A, B, C, D, E, F, G, H, I>);

#[repr(C)]
pub struct EitherProduct<A, B> {
    tag: EitherTag,
    a: MaybeUninit<A>,
    b: MaybeUninit<B>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum EitherTag {
    A,
    B,
    AB,
    BA,
}

impl<A, B> EitherProduct<A, B>
where
    A: Mountable,
    B: Mountable,
{
    const fn a(prod: A) -> Self {
        EitherProduct {
            tag: EitherTag::A,
            a: MaybeUninit::new(prod),
            b: MaybeUninit::uninit(),
        }
    }

    const fn b(prod: B) -> Self {
        EitherProduct {
            tag: EitherTag::B,
            a: MaybeUninit::uninit(),
            b: MaybeUninit::new(prod),
        }
    }

    fn update_a<V>(&mut self, view: V)
    where
        V: View<Product = A>,
    {
        if matches!(self.tag, EitherTag::B) {
            self.a.write(view.build());
        } else {
            view.update(unsafe { self.a.assume_init_mut() })
        }

        if matches!(self.tag, EitherTag::B | EitherTag::BA) {
            let a = unsafe { self.a.assume_init_ref() };
            let b = unsafe { self.b.assume_init_ref() };

            b.replace_with(a.js());

            self.tag = EitherTag::AB;
        }
    }

    fn update_b<V>(&mut self, view: V)
    where
        V: View<Product = B>,
    {
        if matches!(self.tag, EitherTag::A) {
            self.b.write(view.build());
        } else {
            view.update(unsafe { self.b.assume_init_mut() })
        }

        if matches!(self.tag, EitherTag::A | EitherTag::AB) {
            let a = unsafe { self.a.assume_init_ref() };
            let b = unsafe { self.b.assume_init_ref() };

            a.replace_with(b.js());

            self.tag = EitherTag::BA;
        }
    }
}

impl<A, B> Mountable for EitherProduct<A, B>
where
    A: Mountable,
    B: Mountable,
{
    type Js = Node;

    fn js(&self) -> &JsValue {
        match self.tag {
            EitherTag::A | EitherTag::AB => unsafe { self.a.assume_init_ref().js() },
            EitherTag::B | EitherTag::BA => unsafe { self.b.assume_init_ref().js() },
        }
    }

    fn replace_with(&self, new: &JsValue) {
        match self.tag {
            EitherTag::A | EitherTag::AB => unsafe { self.a.assume_init_ref().replace_with(new) },
            EitherTag::B | EitherTag::BA => unsafe { self.b.assume_init_ref().replace_with(new) },
        }
    }

    fn unmount(&self) {
        match self.tag {
            EitherTag::A | EitherTag::AB => unsafe { self.a.assume_init_ref().unmount() },
            EitherTag::B | EitherTag::BA => unsafe { self.b.assume_init_ref().unmount() },
        }
    }
}

pub enum Branch2<A, B> {
    A(A),
    B(B),
}

impl<A, B> View for Branch2<A, B>
where
    A: View,
    B: View,
{
    type Product = EitherProduct<A::Product, B::Product>;

    fn build(self) -> Self::Product {
        match self {
            Branch2::A(view) => EitherProduct::a(view.build()),
            Branch2::B(view) => EitherProduct::b(view.build()),
        }
    }

    fn update(self, p: &mut Self::Product) {
        match self {
            Branch2::A(view) => p.update_a(view),
            Branch2::B(view) => p.update_b(view),
        }
    }
}

impl<A, B> Drop for EitherProduct<A, B> {
    fn drop(&mut self) {
        // drop A if tag is either A, AB, or BA
        if self.tag != EitherTag::B {
            unsafe {
                self.a.assume_init_drop();
            }
        }
        // drop B if tag is either B, AB, or BA
        if self.tag != EitherTag::A {
            unsafe {
                self.b.assume_init_drop();
            }
        }
    }
}

pub struct EmptyNode(Node);

impl Anchor for EmptyNode {
    type Js = Node;
    type Target = Node;

    fn anchor(&self) -> &Node {
        &self.0
    }
}

impl View for () {
    type Product = EmptyNode;

    fn build(self) -> Self::Product {
        EmptyNode(dom::empty_node())
    }

    fn update(self, _: &mut Self::Product) {}
}

impl<T> View for Option<T>
where
    T: View,
{
    type Product = EitherProduct<T::Product, EmptyNode>;

    fn build(self) -> Self::Product {
        match self {
            Some(html) => EitherProduct::a(html.build()),
            None => EitherProduct::b(().build()),
        }
    }

    fn update(self, p: &mut Self::Product) {
        match self {
            Some(html) => p.update_a(html),
            None => p.update_b(()),
        }
    }
}
