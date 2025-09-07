// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
use std::fmt::Write;

use fast_qr::Module;
use fast_qr::qr::QRBuilder;

use kobold::prelude::*;
use kobold::diff::fence;

/// Error Correction Coding has 4 levels
pub enum Ecl {
    /// Low, 7%
    L,
    /// Medium, 15%
    M,
    /// Quartile, 25%
    Q,
    /// High, 30%
    H,
}

impl From<Ecl> for fast_qr::ECL {
    fn from(value: Ecl) -> Self {
        match value {
            Ecl::L => fast_qr::ECL::L,
            Ecl::M => fast_qr::ECL::M,
            Ecl::Q => fast_qr::ECL::Q,
            Ecl::H => fast_qr::ECL::H,
        }
    }
}

impl Default for Ecl {
    fn default() -> Self {
        Ecl::Q
    }
}

// QR code sizes are calculated as 17 + N * 4, where N is in range of (1, 40),
// 33 is therefore a valid size, but 32 is not and we can use it to mark error.
const ERROR_SIZE: usize = 32;
const ERROR_PATH: &str = "M4 0L0 4L12 16L0 28L4 32L16 20L28 32L32 28L20 16L32 4L28 0L16 12L4 0z";

#[component(
    size?: 200,
    ecl?,
)]
pub fn qr(data: &str, size: usize, ecl: Ecl) -> impl View {
    fence(data, move || {
        let (data, qrsize) = match QRBuilder::new(data).ecl(ecl.into()).build() {
            Ok(qr) => (qr.data, qr.size),
            Err(_) => ([Module(0); _], ERROR_SIZE),
        };

        let view_box = format!("0 0 {} {}", qrsize, qrsize);
        let style = format!("width: {size}px; height: {size}px;");
        let d = draw_path(data, qrsize);

        view! {
            <svg {view_box} {style}>
                <path {d} fill="currentColor">
            </svg>
        }
    })
}

fn draw_path<const N: usize>(data: [Module; N], size: usize) -> String {
    // Most QR codes will generate a path with density of ~2.5 bytes per module,
    // allocating 4 bytes per module should be more than sufficient.
    let mut buf = String::with_capacity(size * size * 4);

    if size == ERROR_SIZE {
        buf.push_str(ERROR_PATH);
        return buf;
    }

    for (y, row) in data.chunks_exact(size).take(size).enumerate() {
        let row = &mut row.iter();

        // Find first filled module
        let Some(x) = row.position(|m| m.value()) else {
            continue;
        };

        // Move to the new line
        let _ = write!(&mut buf, "M{x} {y}");

        loop {
            // Find all horizontally adjacent filled modules
            let width = row.take_while(|m| m.value()).count() + 1;

            // `v1`       : draw line 1 segment down
            // `h{width}` : draw line {width} segments right
            // `v-1`      : draw line 1 segment up
            let _ = write!(&mut buf, "v1h{width}v-1");

            // Skip empty modules
            let Some(empty) = row.position(|m| m.value()) else {
                break;
            };

            // Note: we are drawing a line here (`h{}`) instead of moving (`m{} 0`) to
            //       save some bytes, this is fine as long as we don't render line stroke.
            let _ = write!(&mut buf, "h{}", empty + 1);
        }
    }

    buf
}
