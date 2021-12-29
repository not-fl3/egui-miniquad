#!/bin/bash
set -eu

cargo test --workspace --all-features
cargo check --workspace --all-features
cargo check --lib --target wasm32-unknown-unknown --all-features
CARGO_INCREMENTAL=0 cargo clippy --workspace --all-features --  -D warnings -W clippy::all
cargo fmt --all -- --check
