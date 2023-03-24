use crate::dom::NoDiff;
use crate::value::FastDiff;

pub trait Diff {
    type State: 'static;
    type Updated: ?Sized;

    fn init(self) -> Self::State;

    fn update(self, state: &mut Self::State, on_change: impl FnOnce(&Self::Updated));
}

impl Diff for &str {
    type State = String;
    type Updated = str;

    fn init(self) -> String {
        self.into()
    }

    fn update(self, state: &mut String, on_change: impl FnOnce(&str)) {
        if self != state {
            self.clone_into(state);
            on_change(&self);
        }
    }
}

impl Diff for String {
    type State = String;
    type Updated = str;

    fn init(self) -> String {
        self
    }

    fn update(self, state: &mut String, on_change: impl FnOnce(&str)) {
        if &self != state {
            on_change(&self);
            *state = self;
        }
    }
}

impl Diff for &String {
    type State = String;
    type Updated = str;

    fn init(self) -> String {
        self.clone()
    }

    fn update(self, state: &mut String, on_change: impl FnOnce(&str)) {
        if self != state {
            self.clone_into(state);
            on_change(&self);
        }
    }
}

impl Diff for FastDiff<'_> {
    type State = usize;
    type Updated = str;

    fn init(self) -> usize {
        self.as_ptr() as _
    }

    fn update(self, state: &mut usize, on_change: impl FnOnce(&str)) {
        if self.as_ptr() as usize != *state {
            *state = self.as_ptr() as _;
            on_change(&self);
        }
    }
}

impl<T> Diff for NoDiff<T> {
    type State = ();
    type Updated = T;

    fn init(self) {}

    fn update(self, _: &mut (), on_change: impl FnOnce(&T)) {
        on_change(&self)
    }
}

macro_rules! impl_diff {
    ($($ty:ty),*) => {
        $(
            impl Diff for $ty {
                type State = $ty;
                type Updated = $ty;

                fn init(self) -> $ty {
                    self
                }

                fn update(self, state: &mut $ty, on_change: impl FnOnce(&Self)) {
                    if self != *state {
                        on_change(&self);
                        *state = self;
                    }
                }
            }

            impl Diff for &$ty {
                type State = $ty;
                type Updated = $ty;

                fn init(self) -> $ty {
                    *self
                }

                fn update(self, state: &mut $ty, on_change: impl FnOnce(&$ty)) {
                    (*self).update(state, on_change)
                }
            }
        )*
    };
}

impl_diff!(bool, u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64);
