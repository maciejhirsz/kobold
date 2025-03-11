#!/bin/sh
set -e

MIRI_NIGHTLY=nightly-$(curl -s https://rust-lang.github.io/rustup-components-history/x86_64-unknown-linux-gnu/miri)
echo "Installing latest nightly with Miri: $MIRI_NIGHTLY"
rustup set profile minimal
rustup default "$MIRI_NIGHTLY"

rustup component add miri
cargo miri setup

# cd crates/kobold
# MIRIFLAGS='-Zmiri-strict-provenance' cargo miri test

# curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
# . $HOME/.cargo/env
# rustup toolchain install nightly
# rustup default stable
# rustup update
# rustup update nightly
# rustup target add wasm32-unknown-unknown
# rustup target add wasm32-unknown-unknown --toolchain nightly

# rustup set profile minimal

cd examples/npm_lib

npm install -g yarn
yarn
yarn run esbuild

rustup target add wasm32-unknown-unknown
rustup target add wasm32-unknown-unknown --toolchain nightly
cargo install --locked trunk
RUST_LOG=debug trunk --config ./ build
# Note: The `--vers "0.2.100"` must match the version of `wasm-bindgen` in Cargo.toml file
cargo install wasm-bindgen-cli --vers "0.2.100"
cargo +nightly build --package kobold_npm_lib_example --bin main --target wasm32-unknown-unknown -Zdoctest-xcompile --verbose

MIRIFLAGS='-Zmiri-strict-provenance' cargo check
MIRIFLAGS='-Zmiri-strict-provenance' cargo fmt --check

# Check all packages
cd ../../

MIRIFLAGS='-Zmiri-strict-provenance' cargo check --features serde,stateful --workspace --all-targets --bins --examples
MIRIFLAGS='-Zmiri-strict-provenance' cargo fmt --check
