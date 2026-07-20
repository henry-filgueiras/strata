#!/usr/bin/env bash
set -euo pipefail

echo "==> cargo fmt --check"
cargo fmt --check

echo "==> cargo test"
cargo test

echo "==> cargo clippy"
cargo clippy --all-targets --all-features -- -D warnings

echo "==> all checks passed"
