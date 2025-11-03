#!/usr/bin/env bash
# AIT42 Editor - Test Script

set -euo pipefail

readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "Running tests..."
cd "$PROJECT_ROOT"

# Unit tests
echo "==> Running unit tests..."
cargo test --lib

# Integration tests
echo "==> Running integration tests..."
cargo test --test '*'

# Doc tests
echo "==> Running doc tests..."
cargo test --doc

# Generate coverage report (optional)
if command -v cargo-tarpaulin &> /dev/null; then
    echo "==> Generating coverage report..."
    cargo tarpaulin --out Html --output-dir coverage
    echo "Coverage report generated in coverage/"
fi

echo "All tests passed!"
