# AIT42 Editor

A modern, fast TUI (Terminal User Interface) code editor with integrated AI agents for development automation.

[![CI Status](https://github.com/RenTonoduka/AIT42/workflows/CI/badge.svg)](https://github.com/RenTonoduka/AIT42/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust Version](https://img.shields.io/badge/rust-1.75%2B-blue.svg)](https://www.rust-lang.org)

## Features

- **Fast TUI Interface**: Built with Ratatui for responsive terminal UI
- **AI Agent Integration**: 49 specialized AI agents for development tasks
- **LSP Support**: Language Server Protocol for intelligent code completion
- **Tmux Integration**: Seamless tmux session management for agent execution
- **File System Operations**: Efficient file browsing and editing
- **Modern Architecture**: Clean Rust codebase with modular design

## Quick Start

### Installation

#### macOS (Recommended)

For macOS users, we provide a streamlined app bundle experience:

```bash
# Clone the repository
git clone https://github.com/RenTonoduka/AIT42
cd AIT42-Editor

# Run automated installer
./install-macos.sh
```

This will:
- Build the release binary
- Install AIT42.app to /Applications
- Optionally add to PATH

You can then launch by:
- Double-clicking AIT42.app in Applications
- Running `open /Applications/AIT42.app`
- Right-click any file → Open With → AIT42

See [INSTALL_MACOS.md](INSTALL_MACOS.md) for detailed macOS installation instructions.

#### Manual Installation (All Platforms)

```bash
# Clone the repository
git clone https://github.com/RenTonoduka/AIT42
cd AIT42-Editor

# Build release binary
cargo build --release

# Run the editor
./target/release/ait42 [file_or_directory]
```

### Usage

```bash
# Open a specific file
ait42 src/main.rs

# Open current directory (auto-selects README.md, Cargo.toml, etc.)
ait42 .

# Open with options
ait42 --debug --log-file ait42.log src/main.rs
```

## Documentation

- [INSTALL_MACOS.md](INSTALL_MACOS.md) - macOS installation guide
- [PROJECT_STRUCTURE.md](PROJECT_STRUCTURE.md) - Project architecture
- [AGENTS.md](AGENTS.md) - AI agents documentation (if available)

---

Made with ❤️ by the AIT42 Team
