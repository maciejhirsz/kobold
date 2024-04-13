// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::ops::Deref;

use crate::internal::In;
use crate::stateful::IntoState;
use crate::View;

#[repr(transparent)]
pub struct Hook<S> {
    state: S,
}

impl<S> Hook<S> {
    fn new(state: &S) -> &Hook<S> {
        unsafe { &*(state as *const _ as *const Hook<S>) }
    }
}

impl<S> Deref for Hook<S> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

pub fn stateful<S, F, V>(
    state: S,
    render: F,
) -> Stateful<
    S,
    impl FnOnce(&Hook<S::State>, In<V::Product>),
    impl Fn(&Hook<S::State>, &mut V::Product),
>
where
    S: IntoState,
    F: Fn(&Hook<S::State>) -> V,
    F: Copy,
    V: View,
{
    // There is no safe way to represent a generic closure with generic return type
    // that borrows from that closure's arguments, without also slapping a lifetime.
    //
    // The `stateful` function ensures that correct lifetimes are used before we
    // erase them for the use in the `Stateful` struct.
    Stateful {
        state,
        build: move |hook: &_, p: In<'_, _>| {
            render(hook).build(p);
        },
        update: move |hook: &_, p: &mut _| render(hook).update(p),
    }
}

pub struct Stateful<S, B, U> {
    state: S,
    build: B,
    update: U,
}

#[repr(C)]
pub struct StatefulProduct<S, U: ?Sized> {
    state: S,
    update: U,
}

// impl<S, F, V> View for Stateful<S, F>
// where
//     S: IntoState,
//     F: Fn(&'a Hook<S::State>) -> V + 'a,
//     V: View + 'a,
// {
// }

// pub struct StatefulProduct<U: ?Sized> {
//     update: U,
// }
