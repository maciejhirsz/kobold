#!/bin/sh
set -e

cd examples/npm_lib
rustup update
rustup target add wasm32-unknown-unknown
# Note: The `--vers "0.2.100"` must match the version of `wasm-bindgen` in Cargo.toml file
cargo install wasm-bindgen-cli --vers "0.2.100"
cargo +nightly test --package kobold_npm_lib_example --bin main --target wasm32-unknown-unknown -Zdoctest-xcompile --verbose
