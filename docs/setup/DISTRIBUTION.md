# AIT42-Editor Distribution Guide

This guide explains how to build and distribute AIT42-Editor for end users.

## Table of Contents

- [Quick Start](#quick-start)
- [Building for Production](#building-for-production)
- [Creating a Release](#creating-a-release)
- [Manual Distribution](#manual-distribution)
- [Troubleshooting](#troubleshooting)

## Quick Start

### Prerequisites

Before building for production, ensure you have:

- **Node.js**: 20.0 or higher
- **Rust**: 1.75 or higher
- **Platform-specific dependencies** (see below)

#### macOS

```bash
# Install Xcode Command Line Tools
xcode-select --install
```

#### Linux (Ubuntu/Debian)

```bash
sudo apt-get update
sudo apt-get install -y \
  libgtk-3-dev \
  libwebkit2gtk-4.0-dev \
  libappindicator3-dev \
  librsvg2-dev \
  patchelf
```

#### Windows

Install:
- [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/) with C++ tools
- [WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)

## Building for Production

### Local Build

```bash
# Clone the repository
git clone https://github.com/MarkXAI/AIT42-Editor.git
cd AIT42-Editor

# Install dependencies
npm install

# Build for production
npm run tauri build
```

### Build Outputs

After a successful build, installers will be created in `src-tauri/target/release/bundle/`:

#### macOS

```
src-tauri/target/release/bundle/
├── dmg/
│   └── AIT42-Editor_[VERSION]_[ARCH].dmg       # Disk image installer
└── macos/
    └── AIT42-Editor.app/                        # Application bundle
```

- **Apple Silicon**: `AIT42-Editor_[VERSION]_aarch64.dmg`
- **Intel**: `AIT42-Editor_[VERSION]_x64.dmg`

#### Windows

```
src-tauri/target/release/bundle/
├── msi/
│   └── AIT42-Editor_[VERSION]_x64_en-US.msi    # MSI installer
└── nsis/
    └── AIT42-Editor_[VERSION]_x64-setup.exe     # NSIS installer (if configured)
```

#### Linux

```
src-tauri/target/release/bundle/
├── deb/
│   └── ait42-editor_[VERSION]_amd64.deb        # Debian package
├── appimage/
│   └── ait42-editor_[VERSION]_amd64.AppImage   # Portable AppImage
└── rpm/
    └── ait42-editor-[VERSION].x86_64.rpm       # RPM package (if configured)
```

### Cross-Platform Builds

To build for multiple platforms, you'll need to either:

1. **Use GitHub Actions** (recommended) - Automatically builds for all platforms
2. **Cross-compile locally** (advanced) - Requires additional setup

#### GitHub Actions Build (Recommended)

The repository includes a GitHub Actions workflow that automatically builds for all platforms:

```bash
# Create a new tag to trigger release build
git tag v1.6.0
git push origin v1.6.0

# Or manually trigger via GitHub UI:
# Go to Actions → Release → Run workflow
```

This will:
- Build for macOS (Apple Silicon + Intel)
- Build for Windows (x64)
- Build for Linux (x64)
- Create a GitHub Release with all installers attached

## Creating a Release

### Version Bumping

1. **Update version in `package.json`**:
   ```json
   {
     "version": "1.6.0"
   }
   ```

2. **Update version in `src-tauri/tauri.conf.json`**:
   ```json
   {
     "package": {
       "version": "1.6.0"
     }
   }
   ```

3. **Update `CHANGELOG.md`** with release notes

### Automated Release via GitHub Actions

```bash
# Ensure all changes are committed
git add .
git commit -m "chore: bump version to v1.6.0"
git push

# Create and push a tag
git tag v1.6.0
git push origin v1.6.0
```

The GitHub Actions workflow will:
1. Create a draft release
2. Build installers for all platforms
3. Upload installers to the release
4. Publish the release automatically

### Manual Release Process

If you prefer to release manually:

1. **Build locally** for your platform:
   ```bash
   npm run tauri build
   ```

2. **Create a GitHub Release**:
   - Go to https://github.com/MarkXAI/AIT42-Editor/releases/new
   - Create a new tag (e.g., `v1.6.0`)
   - Write release notes
   - Upload installers from `src-tauri/target/release/bundle/`

3. **Sign artifacts** (optional but recommended):
   ```bash
   # macOS: Sign with Apple Developer ID
   codesign -s "Developer ID Application: Your Name" AIT42-Editor.app

   # Windows: Sign with SignTool
   signtool sign /f certificate.pfx /p password AIT42-Editor.msi

   # Linux: Sign with GPG
   gpg --detach-sign --armor ait42-editor.AppImage
   ```

## Manual Distribution

### Distribution Channels

1. **GitHub Releases** (Primary)
   - Users download directly from https://github.com/MarkXAI/AIT42-Editor/releases
   - Automatic update checks via Tauri updater

2. **Homebrew** (macOS, optional)
   ```bash
   # Create a Homebrew formula
   brew tap MarkXAI/ait42-editor
   brew install --cask ait42-editor
   ```

3. **Chocolatey** (Windows, optional)
   ```powershell
   choco install ait42-editor
   ```

4. **Snap Store** (Linux, optional)
   ```bash
   snap install ait42-editor
   ```

### Self-Hosted Distribution

To host installers on your own server:

1. **Build all platform installers**
2. **Upload to your server**:
   ```
   https://downloads.example.com/ait42-editor/
   ├── v1.6.0/
   │   ├── AIT42-Editor_1.6.0_aarch64.dmg
   │   ├── AIT42-Editor_1.6.0_x64.dmg
   │   ├── AIT42-Editor_1.6.0_x64_en-US.msi
   │   ├── ait42-editor_1.6.0_amd64.deb
   │   └── ait42-editor_1.6.0_amd64.AppImage
   └── latest.json  # Update manifest for auto-updater
   ```

3. **Update `tauri.conf.json`** to point to your server:
   ```json
   {
     "updater": {
       "active": true,
       "endpoints": [
         "https://downloads.example.com/ait42-editor/latest.json"
       ]
     }
   }
   ```

## Auto-Update Configuration

AIT42-Editor supports automatic updates via Tauri's built-in updater.

### Setup

1. **Generate update signature keys**:
   ```bash
   npm run tauri signer generate -- -w ~/.tauri/ait42-editor.key
   ```

2. **Add public key to `tauri.conf.json`**:
   ```json
   {
     "updater": {
       "active": true,
       "pubkey": "YOUR_PUBLIC_KEY_HERE"
     }
   }
   ```

3. **Set GitHub secrets** for automatic signing:
   - `TAURI_PRIVATE_KEY`: Private key content
   - `TAURI_KEY_PASSWORD`: Key password (if set)

4. **Create update manifest** (`latest.json`):
   ```json
   {
     "version": "1.6.0",
     "notes": "Bug fixes and improvements",
     "pub_date": "2025-11-09T00:00:00Z",
     "platforms": {
       "darwin-aarch64": {
         "signature": "BASE64_SIGNATURE",
         "url": "https://github.com/MarkXAI/AIT42-Editor/releases/download/v1.6.0/AIT42-Editor_1.6.0_aarch64.dmg"
       },
       "darwin-x86_64": {
         "signature": "BASE64_SIGNATURE",
         "url": "https://github.com/MarkXAI/AIT42-Editor/releases/download/v1.6.0/AIT42-Editor_1.6.0_x64.dmg"
       },
       "windows-x86_64": {
         "signature": "BASE64_SIGNATURE",
         "url": "https://github.com/MarkXAI/AIT42-Editor/releases/download/v1.6.0/AIT42-Editor_1.6.0_x64_en-US.msi"
       },
       "linux-x86_64": {
         "signature": "BASE64_SIGNATURE",
         "url": "https://github.com/MarkXAI/AIT42-Editor/releases/download/v1.6.0/ait42-editor_1.6.0_amd64.AppImage"
       }
     }
   }
   ```

## Troubleshooting

### Build Failures

#### "command not found: tauri"

```bash
# Reinstall Tauri CLI
npm install --save-dev @tauri-apps/cli
```

#### macOS: "xcrun: error: invalid active developer path"

```bash
# Install Xcode Command Line Tools
xcode-select --install
```

#### Linux: "webkit2gtk not found"

```bash
# Install WebKit2GTK
sudo apt-get install libwebkit2gtk-4.0-dev
```

#### Windows: "link.exe not found"

Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/) with C++ support.

### Icon Issues

If icons are not showing properly:

1. **Verify icons exist** in `src-tauri/icons/`:
   ```bash
   ls -la src-tauri/icons/
   ```

2. **Regenerate icons** from a PNG source:
   ```bash
   npm run tauri icon path/to/icon.png
   ```

3. **Required icon formats**:
   - macOS: `icon.icns`
   - Windows: `icon.ico`
   - Linux: `icon.png` (various sizes)

### Code Signing Issues

#### macOS: "Developer ID not found"

1. **Enroll in Apple Developer Program** ($99/year)
2. **Create a Developer ID certificate** in Xcode
3. **Sign the app**:
   ```bash
   codesign -s "Developer ID Application: Your Name" -f --deep AIT42-Editor.app
   ```

#### Windows: "SignTool error"

1. **Obtain a code signing certificate** from a CA
2. **Sign with SignTool**:
   ```powershell
   signtool sign /f certificate.pfx /p password /tr http://timestamp.digicert.com AIT42-Editor.msi
   ```

### Performance Optimization

#### Reduce Bundle Size

```bash
# Enable LTO (Link Time Optimization) in Cargo.toml
[profile.release]
lto = true
codegen-units = 1
opt-level = "z"  # Optimize for size
strip = true     # Strip symbols
```

#### Split Code

Update `vite.config.ts` to enable code splitting:

```typescript
export default defineConfig({
  build: {
    rollupOptions: {
      output: {
        manualChunks: {
          monaco: ['@monaco-editor/react'],
          vendor: ['react', 'react-dom']
        }
      }
    }
  }
})
```

## Support

- **Issues**: [GitHub Issues](https://github.com/MarkXAI/AIT42-Editor/issues)
- **Documentation**: [README.md](README.md)
- **Tauri Docs**: https://tauri.app/v1/guides/

---

**Last Updated**: 2025-11-09
**Maintainer**: AIT42 Team
