#!/bin/sh
set -e
echo "--- Rust Format Check ---"
cargo fmt -- --check
echo "--- Rust Clippy Linter ---"
cargo clippy -- -D warnings
echo "--- Rust Tests ---"
cargo test
echo "--- Rust Build Check ---"
cargo check
echo "--- DONE ---"
