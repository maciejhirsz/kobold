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
            wasm_bindgen::throw_str("Cyclic state borrowing");
        }

        self.borrowed.set(true);
        mutator(unsafe { &mut *self.data.get() });
        self.borrowed.set(false);
    }

    pub unsafe fn ref_unchecked(&self) -> &T {
        debug_assert!(!self.borrowed.get());

        &*self.data.get()
    }

    #[allow(clippy::mut_from_ref)]
    pub unsafe fn mut_unchecked(&self) -> &mut T {
        debug_assert!(!self.borrowed.get());

        &mut *self.data.get()
    }
}
