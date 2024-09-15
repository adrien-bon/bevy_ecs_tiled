#!/bin/bash

cargo clippy --workspace --all-targets --all-features -- --deny warnings
cargo fmt --all -- --check