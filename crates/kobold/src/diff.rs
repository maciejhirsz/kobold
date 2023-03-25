use crate::dom::NoDiff;
use crate::value::FastDiff;

pub trait Diff: Copy {
    type State: 'static;

    fn init(self) -> Self::State;

    fn update(self, state: &mut Self::State) -> bool;
}

impl Diff for &str {
    type State = String;

    fn init(self) -> String {
        self.into()
    }

    fn update(self, state: &mut String) -> bool {
        if self != state {
            self.clone_into(state);
            true
        } else {
            false
        }
    }
}

impl Diff for &String {
    type State = String;

    fn init(self) -> String {
        self.clone()
    }

    fn update(self, state: &mut String) -> bool {
        if self != state {
            self.clone_into(state);
            true
        } else {
            false
        }
    }
}

impl Diff for FastDiff<'_> {
    type State = usize;

    fn init(self) -> usize {
        self.as_ptr() as _
    }

    fn update(self, state: &mut usize) -> bool {
        if self.as_ptr() as usize != *state {
            *state = self.as_ptr() as _;
            true
        } else {
            false
        }
    }
}

impl<T> Diff for NoDiff<T>
where
    T: Copy,
{
    type State = ();

    fn init(self) {}

    fn update(self, _: &mut ()) -> bool {
        false
    }
}

macro_rules! impl_diff {
    ($($ty:ty),*) => {
        $(
            impl Diff for $ty {
                type State = $ty;

                fn init(self) -> $ty {
                    self
                }

                fn update(self, state: &mut $ty) -> bool {
                    if self != *state {
                        *state = self;
                        true
                    } else {
                        false
                    }
                }
            }

            impl Diff for &$ty {
                type State = $ty;

                fn init(self) -> $ty {
                    *self
                }

                fn update(self, state: &mut $ty) -> bool {
                    (*self).update(state)
                }
            }
        )*
    };
}

impl_diff!(bool, u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64);
