use kobold::diff::VString;
use kobold::prelude::*;

struct State {
    name: VString,
    age: u32,
}

impl State {
    fn new() -> Self {
        State {
            name: "Bob".into(),
            age: 42,
        }
    }
}

fn app(state: &Hook<State>) -> impl View + '_ {
    bind! { state:
        // Since we work with a state that owns a `String`,
        // callbacks can mutate it at will.
        let exclaim = move |_| state.name.push('!');

        // Repeatedly clicking the Alice button does not have to do anything.
        let alice = move |_| if state.name != "Alice" {
            "Alice".clone_into(&mut state.name);
            Then::Render
        } else {
            Then::Stop
        };

        let inc_age = move |_| state.age += 1;
        let adult = move |_| state.age = 18;
    }

    view! {
        <div>
            // Render can borrow `name` from state, no need for clones
            <h1>{ &state.name }" is "{ state.age }" years old."</h1>
            <button onclick={alice}>"Alice"</button>
            <button onclick={exclaim}>"!"</button>
            " "
            <button onclick={adult}>"18"</button>
            <button onclick={inc_age}>"+"</button>
    }
}

fn main() {
    kobold::start(stateful(State::new, app));
}
