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
    // Repeatedly clicking the Alice button does not have to do anything.
    let alice = event!(|state| {
        if state.name != "Alice" {
            "Alice".clone_into(&mut state.name);
            Then::Render
        } else {
            Then::Stop
        }
    });

    view! {
        <div>
            // Render can borrow `name` from state, no need for clones
            <h1>{ &state.name }" is "{ state.age }" years old."</h1>
            <button onclick={alice}>"Alice"</button>
            <button onclick={do state.name.push('!')}>"!"</button>
            " "
            <button onclick={do state.age = 18}>"18"</button>
            <button onclick={do state.age += 1}>"+"</button>
    }
}

fn main() {
    kobold::start(stateful(State::new, app));
}
