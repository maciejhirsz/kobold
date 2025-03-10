#!/bin/sh
set -e

MIRI_NIGHTLY=nightly-$(curl -s https://rust-lang.github.io/rustup-components-history/x86_64-unknown-linux-gnu/miri)
echo "Installing latest nightly with Miri: $MIRI_NIGHTLY"
rustup set profile minimal
rustup default "$MIRI_NIGHTLY"

rustup component add miri
cargo miri setup

# cd crates/kobold

cd examples/npm_lib
# rustup update
# PATH=$HOME/.cargo/bin:$PATH
rustup target add wasm32-unknown-unknown --toolchain nightly
# Note: The `--vers "0.2.100"` must match the version of `wasm-bindgen` in Cargo.toml file
cargo install wasm-bindgen-cli --vers "0.2.100"

MIRIFLAGS='-Zmiri-strict-provenance' cargo +nightly miri test --package kobold_npm_lib_example --bin main --target wasm32-unknown-unknown -Zdoctest-xcompile --verbose
