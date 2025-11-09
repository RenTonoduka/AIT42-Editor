# AIT42-Editor Release Quick Start

This guide provides quick instructions for creating a new release of AIT42-Editor.

## TL;DR - Automated Release

```bash
# 1. Run the release script
./scripts/prepare-release.sh

# 2. Follow the prompts to:
#    - Bump version
#    - Update CHANGELOG.md
#    - Create git commit and tag
#    - Push to trigger GitHub Actions

# 3. GitHub Actions will automatically:
#    - Build for macOS (Apple Silicon + Intel)
#    - Build for Windows (x64)
#    - Build for Linux (x64)
#    - Create GitHub Release with installers
```

## Manual Release Process

### Step 1: Prepare Version

```bash
# Update version in package.json
# Update version in src-tauri/tauri.conf.json
# Update version in src-tauri/Cargo.toml
# Update CHANGELOG.md with release notes
```

### Step 2: Build Locally (Optional)

```bash
# Test build on your platform
npm run tauri build

# Check output in src-tauri/target/release/bundle/
```

### Step 3: Create Release

```bash
# Commit changes
git add .
git commit -m "chore: bump version to v1.6.1"
git push

# Create and push tag
git tag v1.6.1
git push origin v1.6.1
```

### Step 4: Monitor GitHub Actions

Visit https://github.com/MarkXAI/AIT42-Editor/actions and wait for builds to complete.

### Step 5: Publish Release

1. Go to https://github.com/MarkXAI/AIT42-Editor/releases
2. Find the draft release created by GitHub Actions
3. Review release notes and artifacts
4. Click "Publish release"

## Icon Generation (One-Time Setup)

If icons are missing:

```bash
# Generate SVG icon
python3 scripts/generate_icon.py

# Convert to PNG
open src-tauri/icons/icon-converter.html
# Click "Download PNG (1024x1024)" and save as icon.png

# Generate all icon formats
npm run tauri icon src-tauri/icons/icon.png
```

## Distribution Channels

### GitHub Releases (Primary)
- Automatic via GitHub Actions
- Users download from: https://github.com/MarkXAI/AIT42-Editor/releases

### Auto-Update (Future)
- Configure Tauri updater in `tauri.conf.json`
- Generate signing keys with: `npm run tauri signer generate`
- Users will receive automatic update notifications

## Troubleshooting

### Build fails on GitHub Actions

**Check logs at**: https://github.com/MarkXAI/AIT42-Editor/actions

**Common issues**:
- Missing dependencies: Ensure `package-lock.json` is committed
- Rust compilation errors: Test locally with `cargo build --release`
- Icon errors: Regenerate icons as described above

### Version mismatch

Ensure version is consistent across:
- `package.json`
- `src-tauri/tauri.conf.json`
- `src-tauri/Cargo.toml`

### Tag already exists

```bash
# Delete local tag
git tag -d v1.6.1

# Delete remote tag
git push origin :refs/tags/v1.6.1

# Recreate tag
git tag v1.6.1
git push origin v1.6.1
```

## Platform-Specific Notes

### macOS
- **Builds**: Apple Silicon (aarch64) + Intel (x64)
- **Format**: .dmg disk images
- **Signing**: Optional (requires Apple Developer ID)

### Windows
- **Builds**: x64 only
- **Format**: .msi installer
- **Signing**: Optional (requires code signing certificate)

### Linux
- **Builds**: x64 only
- **Formats**: .deb (Debian/Ubuntu), .AppImage (universal)
- **Signing**: Optional (GPG signatures)

## Release Checklist

Before creating a release:

- [ ] All tests passing (`cargo test`)
- [ ] CHANGELOG.md updated with changes
- [ ] Version bumped in all config files
- [ ] Icons generated (if missing)
- [ ] Documentation updated (README.md, etc.)
- [ ] Breaking changes documented (if any)
- [ ] Security issues fixed (if any)

After release:

- [ ] GitHub Release published
- [ ] Release notes reviewed
- [ ] All platform artifacts present
- [ ] Installers tested on each platform
- [ ] Users notified (announcements, social media)

## Support

For detailed instructions, see:
- [DISTRIBUTION.md](DISTRIBUTION.md) - Complete distribution guide
- [.github/workflows/release.yml](.github/workflows/release.yml) - GitHub Actions workflow
- [README.md](README.md) - Project documentation

---

**Last Updated**: 2025-11-09
**Maintained by**: AIT42 Team
