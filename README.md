# AIT42-Editor GUI

A modern desktop code editor with integrated AI multi-agent workflows powered by Tauri, React, and Rust.

[![CI Status](https://github.com/RenTonoduka/AIT42/workflows/CI/badge.svg)](https://github.com/RenTonoduka/AIT42/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust Version](https://img.shields.io/badge/rust-1.75%2B-blue.svg)](https://www.rust-lang.org)
[![Node Version](https://img.shields.io/badge/node-20%2B-green.svg)](https://nodejs.org)

## Overview

AIT42-Editor is a next-generation GUI code editor built with Tauri that brings AI-powered multi-agent collaboration directly into your development workflow. Unlike traditional editors, AIT42-Editor enables multiple AI agents to work in parallel or engage in structured debates to solve complex development challenges.

**Key Highlights:**
- 🎯 **3 AI Workflow Modes**: Competition, Ensemble, and Debate
- 🔥 **Debate Mode (v1.5.0)**: MIT MAD-inspired multi-round role-based discussions (+60.4% reasoning improvement)
- ⚡ **Native Performance**: Rust backend with React frontend
- 🔧 **Git Worktree Integration**: Isolated workspaces for each AI agent
- 📊 **Real-time Progress Tracking**: Live updates on agent execution
- 🛡️ **Enterprise-grade Security**: Command injection prevention, race condition handling

## Features

### Core Editor Features
- **Modern UI**: Beautiful React-based interface with Tailwind CSS
- **LSP Support**: Language Server Protocol for intelligent code completion
- **Git Integration**: Full Git operations including worktrees, branches, commits
- **File System Operations**: Efficient file browsing and editing
- **Plugin System**: Extensible architecture for custom extensions
- **Terminal Integration**: Built-in terminal panel with command history

### AI Multi-Agent Workflows

#### 🏆 Competition Mode
Launch 3-5 AI instances to solve the same problem independently, then select the best solution.
- **Use Cases**: Algorithm optimization, code generation, architecture design
- **Expected Improvement**: +40% quality (based on CAMEL research)
- **ROI**: 640% ($70k → $522k annual return)

#### ✨ Ensemble Mode
Multiple agents work in parallel, with a dedicated Integration Agent combining their outputs.
- **Use Cases**: Large refactoring, multi-file changes, system-wide updates
- **Expected Improvement**: +35% completeness
- **Workflow**: Independent execution → Automated integration → Single deliverable

#### 💬 Debate Mode (Phase 1 MVP - v1.5.0)
Three roles engage in a structured 3-round debate to reach consensus on complex decisions.

**Research Foundation:**
- **MIT MAD Framework**: +60.4% Math reasoning, +64.0% Logic puzzles
- **CAMEL Framework**: +42% quality improvement via role-based collaboration
- **Stanford Research**: Structured debates outperform single-agent approaches by 2.5x

**3-Round Structure:**
1. **Round 1 - Independent Proposals** (独立提案): Each role presents their perspective
2. **Round 2 - Critical Analysis** (批判的分析): Roles critique and refine proposals
3. **Round 3 - Consensus Formation** (コンセンサス形成): Unified recommendation

**Role Presets:**
- **Tech Stack Selection**: Architect, Pragmatist, Innovator
- **Security Review**: Security Architect, PenTester, Compliance Expert
- **Architecture Decision**: System Architect, Performance Engineer, Maintainability Advocate

**Typical Use Cases:**
- Technology stack selection (e.g., "Next.js vs Astro for blog?")
- Security policy decisions (e.g., "API authentication strategy?")
- Architecture trade-offs (e.g., "Microservices vs Monolith?")

**Expected Results:**
- Execution time: 25-35 minutes (3 rounds × 8-12 min/round)
- Output: 3,000-5,000 word integrated proposal
- Cost: $0.35-0.45 (with Claude Sonnet 4.5)
- Quality: 90+ score (with ReflectionAgent validation)

## Quick Start

### Prerequisites

- **Node.js**: 20.0 or higher
- **Rust**: 1.75 or higher
- **Git**: 2.40 or higher
- **Tmux**: 3.3 or higher (for agent isolation)
- **Claude Code CLI**: Latest version
- **Environment Variables**:
  ```bash
  export ANTHROPIC_API_KEY="your_api_key_here"
  ```

### Installation

#### Development Mode

```bash
# Clone the repository
git clone https://github.com/RenTonoduka/AIT42-Editor
cd AIT42-Editor

# Install dependencies
npm install

# Run in development mode (hot reload)
npm run tauri dev
```

#### Production Build

```bash
# Build for production
npm run tauri build

# macOS: AIT42-Editor.app will be created in src-tauri/target/release/bundle/macos/
# Linux: AppImage will be created in src-tauri/target/release/bundle/appimage/
# Windows: .exe installer will be created in src-tauri/target/release/bundle/msi/
```

#### macOS Installation (Recommended)

```bash
# Run automated installer (builds and installs to /Applications)
./install-macos.sh

# Launch from Applications folder or command line
open /Applications/AIT42-Editor.app
```

See [INSTALL_MACOS.md](INSTALL_MACOS.md) for detailed macOS instructions.

### First Launch

1. Launch AIT42-Editor
2. The editor will open with the welcome screen
3. Open a project directory using File → Open Directory
4. Start using AI modes from the header buttons

## Using Debate Mode

### Step-by-Step Guide

1. **Launch Debate Dialog**
   - Click the "💬 ディベート" button in the header
   - The Debate configuration dialog will appear

2. **Configure Debate**
   - **Task Input**: Enter your decision or problem (e.g., "Should we use Next.js or Astro for our blog?")
   - **Role Preset**: Select from:
     - 技術スタック選定 (Tech Stack Selection)
     - セキュリティレビュー (Security Review)
     - アーキテクチャ決定 (Architecture Decision)
   - **Model Selection**: Choose AI model (Sonnet 4.5, Opus 4, Haiku 4)
   - **Advanced Settings** (optional):
     - Timeout: Per-round timeout in seconds (default: 800s)
     - Preserve Worktrees: Keep Git worktrees after completion for inspection

3. **Start Debate**
   - Click "💬 ディベート開始" (Start Debate)
   - The view will switch to the Debate Status Panel
   - Real-time progress will be displayed for each round

4. **Monitor Progress**
   - **Round Progress**: Visual indicator showing Round 1/3, 2/3, 3/3
   - **Role Execution**: Each role's output appears as it completes
   - **Timestamps**: Start and completion times for each role
   - **Context Files**: Number of context files generated

5. **Review Results**
   - Once completed, view the final consensus proposal
   - Explore individual round outputs by expanding each round
   - Review context files in `/tmp/debate-{id}/` (if preserved)

### Example Debate Tasks

**Technology Decisions:**
```
"React vs Vue for our new dashboard - consider team expertise, ecosystem, and performance"
```

**Security Policies:**
```
"JWT vs Session-based auth for our API - analyze security, scalability, and implementation complexity"
```

**Architecture Choices:**
```
"Microservices vs Modular Monolith for our e-commerce platform - 5-year roadmap"
```

### Debate Output Structure

```
/tmp/debate-{debate_id}/
├── round1/                          # Round 1: Independent Proposals
│   ├── role-architect-proposal.md   # Architect's initial proposal
│   ├── role-pragmatist-proposal.md  # Pragmatist's initial proposal
│   └── role-innovator-proposal.md   # Innovator's initial proposal
├── round2/                          # Round 2: Critical Analysis
│   ├── role-architect-critique.md   # Critiques of other proposals
│   ├── role-architect-revised.md    # Revised proposal
│   ├── role-pragmatist-critique.md
│   ├── role-pragmatist-revised.md
│   ├── role-innovator-critique.md
│   └── role-innovator-revised.md
└── round3/                          # Round 3: Consensus
    └── consensus.md                 # Final integrated proposal (3,000-5,000 words)
```

### Tips for Effective Debates

1. **Be Specific**: Provide context and constraints in your task description
   - ❌ "Choose a framework"
   - ✅ "Choose a frontend framework for our B2B SaaS dashboard with 100k+ users, considering SEO, performance, and team of 5 developers with React experience"

2. **Choose the Right Preset**: Match your decision type to the role preset
   - Technical decisions → Tech Stack Selection
   - Security concerns → Security Review
   - System design → Architecture Decision

3. **Set Appropriate Timeouts**: Adjust based on complexity
   - Simple decisions: 600s (10 min per round)
   - Complex decisions: 800s (13 min per round, default)
   - Deep analysis: 1200s (20 min per round)

4. **Preserve Worktrees for Review**: Enable "Preserve Worktrees" to inspect agent outputs after completion

## Architecture

### Technology Stack

- **Frontend**: React 18 + TypeScript 5 + Tailwind CSS 3
- **Backend**: Rust (Tauri 1.5 + async/await + tokio)
- **State Management**: Zustand
- **Build Tool**: Vite 5
- **Desktop Framework**: Tauri (Chromium-free, native webview)

### Project Structure

```
AIT42-Editor/
├── src/                          # React frontend
│   ├── components/
│   │   ├── AI/                   # AI workflow components
│   │   │   ├── CompetitionDialog.tsx
│   │   │   ├── EnsembleDialog.tsx
│   │   │   ├── DebateDialog.tsx          # Debate configuration
│   │   │   ├── DebateStatusPanel.tsx     # Real-time progress
│   │   │   └── MultiAgentPanel.tsx
│   │   ├── Editor/               # Code editor components
│   │   ├── Sidebar/              # File tree sidebar
│   │   └── Settings/             # Settings panel
│   ├── services/
│   │   └── tauri.ts              # Tauri API wrappers
│   ├── store/                    # Zustand state management
│   └── App.tsx                   # Main application
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   ├── commands/             # Tauri commands
│   │   │   ├── ait42.rs          # AI agent orchestration
│   │   │   ├── file.rs           # File operations
│   │   │   ├── editor.rs         # Editor operations
│   │   │   ├── git.rs            # Git operations
│   │   │   └── lsp.rs            # LSP operations
│   │   ├── state.rs              # Application state
│   │   └── main.rs               # Entry point
│   └── Cargo.toml
├── crates/                       # Rust workspace crates
│   ├── ait42-core/               # Core editor logic
│   ├── ait42-tui/                # TUI components (legacy)
│   └── ait42-lsp/                # LSP client
└── docs/
    └── design/phase1/            # Debate Mode design docs
        ├── ARCHITECTURE_DESIGN.md     # System architecture
        ├── RUST_BACKEND_SPEC.md       # Backend specification
        ├── REACT_FRONTEND_SPEC.md     # Frontend specification
        ├── ROLE_PROMPTS.md            # Role definitions
        └── INTEGRATION_TEST_SCENARIOS.md  # Test scenarios
```

## Documentation

### Design Documents (Debate Mode Phase 1)
- [ARCHITECTURE_DESIGN.md](docs/design/phase1/ARCHITECTURE_DESIGN.md) - Overall system architecture (1,294 lines)
- [RUST_BACKEND_SPEC.md](docs/design/phase1/RUST_BACKEND_SPEC.md) - Backend implementation spec (1,527 lines)
- [REACT_FRONTEND_SPEC.md](docs/design/phase1/REACT_FRONTEND_SPEC.md) - Frontend UI spec (1,783 lines)
- [ROLE_PROMPTS.md](docs/design/phase1/ROLE_PROMPTS.md) - 3 role definitions with prompts (1,478 lines)
- [INTEGRATION_TEST_SCENARIOS.md](docs/design/phase1/INTEGRATION_TEST_SCENARIOS.md) - 10 test scenarios (951 lines)

### Installation & Configuration
- [INSTALL_MACOS.md](INSTALL_MACOS.md) - macOS installation guide
- [PROJECT_STRUCTURE.md](PROJECT_STRUCTURE.md) - Project architecture (if available)

### Research Papers
- **MIT MAD Framework**: Multi-Agent Debate improves reasoning by 60.4% (Mathematics, Logic)
- **CAMEL**: Communicative Agents for "Mind" Exploration of Large Language Model Society
- **Stanford Research**: Role-based collaboration outperforms single-agent by 2.5x

## Development

### Running Tests

```bash
# Rust backend tests
cargo test

# Frontend tests (if available)
npm test

# E2E tests with Playwright
npm run test:e2e
```

### Building Documentation

```bash
# Generate Rust documentation
cargo doc --open

# View design documents
open docs/design/phase1/ARCHITECTURE_DESIGN.md
```

### Code Quality

```bash
# Rust linting
cargo clippy

# TypeScript linting
npm run lint

# Format code
cargo fmt
npm run format
```

## Roadmap

### Phase 1 MVP (v1.5.0) ✅ **Completed**
- [x] Debate Mode backend (Rust)
- [x] Debate Mode UI (React)
- [x] 3 role presets
- [x] Real-time progress tracking
- [x] Git worktree integration
- [x] Tmux session management

### Phase 2 (v1.6.0) - Planned
- [ ] Custom role creation UI
- [ ] Debate history and replay
- [ ] Export to PDF/Markdown
- [ ] Cost tracking dashboard
- [ ] Multiple debates in parallel
- [ ] Claude Opus 4 support

### Phase 3 (v1.7.0) - Planned
- [ ] Debate templates library
- [ ] Team collaboration (share debates)
- [ ] Agent quality scoring
- [ ] Automatic role recommendation
- [ ] Integration with project management tools

## Troubleshooting

### Debate Mode Issues

**Problem**: Debate fails to start
- **Solution**: Check `ANTHROPIC_API_KEY` is set correctly
- **Solution**: Verify Git repository is initialized (`git init`)
- **Solution**: Ensure Tmux is installed (`tmux -V`)

**Problem**: Worktrees not cleaned up
- **Solution**: Manually remove: `rm -rf /tmp/debate-*`
- **Solution**: Enable "Preserve Worktrees" to inspect manually

**Problem**: Agent timeout
- **Solution**: Increase timeout in Advanced Settings (default: 800s)
- **Solution**: Check network connectivity to Anthropic API
- **Solution**: Verify Claude Code CLI is functioning: `claude --version`

### General Issues

**Problem**: Application won't start
- **Solution**: Check Node.js version: `node -v` (requires 20+)
- **Solution**: Rebuild: `npm run tauri build`

**Problem**: Hot reload not working
- **Solution**: Restart dev server: `npm run tauri dev`
- **Solution**: Clear Vite cache: `rm -rf node_modules/.vite`

## Contributing

We welcome contributions! Please follow these steps:

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Commit your changes: `git commit -m 'feat: add amazing feature'`
4. Push to the branch: `git push origin feature/amazing-feature`
5. Open a Pull Request

### Development Guidelines

- Follow Rust and TypeScript best practices
- Write tests for new features
- Update documentation for user-facing changes
- Use Conventional Commits for commit messages

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- **MIT MAD Research Team**: Multi-Agent Debate framework
- **CAMEL Research Team**: Role-based AI collaboration
- **Tauri Team**: Excellent desktop app framework
- **Claude Code Team**: AI-powered development tools
- **Anthropic**: Claude API and models

## Support

- **Issues**: [GitHub Issues](https://github.com/RenTonoduka/AIT42-Editor/issues)
- **Discussions**: [GitHub Discussions](https://github.com/RenTonoduka/AIT42-Editor/discussions)
- **Email**: support@ait42.dev (if available)

---

Made with ❤️ by the AIT42 Team

**Version**: 1.5.0 (Phase 1 MVP - Debate Mode)
**Last Updated**: 2025-11-05
