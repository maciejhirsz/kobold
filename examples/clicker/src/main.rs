use kobold::prelude::*;

fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    #[derive(Debug, PartialEq, Eq)]
    struct Greeter {
        name: &'static str,
        count: u32,
    }

    impl Default for Greeter {
        fn default() -> Self {
            Greeter {
                name: "Alice",
                count: 2,
            }
        }
    }

    impl Greeter {
        fn render(self) -> impl Html {
            self.stateful(|state, link| {
                let n = state.count;

                let inc = link.callback(|state, _| state.count += 1);
                let dec = link.callback(|state, _| state.count = state.count.saturating_sub(1));

                html! {
                    <div>
                        <h1 class="Greeter">"Hello "{ state.name }"!"</h1>
                        <p>
                            <button onclick={inc}>"+"</button>
                            { state.count }
                            <button onclick={dec}>"-"</button>
                        </p>
                        <p>
                            <strong>{ n }" + 2 = "{ n + 2 }</strong>
                        </p>
                        { (0..n).map(|n| html! { <p>"Item #"{ n }</p> }).list() }
                    </div>
                }
            })

        }
    }

    kobold::start(html! {
        <Greeter name={"Bob"} ../>
    });
}
