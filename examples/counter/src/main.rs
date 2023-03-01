use kobold::prelude::*;

#[component]
fn Counter() -> impl Html {
    stateful(0_u32, |count| {
        let onclick = count.bind(|count, _event| *count += 1);

        html! {
            <p>
                <ShowCount count={count.get()} />

                // `{onclick}` here is shorthand for `onclick={onclick}`
                <button {onclick}>"Click me!"</button>
                <button onclick={count.bind(|count, _| *count = 0)}>"Reset"</button>
            </p>
        }
    })
}

// #[component(branching)]
// fn ShowCount(count: u32) -> impl Html {
//     let count = if count == 0 {
//         html! { "zero times." }
//     } else if count == 1 {
//         html! { "once." }
//     } else {
//         html! { { count }" times." }
//     };

//     html! { <h3>"You've clicked the button "{ count }</h3> }
// }

#[component(branching)]
fn ShowCount(count: u32) -> impl Html {
    let count = match count {
        0 => html! { "zero times." },
        1 => html! { "once." },
        n => html! { { n }" times." },
    };

    html! { <h3>"You've clicked the button "{ count }</h3> }
}

// #[component(branching)]
// fn ShowCount(count: u32) -> impl Html {
//     if count == 0 {
//         return html! { "The counter is empty." };
//     }

//     if count == 1 {
//         return html! { "You've clicked the button once." };
//     }

//     html! { "You've clicked the button "{ count }" times." }
// }

fn main() {
    kobold::start(html! {
        <Counter />
    });
}
