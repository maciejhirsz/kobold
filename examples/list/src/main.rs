use kobold::prelude::*;

#[component]
fn list_example(count: u32) -> impl View {
    stateful(count, |count| {
        let dec = event!(*count = count.saturating_sub(1));
        let inc = event!(*count += 1);

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
                    for (1..=count.get()).map(list_item)
                }
        }
    })
}

#[component]
fn list_item(n: u32) -> impl View {
    view! { <li>"Item #"{ n }</li> }
}

fn main() {
    kobold::start(view! {
        <!list_example count={2}>
    });
}
