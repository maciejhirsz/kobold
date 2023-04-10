// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::diff::Diff;
use crate::stateful::Then;

/// Trait used to create stateful components, see [`stateful`](crate::stateful::stateful) for details.
pub trait IntoState: Sized {
    type State: 'static;

    fn init(self) -> Self::State;

    fn update(self, state: &mut Self::State) -> Then;
}

impl<F, S> IntoState for F
where
    S: 'static,
    F: FnOnce() -> S,
{
    type State = S;

    fn init(self) -> Self::State {
        (self)()
    }

    fn update(self, _: &mut Self::State) -> Then {
        Then::Stop
    }
}

macro_rules! impl_into_state {
    ($($ty:ty),*) => {
        $(
            impl IntoState for $ty {
                type State = <Self as Diff>::Memo;

                fn init(self) -> Self::State {
                    self.into_memo()
                }

                fn update(self, state: &mut Self::State) -> Then {
                    match self.diff(state) {
                        false => Then::Stop,
                        true => Then::Render,
                    }
                }
            }
        )*
    };
}

impl_into_state!(
    &str, &String, bool, u8, u16, u32, u64, u128, usize, isize, i8, i16, i32, i64, i128, f32, f64
);
