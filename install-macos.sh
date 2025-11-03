#!/bin/bash
#
# AIT42 Editor - macOS Installation Script
#
# This script automates the installation of AIT42 Editor on macOS.
# It builds the release binary, sets up the app bundle, and installs to /Applications.
#

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APP_BUNDLE="$SCRIPT_DIR/AIT42.app"
BINARY_PATH="$SCRIPT_DIR/target/release/ait42"
INSTALL_DIR="/Applications"

# Print colored output
print_info() {
    echo -e "${BLUE}ℹ${NC}  $1"
}

print_success() {
    echo -e "${GREEN}✓${NC}  $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC}  $1"
}

print_error() {
    echo -e "${RED}✗${NC}  $1"
}

# Print header
echo ""
echo "╔═══════════════════════════════════════════════════════════╗"
echo "║                  AIT42 Editor Installer                  ║"
echo "║                    macOS Version                          ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo ""

# Check if Rust is installed
print_info "Checking for Rust installation..."
if ! command -v cargo &> /dev/null; then
    print_error "Rust is not installed!"
    echo ""
    echo "Please install Rust from: https://rustup.rs/"
    echo "Run: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi
print_success "Rust found: $(rustc --version)"

# Check if app bundle exists
print_info "Checking app bundle..."
if [ ! -d "$APP_BUNDLE" ]; then
    print_error "App bundle not found at: $APP_BUNDLE"
    exit 1
fi
print_success "App bundle found"

# Build release binary
print_info "Building release binary (this may take a few minutes)..."
cd "$SCRIPT_DIR"
if cargo build --release; then
    print_success "Build completed successfully"
else
    print_error "Build failed"
    exit 1
fi

# Verify binary exists
if [ ! -f "$BINARY_PATH" ]; then
    print_error "Binary not found at: $BINARY_PATH"
    exit 1
fi
print_success "Binary verified: $BINARY_PATH"

# Get binary size
BINARY_SIZE=$(du -h "$BINARY_PATH" | cut -f1)
print_info "Binary size: $BINARY_SIZE"

# Make launcher script executable
print_info "Setting executable permissions..."
chmod +x "$APP_BUNDLE/Contents/MacOS/AIT42"
print_success "Permissions set"

# Check if already installed
if [ -d "$INSTALL_DIR/AIT42.app" ]; then
    print_warning "AIT42.app already exists in $INSTALL_DIR"
    read -p "Do you want to overwrite it? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_info "Installation cancelled"
        exit 0
    fi
    print_info "Removing existing installation..."
    rm -rf "$INSTALL_DIR/AIT42.app"
fi

# Copy app bundle to Applications
print_info "Installing to $INSTALL_DIR..."
if cp -R "$APP_BUNDLE" "$INSTALL_DIR/"; then
    print_success "App installed to $INSTALL_DIR/AIT42.app"
else
    print_error "Failed to copy app bundle. You may need sudo:"
    echo "  sudo cp -R $APP_BUNDLE $INSTALL_DIR/"
    exit 1
fi

# Verify installation
if [ -d "$INSTALL_DIR/AIT42.app" ]; then
    print_success "Installation verified"
else
    print_error "Installation verification failed"
    exit 1
fi

# Print completion message
echo ""
echo "╔═══════════════════════════════════════════════════════════╗"
echo "║              Installation Complete! 🎉                    ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo ""
print_success "AIT42 Editor has been successfully installed!"
echo ""
echo "You can now launch AIT42 Editor in several ways:"
echo ""
echo "  1. Double-click AIT42.app in Applications folder"
echo "  2. Run: open /Applications/AIT42.app"
echo "  3. Right-click any file → Open With → AIT42"
echo ""
print_info "First time launch:"
echo "  - The app will open Terminal.app"
echo "  - AIT42 Editor will run in the terminal"
echo "  - This is normal for TUI applications"
echo ""
print_info "Command line usage (optional):"
echo "  Add to your ~/.zshrc or ~/.bash_profile:"
echo "  export PATH=\"$SCRIPT_DIR/target/release:\$PATH\""
echo ""
print_info "Uninstallation:"
echo "  rm -rf /Applications/AIT42.app"
echo ""
print_info "Documentation:"
echo "  See INSTALL_MACOS.md for detailed usage instructions"
echo ""

# Offer to add to PATH
echo ""
read -p "Do you want to add ait42 to your PATH? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    # Detect shell
    if [ -n "$ZSH_VERSION" ] || [ -f "$HOME/.zshrc" ]; then
        SHELL_RC="$HOME/.zshrc"
    elif [ -n "$BASH_VERSION" ] || [ -f "$HOME/.bash_profile" ]; then
        SHELL_RC="$HOME/.bash_profile"
    else
        SHELL_RC="$HOME/.profile"
    fi

    PATH_EXPORT="export PATH=\"$SCRIPT_DIR/target/release:\$PATH\""

    # Check if already in PATH
    if grep -q "$PATH_EXPORT" "$SHELL_RC" 2>/dev/null; then
        print_info "Already in PATH"
    else
        echo "" >> "$SHELL_RC"
        echo "# AIT42 Editor" >> "$SHELL_RC"
        echo "$PATH_EXPORT" >> "$SHELL_RC"
        print_success "Added to $SHELL_RC"
        print_warning "Restart your terminal or run: source $SHELL_RC"
    fi
fi

echo ""
print_success "Setup complete! Enjoy using AIT42 Editor! 🚀"
echo ""
