* Usage
    * Run the following from the project root directory:
    ```
    cd examples/invoice/
    cargo install --locked trunk
    RUST_LOG=info trunk serve --address=127.0.0.1 --open
    ```
    * Open in web browser http://localhost:8080
    * Upload, edit (saves in local storage), and download a backup to a CSV file for the "Main" table
        * Create a text file similar to the example in folder ./data/main.csv  and `mock_file_main` in state.rs, prefixed with `#main,`.
            * Note: It looks like there is an additional column on the first row but that cell will be removed during the upload process and used to populate a `TableVariant` value in the state that is reflected in Local Storage
        * Upload a file by clicking "Upload CSV file (Main) to upload it in the "Main" table
        * View the file in the UI and serialised in browser Local Storage under key `kobald.invoice.main`
        * Modify the table by double clicking cells and pressing escape or enter to save
        * Save a backup of the file by clicking the associated "Save to CSV file" button
            * Note: The downloaded file should be prefixed with `#main,` to indicate it uses the `TableVariant::Main` table
    * Upload, edit (saves in local storage), and download a backup to a CSV file for the "Details" table
        * Repeat steps used for the "Main" table, but similar to example in folder ./data/details.csv and `mock_file_details` in state.rs, and prefixed with `#details,` instead, and stored under `kobald.invoice.details` in Local Storage instead.

* Contributing Guidelines
    * Format `cargo fmt --all` before pushing commits
    * Test with `cargo test` before pushing commits

* Browser Compatibility:
    * Brave Version 1.50.121 Chromium: 112.0.5615.138 (Official Build) (x86_64)
    * Chrome Version 112.0.5615.137 (Official Build) (x86_64)
    * Firefox Version 112.0.2 (64-bit)

* Notes:
    * Best Practice
        * Use `&str` and avoid `String`
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
