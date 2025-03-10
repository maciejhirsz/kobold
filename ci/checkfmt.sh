#!/bin/sh
set -e

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
. $HOME/.cargo/env
rustup toolchain install nightly
rustup default stable
rustup update
rustup update nightly
rustup target add wasm32-unknown-unknown
rustup target add wasm32-unknown-unknown --toolchain nightly

rustup set profile minimal

cd examples/npm_lib

npm install -g yarn
yarn
yarn run esbuild

rustup target add wasm32-unknown-unknown --toolchain nightly
cargo install --locked trunk
RUST_LOG=debug trunk --config ./ build
# Note: The `--vers "0.2.100"` must match the version of `wasm-bindgen` in Cargo.toml file
cargo install wasm-bindgen-cli --vers "0.2.100"
cargo +nightly build --package kobold_npm_lib_example --bin main --target wasm32-unknown-unknown -Zdoctest-xcompile --verbose

cargo check --features serde
cargo fmt --check
