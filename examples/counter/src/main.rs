use kobold::prelude::*;

#[component]
fn Counter() -> impl View {
    stateful(0_u32, |count| {
        bind! { count:
            let onclick = move |_| *count += 1;
            let reset = move |_| *count = 0;
        }

        view! {
            <p>
                <ShowCount count={count.get()} />

                // `{onclick}` here is shorthand for `onclick={onclick}`
                <button {onclick}>"Click me!"</button>
                <button onclick={reset}>"Reset"</button>
            </p>
        }
    })
}

#[component(auto_branch)]
fn ShowCount(count: u32) -> impl View {
    let count = match count {
        0 => view! { "zero times." },
        1 => view! { "once." },
        n => view! { { n }" times." },
    };

    view! { <h3>"You've clicked the button "{ count }</h3> }
}

fn main() {
    kobold::start(view! {
        <Counter />
    });
}
