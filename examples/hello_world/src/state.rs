// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use gloo_storage::{LocalStorage, Storage};
use wasm_bindgen::throw_str;
use log::{debug};

const KEY: &str = "kobold.hello_world.example";

#[derive(Clone, Copy, Debug)]
pub struct State {
    pub my_state: bool,
}

impl Default for State {
    fn default() -> Self {
        let mut my_state = false;
        if let Some(storage) = LocalStorage::raw().get(KEY).ok() {
            my_state = storage.unwrap().parse::<bool>().is_err();
        }

        State {
            my_state
        }
    }
}

impl State {
    pub fn mock() -> Self {
        State {
            my_state: false
        }
    }

    pub fn get() -> Self {
        let mut current_state = false;
        if let Some(storage) = LocalStorage::raw().get(KEY).ok() {
            if !storage.is_none() {
                current_state = storage.unwrap().parse::<bool>().is_err();
            }  else {
                debug!("my_state is None in storage");
            }
        } else {
            debug!("my_state is None in storage");
        }

        State {
            my_state: current_state
        }
    }

    #[inline(never)]
    pub fn store(&mut self) {
        let mut storage = false;

        storage = self.my_state;

        LocalStorage::raw().set_item(KEY, &storage.to_string()).ok();
    }

    pub fn toggle(&mut self) {
        self.my_state = !self.my_state;

        // self.store();
        State::store(self);
    }
}
