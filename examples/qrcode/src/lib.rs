use kobold::prelude::*;
use kobold::reexport::web_sys::HtmlTextAreaElement;
use kobold_qr::qr;

use kobold::path::popstate;

#[component]
fn app() -> impl View {
    let data = state!("Enter something");

    popstate(move |path| {
        let onkeyup = event!(|data, e: &KeyboardEvent<HtmlTextAreaElement>| {
            *data = e.current_target().value();
        });

        view! {
            <h1>"QR code example"</h1>
            <p>"Current path is: "{path.to_owned()}</p>
            <!qr {data}>
            <textarea {onkeyup}>{ static data.as_str() }</textarea>
        }
    })
}

kobold::start!(app);
