#!/bin/bash
# Prepare AIT42-Editor for release
# This script automates version bumping, changelog updates, and release preparation

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

# Check if we're in the project root
if [ ! -f "package.json" ] || [ ! -d "src-tauri" ]; then
    print_error "This script must be run from the AIT42-Editor project root"
    exit 1
fi

# Get current version
CURRENT_VERSION=$(node -p "require('./package.json').version")
print_info "Current version: v${CURRENT_VERSION}"

# Ask for new version
echo ""
read -p "Enter new version (e.g., 1.6.1): " NEW_VERSION

if [ -z "$NEW_VERSION" ]; then
    print_error "Version cannot be empty"
    exit 1
fi

# Validate version format
if ! [[ $NEW_VERSION =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    print_error "Invalid version format. Use semantic versioning (e.g., 1.6.1)"
    exit 1
fi

print_info "New version will be: v${NEW_VERSION}"

# Ask for confirmation
echo ""
read -p "Continue with version bump? (y/n): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    print_warning "Aborted by user"
    exit 0
fi

echo ""
print_info "Updating version in package.json..."
# Update package.json
node -e "
const fs = require('fs');
const pkg = JSON.parse(fs.readFileSync('package.json', 'utf8'));
pkg.version = '${NEW_VERSION}';
fs.writeFileSync('package.json', JSON.stringify(pkg, null, 2) + '\n');
"
print_success "Updated package.json"

print_info "Updating version in src-tauri/tauri.conf.json..."
# Update tauri.conf.json
node -e "
const fs = require('fs');
const config = JSON.parse(fs.readFileSync('src-tauri/tauri.conf.json', 'utf8'));
config.package.version = '${NEW_VERSION}';
fs.writeFileSync('src-tauri/tauri.conf.json', JSON.stringify(config, null, 2) + '\n');
"
print_success "Updated tauri.conf.json"

# Update Cargo.toml version
print_info "Updating version in src-tauri/Cargo.toml..."
if [ -f "src-tauri/Cargo.toml" ]; then
    sed -i.bak "s/^version = .*/version = \"${NEW_VERSION}\"/" src-tauri/Cargo.toml
    rm -f src-tauri/Cargo.toml.bak
    print_success "Updated Cargo.toml"
fi

# Create CHANGELOG entry
print_info "Preparing CHANGELOG.md..."
if [ ! -f "CHANGELOG.md" ]; then
    cat > CHANGELOG.md << EOF
# Changelog

All notable changes to AIT42-Editor will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

EOF
fi

# Add new version entry
TODAY=$(date +%Y-%m-%d)
cat > /tmp/changelog_entry << EOF
## [${NEW_VERSION}] - ${TODAY}

### Added
-

### Changed
-

### Fixed
-

### Security
-

EOF

# Prepend new entry to CHANGELOG
if grep -q "## \[" CHANGELOG.md 2>/dev/null; then
    # Insert after the header
    awk '/^## \[/{exit} {print}' CHANGELOG.md > /tmp/changelog_header
    awk '/^## \[/{p=1} p' CHANGELOG.md > /tmp/changelog_rest
    cat /tmp/changelog_header /tmp/changelog_entry /tmp/changelog_rest > CHANGELOG.md
    rm -f /tmp/changelog_header /tmp/changelog_rest
else
    cat /tmp/changelog_entry >> CHANGELOG.md
fi
rm -f /tmp/changelog_entry

print_success "Added v${NEW_VERSION} entry to CHANGELOG.md"

# Check if icons are generated
print_info "Checking icon files..."
if [ ! -f "src-tauri/icons/icon.icns" ] || [ ! -f "src-tauri/icons/icon.ico" ]; then
    print_warning "Icon files not found. Please generate icons:"
    print_warning "  1. Generate PNG: node scripts/svg-to-png.js"
    print_warning "  2. Generate icons: npm run tauri icon src-tauri/icons/icon.png"
else
    print_success "Icon files found"
fi

# Run tests
echo ""
read -p "Run tests before release? (recommended) (y/n): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    print_info "Running tests..."
    if cargo test --manifest-path src-tauri/Cargo.toml; then
        print_success "All tests passed"
    else
        print_error "Tests failed. Please fix before releasing."
        exit 1
    fi
fi

# Create git commit
echo ""
read -p "Create git commit for version bump? (y/n): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    git add package.json src-tauri/tauri.conf.json src-tauri/Cargo.toml CHANGELOG.md
    git commit -m "chore: bump version to v${NEW_VERSION}

🤖 Generated with [Claude Code](https://claude.ai/code)

Co-Authored-By: Claude <noreply@anthropic.com>"
    print_success "Created git commit"

    # Ask to push
    read -p "Push to remote? (y/n): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)
        git push origin "$CURRENT_BRANCH"
        print_success "Pushed to origin/${CURRENT_BRANCH}"
    fi
fi

# Create git tag
echo ""
read -p "Create git tag v${NEW_VERSION}? (y/n): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    git tag -a "v${NEW_VERSION}" -m "Release v${NEW_VERSION}

AIT42-Editor v${NEW_VERSION}

See CHANGELOG.md for details.

🤖 Generated with [Claude Code](https://claude.ai/code)

Co-Authored-By: Claude <noreply@anthropic.com>"
    print_success "Created tag v${NEW_VERSION}"

    # Ask to push tag
    read -p "Push tag to trigger release workflow? (y/n): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        git push origin "v${NEW_VERSION}"
        print_success "Pushed tag v${NEW_VERSION}"
        echo ""
        print_success "GitHub Actions will now build and publish the release!"
        print_info "Monitor progress at: https://github.com/MarkXAI/AIT42-Editor/actions"
    fi
fi

echo ""
print_success "Release preparation complete!"
echo ""
print_info "Next steps:"
echo "  1. Update CHANGELOG.md with actual changes"
echo "  2. Wait for GitHub Actions to build artifacts"
echo "  3. Review and publish the draft release on GitHub"
echo "  4. Announce the release to users"
echo ""
print_info "Release checklist:"
echo "  ✓ Version bumped to v${NEW_VERSION}"
echo "  ✓ CHANGELOG.md updated"
echo "  ✓ Git commit created (if selected)"
echo "  ✓ Git tag created (if selected)"
echo ""
