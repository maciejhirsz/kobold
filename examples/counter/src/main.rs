use kobold::prelude::*;
use kobold::branching::Branch3;

#[component]
fn Counter() -> impl Html {
    stateful(0_u32, |count| {
        let onclick = count.bind(|count, _event| *count += 1);

        html! {
            <p>
                // `{onclick}` here is shorthand for `onclick={onclick}`
                <button {onclick}>"Click me!"</button>
                <p>
                    <ShowCount count={**count} />
                </p>
                <button onclick={count.bind(|count, _| *count = 0)}>"Reset"</button>
            </p>
        }
    })
}

#[component(branching)]
fn ShowCount(count: u32) -> impl Html {
    if count == 0 {
        html! { "The counter is empty." }
    } else if count == 1 {
        html! { "You've clicked the button once." }
    } else {
        html! { "You've clicked the button "{ count }" times." }
    }
}

// #[component(branching)]
// fn ShowCountMatch(count: u32) -> impl Html {
//     match count {
//         0 => html! { "The counter is empty." },
//         1 => html! { "You've clicked the button once." },
//         _ => html! { "You've clicked the button "{ count }" times." },
//     }
// }

// #[component(branching)]
// fn ShowCountReturns(count: u32) -> impl Html {
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
