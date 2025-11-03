#!/usr/bin/env bash
# AIT42 Editor - Release Build Script

set -euo pipefail

readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

VERSION="${1:-}"

if [ -z "$VERSION" ]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 0.1.0"
    exit 1
fi

echo "Building release version $VERSION..."
cd "$PROJECT_ROOT"

# Update version in Cargo.toml files
# (This would need a proper implementation)

# Build release
cargo build --release

# Run tests
cargo test --release

# Create release artifacts
RELEASE_DIR="$PROJECT_ROOT/target/release"
DIST_DIR="$PROJECT_ROOT/dist"
mkdir -p "$DIST_DIR"

# Copy binary
cp "$RELEASE_DIR/ait42" "$DIST_DIR/ait42-$VERSION"

# Create tarball
tar -czf "$DIST_DIR/ait42-$VERSION-$(uname -m)-$(uname -s).tar.gz" \
    -C "$RELEASE_DIR" ait42

echo "Release build completed: $DIST_DIR/ait42-$VERSION"
