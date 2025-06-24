#!/bin/bash
set -ex 

cargo clippy --tests
cargo clippy --tests --all-features
cargo clippy --no-default-features

cargo fmt --check

cargo build --all-features
cargo build --no-default-features

cargo test --quiet --all-features
cargo doc --no-deps
