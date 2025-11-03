#!/usr/bin/env bash
# AIT42 Editor - Build Script

set -euo pipefail

readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

MODE="${1:-debug}"

case "$MODE" in
    debug)
        echo "Building in debug mode..."
        cd "$PROJECT_ROOT"
        cargo build
        ;;
    release)
        echo "Building in release mode..."
        cd "$PROJECT_ROOT"
        cargo build --release
        ;;
    *)
        echo "Usage: $0 [debug|release]"
        exit 1
        ;;
esac

echo "Build completed successfully"
