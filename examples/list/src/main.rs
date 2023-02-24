use kobold::prelude::*;

#[component]
fn ListExample(count: u32) -> impl Html {
    stateful(count, |count| {
        let dec = count.bind(|count, _| *count = count.saturating_sub(1));
        let inc = count.bind(|count, _| *count += 1);

        html! {
            <div>
                <h1 class="Greeter">"List example"</h1>
                <p>
                    "This component dynamically creates a list from a range iterator ending at "
                    <button onclick={dec}>"-"</button>
                    " "{ count }" "
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
                    (1..=count.get())
                        .map(|n| html! { <ListItem {n} /> })
                        .list()
                }
                </ul>
            </div>
        }
    })
}

#[component]
fn ListItem(n: u32) -> impl Html {
    html! { <li>"Item #"{ n }</li> }
}

fn main() {
    kobold::start(html! {
        <ListExample count={2} />
    });
}
