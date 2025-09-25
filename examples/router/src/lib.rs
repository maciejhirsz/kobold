// use kobold::path::pop_state;
use kobold::prelude::*;

#[component]
fn app() -> impl View {
    let state: &Hook<Vec<String>> = state!(Vec::new);

    let body = router! {
        "/inventory" => {}
        "/item/{id}" where id: usize => {
            state.get(id).map()

            view! {
                <h1>"Item "{id}</h1>

            },
        }
        _ => view! {
            <h1>"Router Example"</h1>
            <p>"Click a link in the menu to navigate"</p>
        },
    };

    view! {
        <h1>"Router example"</h1>
        <ul.menu>
            <li><a link="/">"Home"</a></li>
            <li><a link="/inventory">"Inventory"</a></li>
        </ul>
        <div.body>{body}</div>
    }
    // pop_state(move |path| {
    //     view! {
    //         <h1>"Router example"</h1>
    //         <p>"Current path is: "{path.to_owned()}</p>
    //         <p><a link="/foo">"Foo"</a>" "<a link="/bar">"Bar"</a></p>
    //     }
    // })
}

kobold::start!(app);
