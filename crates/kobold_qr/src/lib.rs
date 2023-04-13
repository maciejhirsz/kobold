// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use kobold::prelude::*;
use wasm_bindgen::prelude::*;

use fast_qr::qr::QRBuilder;
use kobold::diff::fence;
use web_sys::CanvasRenderingContext2d;

#[component(size?: 200)]
pub fn KoboldQR(data: &str, size: u32) -> impl View + '_ {
    fence(data, move || {
        let qr = QRBuilder::new(data).build().ok()?;
        let qrsize = qr.size * 8;
        let style = format!("width: {size}px; height: {size}px;");

        Some(
            view! {
                <canvas width={qrsize} height={qrsize} {style} />
            }
            .on_render(move |canvas| {
                let ctx = match canvas.get_context("2d") {
                    Ok(Some(ctx)) => ctx.unchecked_into::<CanvasRenderingContext2d>(),
                    _ => return,
                };

                ctx.clear_rect(0., 0., qrsize as f64, qrsize as f64);

                for (y, row) in qr.data.chunks(qr.size).take(qr.size).enumerate() {
                    let mut row = row.iter().enumerate();

                    while let Some((x, m)) = row.next() {
                        if !m.value() {
                            continue;
                        }

                        let w = 1 + (&mut row).take_while(|(_, m)| m.value()).count();

                        ctx.fill_rect((8 * x) as f64, (8 * y) as f64, (w * 8) as f64, 8.)
                    }
                }
            }),
        )
    })
}
