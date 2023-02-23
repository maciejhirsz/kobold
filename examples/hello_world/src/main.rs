use kobold::prelude::*;

#[component]
fn Hello(name: &'static str) -> impl Html {
    html! {
        <h1>"Hello "{ name }"!"</h1>
    }
}

fn main() {
    kobold::start(html! {
        <Hello name="Kobold" />
    });
}
