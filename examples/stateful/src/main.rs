use kobold::prelude::*;

struct State {
    name: String,
    age: u32,
}

impl State {
    fn new() -> Self {
        State {
            name: "Bob".to_owned(),
            age: 42,
        }
    }
}

#[component]
fn App() -> impl Html {
    stateful(State::new, |state| {
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
            let adult = move |_| state.age = 0;
        }

        html! {
            <div>
                // Render can borrow `name` from state, no need for clones
                <h1>{ &state.name }" is "{ state.age }" years old."</h1>
                <button onclick={alice}>"Alice"</button>
                <button onclick={exclaim}>"!"</button>
                " "
                <button onclick={adult}>"18"</button>
                <button onclick={inc_age}>"+"</button>
            </div>
        }
    })
}

fn main() {
    kobold::start(html! {
        <App />
    });
}
