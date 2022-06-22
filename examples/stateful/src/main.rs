use kobold::prelude::*;

// This is our component struct, note that it can take arbitrary lifetimes.
struct Borrowing<'a> {
    name: &'a str,
}

// This is our owned state, it must live for a `'static` lifetime, and may
// contain different fields than those on the component.
struct OwnedState {
    name: String,
}

impl Stateful for Borrowing<'_> {
    // We define that `OwnedState` is the state for this component
    type State = OwnedState;

    // Create `OwnedState` from this component
    fn init(self) -> OwnedState {
        OwnedState {
            name: self.name.into(),
        }
    }

    // Update the pre-existing state
    fn update(self, state: &mut Self::State) -> ShouldRender {
        if self.name != state.name {
            // `state.name = self.name.into()` would have been fine too,
            // but this saves an allocation if the original `String` has
            // enough capacity
            state.name.replace_range(.., self.name);

            ShouldRender::Yes
        } else {
            // If the name hasn't change there is no need to do anything
            ShouldRender::No
        }
    }
}

impl<'a> Borrowing<'a> {
    fn render(self) -> impl Html + 'a {
        // Types here are:
        // state: &OwnedState,
        // link: Link<OwnedState>,
        self.stateful(|state, link| {
            // Since we work with a state that owns a `String`,
            // callbacks can mutate it at will.
            let exclaim = link.callback(|state, _| state.name.push('!'));

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
            let alice = link.callback(|state, _| {
                if state.name != "Alice" {
                    state.name.replace_range(.., "Alice");

                    ShouldRender::Yes
                } else {
                    ShouldRender::No
                }
            });

            html! {
                <div>
                    // Render can borrow `name` from state, no need for clones
                    <h1>"Hello: "{ &state.name }</h1>
                    <button onclick={alice}>"Alice"</button>
                    <button onclick={exclaim}>"!"</button>
                </div>
            }
        })
    }
}

fn main() {
    kobold::start(html! {
        // Constructing the component only requires a `&str` slice.
        <Borrowing name="Bob" />
    });
}
