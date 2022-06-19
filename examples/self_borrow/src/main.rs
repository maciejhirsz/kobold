use kobold::prelude::*;

fn main() {
    struct SelfBorrow<'a> {
        name: &'a str,
    }

    struct State {
        name: String,
    }

    impl Stateful for SelfBorrow<'_> {
        type State = State;

        fn init(self) -> State {
            State {
                name: self.name.into(),
            }
        }
    }

    impl<'a> SelfBorrow<'a> {
        fn render(self) -> impl Html + 'a {
            self.stateful(|state, link| {
                let exclaim = link.callback(|state, _| state.name.push('!'));
                let alice = link.callback(|state, _| state.name.replace_range(.., "Alice"));

                html! {
                    <div>
                        <p>
                            "Should be able to borrow a reference to an owned String: "{ &state.name }
                        </p>
                        <button onclick={alice}>"Alice"</button> <button onclick={exclaim}>"!"</button>
                    </div>
                }
            })
        }
    }

    kobold::start(html! { <SelfBorrow name={"Bob"} /> });
}
