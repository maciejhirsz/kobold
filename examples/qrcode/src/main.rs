use kobold::prelude::*;
use kobold_qr::KoboldQR;

#[component]
fn QRExample() -> impl Html {
    stateful("Enter something", |data| {
        html! {
            <h1>"QR code example"</h1>
            <KoboldQR data={data.as_str()} />
            <textarea
                onkeyup={data.bind(move |data, event| {
                    *data = event.target().value();
                })}
            >
            { data.as_str().no_diff() }
            </textarea>
        }
    })
}

fn main() {
    kobold::start(html! {
        <QRExample />
    });
}
