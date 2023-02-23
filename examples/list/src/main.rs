use kobold::prelude::*;

#[derive(Stateful, PartialEq, Eq)]
struct ListExample {
    count: u32,
}

impl ListExample {
    fn render(self) -> impl Html {
        self.stateful(|state, ctx| {
            let n = state.count;

            let dec = ctx.bind(|state, _| state.count = state.count.saturating_sub(1));
            let inc = ctx.bind(|state, _| state.count += 1);

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
                    {
                        // Use the `list` method on an iterator to turn it into an `Html` type.
                        //
                        // On subsequent renders `Kobold` can very cheaply diff items yielded
                        // by iterators, avoiding allocations unless new items are added.
                        //
                        // `{n}` is just shorthand for `n={n}`.
                        (1..=n)
                            .map(|n| html! { <ListItem {n} /> })
                            .list()
                    }
                    </ul>
                </div>
            }
        })
    }
}

#[kobold::component]
fn ListItem(n: u32) -> impl Html {
    html! { <li>"Item #"{ n }</li> }
}

fn main() {
    kobold::start(html! {
        <ListExample count={2} />
    });
}
