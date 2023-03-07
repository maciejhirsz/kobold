use std::ops::Deref;

use fast_qr::qr::{QRBuilder, QRCode};
use kobold::prelude::*;

struct State {
    input: Vec<u8>,
    qr: Option<QRCode>,
}

struct Data<D>(D);

impl<D> Deref for Data<D>
where
    D: AsRef<[u8]>,
{
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl<D> IntoState for Data<D>
where
    D: AsRef<[u8]>,
{
    type State = State;

    fn init(self) -> State {
        // let qr = QRBuilder::new(&*self).build().ok();
        // let vb = qr.as_ref().map(|qr| format!("0 0 {0} {0}", qr.size)).unwrap_or_default();

        State {
            input: self.to_vec(),
            qr: QRBuilder::new(&*self).build().ok(),
        }
    }

    fn update(self, state: &mut State) -> ShouldRender {
        if state.input == self.as_ref() {
            return ShouldRender::No;
        }

        state.input = self.to_vec();
        state.qr = QRBuilder::new(&*self).build().ok();

        ShouldRender::Yes
    }
}

#[component]
pub fn KoboldQR(data: &str) -> impl Html + '_ {
    stateful(Data(data), |state| {
        let qr = state.qr.as_ref()?;
        let vb = format!("0 0 {0} {0}", qr.size);

        let list = qr.data[..qr.size * qr.size]
            .iter()
            .enumerate()
            .map(|(i, m)| {
                m.value().then(|| {
                    let y = i / qr.size;
                    let x = i % qr.size;

                    html! { <rect x={x} y={y} width="1" height="1" /> }
                })
            })
            .list();

        Some(html! {
            <svg viewBox={vb} style="width: 200px; display: block;" xmlns="http://www.w3.org/2000/svg">{ list }</svg>
        })
    })
}

// fn main() {
//     // QRBuilder::new can fail if content is too big for version,
//     // please check before unwrapping.
//     let qrcode = QRBuilder::new("https://example.com/").build().unwrap();

//     let str = qrcode.to_str(); // .print() exists
//     println!("{}", str);
// }
