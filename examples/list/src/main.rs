use kobold::prelude::*;

struct Foo {

}

impl Foo {
    fn test(this: &Hook<Self>, idx: usize) {

    }
}

#[component]
fn ListExample(count: u32) -> impl View {
    stateful(count, |count| {
        // #[event]
        // fn dec(count: &mut u32) {
        //     *count = count.saturating_sub(1);
        // }

        // #[event]
        // fn inc(count: &mut u32) {
        //     *count += 1;
        // }

        bind! { count:
            // let dec = move |_| *count = count.saturating_sub(1);
            let inc = move |_| *count += 1;
        }

        let dec = count.bind(|c| *c = c.saturating_sub(1));
        let inc = count.bind(|c| *c += 1);

        // let dec = |count: &mut u32| *count = count.saturating_sub(1);
        // let inc = |count: &mut u32| *count += 1;

        // foo(dec);

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
