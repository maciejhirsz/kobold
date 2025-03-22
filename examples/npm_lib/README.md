## Kobold Load NPM Module

### Usage

Install Node.js and dependencies. Build JS to ESM. Run in browser.

```sh
rm -rf node_modules
rm -rf yarn.lock
nvm use
nvm install
npm install -g yarn
yarn
yarn run esbuild
rustup update
rustup target add wasm32-unknown-unknown
cargo install --locked trunk
RUST_LOG=info trunk serve
```

Click "Connect" in the UI at http://localhost:8080 and view it use Polkadot.js API to fetch and display a genesis hash

### Testing

Run tests including doctests:

```sh
cargo clean
# Note: The `--vers "0.2.100"` must match the version of `wasm-bindgen` in Cargo.toml file
cargo install wasm-bindgen-cli --vers "0.2.100"
cargo +nightly test --target wasm32-unknown-unknown -Zdoctest-xcompile
```

### Config

```sh
RUST_LOG=debug trunk --config ./examples/npm_lib config show
```

### References:

* https://rustwasm.github.io/docs/wasm-bindgen
* https://stackoverflow.com/questions/73490625/how-to-load-a-npm-package-to-wasm-bindgen
* https://stackoverflow.com/questions/75422119/using-npm-packages-with-rust-and-webassembly
* https://rustwasm.github.io/wasm-bindgen/wasm-bindgen-test/usage.html#appendix-using-wasm-bindgen-test-without-wasm-pack
