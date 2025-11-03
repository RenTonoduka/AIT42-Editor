# AIT42 Editor - macOS Installation Guide

## Overview

AIT42 Editor is a modern TUI (Text User Interface) code editor with integrated AI agents. This guide will help you install and run it on macOS.

## Prerequisites

- macOS 10.13 (High Sierra) or later
- Terminal.app (included with macOS)
- Rust toolchain (for building from source)

## Installation Methods

### Method 1: Using the macOS App Bundle (Recommended)

The easiest way to run AIT42 Editor on macOS is using the pre-configured app bundle.

#### Step 1: Build the Release Binary

```bash
cd /Users/tonodukaren/Programming/AI/02_Workspace/05_Client/03_Sun/AIT42-Editor
cargo build --release
```

The binary will be created at: `target/release/ait42`

#### Step 2: Install to Applications Folder

```bash
# Copy the app bundle to Applications
cp -R AIT42.app /Applications/

# Make sure the launcher script is executable
chmod +x /Applications/AIT42.app/Contents/MacOS/AIT42
```

#### Step 3: Launch the App

You can now launch AIT42 Editor in three ways:

1. **Double-click the app**: Navigate to `/Applications/` in Finder and double-click `AIT42.app`
2. **Open command**: `open /Applications/AIT42.app`
3. **Open with file**: Right-click any file → "Open With" → AIT42

### Method 2: Command Line Only

If you prefer to use AIT42 Editor directly from the command line without the app bundle:

```bash
# Build release binary
cargo build --release

# Add to PATH (add to ~/.zshrc or ~/.bash_profile)
export PATH="/Users/tonodukaren/Programming/AI/02_Workspace/05_Client/03_Sun/AIT42-Editor/target/release:$PATH"

# Run the editor
ait42 [file_or_directory]
```

## Usage

### Opening Files

```bash
# Open a specific file
ait42 src/main.rs

# Open current directory (will auto-select README.md, Cargo.toml, etc.)
ait42 .

# Open project directory (will auto-find a file to open)
ait42 /path/to/project
```

### Auto File Selection

When you open a directory, AIT42 will automatically look for and open the first file it finds from this list:

1. README.md
2. Cargo.toml
3. package.json
4. main.rs
5. src/main.rs
6. src/lib.rs

### Command Line Options

```bash
# Show version
ait42 --version

# Show help
ait42 --help

# Enable debug logging
ait42 --debug src/main.rs

# Enable verbose logging
ait42 --verbose src/main.rs

# Specify config file
ait42 --config ~/.ait42/config.toml src/main.rs

# Specify log file
ait42 --log-file ~/.ait42/logs/ait42.log src/main.rs
```

## How It Works

### macOS App Bundle Architecture

```
AIT42.app/
├── Contents/
│   ├── Info.plist          # App metadata
│   ├── MacOS/
│   │   └── AIT42          # Launcher script
│   └── Resources/          # (Future: app icons)
```

When you launch `AIT42.app`:

1. macOS executes the launcher script: `Contents/MacOS/AIT42`
2. The script opens Terminal.app using AppleScript
3. Terminal runs the actual binary: `target/release/ait42`
4. The TUI editor launches in the terminal window

### Why Terminal.app?

AIT42 is a **TUI (Text User Interface)** application, not a traditional GUI app. It requires a terminal environment to run. The app bundle provides a seamless way to launch it by automatically opening Terminal.app.

## Troubleshooting

### Issue: "Binary not found" alert

**Solution**: Make sure you've built the release binary:
```bash
cd /Users/tonodukaren/Programming/AI/02_Workspace/05_Client/03_Sun/AIT42-Editor
cargo build --release
```

### Issue: "Permission denied" when launching

**Solution**: Make the launcher script executable:
```bash
chmod +x /Applications/AIT42.app/Contents/MacOS/AIT42
```

### Issue: "Is a directory" error

**Solution**: This has been fixed in the latest version. Update your installation:
```bash
git pull
cargo build --release
```

### Issue: Terminal opens but editor crashes

**Possible causes**:
1. Target file doesn't exist
2. No suitable file found in directory

**Solution**: Specify a valid file path or ensure your project directory contains at least one of the auto-detected files (README.md, Cargo.toml, etc.)

## Uninstallation

To remove AIT42 Editor:

```bash
# Remove app bundle
rm -rf /Applications/AIT42.app

# Remove configuration (optional)
rm -rf ~/.ait42

# Remove from PATH (if using command line method)
# Edit ~/.zshrc or ~/.bash_profile and remove the export PATH line
```

## Development Mode

If you're developing AIT42 Editor, you can run it directly without installing:

```bash
# Run with cargo
cargo run -- src/main.rs

# Run release build directly
./target/release/ait42 src/main.rs

# Run with debug logging
RUST_LOG=debug cargo run -- --debug src/main.rs
```

## Features

- **TUI-based code editing** with syntax highlighting
- **LSP integration** for intelligent code completion
- **49 AI agents** for development automation
- **Tmux session management** for parallel agent execution
- **File system operations** with real-time updates
- **Git integration** (planned)
- **Multiple language support** (planned)

## Configuration

Create a configuration file at `~/.ait42/config.toml`:

```toml
[editor]
theme = "default"
line_numbers = true
auto_save = false

[lsp]
enabled = true
rust_analyzer_path = "rust-analyzer"

[agents]
tmux_enabled = true
parallel_execution = true
```

## Next Steps

1. **Learn the keybindings**: Press `?` in the editor for help
2. **Configure your environment**: Edit `~/.ait42/config.toml`
3. **Try AI agents**: Use tmux commands to launch parallel agents
4. **Read the docs**: Check the main README.md for more details

## Support

- **Issues**: https://github.com/yourusername/AIT42-Editor/issues
- **Documentation**: See README.md in the project root
- **Contributing**: See CONTRIBUTING.md

## License

See LICENSE file in the project root.
