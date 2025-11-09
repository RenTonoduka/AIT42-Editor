# AIT42 Editor User Guide

**Version**: 1.0.0
**Last Updated**: 2025-01-06
**Target Audience**: End Users, Developers

---

## Table of Contents

1. [Introduction](#1-introduction)
2. [Installation](#2-installation)
3. [Getting Started](#3-getting-started)
4. [Basic Editing](#4-basic-editing)
5. [Advanced Features](#5-advanced-features)
6. [AI Agent Integration](#6-ai-agent-integration)
7. [Configuration](#7-configuration)
8. [Keyboard Shortcuts](#8-keyboard-shortcuts)
9. [Troubleshooting](#9-troubleshooting)
10. [FAQ](#10-faq)

---

## 1. Introduction

### What is AIT42 Editor?

AIT42 Editor is a modern, fast terminal-based code editor built specifically for macOS, featuring deep integration with 49 specialized AI agents. It combines the efficiency of Vim-style modal editing with intelligent code completion (LSP) and AI-powered development assistance.

**Think of it as**: Neovim + VSCode Intelligence + 49 AI Development Assistants, all in your terminal.

### Key Features

- **Vim-Style Modal Editing**: Efficient keyboard-driven text manipulation
- **49 Specialized AI Agents**: From backend development to security testing
- **LSP Support**: Intelligent code completion for 15+ languages
- **Rope-Based Text Buffer**: Handles large files (100MB+) efficiently
- **Real-Time File Synchronization**: Auto-reload on external changes
- **Tmux Session Management**: Run multiple AI agents in parallel
- **macOS Native**: Optimized for macOS terminal experience

### System Requirements

- **Operating System**: macOS 11.0 (Big Sur) or later
- **Terminal Emulator**: iTerm2, Terminal.app, Alacritty, or Kitty
- **RAM**: 4GB minimum, 8GB recommended
- **Disk Space**: 100MB for installation
- **Optional**:
  - Rust 1.75+ (for building from source)
  - Tmux 2.0+ (for parallel agent execution)
  - Language servers (rust-analyzer, typescript-language-server, etc.)

### Who Should Use AIT42 Editor?

- **Terminal Enthusiasts**: Developers who prefer keyboard-driven workflows
- **AI-Assisted Developers**: Teams leveraging AI for code generation, review, and testing
- **macOS Developers**: Native macOS terminal experience
- **Performance Seekers**: <500ms startup time, handles 100MB+ files
- **Multi-Agent Workflows**: Coordinate multiple AI agents for complex tasks

---

## 2. Installation

### Option 1: Homebrew (Recommended)

```bash
# Add AIT42 tap
brew tap ait42/ait42-editor

# Install
brew install ait42-editor

# Verify installation
ait42-editor --version
# Expected output: ait42-editor 1.0.0
```

### Option 2: Download Binary

1. Visit the [Releases Page](https://github.com/RenTonoduka/AIT42/releases)
2. Download the latest `ait42-editor-macos.tar.gz`
3. Extract and install:

```bash
tar -xzf ait42-editor-macos.tar.gz
cd ait42-editor-macos
sudo cp ait42-editor /usr/local/bin/
chmod +x /usr/local/bin/ait42-editor
```

### Option 3: Build from Source

```bash
# Clone repository
git clone https://github.com/RenTonoduka/AIT42
cd AIT42-Editor

# Run setup script
./scripts/setup.sh

# Build release binary
cargo build --release

# Install to system
sudo cp target/release/ait42-editor /usr/local/bin/
```

### Post-Installation Setup

1. **Create configuration directory**:
   ```bash
   mkdir -p ~/.config/ait42-editor
   ```

2. **Install optional dependencies**:
   ```bash
   # Tmux (for parallel agent execution)
   brew install tmux

   # Language servers (optional, for LSP features)
   # Rust
   rustup component add rust-analyzer

   # TypeScript
   npm install -g typescript-language-server typescript

   # Python
   pip install pyright
   ```

3. **Set AIT42 root path** (for AI agents):
   ```bash
   export AIT42_ROOT=/path/to/AIT42
   echo 'export AIT42_ROOT=/path/to/AIT42' >> ~/.zshrc
   ```

### Verification

```bash
# Launch editor
ait42-editor

# You should see the welcome screen with version info
# Press 'q' to quit
```

---

## 3. Getting Started

### First Launch

```bash
# Launch editor (empty buffer)
ait42-editor

# Launch with interactive tutorial
ait42-editor --tutorial

# Open specific file
ait42-editor README.md

# Open directory (file explorer)
ait42-editor .
```

### Interactive Tutorial

On first launch, AIT42 Editor displays an interactive tutorial:

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Welcome to AIT42 Editor v1.0.0
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Press ENTER to start the tutorial
Press 'q' to quit
Press 'h' for help

Features:
  • Vim-style modal editing
  • 49 AI agents (Ctrl+P)
  • LSP code completion
  • Tmux session management
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### Opening Files

```bash
# Single file
ait42-editor src/main.rs

# Multiple files (buffers)
ait42-editor src/main.rs src/lib.rs

# Open at specific line:column
ait42-editor src/main.rs:42:10

# Open directory
ait42-editor src/
```

### Understanding the Interface

```
┌─────────────────────────────────────────────┐
│ 1 │ fn main() {                             │  ← Buffer (editing area)
│ 2 │     println!("Hello, World!");          │
│ 3 │ }                                       │
│ 4 │                                         │
├─────────────────────────────────────────────┤
│ NORMAL | src/main.rs | 2:5 | UTF-8 | Rust  │  ← Status bar
└─────────────────────────────────────────────┘
   ↑        ↑             ↑      ↑      ↑
  Mode    File         Cursor  Encoding Lang
```

**Components**:
- **Buffer**: Main editing area (text content)
- **Status Bar**: Mode, file path, cursor position, encoding, language
- **Command Palette**: `Ctrl+P` to open (agent selection, file search)
- **File Tree**: `Ctrl+E` to toggle (directory navigation)

---

## 4. Basic Editing

### Modal Editing

AIT42 uses Vim-style modal editing with 4 modes:

#### Normal Mode (Default)

Navigate and manipulate text. Press `Esc` to return to Normal mode from any other mode.

**Movement**:
```
h, j, k, l    ← Move left, down, up, right
w, b          ← Next word, previous word
0, $          ← Start of line, end of line
gg, G         ← Start of file, end of file
Ctrl+F, Ctrl+B ← Page down, page up
```

**Editing**:
```
x       ← Delete character
dd      ← Delete line
yy      ← Copy line
p       ← Paste
u       ← Undo
Ctrl+R  ← Redo
```

#### Insert Mode

Type text normally. Press `i` to enter Insert mode.

**Entry**:
```
i   ← Insert before cursor
a   ← Insert after cursor
I   ← Insert at line start
A   ← Insert at line end
o   ← Open line below
O   ← Open line above
```

**Exit**: Press `Esc` to return to Normal mode

#### Visual Mode

Select text. Press `v` to enter Visual mode.

**Selection**:
```
v       ← Character-wise selection
V       ← Line-wise selection
Ctrl+V  ← Block-wise selection (Phase 2)
```

**Operations** (after selection):
```
y   ← Copy selection
d   ← Delete selection
c   ← Change selection (delete and enter Insert mode)
```

#### Command Mode

Execute commands. Press `:` to enter Command mode.

**Common Commands**:
```
:w          ← Save file
:q          ← Quit
:wq         ← Save and quit
:q!         ← Quit without saving
:e file.rs  ← Open file
:bn         ← Next buffer
:bp         ← Previous buffer
:bd         ← Close current buffer
```

### Basic Workflow Example

```
1. Open file:      ait42-editor src/main.rs
2. Navigate:       j j j  (move to line 3)
3. Enter Insert:   i
4. Type text:      // TODO: Implement feature
5. Exit Insert:    Esc
6. Save:           :w
7. Quit:           :q
```

---

## 5. Advanced Features

### LSP Features

Language Server Protocol provides intelligent code intelligence.

#### Code Completion

**Automatic**:
- Completion suggestions appear as you type
- `Tab` to accept suggestion
- `Esc` to dismiss

**Manual**:
- `Ctrl+Space` to manually trigger completion

Example (TypeScript):
```typescript
const user = {
  name: "John",
  age: 30
};

user.   ← Type '.' and completion shows: name, age
```

#### Go to Definition

Place cursor on symbol and press `gd` in Normal mode.

```rust
fn main() {
    hello();  ← Cursor here, press 'gd'
}

fn hello() {  ← Jumps to this definition
    println!("Hello!");
}
```

#### Hover Information

Place cursor on symbol and press `K` for documentation.

```rust
fn main() {
    Vec::new()  ← Cursor on 'Vec', press 'K'
}

// Shows popup:
// pub struct Vec<T, A = Global> { ... }
// A contiguous growable array type...
```

#### Diagnostics

Errors and warnings shown in real-time:

```
┌─────────────────────────────────────────────┐
│ 1 │ fn main() {                             │
│ 2 │     let x = 5;                          │
│ 3 │     println!("{}", y);  ← ⚠ Error       │
│   │                           └─ cannot find value `y`
│ 4 │ }                                       │
├─────────────────────────────────────────────┤
│ ⚠ 1 error | Press Ctrl+D for diagnostics   │
└─────────────────────────────────────────────┘
```

Commands:
```
:diagnostics     ← View all issues
:lsp-restart     ← Restart LSP server
:lsp-disable     ← Disable LSP for current buffer
```

### Multi-Buffer Editing

Work with multiple files simultaneously:

```
:e src/lib.rs     ← Open new buffer
:bn               ← Next buffer
:bp               ← Previous buffer
:bd               ← Close current buffer
:buffers          ← List all buffers

Output:
  1  src/main.rs (active)
  2  src/lib.rs
  3  Cargo.toml (modified)
```

**Quick Buffer Switching**:
- `Ctrl+6` - Toggle between last two buffers
- `:b <number>` - Jump to buffer number

### Search and Replace

**Search**:
```
/pattern      ← Search forward
?pattern      ← Search backward
n             ← Next match
N             ← Previous match
*             ← Search word under cursor
```

**Replace**:
```
:%s/old/new/g       ← Replace all in file
:s/old/new/g        ← Replace all in current line
:%s/old/new/gc      ← Replace with confirmation
:'<,'>s/old/new/g   ← Replace in visual selection
```

Example:
```
:%s/console.log/logger.info/g
# Replaces all console.log with logger.info
```

### File Explorer

Toggle file tree with `Ctrl+E`:

```
┌─────────────────┬───────────────────────────┐
│ src/            │ fn main() {               │
│ ├─ main.rs *    │     println!("Hello!");   │
│ ├─ lib.rs       │ }                         │
│ └─ utils/       │                           │
│    └─ helper.rs │                           │
│ tests/          │                           │
│ Cargo.toml      │                           │
└─────────────────┴───────────────────────────┘
    File Tree           Active Buffer
```

**Navigation**:
```
j, k     ← Move down, up
Enter    ← Open file / Expand directory
h        ← Collapse directory
l        ← Expand directory
/        ← Search files (fuzzy)
```

### Splits (Phase 2)

Horizontal and vertical splits (coming in Phase 2):

```
:split        ← Horizontal split
:vsplit       ← Vertical split
Ctrl+W h/j/k/l ← Navigate splits
```

---

## 6. AI Agent Integration

AIT42's most powerful feature: 49 specialized AI agents.

### Command Palette

Press `Ctrl+P` to open the command palette:

```
┌────────────────────────────────────────┐
│ Search: backend                        │
├────────────────────────────────────────┤
│ ► backend-developer                    │
│   Backend API, authentication, logic   │
│                                        │
│   api-developer                        │
│   REST/GraphQL/WebSocket APIs          │
│                                        │
│   database-developer                   │
│   DB implementation, migrations        │
└────────────────────────────────────────┘
```

### Using the Coordinator (Auto-Select)

The **Coordinator** automatically selects the best agent(s) for your task.

**Usage**: Just describe what you want, no need to specify agent name.

```
Ctrl+P → Type your request
"Implement user authentication API"

→ Coordinator selects: backend-developer
→ Agent executes task
→ Result displayed in status bar
```

**Examples**:
```
"Review this code for security issues"
→ Coordinator selects: security-tester + code-reviewer

"Design an e-commerce system"
→ Coordinator selects: system-architect + api-designer + database-designer (parallel)

"Generate tests for this module"
→ Coordinator selects: test-generator
```

### Running Specific Agents

#### Backend Development

```
Ctrl+P → Type "backend-developer"
Task: "Implement REST API endpoint for user authentication"

Output:
✓ Generated src/routes/auth.ts
✓ Generated src/middleware/auth.ts
✓ Generated src/services/user-service.ts
✓ Tests created in tests/auth.test.ts
```

#### Code Review

```
Ctrl+P → Type "code-reviewer"
Task: "Review this file for quality and security"

Output:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Code Review Report
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Quality Score: 87/100

✓ Strengths:
  - Good error handling
  - Proper TypeScript types
  - Clear function names

⚠ Issues:
  - Line 42: SQL injection risk
  - Line 67: Missing input validation
  - Line 89: Complexity too high (15)

💡 Recommendations:
  1. Use parameterized queries
  2. Add Joi validation
  3. Extract to smaller functions
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

#### Test Generation

```
Ctrl+P → Type "test-generator"
Task: "Generate unit tests for UserService"

Output:
✓ Generated tests/user-service.test.ts
  - 15 test cases
  - Coverage: 92%
  - Edge cases included
```

### Agent Execution on Selection

1. Enter Visual mode (`v`)
2. Select text (e.g., a function)
3. Press `Ctrl+P`
4. Choose agent (e.g., "refactor-specialist")
5. Agent processes only the selected code

Example:
```rust
// Select this function in Visual mode
fn calculate_total(items: &[Item]) -> f64 {
    let mut total = 0.0;
    for item in items {
        total += item.price;
    }
    total
}

// After running "refactor-specialist":
fn calculate_total(items: &[Item]) -> f64 {
    items.iter().map(|item| item.price).sum()
}
```

### Monitoring Agent Progress

**Real-time output** in status bar:
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Agent: backend-developer [Running...]
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

**View all running agents**:
```
:agents

Output:
  ID   Agent                 Status      Time
  1    backend-developer     Running     2m 34s
  2    test-generator        Queued      -
```

**View agent output**:
```
:agent-output 1

Output: (real-time log)
```

### Parallel Agent Execution

Run multiple agents in parallel using Tmux:

```
Ctrl+P → "Run Parallel Agents"

Select agents (Space to toggle):
☑ api-designer
☑ database-designer
☐ backend-developer

Enter task: "Design e-commerce system"
Press Enter → Agents execute in parallel

Output:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Parallel Execution (2 agents)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
[api-designer] Session: ait42-api-designer-1234
[database-designer] Session: ait42-db-designer-1235

Press Ctrl+T to view Tmux sessions
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

**Tmux Integration**:
```
Ctrl+T     ← View Tmux sessions
:tmux-attach 1234  ← Attach to specific session
:tmux-kill 1234    ← Kill session
```

### Available Agents (49 Total)

#### Planning & Design (8 agents)
- **system-architect**: System design, architecture patterns
- **api-designer**: API design, OpenAPI specs
- **database-designer**: DB design, ERD, normalization
- **ui-ux-designer**: UI/UX design, wireframes
- **security-architect**: Security design, threat modeling
- **cloud-architect**: Cloud architecture, AWS/GCP/Azure
- **integration-planner**: Integration planning, data flow
- **requirements-elicitation**: Requirements, stakeholder analysis

#### Implementation (9 agents)
- **backend-developer**: Backend implementation, APIs
- **frontend-developer**: Frontend, React/Vue/Angular
- **api-developer**: API implementation
- **database-developer**: DB implementation, migrations
- **feature-builder**: New feature implementation
- **integration-developer**: Third-party API integration
- **migration-developer**: Data migration
- **script-writer**: Automation scripts
- **implementation-assistant**: AI system implementation

#### Quality Assurance (11 agents)
- **code-reviewer**: Code review, quality scoring
- **test-generator**: Unit/Integration/E2E tests
- **integration-tester**: Integration testing
- **performance-tester**: Performance, load testing
- **security-tester**: Security testing, OWASP Top 10
- **mutation-tester**: Mutation testing
- **qa-validator**: Quality validation, coverage
- **refactor-specialist**: Refactoring, SOLID principles
- **complexity-analyzer**: Complexity analysis
- **doc-reviewer**: Documentation review
- **bug-fixer**: Bug fixing, root cause analysis

#### Operations (10 agents)
- **devops-engineer**: DevOps, IaC, Terraform, K8s
- **cicd-manager**: CI/CD, pipelines, deployment
- **container-specialist**: Docker, K8s optimization
- **monitoring-specialist**: Monitoring, Prometheus, Grafana
- **incident-responder**: Incident management, RCA
- **security-scanner**: Security scanning, SAST/DAST
- **backup-manager**: Backup, DR planning
- **chaos-engineer**: Chaos engineering
- **release-manager**: Release management, SemVer
- **config-manager**: Configuration management

#### Meta (11 agents)
- **process-optimizer**: Process optimization
- **workflow-coordinator**: Workflow design
- **learning-agent**: Learning capture, best practices
- **feedback-analyzer**: Feedback analysis
- **metrics-collector**: Metrics, DORA metrics
- **knowledge-manager**: Knowledge management
- **innovation-scout**: Technology evaluation
- **tech-writer**: Technical documentation
- **coordinator**: Auto-select optimal agents
- **tmux-session-manager**: Tmux session management
- **session-summarizer**: Session summary generation

---

## 7. Configuration

### Configuration File

**Location**: `~/.config/ait42-editor/config.toml`

### Default Configuration

```toml
[editor]
tab_size = 4
auto_save = true
auto_save_delay = 5000  # milliseconds
line_numbers = true
wrap_lines = false
highlight_current_line = true

[theme]
name = "monokai"  # monokai, solarized-dark, gruvbox

[keybindings]
mode = "vim"  # vim, emacs (Phase 2)

# Custom keybindings (Phase 2)
# [keybindings.custom]
# save = "Ctrl+S"
# command_palette = "Ctrl+P"

[lsp]
# Rust
[lsp.rust]
command = "rust-analyzer"
args = []

# TypeScript
[lsp.typescript]
command = "typescript-language-server"
args = ["--stdio"]

# Python
[lsp.python]
command = "pyright-langserver"
args = ["--stdio"]

# Go
[lsp.go]
command = "gopls"
args = []

[ait42]
ait42_root = "/path/to/AIT42"  # Required for agent integration
max_parallel_agents = 5
session_timeout = 300  # seconds
auto_cleanup = true
auto_tmux = true  # Auto-use tmux for parallel/long tasks

[appearance]
show_whitespace = false
show_trailing_newline = true
indent_guides = true
```

### Themes

Built-in themes:
- **monokai** (default): Dark theme with vibrant colors
- **solarized-dark**: Popular dark theme
- **gruvbox**: Retro dark theme

**Switching themes**:
```toml
[theme]
name = "gruvbox"
```

Or in editor:
```
:theme solarized-dark
```

### Custom Theme (Phase 2)

```toml
[theme]
name = "custom"

[theme.colors]
background = "#1e1e1e"
foreground = "#d4d4d4"
cursor = "#aeafad"
selection = "#264f78"
comment = "#6a9955"
keyword = "#569cd6"
string = "#ce9178"
function = "#dcdcaa"
error = "#f44747"
warning = "#cca700"
```

### LSP Configuration

Add language servers:

```toml
# Java
[lsp.java]
command = "jdtls"
args = []

# C++
[lsp.cpp]
command = "clangd"
args = ["--background-index"]

# HTML
[lsp.html]
command = "vscode-html-language-server"
args = ["--stdio"]
```

### AIT42 Agent Configuration

```toml
[ait42]
# Path to AIT42 root directory
ait42_root = "/Users/username/AIT42"

# Maximum parallel agents
max_parallel_agents = 5

# Agent execution timeout
session_timeout = 600  # 10 minutes

# Auto-cleanup tmux sessions
auto_cleanup = true

# Auto-use tmux for parallel/long tasks
auto_tmux = true

# Coordinator settings
[ait42.coordinator]
enabled = true
auto_select = true
```

---

## 8. Keyboard Shortcuts

### Global

| Shortcut | Action |
|----------|--------|
| `Ctrl+P` | Open command palette (agents, files) |
| `Ctrl+E` | Toggle file tree |
| `Ctrl+T` | View Tmux sessions |
| `Ctrl+D` | View diagnostics (LSP errors/warnings) |
| `Ctrl+Q` | Quit editor |
| `Esc` | Return to Normal mode |

### Normal Mode

#### Movement
| Shortcut | Action |
|----------|--------|
| `h, j, k, l` | Left, Down, Up, Right |
| `w, b` | Next word, Previous word |
| `0, $` | Start of line, End of line |
| `gg, G` | Start of file, End of file |
| `Ctrl+F, Ctrl+B` | Page down, Page up |
| `{, }` | Previous paragraph, Next paragraph |
| `%` | Jump to matching bracket |

#### Editing
| Shortcut | Action |
|----------|--------|
| `i, a` | Insert before/after cursor |
| `I, A` | Insert at line start/end |
| `o, O` | Open line below/above |
| `x` | Delete character |
| `dd` | Delete line |
| `yy` | Copy line |
| `p, P` | Paste after/before cursor |
| `u` | Undo |
| `Ctrl+R` | Redo |

#### Search
| Shortcut | Action |
|----------|--------|
| `/` | Search forward |
| `?` | Search backward |
| `n, N` | Next/Previous match |
| `*` | Search word under cursor |

#### LSP
| Shortcut | Action |
|----------|--------|
| `gd` | Go to definition |
| `gr` | Go to references |
| `K` | Hover documentation |
| `Ctrl+Space` | Trigger completion |

### Insert Mode

| Shortcut | Action |
|----------|--------|
| `Esc` | Exit to Normal mode |
| `Ctrl+W` | Delete word backward |
| `Ctrl+U` | Delete line |
| `Tab` | Accept completion / Indent |

### Visual Mode

| Shortcut | Action |
|----------|--------|
| `v` | Character-wise selection |
| `V` | Line-wise selection |
| `y` | Copy selection |
| `d` | Delete selection |
| `c` | Change selection |
| `Esc` | Cancel selection |

### Command Mode

| Command | Action |
|---------|--------|
| `:w` | Save file |
| `:q` | Quit |
| `:wq` | Save and quit |
| `:q!` | Quit without saving |
| `:e file` | Open file |
| `:bn, :bp` | Next/Previous buffer |
| `:bd` | Close buffer |
| `:buffers` | List all buffers |
| `:help` | Show help |

---

## 9. Troubleshooting

### LSP Server Not Starting

**Symptoms**: No code completion, diagnostics not showing

**Diagnosis**:
```bash
# Check if LSP server is installed
which rust-analyzer
# Expected: /usr/local/bin/rust-analyzer

# Check logs
tail -f ~/.local/share/ait42-editor/logs/lsp.log
```

**Solutions**:
1. **Install LSP server**:
   ```bash
   # Rust
   rustup component add rust-analyzer

   # TypeScript
   npm install -g typescript-language-server
   ```

2. **Restart LSP in editor**:
   ```
   :lsp-restart
   ```

3. **Check configuration**:
   ```toml
   # ~/.config/ait42-editor/config.toml
   [lsp.rust]
   command = "rust-analyzer"  # Ensure correct path
   ```

### Agent Execution Failing

**Symptoms**: Agent doesn't start, "Agent execution failed" error

**Diagnosis**:
```bash
# Check AIT42_ROOT is set
echo $AIT42_ROOT
# Expected: /path/to/AIT42

# Check tmux is installed (for parallel agents)
which tmux
# Expected: /usr/local/bin/tmux

# View agent logs
:agent-logs
```

**Solutions**:
1. **Set AIT42_ROOT**:
   ```bash
   export AIT42_ROOT=/path/to/AIT42
   echo 'export AIT42_ROOT=/path/to/AIT42' >> ~/.zshrc
   ```

2. **Install tmux** (if using parallel agents):
   ```bash
   brew install tmux
   ```

3. **Verify agent files exist**:
   ```bash
   ls $AIT42_ROOT/.claude/agents/
   # Should show: backend-developer.md, coordinator.md, etc.
   ```

### Performance Issues

**Symptoms**: Slow rendering, lag when typing, high CPU usage

**Diagnosis**:
```bash
# Check file size
ls -lh current-file.txt
# If > 100MB, consider splitting

# Check buffer count
:buffers
# If > 20 buffers, close unused ones
```

**Solutions**:
1. **Close unused buffers**:
   ```
   :bd  # Close current buffer
   :bufdo bd  # Close all buffers (be careful!)
   ```

2. **Disable LSP for large files**:
   ```
   :lsp-disable
   ```

3. **Reduce parallel agents**:
   ```toml
   # config.toml
   [ait42]
   max_parallel_agents = 3  # Reduce from 5 to 3
   ```

4. **Disable syntax highlighting** (temporary):
   ```
   :syntax off
   ```

### Editor Crashes

**Symptoms**: Editor closes unexpectedly, segfaults

**Diagnosis**:
```bash
# Check crash logs
cat ~/.local/share/ait42-editor/logs/crash.log

# Generate bug report
ait42-editor --bug-report
```

**Solutions**:
1. **Update to latest version**:
   ```bash
   brew upgrade ait42-editor
   ```

2. **Report issue** with crash log:
   - Visit: https://github.com/RenTonoduka/AIT42/issues
   - Attach: `crash.log` and `bug-report.txt`

### File Not Saving

**Symptoms**: `:w` command doesn't save file, "Permission denied" error

**Diagnosis**:
```bash
# Check file permissions
ls -l current-file.txt
# Should show: -rw-r--r-- (writable by owner)
```

**Solutions**:
1. **Fix permissions**:
   ```bash
   chmod u+w current-file.txt
   ```

2. **Save with sudo** (not recommended):
   ```
   :w !sudo tee %
   ```

3. **Save to different location**:
   ```
   :w ~/backup/file.txt
   ```

---

## 10. FAQ

### General Questions

**Q: How do I exit the editor?**

A: Press `:q` in Normal mode (or `:wq` to save and quit, `:q!` to quit without saving).

**Q: Can I use AIT42 Editor over SSH?**

A: Yes! It's terminal-based and works perfectly over SSH. Just ensure the remote server has the editor installed.

**Q: Is it similar to Vim/Neovim?**

A: Yes, it uses Vim-style modal editing. If you know Vim, you'll feel at home. The main difference is integrated AI agent support.

**Q: Does it support plugins?**

A: Phase 2 will add a Wasm-based plugin system. Currently, you can extend via custom agents.

**Q: Is it free and open source?**

A: Yes, AIT42 Editor is open source under the MIT license.

### AI Agent Questions

**Q: How many agents can run in parallel?**

A: Default is 5, configurable via `max_parallel_agents` in config.toml.

**Q: Do I need to know all 49 agent names?**

A: No! Use the **Coordinator** by just describing your task. It auto-selects the best agents.

**Q: Can I create custom agents?**

A: Yes! Add agent definitions to `$AIT42_ROOT/.claude/agents/your-agent.md` following the YAML frontmatter format.

**Q: Are agent API calls free?**

A: Agents use the Anthropic API. You need an API key and will be charged based on usage. See [Anthropic Pricing](https://www.anthropic.com/pricing).

### LSP Questions

**Q: Which languages support LSP?**

A: 15+ languages including: Rust, TypeScript/JavaScript, Python, Go, Java, C/C++, HTML/CSS, and more. See [LSP Configuration](#lsp-configuration).

**Q: Why is code completion slow?**

A: Check if your LSP server is responding:
   ```
   :lsp-status
   ```
   If slow, try restarting: `:lsp-restart`

**Q: Can I disable LSP?**

A: Yes, per buffer: `:lsp-disable` or globally in config:
   ```toml
   [lsp]
   enabled = false
   ```

### Performance Questions

**Q: What's the maximum file size supported?**

A: Theoretically unlimited (rope data structure), but performance is optimal for files < 100MB.

**Q: How much RAM does the editor use?**

A: ~50MB idle, ~150MB with 10 buffers and LSP active, ~300MB with 5 parallel agents.

**Q: Is it faster than VSCode?**

A: Startup time: Yes (<500ms vs 2-3s). Editing performance: Comparable. VSCode has more features but higher resource usage.

### Tmux Questions

**Q: Do I need tmux installed?**

A: Only if you want to run multiple agents in parallel or monitor long-running agent tasks. Single agents work fine without tmux.

**Q: How do I view tmux sessions?**

A: Press `Ctrl+T` in the editor, or use:
   ```bash
   tmux list-sessions
   ```

**Q: Can I manually attach to an agent's tmux session?**

A: Yes:
   ```bash
   tmux attach -t ait42-backend-dev-1234
   ```

---

## Appendices

### A. Command Reference

Full command list: `:help commands`

### B. Agent Capabilities

Detailed agent descriptions: See [AGENT_INTEGRATION.md](AGENT_INTEGRATION.md)

### C. Supported Languages

LSP support matrix:

| Language | LSP Server | Auto-Install |
|----------|-----------|-------------|
| Rust | rust-analyzer | ✓ |
| TypeScript | typescript-language-server | ✓ |
| JavaScript | typescript-language-server | ✓ |
| Python | pyright | ✓ |
| Go | gopls | ✓ |
| Java | jdtls | ✗ |
| C/C++ | clangd | ✗ |
| HTML | vscode-html-language-server | ✓ |
| CSS | vscode-css-language-server | ✓ |
| JSON | vscode-json-language-server | ✓ |

### D. Performance Benchmarks

Measured on MacBook Pro M1, 16GB RAM:

| Metric | Value |
|--------|-------|
| Startup Time | 347ms |
| File Load (1MB) | 23ms |
| File Load (10MB) | 154ms |
| File Load (100MB) | 876ms |
| Memory (Idle) | 48MB |
| Memory (10 buffers + LSP) | 142MB |
| LSP Completion (avg) | 67ms |
| LSP Goto Definition | 34ms |
| Render Frame (60 FPS) | 12ms |

---

**End of User Guide**

For developer documentation, see [DEVELOPER_GUIDE.md](DEVELOPER_GUIDE.md).

For contributing, see [CONTRIBUTING.md](CONTRIBUTING.md).

For agent integration, see [AGENT_INTEGRATION.md](AGENT_INTEGRATION.md).

**Support**: https://github.com/RenTonoduka/AIT42/issues

**License**: MIT
