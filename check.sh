#!/bin/bash
set -eu

cargo test --workspace --all-targets --all-features
cargo check --workspace --all-targets --all-features
cargo check --lib --target wasm32-unknown-unknown --all-features
CARGO_INCREMENTAL=0 cargo clippy --workspace --all-targets --all-features --  -D warnings -W clippy::all
cargo fmt --all -- --check
