// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use log::debug;

use kobold::prelude::*;
use kobold_qr::KoboldQR;

use crate::components::util::sword::{sword};

#[component]
pub fn QRForTask(value: &str) -> impl View + '_ {
    let (left, right): (&str, &str) = sword(value);
    // assert_eq!(&v, &Vec::from(["0x100", "h160"]));
    debug!("{:#?} {:#?}", &left, &right);
    let data: &str = left;
    let format: &str = right;

    view! {
        <div.qr>
            <KoboldQR data={data} />
            <div>{data}</div>
            <div>{format}</div>
        </div>
    }
}
