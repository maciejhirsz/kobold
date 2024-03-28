use kobold::prelude::*;

fn app(count: &Hook<u32>) -> impl View + '_ {
    bind! { count:
        let onclick = move |_| *count += 1;
        let reset = move |_| *count = 0;
    }

    view! {
        <p>
            <!counter count={count.get()}>

            // `{onclick}` here is shorthand for `onclick={onclick}`
            <button {onclick}>"Click me!"</button>
            <button onclick={reset}>"Reset"</button>
    }
}

#[component(auto_branch)]
fn counter(count: u32) -> impl View {
    let count = match count {
        0 => view! { "zero times." },
        1 => view! { "once." },
        n => view! { { n }" times." },
    };

    view! { <h3> "You've clicked the button "{ count } }
}

fn main() {
    kobold::start(stateful(0_u32, app));
}
