# AIT42 Editor - Cargo Workspace Structure

**Version**: 1.0.0
**Date**: 2025-01-06
**Status**: Design Phase

---

## Table of Contents

1. [Workspace Overview](#workspace-overview)
2. [Workspace Root Configuration](#workspace-root-configuration)
3. [Crate Specifications](#crate-specifications)
4. [Dependency Management](#dependency-management)
5. [Build Configuration](#build-configuration)
6. [Development Workflow](#development-workflow)

---

## Workspace Overview

### Directory Structure

```
ait42-editor/                      # Workspace root
├── Cargo.toml                     # Workspace manifest
├── Cargo.lock                     # Locked dependencies
├── .cargo/
│   └── config.toml                # Cargo configuration
│
├── crates/                        # All library crates
│   ├── ait42-core/                # Core editor logic
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── buffer/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── text_buffer.rs
│   │   │   │   ├── buffer_manager.rs
│   │   │   │   └── undo_tree.rs
│   │   │   ├── cursor/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── cursor.rs
│   │   │   │   └── cursor_set.rs
│   │   │   ├── mode/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── normal.rs
│   │   │   │   ├── insert.rs
│   │   │   │   ├── visual.rs
│   │   │   │   └── command.rs
│   │   │   ├── command/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── trait.rs
│   │   │   │   └── builtin.rs
│   │   │   └── state.rs
│   │   └── tests/
│   │
│   ├── ait42-tui/                 # TUI rendering layer
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── widgets/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── editor.rs
│   │   │   │   ├── status_bar.rs
│   │   │   │   ├── command_palette.rs
│   │   │   │   ├── file_tree.rs
│   │   │   │   └── tmux_panel.rs
│   │   │   ├── layout/
│   │   │   │   ├── mod.rs
│   │   │   │   └── manager.rs
│   │   │   ├── theme/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── color_scheme.rs
│   │   │   │   └── builtin_themes.rs
│   │   │   └── input/
│   │   │       ├── mod.rs
│   │   │       └── handler.rs
│   │   └── tests/
│   │
│   ├── ait42-lsp/                 # LSP client integration
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── client/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── lsp_client.rs
│   │   │   │   └── server_manager.rs
│   │   │   ├── handlers/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── completion.rs
│   │   │   │   ├── hover.rs
│   │   │   │   ├── goto_definition.rs
│   │   │   │   └── diagnostics.rs
│   │   │   └── facade.rs
│   │   └── tests/
│   │
│   ├── ait42-ait42/               # AIT42 agent integration
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── agent/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── loader.rs
│   │   │   │   ├── registry.rs
│   │   │   │   └── metadata.rs
│   │   │   ├── coordinator/
│   │   │   │   ├── mod.rs
│   │   │   │   └── client.rs
│   │   │   ├── tmux/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── session_manager.rs
│   │   │   │   └── monitor.rs
│   │   │   └── ui/
│   │   │       ├── mod.rs
│   │   │       ├── agent_palette.rs
│   │   │       └── status_panel.rs
│   │   └── tests/
│   │
│   ├── ait42-fs/                  # File system operations
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── watcher/
│   │   │   │   ├── mod.rs
│   │   │   │   └── file_watcher.rs
│   │   │   ├── explorer/
│   │   │   │   ├── mod.rs
│   │   │   │   └── file_tree.rs
│   │   │   └── search/
│   │   │       ├── mod.rs
│   │   │       └── fuzzy.rs
│   │   └── tests/
│   │
│   └── ait42-config/              # Configuration management
│       ├── Cargo.toml
│       ├── src/
│       │   ├── lib.rs
│       │   ├── parser/
│       │   │   ├── mod.rs
│       │   │   └── toml_parser.rs
│       │   ├── schema/
│       │   │   ├── mod.rs
│       │   │   ├── editor.rs
│       │   │   ├── keybindings.rs
│       │   │   ├── ait42.rs
│       │   │   └── lsp.rs
│       │   └── defaults.rs
│       └── tests/
│
├── ait42-bin/                     # Main binary crate
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs
│   │   ├── app.rs
│   │   └── event_loop.rs
│   └── build.rs                   # Build script (if needed)
│
├── tests/                         # Integration tests
│   ├── integration/
│   │   ├── buffer_tests.rs
│   │   ├── lsp_tests.rs
│   │   └── agent_tests.rs
│   └── e2e/
│       ├── editor_workflow.rs
│       └── agent_execution.rs
│
├── benches/                       # Benchmarks
│   ├── buffer_performance.rs
│   ├── rendering_performance.rs
│   └── lsp_performance.rs
│
├── docs/                          # Documentation
│   ├── ARCHITECTURE.md
│   ├── COMPONENT_DESIGN.md
│   └── CARGO_WORKSPACE.md         # This file
│
├── .github/
│   └── workflows/
│       ├── ci.yml
│       └── release.yml
│
├── .gitignore
├── README.md
├── LICENSE
└── CHANGELOG.md
```

---

## Workspace Root Configuration

### `Cargo.toml` (Workspace Root)

```toml
[workspace]
resolver = "2"
members = [
    "crates/ait42-core",
    "crates/ait42-tui",
    "crates/ait42-lsp",
    "crates/ait42-ait42",
    "crates/ait42-fs",
    "crates/ait42-config",
    "ait42-bin",
]

# Shared workspace dependencies
[workspace.dependencies]
# Async runtime
tokio = { version = "1.35", features = ["full"] }

# Error handling
anyhow = "1.0"
thiserror = "1.0"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"
toml = "0.8"

# Text processing
ropey = "1.6"
unicode-segmentation = "1.10"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# TUI
ratatui = "0.25"
crossterm = "0.27"

# LSP
tower-lsp = "0.20"
lsp-types = "0.95"

# File system
notify = "6.1"
ignore = "0.4"

# Syntax highlighting
tree-sitter = "0.20"
tree-sitter-rust = "0.20"
tree-sitter-typescript = "0.20"
tree-sitter-python = "0.20"

# Utilities
once_cell = "1.19"
parking_lot = "0.12"
dashmap = "5.5"

# Testing
mockall = "0.12"

[workspace.package]
version = "1.0.0"
edition = "2021"
rust-version = "1.75"
authors = ["AIT42 Team"]
license = "MIT"
repository = "https://github.com/your-org/ait42-editor"
keywords = ["editor", "tui", "ai", "agents"]
categories = ["command-line-utilities", "text-editors"]

# Workspace-wide profile settings
[profile.dev]
opt-level = 0
debug = true
split-debuginfo = "unpacked"
strip = "none"
debug-assertions = true
overflow-checks = true
lto = false
panic = "unwind"
incremental = true
codegen-units = 256
rpath = false

[profile.release]
opt-level = 3
debug = false
split-debuginfo = "off"
strip = "symbols"
debug-assertions = false
overflow-checks = false
lto = "thin"
panic = "abort"
incremental = false
codegen-units = 1
rpath = false

[profile.release.package."*"]
opt-level = 3
codegen-units = 1

# Benchmark profile (similar to release but with debug info)
[profile.bench]
inherits = "release"
debug = true
strip = "none"

# Profile for faster development builds
[profile.dev-fast]
inherits = "dev"
opt-level = 1
incremental = true
```

---

## Crate Specifications

### 1. `ait42-core` (Core Editor Logic)

**`crates/ait42-core/Cargo.toml`**:

```toml
[package]
name = "ait42-core"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
authors.workspace = true
license.workspace = true

[dependencies]
# Workspace dependencies
tokio = { workspace = true }
anyhow = { workspace = true }
thiserror = { workspace = true }
serde = { workspace = true }
ropey = { workspace = true }
unicode-segmentation = { workspace = true }
tracing = { workspace = true }
once_cell = { workspace = true }
parking_lot = { workspace = true }

# Tree-sitter for syntax highlighting
tree-sitter = { workspace = true }
tree-sitter-rust = { workspace = true }
tree-sitter-typescript = { workspace = true }
tree-sitter-python = { workspace = true }

[dev-dependencies]
mockall = { workspace = true }
tokio-test = "0.4"

[features]
default = []
# Optional features
multi-cursor = []  # Phase 2
macro-recording = []  # Phase 2
```

**Key Exports** (`src/lib.rs`):

```rust
pub mod buffer;
pub mod cursor;
pub mod mode;
pub mod command;
pub mod state;

pub use buffer::{TextBuffer, BufferManager, BufferId};
pub use cursor::{Cursor, CursorSet};
pub use mode::{Mode, ModeTransition, NormalMode, InsertMode, VisualMode, CommandMode};
pub use command::Command;
pub use state::EditorContext;

// Re-exports for convenience
pub use ropey::{Rope, RopeSlice};
pub use tree_sitter::{Language, Tree, Parser};
```

---

### 2. `ait42-tui` (TUI Rendering Layer)

**`crates/ait42-tui/Cargo.toml`**:

```toml
[package]
name = "ait42-tui"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
authors.workspace = true
license.workspace = true

[dependencies]
# Workspace dependencies
tokio = { workspace = true }
anyhow = { workspace = true }
thiserror = { workspace = true }
ratatui = { workspace = true }
crossterm = { workspace = true }
tracing = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }

# Internal dependencies
ait42-core = { path = "../ait42-core" }

[dev-dependencies]
mockall = { workspace = true }

[features]
default = ["enhanced-graphics"]
enhanced-graphics = []  # Use Unicode box drawing characters
```

**Key Exports** (`src/lib.rs`):

```rust
pub mod widgets;
pub mod layout;
pub mod theme;
pub mod input;

pub use widgets::{EditorWidget, StatusBar, CommandPalette, FileTree, TmuxPanel};
pub use layout::LayoutManager;
pub use theme::{ColorScheme, ThemeManager};
pub use input::InputHandler;

// Re-exports
pub use ratatui;
pub use crossterm;
```

---

### 3. `ait42-lsp` (LSP Client Integration)

**`crates/ait42-lsp/Cargo.toml`**:

```toml
[package]
name = "ait42-lsp"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
authors.workspace = true
license.workspace = true

[dependencies]
# Workspace dependencies
tokio = { workspace = true }
anyhow = { workspace = true }
thiserror = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
tracing = { workspace = true }
dashmap = { workspace = true }

# LSP specific
tower-lsp = { workspace = true }
lsp-types = { workspace = true }

# Internal dependencies
ait42-core = { path = "../ait42-core" }

[dev-dependencies]
mockall = { workspace = true }
tokio-test = "0.4"

[features]
default = []
# Optional features
semantic-tokens = []  # LSP semantic tokens support (Phase 2)
```

**Key Exports** (`src/lib.rs`):

```rust
pub mod client;
pub mod handlers;
pub mod facade;

pub use client::{LspClient, ServerManager};
pub use handlers::{CompletionHandler, HoverHandler, GotoDefinitionHandler, DiagnosticsHandler};
pub use facade::LspFacade;

// Re-exports
pub use lsp_types::{CompletionItem, Hover, Location, Diagnostic, DiagnosticSeverity};
```

---

### 4. `ait42-ait42` (AIT42 Agent Integration)

**`crates/ait42-ait42/Cargo.toml`**:

```toml
[package]
name = "ait42-ait42"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
authors.workspace = true
license.workspace = true

[dependencies]
# Workspace dependencies
tokio = { workspace = true }
anyhow = { workspace = true }
thiserror = { workspace = true }
serde = { workspace = true }
serde_yaml = { workspace = true }
tracing = { workspace = true }
dashmap = { workspace = true }

# Internal dependencies
ait42-core = { path = "../ait42-core" }

# External process management
which = "6.0"  # Find tmux binary

[dev-dependencies]
mockall = { workspace = true }
tokio-test = "0.4"

[features]
default = []
# Optional features
coordinator-integration = []  # Phase 2: Deep Coordinator integration
```

**Key Exports** (`src/lib.rs`):

```rust
pub mod agent;
pub mod coordinator;
pub mod tmux;
pub mod ui;

pub use agent::{AgentLoader, AgentRegistry, AgentMetadata};
pub use coordinator::CoordinatorClient;
pub use tmux::{TmuxSessionManager, TmuxSession, SessionStatus};
pub use ui::{AgentPalette, StatusPanel};
```

---

### 5. `ait42-fs` (File System Operations)

**`crates/ait42-fs/Cargo.toml`**:

```toml
[package]
name = "ait42-fs"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
authors.workspace = true
license.workspace = true

[dependencies]
# Workspace dependencies
tokio = { workspace = true, features = ["fs"] }
anyhow = { workspace = true }
thiserror = { workspace = true }
tracing = { workspace = true }

# File system specific
notify = { workspace = true }
ignore = { workspace = true }  # Gitignore support

# Fuzzy search
fuzzy-matcher = "0.3"

# Internal dependencies
ait42-core = { path = "../ait42-core" }

[dev-dependencies]
tempfile = "3.9"

[features]
default = []
# Optional features
ripgrep-integration = []  # Phase 2: Content search
```

**Key Exports** (`src/lib.rs`):

```rust
pub mod watcher;
pub mod explorer;
pub mod search;

pub use watcher::{FileWatcher, WatchEvent};
pub use explorer::{FileTree, FileNode};
pub use search::{FuzzySearch, SearchResult};
```

---

### 6. `ait42-config` (Configuration Management)

**`crates/ait42-config/Cargo.toml`**:

```toml
[package]
name = "ait42-config"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
authors.workspace = true
license.workspace = true

[dependencies]
# Workspace dependencies
anyhow = { workspace = true }
thiserror = { workspace = true }
serde = { workspace = true }
toml = { workspace = true }
tracing = { workspace = true }

# Platform-specific paths
directories = "5.0"

[dev-dependencies]
tempfile = "3.9"

[features]
default = []
```

**Key Exports** (`src/lib.rs`):

```rust
pub mod parser;
pub mod schema;
pub mod defaults;

pub use parser::ConfigParser;
pub use schema::{Config, EditorConfig, KeybindingConfig, AIT42Config, LspConfig};
pub use defaults::default_config;
```

---

### 7. `ait42-bin` (Main Binary)

**`ait42-bin/Cargo.toml`**:

```toml
[package]
name = "ait42-editor"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
authors.workspace = true
license.workspace = true

[[bin]]
name = "ait42"
path = "src/main.rs"

[dependencies]
# Workspace dependencies
tokio = { workspace = true }
anyhow = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }

# CLI argument parsing
clap = { version = "4.4", features = ["derive", "cargo"] }

# Internal dependencies
ait42-core = { path = "../crates/ait42-core" }
ait42-tui = { path = "../crates/ait42-tui" }
ait42-lsp = { path = "../crates/ait42-lsp" }
ait42-ait42 = { path = "../crates/ait42-ait42" }
ait42-fs = { path = "../crates/ait42-fs" }
ait42-config = { path = "../crates/ait42-config" }

[features]
default = []
# Optional features
telemetry = []  # Phase 2: Optional telemetry
```

**Key Files**:

- `src/main.rs` - Entry point, CLI parsing
- `src/app.rs` - Main application struct
- `src/event_loop.rs` - Main event loop

---

## Dependency Management

### Dependency Categories

#### 1. Core Dependencies (Required for MVP)

```toml
tokio = "1.35"           # Async runtime
ropey = "1.6"            # Text buffer
ratatui = "0.25"         # TUI framework
crossterm = "0.27"       # Terminal handling
tower-lsp = "0.20"       # LSP client
tree-sitter = "0.20"     # Syntax parsing
```

#### 2. Optional Dependencies (Phase 2)

```toml
libgit2-sys = "0.16"     # Git integration (Phase 2)
regex = "1.10"           # Advanced search (Phase 2)
lua = "0.54"             # Lua scripting (Phase 2)
```

#### 3. Development Dependencies

```toml
mockall = "0.12"         # Mocking for tests
tokio-test = "0.4"       # Async test utilities
criterion = "0.5"        # Benchmarking
proptest = "1.4"         # Property-based testing
```

---

### Version Pinning Strategy

**Semantic Versioning Rules**:

1. **Major version 0.x**: Always pin exact version (e.g., `"0.25"`)
   - Breaking changes common
   - Examples: `ratatui = "0.25"`

2. **Major version 1.x+**: Pin minor version (e.g., `"1.35"`)
   - Allows patch updates
   - Examples: `tokio = "1.35"`

3. **Internal crates**: Use `path` dependencies
   - No version pinning needed
   - Example: `ait42-core = { path = "../ait42-core" }`

---

### Cargo.lock Management

**Commit `Cargo.lock`**: ✅ YES

**Reason**: Binary project - reproducible builds critical

**Update Strategy**:
```bash
# Regular dependency updates (weekly)
cargo update

# Specific package update
cargo update -p tokio

# Conservative update (respect Cargo.toml constraints)
cargo update --conservative
```

---

## Build Configuration

### `.cargo/config.toml`

```toml
[build]
# Use all available CPU cores
jobs = -1

[target.x86_64-apple-darwin]
# macOS-specific optimizations
rustflags = [
    "-C", "target-cpu=native",
    "-C", "link-arg=-fuse-ld=lld",  # Faster linking (if lld installed)
]

[target.aarch64-apple-darwin]
# Apple Silicon optimizations
rustflags = [
    "-C", "target-cpu=native",
]

[alias]
# Convenient aliases
r = "run --release"
b = "build --release"
t = "test --workspace"
c = "clippy --workspace -- -D warnings"
doc-open = "doc --open --no-deps"

# Development aliases
dev = "run --features dev-fast"
bench-all = "bench --workspace"

[profile.dev]
# Enable some optimizations for dependencies in dev mode
[profile.dev.package."*"]
opt-level = 2

[net]
# Faster cargo operations
git-fetch-with-cli = true
```

---

### Build Scripts

#### `ait42-bin/build.rs` (Optional)

```rust
// Build script for embedding version info

use std::process::Command;

fn main() {
    // Embed git commit hash
    let output = Command::new("git")
        .args(&["rev-parse", "--short", "HEAD"])
        .output()
        .ok();

    if let Some(output) = output {
        if output.status.success() {
            let hash = String::from_utf8_lossy(&output.stdout);
            println!("cargo:rustc-env=GIT_HASH={}", hash.trim());
        }
    }

    // Embed build timestamp
    let timestamp = chrono::Utc::now().to_rfc3339();
    println!("cargo:rustc-env=BUILD_TIMESTAMP={}", timestamp);

    // Re-run if git HEAD changes
    println!("cargo:rerun-if-changed=../.git/HEAD");
}
```

---

## Development Workflow

### Common Commands

#### Build

```bash
# Debug build (fast compilation)
cargo build

# Release build (optimized)
cargo build --release

# Specific crate
cargo build -p ait42-core

# Check without building (faster)
cargo check --workspace
```

#### Run

```bash
# Run in debug mode
cargo run

# Run in release mode
cargo run --release

# Run with specific file
cargo run -- /path/to/file.rs

# Run with log level
RUST_LOG=debug cargo run
```

#### Test

```bash
# Run all tests
cargo test --workspace

# Run specific crate tests
cargo test -p ait42-core

# Run specific test
cargo test -p ait42-core buffer_insert

# Run with output
cargo test -- --nocapture

# Run integration tests only
cargo test --test integration_tests
```

#### Lint & Format

```bash
# Run clippy
cargo clippy --workspace -- -D warnings

# Format code
cargo fmt --all

# Check formatting without modifying
cargo fmt --all -- --check
```

#### Benchmark

```bash
# Run all benchmarks
cargo bench --workspace

# Run specific benchmark
cargo bench -p ait42-core buffer_performance

# Generate benchmark report
cargo bench -- --save-baseline main
```

#### Documentation

```bash
# Generate and open docs
cargo doc --open --no-deps

# Generate docs for specific crate
cargo doc -p ait42-core --open

# Include private items
cargo doc --document-private-items
```

---

### CI/CD Pipeline

#### `.github/workflows/ci.yml`

```yaml
name: CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main, develop]

env:
  RUST_BACKTRACE: 1
  CARGO_TERM_COLOR: always

jobs:
  test:
    name: Test
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Cache cargo
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Install tmux
        run: brew install tmux

      - name: Run tests
        run: cargo test --workspace --all-features

  lint:
    name: Lint
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy

      - name: Check formatting
        run: cargo fmt --all -- --check

      - name: Run clippy
        run: cargo clippy --workspace --all-features -- -D warnings

  benchmark:
    name: Benchmark
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Run benchmarks
        run: cargo bench --workspace --no-fail-fast

  build:
    name: Build Release
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Build release binary
        run: cargo build --release

      - name: Upload artifact
        uses: actions/upload-artifact@v3
        with:
          name: ait42-editor-macos
          path: target/release/ait42
```

---

### Release Process

#### Semantic Versioning

```
MAJOR.MINOR.PATCH
  1  .  0  .  0

MAJOR: Breaking changes (API incompatible)
MINOR: New features (backward compatible)
PATCH: Bug fixes (backward compatible)
```

#### Release Checklist

1. **Update version** in `Cargo.toml` (workspace root)
2. **Update `CHANGELOG.md`** with new version
3. **Run full test suite**: `cargo test --workspace`
4. **Run benchmarks**: `cargo bench --workspace`
5. **Build release binary**: `cargo build --release`
6. **Create git tag**: `git tag v1.0.0`
7. **Push tag**: `git push origin v1.0.0`
8. **GitHub Actions** will build and create release

---

### Performance Optimization Tips

#### 1. Link-Time Optimization (LTO)

Already enabled in `Cargo.toml`:
```toml
[profile.release]
lto = "thin"  # Balance between speed and build time
```

#### 2. Codegen Units

Already optimized:
```toml
[profile.release]
codegen-units = 1  # Maximum optimization
```

#### 3. Target CPU

Use `.cargo/config.toml`:
```toml
rustflags = ["-C", "target-cpu=native"]
```

#### 4. Profile-Guided Optimization (PGO)

Future optimization (Phase 2):
```bash
# Step 1: Build with instrumentation
RUSTFLAGS="-C profile-generate=/tmp/pgo-data" cargo build --release

# Step 2: Run typical workload
target/release/ait42 <typical-usage>

# Step 3: Build with profile data
RUSTFLAGS="-C profile-use=/tmp/pgo-data/merged.profdata" cargo build --release
```

---

## Troubleshooting

### Common Build Issues

#### 1. Missing `tmux`

**Error**: Tests fail with "tmux: command not found"

**Solution**:
```bash
brew install tmux
```

#### 2. Linker Errors

**Error**: `ld: symbol(s) not found`

**Solution**: Update Xcode Command Line Tools
```bash
xcode-select --install
```

#### 3. Slow Compilation

**Solution**: Enable parallel compilation
```bash
# .cargo/config.toml
[build]
jobs = -1  # Use all CPU cores
```

#### 4. Out of Memory

**Solution**: Reduce codegen units in debug mode
```toml
[profile.dev]
codegen-units = 256  # Default, reduce if OOM
```

---

## Next Steps

1. **Create repository structure**:
   ```bash
   mkdir -p ait42-editor/{crates,ait42-bin,tests,benches,docs}
   ```

2. **Initialize workspace**:
   ```bash
   cd ait42-editor
   cargo init --lib crates/ait42-core
   cargo init --lib crates/ait42-tui
   cargo init --lib crates/ait42-lsp
   cargo init --lib crates/ait42-ait42
   cargo init --lib crates/ait42-fs
   cargo init --lib crates/ait42-config
   cargo init --bin ait42-bin
   ```

3. **Copy workspace `Cargo.toml`** from this document

4. **Start implementation** with `ait42-core` (Week 3)

---

**End of Cargo Workspace Document**

Generated by: system-architect agent
Date: 2025-01-06
Version: 1.0.0
