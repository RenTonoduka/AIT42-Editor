# Contributing to AIT42 Editor

Thank you for your interest in contributing to AIT42 Editor!

This document provides guidelines for contributing to the project.

---

## Table of Contents

1. [Code of Conduct](#code-of-conduct)
2. [Getting Started](#getting-started)
3. [Development Workflow](#development-workflow)
4. [Coding Standards](#coding-standards)
5. [Testing Guidelines](#testing-guidelines)
6. [Pull Request Process](#pull-request-process)
7. [Issue Guidelines](#issue-guidelines)
8. [Documentation](#documentation)
9. [Community](#community)

---

## Code of Conduct

### Our Pledge

We are committed to providing a welcoming and inclusive environment for all contributors, regardless of:
- Experience level
- Gender identity
- Sexual orientation
- Disability
- Personal appearance
- Race or ethnicity
- Age
- Religion

### Our Standards

**Positive Behavior**:
- Using welcoming and inclusive language
- Being respectful of differing viewpoints
- Gracefully accepting constructive criticism
- Focusing on what is best for the community
- Showing empathy towards other community members

**Unacceptable Behavior**:
- Harassment or discriminatory language
- Trolling, insulting, or derogatory comments
- Personal or political attacks
- Publishing others' private information without permission
- Other conduct which could reasonably be considered inappropriate

### Enforcement

Instances of abusive, harassing, or otherwise unacceptable behavior may be reported to the project maintainers. All complaints will be reviewed and investigated promptly and fairly.

---

## Getting Started

### Prerequisites

- **Rust 1.75+**: Install from [rustup.rs](https://rustup.rs)
- **Git**: Version control
- **macOS 11.0+**: Currently macOS-only (Linux/Windows support planned)
- **Optional**:
  - Tmux 2.0+ for agent development
  - Language servers (rust-analyzer, etc.) for LSP features

### Installation

```bash
# Clone repository
git clone https://github.com/RenTonoduka/AIT42
cd AIT42-Editor

# Run setup script
./scripts/setup.sh

# Verify build
cargo check

# Run tests
cargo test

# Run editor
cargo run
```

### Development Tools

```bash
# Install recommended tools
cargo install cargo-watch    # Auto-rebuild on changes
cargo install cargo-tarpaulin # Code coverage
cargo install cargo-flamegraph # Profiling
cargo install cargo-audit    # Security audit
cargo install cargo-nextest  # Faster test runner
```

### IDE Setup

**VS Code** (Recommended):
```bash
# Install rust-analyzer extension
code --install-extension rust-lang.rust-analyzer

# Recommended extensions
code --install-extension vadimcn.vscode-lldb  # Debugging
code --install-extension tamasfe.even-better-toml
code --install-extension serayuzgur.crates
```

Create `.vscode/settings.json`:
```json
{
  "rust-analyzer.cargo.features": "all",
  "rust-analyzer.check.command": "clippy",
  "rust-analyzer.check.allTargets": true,
  "editor.formatOnSave": true,
  "editor.rulers": [100],
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  }
}
```

---

## Development Workflow

### 1. Find or Create an Issue

Before starting work:
- Check [existing issues](https://github.com/RenTonoduka/AIT42/issues)
- Comment on the issue to indicate you're working on it
- If no issue exists, create one describing the bug or feature

### 2. Fork and Branch

```bash
# Fork the repository on GitHub
# Then clone your fork
git clone https://github.com/YOUR_USERNAME/AIT42
cd AIT42-Editor

# Add upstream remote
git remote add upstream https://github.com/RenTonoduka/AIT42

# Create feature branch
git checkout -b feature/my-feature
# or
git checkout -b fix/my-bugfix
```

### 3. Make Changes

- Write clean, well-documented code
- Follow coding standards (see below)
- Add tests for new functionality
- Update documentation as needed

### 4. Commit Changes

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```bash
git commit -m "feat(lsp): add support for Ruby language server"
git commit -m "fix(buffer): handle emoji grapheme clusters correctly"
git commit -m "docs: update installation instructions"
git commit -m "test: add integration tests for agent executor"
```

**Commit Types**:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `style`: Code style (formatting, missing semicolons, etc.)
- `refactor`: Code refactoring
- `perf`: Performance improvement
- `test`: Adding or updating tests
- `chore`: Maintenance tasks (dependencies, build, etc.)

### 5. Push and Create PR

```bash
# Push to your fork
git push origin feature/my-feature

# Create Pull Request on GitHub
# Fill out the PR template
```

---

## Coding Standards

### Rust Style Guide

Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/):

1. **Formatting**: Use `rustfmt` (automatically applied)
   ```bash
   cargo fmt
   ```

2. **Linting**: Fix all `clippy` warnings
   ```bash
   cargo clippy -- -W clippy::all
   ```

3. **Error Handling**: Never use `unwrap()` or `expect()` in production code
   ```rust
   // Bad
   let file = fs::read_to_string(path).unwrap();

   // Good
   let file = fs::read_to_string(path)
       .map_err(|e| EditorError::FileReadError(path.clone(), e))?;
   ```

4. **Naming Conventions**:
   - Types: `PascalCase` (e.g., `EditorState`, `BufferId`)
   - Functions/Variables: `snake_case` (e.g., `read_file`, `line_count`)
   - Constants: `SCREAMING_SNAKE_CASE` (e.g., `MAX_BUFFER_SIZE`)

5. **Documentation**:
   ```rust
   /// Brief description of function
   ///
   /// More detailed explanation if needed.
   ///
   /// # Arguments
   ///
   /// * `path` - The file path to read
   /// * `encoding` - The character encoding to use
   ///
   /// # Returns
   ///
   /// Returns the file content as a String
   ///
   /// # Errors
   ///
   /// Returns `EditorError::FileNotFound` if file doesn't exist
   ///
   /// # Examples
   ///
   /// ```no_run
   /// use ait42_core::read_file;
   /// let content = read_file("test.txt", "utf-8")?;
   /// ```
   pub fn read_file(path: &Path, encoding: &str) -> Result<String> {
       // Implementation
   }
   ```

6. **Module Organization**:
   - One public module per file
   - Use `mod.rs` only for re-exports
   - Keep modules small and focused

### Performance Guidelines

1. **Avoid allocations in hot paths**:
   ```rust
   // Bad: Allocates on every iteration
   for line in buffer.lines() {
       let owned = line.to_string();
       process(owned);
   }

   // Good: Use Cow<str> or references
   for line in buffer.lines() {
       process(&line);
   }
   ```

2. **Use appropriate data structures**:
   - `Vec<T>` for sequential access
   - `HashMap<K, V>` for key-value lookups
   - `BTreeMap<K, V>` for sorted iteration
   - `Rope` for text editing

3. **Profile before optimizing**:
   ```bash
   cargo flamegraph --bin ait42-editor
   ```

### Security Guidelines

1. **Input validation**:
   ```rust
   pub fn set_cursor(&mut self, pos: usize) -> Result<()> {
       if pos > self.buffer.len_chars() {
           return Err(EditorError::InvalidCursorPosition(pos));
       }
       self.cursor = pos;
       Ok(())
   }
   ```

2. **No unsafe code** without documentation:
   ```rust
   // Only allowed when absolutely necessary
   // SAFETY: Caller must ensure buffer is valid UTF-8
   unsafe {
       // ... unsafe operations
   }
   ```

3. **Sanitize outputs**:
   ```rust
   fn sanitize_output(text: &str) -> String {
       text.chars()
           .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
           .collect()
   }
   ```

---

## Testing Guidelines

### Test Coverage Requirements

| Component | Target Coverage | Current |
|-----------|----------------|---------|
| ait42-core | 90%+ | 89% |
| ait42-lsp | 80%+ | 82% |
| ait42-ait42 | 85%+ | 87% |
| ait42-tui | 70%+ | 68% |
| **Overall** | **85%+** | **83%** |

### Writing Tests

#### Unit Tests

Place tests in the same file:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_insert() {
        // Arrange
        let mut buffer = Buffer::new();

        // Act
        buffer.insert(0, "Hello").unwrap();

        // Assert
        assert_eq!(buffer.to_string(), "Hello");
        assert!(buffer.is_dirty());
    }

    #[test]
    #[should_panic(expected = "out of bounds")]
    fn test_buffer_insert_out_of_bounds() {
        let mut buffer = Buffer::new();
        buffer.insert(100, "test").unwrap();
    }
}
```

#### Integration Tests

Place in `tests/` directory:

```rust
// tests/integration_tests.rs
use ait42_core::*;

#[tokio::test]
async fn test_full_editing_workflow() {
    let mut state = EditorState::new();
    let buffer = Buffer::from_string("test".to_string(), None);
    let id = state.open_buffer(buffer);

    // Make edit
    let cmd = Box::new(InsertCommand::new(id, 0, "// Comment\n"));
    state.execute_command(cmd).await.unwrap();

    // Verify
    assert_eq!(state.buffer(id).unwrap().to_string(), "// Comment\ntest");
}
```

#### Property-Based Tests

Use `proptest`:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_insert_delete_roundtrip(text in "\\PC*") {
        let mut buffer = Buffer::new();
        buffer.insert(0, &text).unwrap();
        assert_eq!(buffer.to_string(), text);

        let len = buffer.len_chars();
        buffer.delete(0..len).unwrap();
        assert_eq!(buffer.len_chars(), 0);
    }
}
```

### Running Tests

```bash
# All tests
cargo test

# Specific crate
cargo test --package ait42-core

# Specific test
cargo test test_buffer_insert

# With output
cargo test -- --nocapture

# Coverage
cargo tarpaulin --out Html --output-dir coverage

# Watch mode
cargo watch -x test

# Faster test runner
cargo nextest run
```

### Test Checklist

Before submitting PR:
- [ ] All tests pass: `cargo test`
- [ ] No clippy warnings: `cargo clippy`
- [ ] Code formatted: `cargo fmt`
- [ ] Coverage maintained or improved
- [ ] New features have tests
- [ ] Edge cases covered

---

## Pull Request Process

### PR Title

Follow Conventional Commits format:

```
feat(lsp): add Ruby language server support
fix(buffer): handle emoji grapheme clusters correctly
docs: update installation instructions
```

### PR Description Template

```markdown
## Description

Brief description of changes made.

## Motivation and Context

Why is this change required? What problem does it solve?
Fixes #(issue)

## Type of Change

- [ ] Bug fix (non-breaking change which fixes an issue)
- [ ] New feature (non-breaking change which adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to change)
- [ ] Documentation update

## How Has This Been Tested?

Describe the tests you ran to verify your changes:
- [ ] Unit tests
- [ ] Integration tests
- [ ] Manual testing

Test Configuration:
* macOS version:
* Rust version:
* Terminal: (iTerm2, Terminal.app, Alacritty)

## Screenshots (if applicable)

<!-- Add screenshots here -->

## Checklist

- [ ] My code follows the code style of this project
- [ ] I have performed a self-review of my own code
- [ ] I have commented my code, particularly in hard-to-understand areas
- [ ] I have made corresponding changes to the documentation
- [ ] My changes generate no new warnings
- [ ] I have added tests that prove my fix is effective or that my feature works
- [ ] New and existing unit tests pass locally with my changes
- [ ] Any dependent changes have been merged and published

## Additional Notes

<!-- Any additional information -->
```

### Review Process

1. **Automated Checks**: CI runs automatically
   - Tests must pass
   - Clippy must pass
   - Formatting must be correct

2. **Code Review**: Maintainer reviews code
   - Focus on correctness, readability, performance
   - May request changes

3. **Approval**: Once approved, PR can be merged

4. **Merge**: Maintainer merges PR (usually squash merge)

### Review Checklist

Reviewers will check:
- [ ] Code quality and style
- [ ] Test coverage
- [ ] Documentation updates
- [ ] No breaking changes (or properly documented)
- [ ] Performance implications
- [ ] Security considerations

---

## Issue Guidelines

### Reporting Bugs

Use the bug report template:

```markdown
**Describe the bug**
A clear and concise description of what the bug is.

**To Reproduce**
Steps to reproduce the behavior:
1. Open file '...'
2. Press '....'
3. See error

**Expected behavior**
A clear and concise description of what you expected to happen.

**Screenshots**
If applicable, add screenshots to help explain your problem.

**Environment:**
 - macOS version: [e.g. 14.2]
 - Terminal: [e.g. iTerm2]
 - AIT42 Editor version: [e.g. 1.0.0]
 - Rust version: [e.g. 1.75.0]

**Additional context**
Add any other context about the problem here.
```

### Feature Requests

Use the feature request template:

```markdown
**Is your feature request related to a problem? Please describe.**
A clear and concise description of what the problem is. Ex. I'm always frustrated when [...]

**Describe the solution you'd like**
A clear and concise description of what you want to happen.

**Describe alternatives you've considered**
A clear and concise description of any alternative solutions or features you've considered.

**Additional context**
Add any other context or screenshots about the feature request here.
```

---

## Documentation

### What to Document

- **Public APIs**: All public functions, structs, enums need rustdoc
- **User-facing features**: Update USER_GUIDE.md
- **Architecture changes**: Update DEVELOPER_GUIDE.md or ARCHITECTURE.md
- **Configuration options**: Update config examples
- **Breaking changes**: Update CHANGELOG.md

### Documentation Style

```rust
/// Brief one-line description
///
/// More detailed explanation. Use Markdown formatting:
/// - **Bold** for emphasis
/// - `code` for inline code
/// - ```rust for code blocks
///
/// # Arguments
///
/// * `buffer_id` - ID of the buffer to operate on
/// * `position` - Byte position where to insert
/// * `text` - Text to insert
///
/// # Returns
///
/// Returns `Ok(())` on success, or an error if:
/// - Buffer not found
/// - Position out of bounds
///
/// # Examples
///
/// ```no_run
/// # use ait42_core::*;
/// # fn example() -> Result<()> {
/// let mut state = EditorState::new();
/// state.insert_text(buffer_id, 0, "Hello")?;
/// # Ok(())
/// # }
/// ```
///
/// # Panics
///
/// This function does not panic.
///
/// # Safety
///
/// (Only for unsafe functions)
/// Caller must ensure...
pub fn insert_text(
    &mut self,
    buffer_id: BufferId,
    position: usize,
    text: &str,
) -> Result<()> {
    // Implementation
}
```

### Generating Documentation

```bash
# Generate and open docs
cargo doc --no-deps --open

# Generate with private items (for internal development)
cargo doc --no-deps --document-private-items --open
```

---

## Community

### Communication Channels

- **GitHub Issues**: Bug reports, feature requests
- **GitHub Discussions**: Questions, ideas, general discussion
- **Pull Requests**: Code contributions

### Getting Help

- Check [USER_GUIDE.md](USER_GUIDE.md) for usage questions
- Check [DEVELOPER_GUIDE.md](DEVELOPER_GUIDE.md) for development questions
- Search existing [issues](https://github.com/RenTonoduka/AIT42/issues)
- Ask in [GitHub Discussions](https://github.com/RenTonoduka/AIT42/discussions)

### Recognition

Contributors will be:
- Listed in CONTRIBUTORS.md
- Mentioned in release notes
- Acknowledged in commit history

Thank you for contributing to AIT42 Editor!

---

## License

By contributing to AIT42 Editor, you agree that your contributions will be licensed under the MIT License.

---

## Questions?

If you have questions not covered here:
- Open a [GitHub Discussion](https://github.com/RenTonoduka/AIT42/discussions)
- Tag maintainers in an issue

**Happy Contributing! 🎉**
