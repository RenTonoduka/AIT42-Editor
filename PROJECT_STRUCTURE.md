# AIT42 Editor - Project Structure Documentation

## Overview

This document describes the complete project structure of the AIT42 Editor Rust workspace.

## Directory Tree

```
AIT42-Editor/
├── Cargo.toml                 # Workspace root configuration
├── Cargo.lock                 # Dependency lock file
├── rust-toolchain.toml        # Rust version pinning
├── .gitignore                 # Git ignore rules
├── .rustfmt.toml             # Code formatting configuration
├── .clippy.toml              # Linter configuration
├── deny.toml                 # Dependency audit configuration
│
├── README.md                 # Project README
├── LICENSE                   # MIT License
├── CONTRIBUTING.md           # Contribution guidelines
├── CHANGELOG.md              # Version history
├── PROJECT_STRUCTURE.md      # This file
│
├── ait42-bin/                # Main binary crate
│   ├── Cargo.toml
│   └── src/
│       └── main.rs           # Application entry point
│
├── crates/                   # Library crates
│   ├── ait42-core/           # Core editor logic
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs        # Module root
│   │       ├── buffer.rs     # Buffer management (Rope-based)
│   │       ├── cursor.rs     # Cursor handling
│   │       ├── selection.rs  # Text selection
│   │       ├── history.rs    # Undo/redo history
│   │       └── editor.rs     # Main editor state
│   │
│   ├── ait42-tui/            # TUI rendering
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs        # TUI entry point
│   │       ├── app.rs        # Application state
│   │       └── ui/           # UI components
│   │           ├── editor.rs
│   │           ├── file_tree.rs
│   │           └── status_bar.rs
│   │
│   ├── ait42-lsp/            # LSP client
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs        # LSP manager
│   │
│   ├── ait42-ait42/          # AIT42 integration
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs        # Agent & tmux manager
│   │
│   ├── ait42-fs/             # File system
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs        # File operations
│   │
│   └── ait42-config/         # Configuration
│       ├── Cargo.toml
│       └── src/
│           └── lib.rs        # Config management
│
├── tests/                    # Integration tests
│   └── integration_tests.rs
│
├── benches/                  # Benchmarks
│   └── buffer_bench.rs
│
├── scripts/                  # Build & dev scripts
│   ├── setup.sh             # Development setup
│   ├── build.sh             # Build script
│   ├── test.sh              # Test runner
│   └── release.sh           # Release build
│
└── .github/
    └── workflows/
        ├── ci.yml           # CI workflow
        └── release.yml      # Release workflow
```

## Crate Dependency Graph

```
ait42-bin
  ├─> ait42-core
  │     ├─> ait42-config
  │     ├─> ait42-fs
  │     └─> ait42-lsp
  ├─> ait42-tui
  │     ├─> ait42-core
  │     └─> ait42-config
  ├─> ait42-config
  └─> ait42-ait42
```

## File Descriptions

### Root Configuration Files

| File | Purpose |
|------|---------|
| `Cargo.toml` | Workspace configuration, shared dependencies |
| `rust-toolchain.toml` | Pins Rust version to 1.75 |
| `.gitignore` | Git ignore patterns |
| `.rustfmt.toml` | Code formatting rules (rustfmt) |
| `.clippy.toml` | Linter configuration (clippy) |
| `deny.toml` | Dependency security/license auditing |

### Documentation Files

| File | Purpose |
|------|---------|
| `README.md` | Project overview and quick start |
| `CONTRIBUTING.md` | Development guidelines |
| `CHANGELOG.md` | Version history and changes |
| `PROJECT_STRUCTURE.md` | This file - project structure |
| `LICENSE` | MIT License text |

### Binary Crate (ait42-bin)

**Purpose**: Main application entry point

**Files**:
- `src/main.rs`: CLI argument parsing, logging setup, TUI initialization

**Dependencies**:
- Internal: All library crates
- External: tokio, clap, tracing

### Core Crate (ait42-core)

**Purpose**: Core editor logic and state management

**Modules**:
- `buffer.rs`: Text buffer using Rope data structure
- `cursor.rs`: Cursor position and movement
- `selection.rs`: Text selection ranges
- `history.rs`: Undo/redo stack
- `editor.rs`: Main editor state coordinator

**Key Types**:
- `Buffer`: Rope-based text storage
- `BufferManager`: Multi-buffer management
- `Cursor`: Cursor state
- `History`: Edit history for undo/redo
- `Editor`: Main editor state

