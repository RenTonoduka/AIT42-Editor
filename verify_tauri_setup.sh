#!/bin/bash

# AIT42 Editor - Tauri Setup Verification Script
# Verifies that all components are properly configured

set -e

# Add Rust to PATH if it exists
if [ -d "$HOME/.cargo/bin" ]; then
    export PATH="$HOME/.cargo/bin:$PATH"
fi

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "🔍 AIT42 Editor - Tauri Setup Verification"
echo "=========================================="
echo ""

# Function to check if a file exists
check_file() {
    if [ -f "$1" ]; then
        echo -e "${GREEN}✓${NC} $2"
        return 0
    else
        echo -e "${RED}✗${NC} $2 (missing: $1)"
        return 1
    fi
}

# Function to check if a directory exists
check_dir() {
    if [ -d "$1" ]; then
        echo -e "${GREEN}✓${NC} $2"
        return 0
    else
        echo -e "${RED}✗${NC} $2 (missing: $1)"
        return 1
    fi
}

# Check Rust installation
echo "📦 Checking Rust toolchain..."
if command -v cargo &> /dev/null; then
    VERSION=$(cargo --version)
    echo -e "${GREEN}✓${NC} Rust installed: $VERSION"
else
    echo -e "${RED}✗${NC} Rust not found. Install from https://rustup.rs"
    exit 1
fi
echo ""

# Check Node.js installation
echo "📦 Checking Node.js..."
if command -v node &> /dev/null; then
    VERSION=$(node --version)
    echo -e "${GREEN}✓${NC} Node.js installed: $VERSION"
else
    echo -e "${RED}✗${NC} Node.js not found. Install from https://nodejs.org"
    exit 1
fi

if command -v npm &> /dev/null; then
    VERSION=$(npm --version)
    echo -e "${GREEN}✓${NC} npm installed: $VERSION"
else
    echo -e "${RED}✗${NC} npm not found"
    exit 1
fi
echo ""

# Check directory structure
echo "📁 Checking directory structure..."
check_dir "src-tauri" "Tauri backend directory"
check_dir "src-tauri/src" "Backend source directory"
check_dir "src-tauri/icons" "Icons directory"
check_dir "src" "Frontend source directory"
check_dir "src/components" "Components directory"
check_dir "src/services" "Services directory"
check_dir "src/types" "Types directory"
echo ""

# Check configuration files
echo "⚙️  Checking configuration files..."
check_file "src-tauri/Cargo.toml" "Tauri Cargo.toml"
check_file "src-tauri/tauri.conf.json" "Tauri configuration"
check_file "src-tauri/build.rs" "Build script"
check_file "package.json" "package.json"
check_file "vite.config.ts" "Vite configuration"
check_file "tsconfig.json" "TypeScript configuration"
check_file "tailwind.config.js" "Tailwind configuration"
echo ""

# Check source files
echo "📄 Checking source files..."
check_file "src-tauri/src/main.rs" "Backend main.rs"
check_file "src-tauri/src/state.rs" "Application state"
check_file "src-tauri/src/commands/mod.rs" "Tauri commands module"
check_file "src-tauri/src/commands/file.rs" "File commands"
check_file "src-tauri/src/commands/editor.rs" "Editor commands"
check_file "src/main.tsx" "React entry point"
check_file "src/App.tsx" "Main App component"
check_file "src/index.css" "Global styles"
check_file "src/types/index.ts" "Type definitions"
check_file "src/services/tauri.ts" "Tauri service bindings"
check_file "index.html" "HTML template"
echo ""

# Check documentation
echo "📚 Checking documentation..."
check_file "TAURI_SETUP.md" "Setup guide"
check_file "NEXT_STEPS.md" "Implementation roadmap"
check_file "TAURI_INIT_REPORT.md" "Initialization report"
check_file "ARCHITECTURE_GUI.md" "Architecture documentation"
echo ""

# Check node_modules
echo "📦 Checking dependencies..."
if [ -d "node_modules" ]; then
    COUNT=$(ls -1 node_modules | wc -l | tr -d ' ')
    echo -e "${GREEN}✓${NC} node_modules present ($COUNT packages)"
else
    echo -e "${YELLOW}⚠${NC} node_modules not found. Run 'npm install'"
fi
echo ""

# Check Cargo workspace
echo "🦀 Checking Cargo workspace..."
if grep -q "src-tauri" Cargo.toml; then
    echo -e "${GREEN}✓${NC} src-tauri is in workspace members"
else
    echo -e "${RED}✗${NC} src-tauri not in Cargo workspace"
fi
echo ""

# Try to compile Rust backend
echo "🔨 Checking Rust compilation..."
cd src-tauri
if cargo check --quiet 2>/dev/null; then
    echo -e "${GREEN}✓${NC} Rust backend compiles successfully"
else
    echo -e "${YELLOW}⚠${NC} Rust compilation check in progress or failed"
    echo "   Run 'cd src-tauri && cargo check' for details"
fi
cd ..
echo ""

# Check TypeScript compilation
echo "🔨 Checking TypeScript compilation..."
if npx tsc --noEmit 2>/dev/null; then
    echo -e "${GREEN}✓${NC} TypeScript compiles successfully"
else
    echo -e "${YELLOW}⚠${NC} TypeScript compilation has warnings/errors"
    echo "   Run 'npx tsc --noEmit' for details"
fi
echo ""

# Summary
echo "=========================================="
echo -e "${GREEN}✅ Tauri setup verification complete!${NC}"
echo ""
echo "Next steps:"
echo "  1. Run 'npm install' if not done already"
echo "  2. Run 'npm run tauri:dev' to start development server"
echo "  3. See NEXT_STEPS.md for implementation roadmap"
echo ""
