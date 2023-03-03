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
        // Since we work with a state that owns a `String`,
        // callbacks can mutate it at will.
        let exclaim = state.bind(|state, _| state.name.push('!'));

        // Repeatedly clicking the Alice button does not have to do anything.
        //
        // NOTE: This is quite an overkill for this example, as updates on
        // this render function only do two things:
        //
        //    1. Compare the `&state.name` with previous render.
        //    2. Update closures, which is nearly free as these are
        //       zero-sized (they don't capture anything).
        //
        // For any more robust states and renders logic `ShouldRender::No`
        // when no changes in DOM are necessary is always a good idea.
        let alice = state.bind(|state, _| {
            if state.name != "Alice" {
                "Alice".clone_into(&mut state.name);
                ShouldRender::Yes
            } else {
                ShouldRender::No
            }
        });

        let inc_age = state.bind(|state, _| state.age += 1);
        let adult = state.bind(|state, _| state.age = 18);

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
