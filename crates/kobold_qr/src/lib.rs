// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
use std::fmt::Write;

use kobold::prelude::*;

use fast_qr::qr::QRBuilder;
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

#[component(
    size?: 200,
    ecl?,
)]
pub fn qr(data: &str, size: usize, ecl: Ecl) -> impl View {
    fence(data, move || {
        let qr = QRBuilder::new(data).ecl(ecl.into()).build().ok()?;

        let viewbox = format!("0 0 {} {}", qr.size, qr.size);
        let style = format!("width: {size}px; height: {size}px;");

        // Most QR codes will generate a path with density of ~2.5 bytes per module,
        // allocating 4 bytes per module should be more than sufficient.
        let mut path = String::with_capacity(qr.data.len() * 4);

        for (y, row) in qr.data.chunks_exact(qr.size).enumerate() {
            let row = &mut row.iter();

            // Find first filled module
            let Some(x) = row.position(|m| m.value()) else {
                continue;
            };

            // Move to the new line
            let _ = write!(&mut path, "M{x} {y}");

            loop {
                // Draw a recangle for all continous modules
                let width = row.take_while(|m| m.value()).count() + 1;

                let _ = write!(&mut path, "v1h{width}v-1");

                // Skip empty modules
                let Some(empty) = row.position(|m| m.value()) else {
                    break;
                };

                // Note: we are drawing a line here (`hN`) instead of moving (`mN 0`) to
                //       save some bytes, this is fine as long as we don't render line stroke.
                let _ = write!(&mut path, "h{}", empty + 1);
            }
        }

        Some(
            view! {
                <svg viewBox={viewbox} {style}>
                    <path d={path} fill="currentColor">
                </svg>
            }
        )
    })
}
