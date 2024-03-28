use kobold::prelude::*;
use kobold::reexport::web_sys::HtmlTextAreaElement;
use kobold_qr::qr;

#[component]
fn qr_example() -> impl View {
    stateful("Enter something", |data| {
        bind! {
            data:

            let onkeyup = move |event: KeyboardEvent<HtmlTextAreaElement>| *data = event.target().value();
        }

        view! {
            <h1>"QR code example"</h1>
            <!qr {data}>
            <textarea {onkeyup}>{ static data.as_str() }</textarea>
        }
    })
}

fn main() {
    kobold::start(qr_example());
}
