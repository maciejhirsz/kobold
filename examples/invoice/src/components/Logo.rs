// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use kobold::prelude::*;

#[component(
    // using `?` makes the property an optional parameter and falls back to the default
    alt?: "logo".to_string(),
    caption?: "Kobold".to_string(), 
    // TODO - handle SVG images
    // image_url?: "https://raw.githubusercontent.com/maciejhirsz/kobold/master/kobold.svg",
    image_url?: "https://github.githubassets.com/images/mona-loading-default.gif",
    width?: "70px",
    height?: "100px",
)]
pub fn Logo<'a>(
    alt: String,
    caption: String,
    height: &'a str,
    width: &'a str,
    image_url: &'a str,
) -> impl View + 'a {
    view! {
        <div>
            <img src={image_url} {width} {height} alt={alt.to_string()}>
            <span> { caption.to_string() }
    }
}