**Dependencies**:
- `ropey`: Efficient text buffer
- `uuid`: Buffer identification

### TUI Crate (ait42-tui)

**Purpose**: Terminal user interface rendering

**Modules**:
- `lib.rs`: TUI initialization and main loop
- `app.rs`: Application state
- `ui/`: UI component modules

**Key Types**:
- `run()`: Main TUI entry point
- `App`: Application state

**Dependencies**:
- `ratatui`: TUI framework
- `crossterm`: Terminal manipulation

### LSP Crate (ait42-lsp)

**Purpose**: Language Server Protocol client

**Key Types**:
- `LspManager`: LSP client coordinator
- `ServerInfo`: LSP server state

**Dependencies**:
- `tower-lsp`: LSP framework
- `lsp-types`: LSP type definitions

### AIT42 Integration Crate (ait42-ait42)

**Purpose**: AI agent and tmux integration

**Key Types**:
- `AgentManager`: Agent discovery and execution
- `TmuxManager`: Tmux session management
- `Agent`: Agent metadata
- `TmuxSession`: Session state

**Dependencies**:
- `tokio::process`: Process spawning

### File System Crate (ait42-fs)

**Purpose**: Async file operations

**Key Types**:
- `FileNode`: File tree node
- `read_dir()`: Directory reading
- `read_file()`: File reading
- `write_file()`: File writing

**Dependencies**:
- `tokio::fs`: Async file operations
- `notify`: File watching
- `ignore`: Gitignore support

### Config Crate (ait42-config)

**Purpose**: Configuration management

**Key Types**:
- `Config`: Main configuration
- `EditorConfig`: Editor settings
- `UiConfig`: UI settings
- `Ait42Config`: AIT42 integration settings

**Dependencies**:
- `config`: Configuration loading
- `directories`: Config directory location
- `toml`: TOML parsing

## Build Profiles

| Profile | Optimization | Debug Info | Use Case |
|---------|-------------|------------|----------|
| `dev` | 0 | Full | Development |
| `release` | 3 | Stripped | Production |
| `test` | 1 | Full | Testing |
| `bench` | 3 | Partial | Benchmarking |

## Testing Structure

- **Unit Tests**: In each crate's source files
- **Integration Tests**: `tests/integration_tests.rs`
- **Benchmarks**: `benches/buffer_bench.rs`

## CI/CD Workflows

### CI Workflow (.github/workflows/ci.yml)

**Triggers**: Push to main/develop, Pull requests

**Jobs**:
1. **Check**: Format, clippy, cargo check
2. **Test**: Unit, integration, doc tests
3. **Build**: Debug and release builds
4. **Security**: cargo-deny audit

### Release Workflow (.github/workflows/release.yml)

**Triggers**: Git tags (v*.*.*)

**Jobs**:
1. Create GitHub release
2. Build for x86_64 and aarch64
3. Upload release artifacts

## Development Scripts

| Script | Purpose |
|--------|---------|
| `scripts/setup.sh` | Install tools, build project |
| `scripts/build.sh` | Build debug or release |
| `scripts/test.sh` | Run all tests with coverage |
| `scripts/release.sh` | Create release builds |

## External Dependencies

### Core Dependencies
- `tokio`: Async runtime
- `serde`: Serialization
- `anyhow`/`thiserror`: Error handling
- `tracing`: Logging

### TUI Dependencies
- `ratatui`: Terminal UI framework
- `crossterm`: Terminal manipulation
- `tui-textarea`: Text area widget

### Editor Dependencies
- `ropey`: Text buffer (Rope data structure)
- `unicode-width`: Unicode handling
- `unicode-segmentation`: Text segmentation

### LSP Dependencies
- `tower-lsp`: LSP framework
- `lsp-types`: LSP type definitions

### File System Dependencies
- `notify`: File watching
- `ignore`: Gitignore support
- `walkdir`: Directory traversal

## Configuration File Locations

| File | Location |
|------|----------|
| Config | `~/.config/ait42/config.toml` |
| Logs | `~/.local/share/ait42/logs/` |
| Cache | `~/.cache/ait42/` |

## Build Artifacts

| Path | Contents |
|------|----------|
| `target/debug/` | Debug builds |
| `target/release/` | Release builds |
| `target/doc/` | Generated documentation |
| `coverage/` | Test coverage reports |

## Next Steps

1. Initialize Git repository
2. Run `./scripts/setup.sh`
3. Build with `cargo build`
4. Run tests with `cargo test`
5. Generate docs with `cargo doc --open`

---

Last Updated: 2024-11-03
