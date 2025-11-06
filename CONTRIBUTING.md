# Contributing to AIT42-Editor

First off, thank you for considering contributing to AIT42-Editor! It's people like you that make AIT42-Editor such a great tool for the development community.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Environment Setup](#development-environment-setup)
- [Project Structure](#project-structure)
- [Development Workflow](#development-workflow)
- [Coding Standards](#coding-standards)
- [Testing Requirements](#testing-requirements)
- [Pull Request Process](#pull-request-process)
- [Issue Reporting Guidelines](#issue-reporting-guidelines)
- [Community](#community)

---

## Code of Conduct

This project and everyone participating in it is governed by our commitment to creating a welcoming and inclusive environment. By participating, you are expected to uphold this code:

- **Be respectful**: Treat everyone with respect. Disagreements are fine, but be constructive.
- **Be inclusive**: Welcome newcomers and help them get started.
- **Be collaborative**: Share knowledge and credit others for their contributions.
- **Be patient**: Remember that everyone was a beginner once.

If you experience or witness unacceptable behavior, please report it to support@ait42.dev.

---

## Getting Started

### Prerequisites

Before you begin, ensure you have the following installed:

- **Node.js** 20.0 or higher
- **Rust** 1.75 or higher (with `cargo` and `rustup`)
- **Git** 2.40 or higher
- **Tmux** 3.3 or higher (for testing debate mode)
- **Claude Code CLI** (optional, for testing AI workflows)

### Quick Setup

```bash
# Fork the repository on GitHub
# Clone your fork
git clone https://github.com/YOUR_USERNAME/AIT42-Editor
cd AIT42-Editor

# Add upstream remote
git remote add upstream https://github.com/RenTonoduka/AIT42-Editor

# Install dependencies
npm install

# Set up environment variables
cp .env.example .env
# Edit .env and add your ANTHROPIC_API_KEY

# Run tests to verify setup
cargo test
npm test

# Start development server
npm run tauri dev
```

---

## Development Environment Setup

### Rust Development

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install required components
rustup component add rustfmt clippy

# Verify installation
rustc --version  # Should be 1.75 or higher
cargo --version
```

### Node.js Development

```bash
# Install Node.js 20+ (using nvm)
nvm install 20
nvm use 20

# Verify installation
node --version  # Should be 20.0 or higher
npm --version
```

### IDE Setup

#### VS Code (Recommended)

Install the following extensions:

- **rust-analyzer**: Rust language support
- **Tauri**: Tauri development tools
- **ESLint**: TypeScript/JavaScript linting
- **Prettier**: Code formatting
- **React Developer Tools**: React debugging

Settings (`.vscode/settings.json`):
```json
{
  "rust-analyzer.cargo.features": ["all"],
  "editor.formatOnSave": true,
  "editor.defaultFormatter": "esbenp.prettier-vscode",
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  }
}
```

#### IntelliJ IDEA / CLion

Install plugins:
- **Rust**
- **Tauri**
- **ESLint**
- **Prettier**

---

## Project Structure

```
AIT42-Editor/
├── src/                           # React frontend (TypeScript)
│   ├── components/
│   │   ├── AI/                    # AI workflow components
│   │   ├── Optimizer/             # v1.6.0 complexity analysis UI
│   │   ├── Editor/                # Code editor components
│   │   ├── Sidebar/               # File tree
│   │   └── Settings/              # Settings panel
│   ├── services/
│   │   └── tauri.ts               # Tauri IPC wrappers
│   ├── store/                     # Zustand state management
│   └── App.tsx                    # Main application
├── src-tauri/                     # Rust backend
│   ├── src/
│   │   ├── commands/              # Tauri commands (IPC)
│   │   │   ├── ait42.rs           # AI agent orchestration
│   │   │   ├── optimizer.rs       # v1.6.0 optimizer IPC
│   │   │   ├── file.rs            # File operations
│   │   │   ├── editor.rs          # Editor operations
│   │   │   ├── git.rs             # Git operations
│   │   │   └── lsp.rs             # LSP operations
│   │   ├── optimizer/             # v1.6.0 optimizer logic
│   │   │   ├── mod.rs
│   │   │   ├── subtask.rs         # SubtaskOptimizer
│   │   │   └── instance.rs        # InstanceCalculator
│   │   ├── ab_test/               # v1.6.0 A/B testing
│   │   ├── state.rs               # Application state
│   │   └── main.rs                # Entry point
│   └── Cargo.toml
├── crates/                        # Rust workspace crates
│   ├── ait42-core/                # Core editor logic
│   ├── ait42-tui/                 # TUI components
│   ├── ait42-lsp/                 # LSP client
│   ├── ait42-fs/                  # File system operations
│   ├── ait42-config/              # Configuration management
│   ├── ait42-ait42/               # AI agent integration
│   ├── omega-theory/              # v1.6.0 Ω-theory engine
│   └── llm-estimator/             # v1.6.0 LLM estimation
├── docs/                          # Documentation
│   ├── design/phase1/             # Debate Mode design docs
│   ├── OMEGA_THEORY_EXPLAINED.md  # Ω-theory deep dive
│   ├── AB_TESTING_RESULTS.md      # A/B test analysis
│   └── TROUBLESHOOTING.md         # Common issues
├── tests/                         # E2E tests (Playwright)
├── .github/                       # GitHub Actions CI/CD
├── README.md                      # User documentation
├── CHANGELOG.md                   # Version history
├── CONTRIBUTING.md                # This file
└── LICENSE                        # MIT License
```

---

## Development Workflow

### 1. Pick an Issue

- Browse [open issues](https://github.com/RenTonoduka/AIT42-Editor/issues)
- Comment on the issue to claim it
- Wait for maintainer approval before starting work
- For new features, create an issue first to discuss the proposal

### 2. Create a Branch

```bash
# Update your fork
git checkout main
git pull upstream main

# Create a feature branch
git checkout -b feature/your-feature-name

# Or for bug fixes
git checkout -b fix/bug-description
```

### 3. Make Changes

Follow our [Coding Standards](#coding-standards) and write tests for your changes.

### 4. Test Your Changes

```bash
# Rust tests
cargo test
cargo clippy  # Linting
cargo fmt     # Formatting

# Frontend tests
npm test
npm run lint
npm run format

# E2E tests
npm run test:e2e

# Manual testing
npm run tauri dev
```

### 5. Commit Your Changes

We use [Conventional Commits](https://www.conventionalcommits.org/) for commit messages:

```bash
# Format: <type>(<scope>): <subject>

# Examples:
git commit -m "feat(optimizer): add memory-based adjustment"
git commit -m "fix(debate): resolve worktree cleanup race condition"
git commit -m "docs(api): update optimize_task documentation"
git commit -m "test(llm-estimator): add cache expiration tests"
git commit -m "refactor(ui): extract ComplexityBadge component"
```

**Commit Types**:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `test`: Adding or updating tests
- `refactor`: Code refactoring (no functional changes)
- `perf`: Performance improvements
- `style`: Code style changes (formatting, etc.)
- `chore`: Maintenance tasks (dependencies, build, etc.)

### 6. Push and Create PR

```bash
# Push to your fork
git push origin feature/your-feature-name

# Create PR on GitHub
# Fill out the PR template completely
# Link related issues
```

---

## Coding Standards

### Rust Style Guide

#### Formatting

```bash
# Format all Rust code before committing
cargo fmt
```

#### Linting

```bash
# Run Clippy with strict linting
cargo clippy -- -D warnings
```

#### Naming Conventions

- **Modules**: `snake_case` (e.g., `optimizer`, `ab_test`)
- **Structs/Enums**: `PascalCase` (e.g., `SubtaskOptimizer`, `ComplexityClass`)
- **Functions**: `snake_case` (e.g., `optimize_subtask_count`)
- **Constants**: `SCREAMING_SNAKE_CASE` (e.g., `MAX_INSTANCES`)

#### Documentation

```rust
/// Optimize subtask count for a given task description.
///
/// # Arguments
/// * `task_description` - Task description to analyze
/// * `current_subtasks` - Current number of subtasks (0 if none)
///
/// # Returns
/// * `Ok(OptimizationResult)` - Optimization recommendation
/// * `Err(OptimizerError)` - Error during optimization
///
/// # Example
/// ```
/// let result = optimizer.optimize_subtask_count("Implement REST API", 0).await?;
/// assert_eq!(result.complexity_class, ComplexityClass::Linear);
/// ```
pub async fn optimize_subtask_count(
    &self,
    task_description: &str,
    current_subtasks: usize,
) -> Result<OptimizationResult, OptimizerError> {
    // Implementation
}
```

#### Error Handling

```rust
// Use thiserror for custom error types
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OptimizerError {
    #[error("LLM estimation failed: {0}")]
    EstimationFailed(#[from] EstimationError),

    #[error("Operation timed out after {0:?}")]
    Timeout(Duration),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

// Propagate errors with ?
let result = self.estimator.estimate_complexity(task).await?;
```

### TypeScript Style Guide

#### Formatting

```bash
# Format TypeScript code
npm run format
```

#### Linting

```bash
# Run ESLint
npm run lint

# Auto-fix issues
npm run lint -- --fix
```

#### Naming Conventions

- **Components**: `PascalCase` (e.g., `ComplexityBadge`, `TaskAnalyzer`)
- **Functions/Variables**: `camelCase` (e.g., `optimizeTask`, `instanceCount`)
- **Constants**: `SCREAMING_SNAKE_CASE` (e.g., `MAX_CONFIDENCE`)
- **Interfaces**: `PascalCase` with `I` prefix (e.g., `IOptimizerProps`)

#### React Components

```typescript
// Functional components with TypeScript
interface ComplexityBadgeProps {
  complexity: string;
  subtaskRange: string;
  className?: string;
}

export const ComplexityBadge: React.FC<ComplexityBadgeProps> = ({
  complexity,
  subtaskRange,
  className = '',
}) => {
  // Implementation
  return (
    <div className={`badge ${className}`}>
      <span>{complexity}</span>
      <span>{subtaskRange}</span>
    </div>
  );
};
```

#### Tauri IPC

```typescript
// Type-safe Tauri IPC calls
interface OptimizeTaskRequest {
  taskDescription: string;
  currentSubtasks: number;
}

interface OptimizeTaskResponse {
  complexityClass: string;
  recommendedSubtasks: number;
  confidence: number;
  reasoning: string;
}

export async function optimizeTask(
  taskDescription: string,
  currentSubtasks: number
): Promise<OptimizeTaskResponse> {
  return invoke<OptimizeTaskResponse>('optimize_task', {
    taskDescription,
    currentSubtasks,
  });
}
```

---

## Testing Requirements

### Rust Tests

#### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complexity_class_parsing() {
        assert_eq!(
            parse_complexity_class("Linear").unwrap(),
            ComplexityClass::Linear
        );
    }

    #[tokio::test]
    async fn test_optimize_subtask_count() {
        let optimizer = SubtaskOptimizer::from_env().unwrap();
        let result = optimizer
            .optimize_subtask_count("Implement REST API", 0)
            .await
            .unwrap();

        assert_eq!(result.complexity_class, ComplexityClass::Linear);
        assert!(result.recommended_subtasks >= 3 && result.recommended_subtasks <= 5);
    }
}
```

#### Integration Tests

```rust
// tests/integration_tests.rs
#[tokio::test]
async fn test_end_to_end_optimization() {
    let optimizer = SubtaskOptimizer::from_env().unwrap();
    let calculator = InstanceCalculator::new();

    let result = optimizer
        .optimize_subtask_count("Build e-commerce platform", 0)
        .await
        .unwrap();

    let instances = calculator.calculate_instances(
        result.complexity_class,
        result.recommended_subtasks,
    );

    assert!(instances.recommended_instances <= 10);
}
```

### TypeScript Tests (Jest)

```typescript
// src/components/Optimizer/ComplexityBadge.test.tsx
import { render, screen } from '@testing-library/react';
import { ComplexityBadge } from './ComplexityBadge';

describe('ComplexityBadge', () => {
  it('renders complexity class correctly', () => {
    render(<ComplexityBadge complexity="Linear" subtaskRange="3-5" />);
    expect(screen.getByText('Linear')).toBeInTheDocument();
    expect(screen.getByText('3-5')).toBeInTheDocument();
  });

  it('applies correct color for Linear complexity', () => {
    const { container } = render(
      <ComplexityBadge complexity="Linear" subtaskRange="3-5" />
    );
    expect(container.firstChild).toHaveClass('bg-green-100');
  });
});
```

### E2E Tests (Playwright)

```typescript
// tests/e2e/optimizer.spec.ts
import { test, expect } from '@playwright/test';

test('task optimization workflow', async ({ page }) => {
  await page.goto('http://localhost:1420/optimizer');

  // Enter task description
  await page.fill('textarea[name="taskDescription"]', 'Implement REST API');

  // Click analyze button
  await page.click('button:has-text("Analyze Task")');

  // Wait for results
  await page.waitForSelector('.complexity-badge');

  // Verify results
  const complexity = await page.textContent('.complexity-badge');
  expect(complexity).toContain('Linear');
});
```

### Test Coverage Requirements

- **Minimum Coverage**: 80% for new code
- **Critical Paths**: 100% coverage for:
  - Optimizer logic (SubtaskOptimizer, InstanceCalculator)
  - LLM estimation (ComplexityEstimator, AnthropicClient)
  - Tauri IPC commands
- **Run Coverage**:
  ```bash
  cargo tarpaulin --out Html  # Rust coverage
  npm run test:coverage        # TypeScript coverage
  ```

---

## Pull Request Process

### Before Submitting

1. **Rebase on Main**
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

2. **Run All Tests**
   ```bash
   cargo test
   npm test
   npm run test:e2e
   ```

3. **Check Code Quality**
   ```bash
   cargo clippy -- -D warnings
   cargo fmt --check
   npm run lint
   npm run format
   ```

4. **Update Documentation**
   - Update README.md if user-facing features changed
   - Update API_REFERENCE.md for new Tauri commands
   - Add JSDoc/RustDoc comments for new functions

### PR Template

```markdown
## Description
Brief description of what this PR does.

## Related Issue
Closes #123

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
Describe tests added/updated.

## Checklist
- [ ] Code follows project style guidelines
- [ ] Tests added/updated and passing
- [ ] Documentation updated
- [ ] Commit messages follow Conventional Commits
- [ ] No merge conflicts with main
```

### Review Process

1. **Automated Checks**: CI/CD will run tests and linting
2. **Code Review**: At least 1 maintainer approval required
3. **Feedback**: Address review comments promptly
4. **Approval**: Maintainer will merge once approved

---

## Issue Reporting Guidelines

### Bug Reports

Use the bug report template:

```markdown
**Describe the bug**
A clear description of what the bug is.

**To Reproduce**
Steps to reproduce:
1. Go to '...'
2. Click on '...'
3. See error

**Expected behavior**
What you expected to happen.

**Screenshots**
If applicable, add screenshots.

**Environment**
- OS: [e.g., macOS 14.0]
- AIT42-Editor version: [e.g., 1.6.0]
- Node.js version: [e.g., 20.10.0]
- Rust version: [e.g., 1.75.0]

**Additional context**
Any other relevant information.
```

### Feature Requests

Use the feature request template:

```markdown
**Is your feature request related to a problem?**
A clear description of the problem.

**Describe the solution you'd like**
A clear description of what you want to happen.

**Describe alternatives you've considered**
Other solutions you've thought about.

**Additional context**
Mockups, examples, or other context.
```

---

## Community

### Communication Channels

- **GitHub Issues**: Bug reports, feature requests
- **GitHub Discussions**: Questions, ideas, general discussion
- **Email**: support@ait42.dev (for private matters)

### Getting Help

- **Documentation**: Start with [README.md](README.md) and [USER_GUIDE.md](USER_GUIDE.md)
- **Troubleshooting**: Check [TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md)
- **API Reference**: See [API_REFERENCE.md](API_REFERENCE.md)
- **Architecture**: Read [ARCHITECTURE.md](ARCHITECTURE.md)

### Recognition

Contributors will be:
- Listed in CHANGELOG.md for each release
- Mentioned in release notes
- Added to GitHub contributors page

---

## Thank You!

Thank you for taking the time to contribute to AIT42-Editor. Every contribution, no matter how small, is valuable and appreciated. Together, we're building a better tool for the development community!

---

**Questions?** Open a [GitHub Discussion](https://github.com/RenTonoduka/AIT42-Editor/discussions) or email support@ait42.dev.
