use kobold::prelude::*;

fn main() {
    #[derive(PartialEq, Eq, Default)]
    struct Counter {
        count: u32,
    }

    impl Counter {
        fn render(self) -> impl Html {
            self.stateful(|state, link| {
                let onclick = link.callback(|state, _| state.count += 1);

                html! {
                    <p>
                        "You clicked the "
                        <button {onclick}>
                            "Button"
                        </button>
                        " "
                        { state.count }
                        " times."
                    </p>
                }
            })
        }
    }

    kobold::start(html! {
        // The `..` notation fills in the rest of the component with values from the `Default` impl.
        <Counter ../>
    });
}
