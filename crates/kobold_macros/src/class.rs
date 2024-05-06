// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::hash::{Hash, Hasher};

use crate::gen::JsFnName;
use crate::parse::prelude::*;
use crate::tokenize::prelude::*;
use crate::TokenStreamExt;
use tokens::TokenStream;

pub fn parse(stream: TokenStream) -> Result<TokenStream, ParseError> {
    let mut stream = stream.parse_stream();

    let class = stream.expect(Lit)?;

    let class = class.to_string();
    let class = &class[1..class.len() - 1];

    stream.expect("if")?;

    let mut hasher = fnv::FnvHasher::default();
    class.hash(&mut hasher);

    let hash = hasher.finish();
    let fn_name = JsFnName::try_from(format_args!("__class_{hash:016x}")).unwrap();

    let condition: TokenStream = stream.collect();

    let tokens = block((format_args!("\
        use ::kobold::reexport::wasm_bindgen;\
        use wasm_bindgen::prelude::wasm_bindgen;\
        \
        #[wasm_bindgen(inline_js = \"export function {fn_name}(n,v) {{ n.classList.toggle(\\\"{class}\\\",v); }}\")]\
        extern \"C\" {{\
            #[wasm_bindgen(js_name = \"{fn_name}\")]\
            pub fn t(node: &::kobold::reexport::web_sys::Node, on: bool);\
        }}"),
        call("::kobold::attribute::StaticClass::new", ("t,", condition)),
    )).tokenize();

    // panic!("tokens: {}", tokens);

    Ok(tokens)
}
