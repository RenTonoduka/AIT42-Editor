#!/usr/bin/env bash
# AIT42 Editor - Development Environment Setup Script

set -euo pipefail

readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Colors
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly RED='\033[0;31m'
readonly NC='\033[0m'

log_info() {
    echo -e "${GREEN}[INFO]${NC} $*"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $*"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $*"
}

check_rust() {
    log_info "Checking Rust installation..."
    if ! command -v rustc &> /dev/null; then
        log_error "Rust is not installed. Please install from https://rustup.rs/"
        exit 1
    fi

    local rust_version=$(rustc --version | awk '{print $2}')
    log_info "Rust version: $rust_version"

    if ! command -v cargo &> /dev/null; then
        log_error "Cargo is not installed"
        exit 1
    fi
}

install_tools() {
    log_info "Installing development tools..."

    # Install rustfmt and clippy
    if ! command -v rustfmt &> /dev/null; then
        log_info "Installing rustfmt..."
        rustup component add rustfmt
    fi

    if ! command -v cargo-clippy &> /dev/null; then
        log_info "Installing clippy..."
        rustup component add clippy
    fi

    # Install cargo-deny for dependency auditing
    if ! command -v cargo-deny &> /dev/null; then
        log_info "Installing cargo-deny..."
        cargo install cargo-deny
    fi

    # Install cargo-watch for development
    if ! command -v cargo-watch &> /dev/null; then
        log_info "Installing cargo-watch..."
        cargo install cargo-watch
    fi

    log_info "All tools installed successfully"
}

build_project() {
    log_info "Building project..."
    cd "$PROJECT_ROOT"
    cargo build
    log_info "Build completed successfully"
}

main() {
    log_info "Setting up AIT42 Editor development environment..."

    check_rust
    install_tools
    build_project

    log_info "Setup complete! You can now run:"
    log_info "  cargo run              # Run the editor"
    log_info "  cargo test             # Run tests"
    log_info "  cargo clippy           # Run linter"
    log_info "  ./scripts/test.sh      # Run full test suite"
}

main "$@"
