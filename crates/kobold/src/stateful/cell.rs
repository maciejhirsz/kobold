// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::cell::{Cell, UnsafeCell};

pub struct WithCell<T> {
    borrowed: Cell<bool>,
    data: UnsafeCell<T>,
}

impl<T> WithCell<T> {
    pub const fn new(data: T) -> Self {
        WithCell {
            borrowed: Cell::new(false),
            data: UnsafeCell::new(data),
        }
    }

    pub fn with<F>(&self, mutator: F)
    where
        F: FnOnce(&mut T),
    {
        if self.borrowed.get() {
            return;
        }

        self.borrowed.set(true);
        mutator(unsafe { &mut *self.data.get() });
        self.borrowed.set(false);
    }

    pub unsafe fn borrow_unchecked(&self) -> &T {
        &*self.data.get()
    }
}
