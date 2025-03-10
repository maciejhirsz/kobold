use kobold::prelude::*;

fn app(count: &Hook<u32>) -> impl View + '_ {
    view! {
        <p>
            <!counter count={count.get()}>

            <button onclick={do *count += 1}>"Click me!"</button>
            <button onclick={do *count = 0}>"Reset"</button>
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
