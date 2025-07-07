#!/bin/bash

cargo clippy --workspace --all-targets --all-features -- --deny warnings || { echo "cargo clippy failed"; exit 1; }
cargo fmt --all -- --check || { echo "cargo fmt failed"; exit 1; }
cargo test --workspace --all-features --all-targets || { echo "cargo test failed"; exit 1; }
cargo test --workspace --doc --all-features || { echo "cargo test --doc failed"; exit 1; }
cargo check-all-features || { echo "cargo check-all-features failed"; exit 1; }
echo "All OK! :)"