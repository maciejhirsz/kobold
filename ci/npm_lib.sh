#!/bin/sh
set -e

cd examples/npm_lib
cargo +nightly test --package kobold_npm_lib_example --bin main --target wasm32-unknown-unknown -Zdoctest-xcompile --verbose
