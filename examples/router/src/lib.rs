use kobold::path::pop_state;
use kobold::prelude::*;

#[component]
fn app() -> impl View {
    // let data = state!("Enter something");

    pop_state(move |path| {
        view! {
            <h1>"Router example"</h1>
            <p>"Current path is: "{path.to_owned()}</p>
            <p><a link="/foo">"Foo"</a>" "<a link="/bar">"Bar"</a></p>
        }
    })
}

kobold::start!(app);
