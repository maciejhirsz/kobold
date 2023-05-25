## Kobold Load NPM Module 
* Install Node.js and dependencies. Build JS to ESM. Run in browser
```bash
yarn
yarn run esbuild
RUST_LOG=info trunk serve --address=127.0.0.1 --open
```
* Test
```bash
cargo test --target wasm32-unknown-unknown
```

* References:
    * https://rustwasm.github.io/docs/wasm-bindgen
    * https://stackoverflow.com/questions/73490625/how-to-load-a-npm-package-to-wasm-bindgen
    * https://stackoverflow.com/questions/75422119/using-npm-packages-with-rust-and-webassembly
