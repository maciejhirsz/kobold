// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::cell::UnsafeCell;
use std::fmt::{self, Display};

pub struct Join<'a, I> {
    iter: UnsafeCell<I>,
    sep: &'a str,
}

impl<I> Display for Join<'_, I>
where
    I: Iterator,
    I::Item: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let iter = unsafe { &mut *self.iter.get() };

        if let Some(first) = iter.next() {
            first.fmt(f)?;

            for item in iter {
                f.write_str(self.sep)?;
                item.fmt(f)?;
            }
        }

        Ok(())
    }
}

pub trait IteratorExt: Iterator + Sized {
    fn join(self, sep: &str) -> Join<Self> {
        Join {
            iter: UnsafeCell::new(self),
            sep,
        }
    }
}

impl<I: Iterator> IteratorExt for I {}
