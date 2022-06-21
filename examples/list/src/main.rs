use kobold::prelude::*;

fn main() {
    #[derive(PartialEq, Eq)]
    struct ListExample {
        count: u32,
    }

    impl ListExample {
        fn render(self) -> impl Html {
            self.stateful(|state, link| {
                let n = state.count;

                let dec = link.callback(|state, _| state.count = state.count.saturating_sub(1));
                let inc = link.callback(|state, _| state.count += 1);

                html! {
                    <div>
                        <h1 class="Greeter">"List example"</h1>
                        <p>
                            "This component dynamically creates a list from a range iterator ending at "
                            <button onclick={dec}>"-"</button>
                            " "{ state.count }" "
                            <button onclick={inc}>"+"</button>
                        </p>
                        <ul>
                            // Just an iterator, you don't need to collect it to a `Vec`.
                            //
                            // On subsequent renders `Kobold` can very cheaply diff items yielded
                            // by iterators, avoiding allocations unless new items are added.
                            { (1..=n).map(|n| html! { <li>"Item #"{ n }</li> }) }
                        </ul>
                    </div>
                }
            })
        }
    }

    kobold::start(html! {
        <ListExample count={2} />
    });
}
