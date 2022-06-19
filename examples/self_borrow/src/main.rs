use kobold::prelude::*;

fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    struct Greeter<'a> {
        name: &'a str,
    }

    struct GreeterState {
        name: String,
    }

    impl Stateful for Greeter<'_> {
        type State = GreeterState;

        fn init(self) -> GreeterState {
            GreeterState {
                name: self.name.into()
            }
        }
    }

    impl<'a> Greeter<'a> {
        fn render(self) -> impl Html + 'a {
            self.stateful(|state, link| {
                let exclaim = link.callback(|state, _| state.name.push('!'));
                let alice = link.callback(|state, _| {
                    state.name.clear();
                    state.name.push_str("Alice");
                });

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

    kobold::start(Greeter { name: "Bob" }.render());
}
