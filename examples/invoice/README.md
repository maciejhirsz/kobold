* Browser Compatibility:
    * Brave Version 1.50.121 Chromium: 112.0.5615.138 (Official Build) (x86_64)
    * Chrome Version 112.0.5615.137 (Official Build) (x86_64)
    * Firefox Version 112.0.2 (64-bit)

* Notes:
    * Closure (e.g. `state.update(|state| state.store())` has access to Signal of state
        * `update` doesn't implement Deref so you can't access fields on it like you can with a Hook
        * `update_silent` gives access to the actual state without triggering a render
    * State
        * Tables
            * all `Table` cells should be populated with `Insitu` by default, the only exception is when you have escapes in the loaded CSV. e.g. if your CSV contains quotes in quotes, the parser needs to change escapes quotes into unescaped ones, so it will allocate a String to do it in. for a value in quotes it slices with +1/-1 to skip quotes, and then for escapes it also skips quotes and then replaces escaped quotes inside. if you put something like: `"hello ""world"""` in your CSV file, that will be `Text::Owned`
            * the `Table` `source` property values should be read only
            * if you edit a `Table` cell, just swap it from `Insitu` to `Owned` text
            * you get an owned string from `.value()` so there is no point in trying to avoid it
            * loading a file prefers `Insitu` since it can just borrow all unescaped values from the `source` without allocations
            * it uses `fn parse_row` in csv.rs to magically know whether to store in `Insitu` instead of `Owned`, otherwise we explicitly tell it to use `Insitu` when setting the default value `Text::Insitu(0..0)` in this file and when we edit a field in the UI so it becomes `Owned("text")` (where text is what we enter)
            * `Text` is used instead of just String to avoid unnecessary allocations that are expensive, since subslicing the `source` with an `Insitu` `range` is a const operation, so it's just fiddling with a pointer and the length - so it's not exactly free, but it's as close to free as you can get. even better would be for `Insitu` to contain `&str`, but internal borrowing is a bit of a pain

* References:
    * [kobold docs](https://docs.rs/kobold/latest/kobold/)
    * [wasm-bindgen docs](https://rustwasm.github.io/docs/wasm-bindgen/introduction.html)
        * [web-sys File](https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.File.html#)
        * [js-sys](https://docs.rs/js-sys/latest/js_sys)
            * Example: https://yew.rs/docs/0.18.0/concepts/wasm-bindgen/web-sys
    * [std::fs::File::create](https://doc.rust-lang.org/std/fs/struct.File.html#method.create)

* Credits:
    * Maciej Hirsz
