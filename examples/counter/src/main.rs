use kobold::prelude::*;

#[component]
fn Counter() -> impl Html {
    stateful(0_u32, |count| {
        let onclick = count.bind(|count, _event| *count += 1);

        html! {
            <p>
                "You clicked on the "
                // `{onclick}` here is shorthand for `onclick={onclick}`
                <button {onclick}>"Button"</button>
                " "{ count }" times."
            </p>
        }
    })
}

fn main() {
    kobold::start(html! {
        <Counter />
    });
}
