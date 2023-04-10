use kobold::prelude::*;

#[component]
fn ListExample(count: u32) -> impl View {
    stateful(count, |count| {
        bind! { count:
            let dec = move |_| *count = count.saturating_sub(1);
            let inc = move |_| *count += 1;
        }

        view! {
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
                    // Use the `for` keyword to turn an iterator into a `View`.
                    //
                    // On subsequent renders `Kobold` can very cheaply diff items yielded
                    // by iterators, avoiding allocations unless new items are added.
                    //
                    // `{n}` is just shorthand for `n={n}`.
                    for (1..=count.get()).map(|n| view! { <ListItem {n} /> })
                }
        }
    })
}

#[component]
fn ListItem(n: u32) -> impl View {
    view! { <li>"Item #"{ n }</li> }
}

fn main() {
    kobold::start(view! {
        <ListExample count={2} />
    });
}
