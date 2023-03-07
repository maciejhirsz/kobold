use kobold::prelude::*;
use kobold_qr::KoboldQR;

#[component]
fn QRExample() -> impl Html {
    stateful("Enter something", |data| {
        html! {
            <h1>"QR code example"</h1>
            <p>
                <input
                    value={data.as_str()}
                    onkeyup={data.bind(move |data, event| {
                        *data = event.target().value();
                    })}
                />
            </p>
            <KoboldQR data={data.as_str()} />
        }
    })
}

fn main() {
    kobold::start(html! {
        <QRExample />
    });
}
