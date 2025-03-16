#!/bin/bash

# Reference: https://github.com/rustwasm/wasm-pack/issues/1420
XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-"/run/user/$UID"}"
exec flock "$XDG_RUNTIME_DIR/wasm-pack-lock" "${CARGO_HOME:-"$HOME/.cargo"}/bin/wasm-pack" "$@"
