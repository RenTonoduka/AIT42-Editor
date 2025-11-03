# Documentation Style Guide - AIT42 Editor

**Version**: 1.0.0
**Last Updated**: 2025-11-03
**Applies To**: All project documentation

---

## Purpose

This style guide ensures consistency, clarity, and professionalism across all AIT42 Editor documentation. All contributors must follow these guidelines when creating or modifying documentation.

---

## Table of Contents

1. [Writing Style](#writing-style)
2. [Markdown Formatting](#markdown-formatting)
3. [Code Examples](#code-examples)
4. [File Organization](#file-organization)
5. [Terminology](#terminology)
6. [Document Templates](#document-templates)
7. [Review Checklist](#review-checklist)

---

## Writing Style

### Voice and Tone

#### Use Active Voice

**Good**:
> The editor loads the file when you press Enter.

**Bad**:
> The file is loaded by the editor when Enter is pressed.

---

#### Use Present Tense

**Good**:
> The agent executes the task in a tmux session.

**Bad**:
> The agent will execute the task in a tmux session.

---

#### Be Direct and Concise

**Good**:
> Press `Ctrl+S` to save.

**Bad**:
> In order to save the file, you should press the keyboard combination Ctrl+S.

---

#### Avoid Jargon (in User Docs)

**Good** (User Guide):
> The editor creates a backup copy before saving.

**Bad**:
> The editor performs atomic write operations with fsync.

**Acceptable** (Developer Guide):
> The editor uses atomic writes with fsync for data integrity.

---

### Pronouns

#### Use "You" for Users

**Good**:
> You can open the agent palette by pressing `Ctrl+Shift+A`.

**Bad**:
> One can open the agent palette by pressing `Ctrl+Shift+A`.

---

#### Avoid Gendered Pronouns

**Good**:
> When a developer opens a file, they see syntax highlighting.

**Bad**:
> When a developer opens a file, he sees syntax highlighting.

---

### Sentence Structure

#### Keep Sentences Short

- Target: 15-20 words per sentence
- Maximum: 25 words
- Use bullet points for lists

**Good**:
> The editor supports multiple cursors. Press `Ctrl+D` to add a cursor at the next occurrence.

**Bad**:
> The editor supports multiple cursors which can be added by pressing `Ctrl+D` to select the next occurrence of the currently selected text.

---

#### Use Parallel Structure

**Good**:
> The agent can:
> - Read files
> - Write code
> - Execute commands

**Bad**:
> The agent can:
> - Read files
> - Writing code
> - Commands can be executed

---

## Markdown Formatting

### Headers

#### Use ATX-Style Headers

```markdown
# H1: Document Title (one per document)
## H2: Major Sections
### H3: Subsections
#### H4: Minor Subsections
```

#### Header Rules

- One H1 per document (document title)
- No skipping levels (H1 → H3)
- Add blank line before and after headers
- Use sentence case (capitalize first word only)

**Good**:
```markdown
## Installation guide

This section covers installation.

### Prerequisites

You need these tools...
```

**Bad**:
```markdown
## Installation Guide
This section covers installation.
####Prerequisites
You need these tools...
```

---

### Lists

#### Unordered Lists

Use `-` for unordered lists (not `*` or `+`):

```markdown
- First item
- Second item
  - Nested item (2 spaces indent)
  - Another nested item
- Third item
```

#### Ordered Lists

Use numbers for ordered lists:

```markdown
1. First step
2. Second step
3. Third step
```

#### List Punctuation

- No periods for single-line items
- Use periods for multi-line items
- Be consistent within a list

---

### Code Formatting

#### Inline Code

Use backticks for:
- Commands: `cargo build`
- File names: `main.rs`
- Code elements: `Buffer`, `TextBuffer::new()`
- Keyboard keys: `Ctrl+S`

```markdown
The `cargo test` command runs all tests.
```

#### Code Blocks

Always specify language:

````markdown
```rust
fn main() {
    println!("Hello, world!");
}
```

```bash
cargo build --release
```

```toml
[package]
name = "ait42-editor"
```
````

#### Supported Languages

- `rust` - Rust code
- `bash` / `sh` - Shell commands
- `toml` - Configuration files
- `yaml` - YAML files
- `json` - JSON data
- `markdown` - Markdown examples
- `text` - Plain text output

---

### Links

#### Internal Links

Use relative paths:

```markdown
See [Architecture](ARCHITECTURE.md) for details.
See [API Specification](API_SPECIFICATION.md#buffer-api).
```

#### External Links

Use full URLs:

```markdown
See [Rust Book](https://doc.rust-lang.org/book/).
```

#### Link Text

Make link text descriptive:

**Good**:
```markdown
See the [installation guide](INSTALLATION.md) for details.
```

**Bad**:
```markdown
Click [here](INSTALLATION.md) for details.
```

---

### Tables

#### Use Pipe Tables

```markdown
| Header 1 | Header 2 | Header 3 |
|----------|----------|----------|
| Cell 1   | Cell 2   | Cell 3   |
| Cell 4   | Cell 5   | Cell 6   |
```

#### Table Alignment

```markdown
| Left | Center | Right |
|:-----|:------:|------:|
| Text | Text   | 123   |
```

#### Keep Tables Simple

- Maximum 5 columns
- Keep cell content brief
- Use lists for complex data

---

### Emphasis

#### Bold for UI Elements

Use **bold** for:
- Menu items
- Button names
- Tab names

```markdown
Click the **Save** button.
Go to **File** → **Open**.
```

#### Italic for Emphasis

Use *italic* sparingly:

```markdown
This is *not* recommended for production.
```

#### Code for Technical Terms

```markdown
The `Buffer` struct stores text data.
```

---

## Code Examples

### Requirements

All code examples must:

1. **Be syntactically correct**
2. **Be runnable** (or clearly marked as pseudocode)
3. **Include comments** for complex logic
4. **Follow project coding standards**
5. **Be tested** before inclusion

---

### Structure

#### Complete Examples

For standalone examples:

```rust
// main.rs
use ait42_core::Buffer;

fn main() {
    // Create new buffer
    let mut buffer = Buffer::new();

    // Insert text
    buffer.insert(0, "Hello, world!").unwrap();

    // Print result
    println!("{}", buffer.to_string());
}
```

#### Snippet Examples

For focused examples:

```rust
// Insert text at cursor position
let pos = cursor.pos();
buffer.insert(pos, "text")?;
```

---

### Comments

#### Use Comments Sparingly

Only comment non-obvious code:

**Good**:
```rust
// Canonicalize path to prevent traversal attacks
let path = path.canonicalize()?;
```

**Bad**:
```rust
// Create a buffer
let buffer = Buffer::new();
```

---

### Error Handling

Show proper error handling:

**Good**:
```rust
let buffer = Buffer::from_file(path)
    .map_err(|e| Error::FileReadFailed(path.to_path_buf(), e))?;
```

**Bad**:
```rust
let buffer = Buffer::from_file(path).unwrap();
```

---

### Command Examples

#### Format

```bash
# Comment describing what this does
command arg1 arg2

# Example output
Expected output here
```

#### Example

```bash
# Build the project in release mode
cargo build --release

# Output:
# Compiling ait42-core v1.0.0
# Compiling ait42-editor v1.0.0
#    Finished release [optimized] target(s) in 2.34s
```

---

## File Organization

### Document Structure

Every document should have:

1. **Title** (H1)
2. **Metadata** (version, date, status)
3. **Table of Contents** (for docs >500 lines)
4. **Introduction** (what this doc covers)
5. **Main Content** (organized in sections)
6. **Conclusion** (summary, next steps)
7. **References** (related docs, external links)

---

### Template

```markdown
# Document Title

**Version**: 1.0.0
**Date**: 2025-11-03
**Status**: Complete / Draft / In Review
**Author**: Name or Team

---

## Table of Contents

1. [Section 1](#section-1)
2. [Section 2](#section-2)

---

## Introduction

Brief overview of what this document covers.

## Section 1

Content...

## Section 2

Content...

## Conclusion

Summary and next steps.

---

**Last Updated**: 2025-11-03
**Maintained By**: Team Name
```

---

### File Naming

#### Use Consistent Names

- ALL_CAPS for major docs: `README.md`, `CONTRIBUTING.md`
- Hyphens for multi-word: `USER_GUIDE.md`
- Descriptive names: `API_SPECIFICATION.md` (not `API.md`)

#### File Organization

```
project-root/
├── README.md                    # Project overview
├── CONTRIBUTING.md              # Contribution guide
├── ARCHITECTURE.md              # Architecture
├── API_SPECIFICATION.md         # API docs
├── docs/                        # Additional docs
│   ├── guides/                  # User guides
│   │   ├── USER_GUIDE.md
│   │   └── INSTALLATION.md
│   ├── development/             # Developer docs
│   │   ├── DEVELOPER_GUIDE.md
│   │   └── TESTING_GUIDE.md
│   └── security/                # Security docs
│       ├── SECURITY_ARCHITECTURE.md
│       └── THREAT_MODEL.md
```

---

## Terminology

### Consistent Terms

Always use the same term for the same concept:

| Use This | Not This |
|----------|----------|
| Agent | Bot, Assistant, AI |
| Buffer | File, Document, Text |
| Palette | Menu, Picker, Selector |
| Tmux session | Terminal, Shell, Console |
| LSP | Language Server, Language Support |

---

### Capitalization

| Term | Capitalization | Example |
|------|----------------|---------|
| AIT42 Editor | Title case | "AIT42 Editor is a terminal-based..." |
| Rust | Capitalized | "Written in Rust" |
| macOS | Proper case | "Supports macOS 12.0+" |
| Vim | Capitalized | "Vim-style editing" |
| LSP | All caps | "LSP client" |
| API | All caps | "REST API" |

---

### Product Names

- **AIT42 Editor** - Full product name
- **AIT42** - Short form (after first mention)
- **the editor** - Generic reference

---

## Document Templates

### User Guide Template

```markdown
# [Feature Name] - User Guide

## Overview

[What this feature does, why it's useful]

## Prerequisites

- Requirement 1
- Requirement 2

## Usage

### Basic Usage

[Simple example]

```bash
# Example command
ait42-editor file.rs
```

### Advanced Usage

[Complex examples]

## Configuration

[Configuration options]

```toml
[section]
option = "value"
```

## Troubleshooting

### Issue 1

**Problem**: [Description]

**Solution**:
1. Step 1
2. Step 2

---

**Last Updated**: 2025-11-03
```

---

### API Documentation Template

```markdown
# [Module Name] API

## Overview

[Brief description of module purpose]

## Types

### `StructName`

[Description]

**Fields**:
- `field_name: Type` - Description

**Example**:
```rust
let instance = StructName {
    field_name: value,
};
```

## Functions

### `function_name(arg: Type) -> ReturnType`

[Description]

**Parameters**:
- `arg` - Description

**Returns**:
- Description of return value

**Errors**:
- `ErrorType` - When this error occurs

**Example**:
```rust
let result = function_name(arg)?;
```

---

**Last Updated**: 2025-11-03
```

---

### Security Document Template

```markdown
# Security [Topic]

**Version**: 1.0.0
**Classification**: Internal / Public
**Last Updated**: 2025-11-03

## Summary

[Brief security overview]

## Threat Model

### Assets
- Asset 1
- Asset 2

### Threats
| Threat | Severity | Mitigation |
|--------|----------|------------|
| Threat 1 | High | Mitigation strategy |

## Security Controls

### Control 1

**Purpose**: [What this protects]

**Implementation**:
```rust
// Code example
```

## Testing

[How to verify security]

## Incident Response

[What to do if security issue found]

---

**Last Updated**: 2025-11-03
```

---

## Review Checklist

Before submitting documentation:

### Content

- [ ] **Accurate**: All information is correct
- [ ] **Complete**: No missing sections
- [ ] **Clear**: Easy to understand
- [ ] **Concise**: No unnecessary verbosity
- [ ] **Current**: Reflects current version

### Formatting

- [ ] **Headers**: Proper hierarchy (H1 → H2 → H3)
- [ ] **Lists**: Consistent formatting
- [ ] **Code blocks**: Language specified
- [ ] **Links**: All links work
- [ ] **Tables**: Properly formatted

### Writing Style

- [ ] **Active voice**: Used throughout
- [ ] **Present tense**: Used consistently
- [ ] **Consistent terms**: Same concepts, same words
- [ ] **Proper capitalization**: Per style guide
- [ ] **Short sentences**: Average 15-20 words

### Code Examples

- [ ] **Syntactically correct**: No errors
- [ ] **Runnable**: Examples work
- [ ] **Commented**: Complex logic explained
- [ ] **Error handling**: Proper error handling shown
- [ ] **Tested**: Examples verified

### Accessibility

- [ ] **Alt text**: Images have descriptions
- [ ] **Heading structure**: Logical hierarchy
- [ ] **Link text**: Descriptive, not "click here"
- [ ] **Abbreviations**: Defined on first use

### Metadata

- [ ] **Version**: Document version specified
- [ ] **Date**: Last update date included
- [ ] **Author**: Author or team identified
- [ ] **Status**: Draft / In Review / Complete

---

## Common Mistakes

### 1. Inconsistent Terminology

**Bad**:
> Open the agent menu. Select a bot from the picker.

**Good**:
> Open the agent palette. Select an agent from the palette.

---

### 2. Passive Voice

**Bad**:
> The file is saved by the editor when Ctrl+S is pressed.

**Good**:
> The editor saves the file when you press Ctrl+S.

---

### 3. Future Tense

**Bad**:
> The editor will display an error message.

**Good**:
> The editor displays an error message.

---

### 4. Missing Code Language

**Bad**:
````markdown
```
fn main() {}
```
````

**Good**:
````markdown
```rust
fn main() {}
```
````

---

### 5. Broken Links

**Bad**:
```markdown
See [Documentation](docs.md) for details.
```

**Good**:
```markdown
See [Documentation](USER_GUIDE.md) for details.
```

---

### 6. Unclear Examples

**Bad**:
```rust
let x = func(y);
```

**Good**:
```rust
// Create buffer from file
let buffer = Buffer::from_file(&path)?;
```

---

### 7. No Error Handling

**Bad**:
```rust
let file = fs::read_to_string(path).unwrap();
```

**Good**:
```rust
let file = fs::read_to_string(path)
    .map_err(|e| Error::FileReadFailed(path, e))?;
```

---

## Tools and Automation

### Markdown Linters

Use `markdownlint`:

```bash
# Install
npm install -g markdownlint-cli

# Run
markdownlint '**/*.md'
```

### Spell Checking

Use `cspell`:

```bash
# Install
npm install -g cspell

# Run
cspell '**/*.md'
```

### Link Checking

Use `markdown-link-check`:

```bash
# Install
npm install -g markdown-link-check

# Run
find . -name \*.md -exec markdown-link-check {} \;
```

---

## Documentation Review Process

### 1. Self-Review

- Read your document out loud
- Check all links
- Run spell checker
- Verify code examples

### 2. Peer Review

- At least one peer review
- Check for clarity and accuracy
- Verify technical content

### 3. User Testing

- For user-facing docs, test with actual users
- Observe where they get stuck
- Iterate based on feedback

### 4. Final Approval

- Maintainer approval required
- All checklist items complete
- CI checks pass

---

## Continuous Improvement

### Feedback Collection

Encourage feedback:

```markdown
## Feedback

Found an issue with this documentation?
- [Open an issue](https://github.com/RenTonoduka/AIT42/issues)
- [Submit a PR](https://github.com/RenTonoduka/AIT42/pulls)
- Email: docs@ait42-editor.com
```

### Regular Reviews

- Review all docs quarterly
- Update for new features
- Fix reported issues promptly
- Track common questions (→ FAQ)

---

## Examples

### Good Documentation Example

```markdown
# Buffer API

## Overview

The Buffer API provides text manipulation operations with efficient performance
on large files.

## Creating a Buffer

### `Buffer::new()`

Creates an empty buffer.

**Example**:
```rust
use ait42_core::Buffer;

let buffer = Buffer::new();
```

### `Buffer::from_file(path: &Path) -> Result<Self>`

Loads a buffer from a file.

**Parameters**:
- `path` - Path to the file

**Returns**:
- `Ok(Buffer)` on success
- `Err(Error)` if file cannot be read

**Example**:
```rust
use std::path::Path;

let path = Path::new("main.rs");
let buffer = Buffer::from_file(&path)?;
```

## Inserting Text

### `buffer.insert(pos: usize, text: &str) -> Result<()>`

Inserts text at the specified position.

**Parameters**:
- `pos` - Byte offset (0-based)
- `text` - Text to insert

**Errors**:
- `Error::InvalidOffset` if position is out of bounds

**Example**:
```rust
// Insert at beginning
buffer.insert(0, "Hello, ")?;

// Insert at end
let end = buffer.len_bytes();
buffer.insert(end, "world!")?;
```

**Performance**: O(log n) where n is buffer size
```

This example demonstrates:
- Clear structure
- Active voice
- Present tense
- Complete code examples
- Error handling
- Performance notes
- Proper formatting

---

## Style Guide Maintenance

**Version History**:
- 1.0.0 (2025-11-03): Initial version

**Review Schedule**: Quarterly

**Owner**: Documentation Team

**Feedback**: docs@ait42-editor.com

---

**End of Style Guide**
