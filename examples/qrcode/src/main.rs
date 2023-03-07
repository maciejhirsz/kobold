use kobold::prelude::*;
use kobold_qr::KoboldQR;
use web_sys::HtmlInputElement as InputElement;

#[component]
fn QRExample() -> impl Html {
    stateful("Enter something", |data| {
        let onkeypress = data.bind(move |data, event: &KeyboardEvent<InputElement>| {
            *data = event.target().value();
        });

        html! {
            <p>"QR code example"</p>
            <input value={data.as_str()} {onkeypress} />
            <KoboldQR data={data.as_str()} />
        }
    })
}

fn main() {
    kobold::start(html! {
        <QRExample />
    });
}
