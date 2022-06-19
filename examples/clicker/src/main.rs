use kobold::prelude::*;

fn main() {
    #[derive(PartialEq, Eq)]
    struct Clicker {
        name: &'static str,
        count: u32,
    }

    impl Default for Clicker {
        fn default() -> Self {
            Clicker {
                name: "Alice",
                count: 2,
            }
        }
    }

    impl Clicker {
        fn render(self) -> impl Html {
            self.stateful(|state, link| {
                let n = state.count;

                let inc = link.callback(|state, _| state.count += 1);
                let dec = link.callback(|state, _| state.count = state.count.saturating_sub(1));

                html! {
                    <div>
                        <h1 class="Greeter">"Hello "{ state.name }"!"</h1>
                        <p>
                            "This component dynamically creates a list from a range iterator ending at "
                            { state.count }
                            <button onclick={inc}>"+"</button>
                            <button onclick={dec}>"-"</button>
                        </p>
                        <ul>
                            // `.list()` wraps the iterator in a helper struct that implements `Html`
                            { (1..=n).map(|n| html! { <li>"Item #"{ n }</li> }).list() }
                        </ul>
                    </div>
                }
            })
        }
    }

    kobold::start(html! {
        // The `..` notation fills in the rest of the component with values from the `Default` impl.
        <Clicker name={"Bob"} ../>
    });
}
