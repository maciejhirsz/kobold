use kobold::prelude::*;
use log::{debug};

mod state;

use state::{State};

#[component]
fn Hello(state: bool) -> impl View + 'static {
    stateful(State::mock, |state| {
        let signal: Signal<State> = state.signal();

        let s = state.get();
        debug!("my_state {:#?}", s);

        signal.update(|state| state.toggle());
        signal.set(State { my_state: true });
        debug!("my_state new {:#?}", s);
        let s_new = state.get();
        debug!("my_state new {:#?}", s_new);
        let s_more = State::get();
        debug!("my_state more {:#?}", s_more);

        view! {
            <h1>"Hello "{ use s.my_state }"!"</h1>
        }
    })
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    kobold::start(view! {
        <Hello state=true />
    });
}
