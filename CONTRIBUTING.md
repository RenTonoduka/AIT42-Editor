# Contributing to AIT42 Editor

Thank you for your interest in contributing to AIT42 Editor!

## Development Setup

1. Install Rust 1.75 or later:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. Clone the repository:
   ```bash
   git clone https://github.com/RenTonoduka/AIT42
   cd AIT42-Editor
   ```

3. Run setup script:
   ```bash
   ./scripts/setup.sh
   ```

## Development Workflow

### Running Tests

```bash
# All tests
cargo test

# Specific crate
cargo test -p ait42-core

# With output
cargo test -- --nocapture
```

### Code Quality

```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Fix lint issues
cargo clippy --fix
```

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release
```

## Project Structure

```
AIT42-Editor/
├── ait42-bin/          # Main binary
├── crates/
│   ├── ait42-core/     # Core editor logic
│   ├── ait42-tui/      # TUI rendering
│   ├── ait42-lsp/      # LSP client
│   ├── ait42-ait42/    # AIT42 integration
│   ├── ait42-fs/       # File system
│   └── ait42-config/   # Configuration
├── tests/              # Integration tests
└── benches/            # Benchmarks
```

## Coding Standards

- Follow Rust standard style (enforced by `rustfmt`)
- Write tests for new features
- Update documentation for public APIs
- Keep commits atomic and well-described
- Ensure CI passes before submitting PR

## Pull Request Process

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests and linter
5. Submit PR with clear description
6. Address review feedback

## Questions?

Open an issue or reach out to the maintainers.
