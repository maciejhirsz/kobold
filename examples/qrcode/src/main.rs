use kobold::prelude::*;
use kobold::reexport::web_sys::HtmlTextAreaElement;
use kobold_qr::KoboldQR;

#[component]
fn QRExample() -> impl View {
    stateful("Enter something", |data| {
        bind! {
            data:

            let onkeyup = move |event: KeyboardEvent<HtmlTextAreaElement>| *data = event.target().value();
        }

        view! {
            <h1>"QR code example"</h1>
            <KoboldQR {data} />
            <textarea {onkeyup}>{ static data.as_str() }</textarea>
        }
    })
}

fn main() {
    kobold::start(view! {
        <QRExample />
    });
}
