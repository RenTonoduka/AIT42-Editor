# ait42-ait42

AIT42 Agent Integration System for the AIT42 Editor.

## Overview

This crate provides complete integration with the AIT42 agent system, enabling the editor to orchestrate 49 specialized AI agents for various development tasks.

## Features

- **Agent Registry**: Discover and manage 49 AI agents
- **Tmux Session Management**: Run agents in isolated tmux sessions
- **Coordinator**: Intelligent agent selection and orchestration
- **Executor**: Single, parallel, and sequential execution modes
- **Output Streaming**: Real-time output from running agents
- **Command Palette**: Integration with editor commands
- **Editor Bridge**: Run agents on buffer content and selections

## Quick Start

```rust
use ait42_ait42::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration
    let config = AIT42Config::load()?;

    // Create coordinator and executor
    let coordinator = Coordinator::new(config)?;
    let mut executor = AgentExecutor::new(coordinator);

    // Execute a task
    let results = executor
        .execute_single("backend-developer", "Implement user authentication")
        .await?;

    println!("Output: {}", results.output);
    Ok(())
}
```

## Available Agents (49)

### Backend (13)
- backend-developer, api-developer, api-designer, database-developer, database-designer, integration-developer, migration-developer, script-writer

### Frontend (1)
- frontend-developer

### Testing (7)
- test-generator, integration-tester, performance-tester, security-tester, mutation-tester, qa-validator, bug-fixer

### Documentation (2)
- tech-writer, doc-reviewer

### Security (4)
- security-architect, security-scanner, security-tester

### Infrastructure (7)
- devops-engineer, cicd-manager, container-specialist, cloud-architect, monitoring-specialist, backup-manager, chaos-engineer

### Coordination (2)
- coordinator, workflow-coordinator

### Planning (7)
- system-architect, ui-ux-designer, integration-planner, requirements-elicitation

### QA (4)
- code-reviewer, refactor-specialist, complexity-analyzer, feedback-analyzer

### Operations (9)
- incident-responder, release-manager, config-manager

### Meta (5)
- process-optimizer, learning-agent, metrics-collector, knowledge-manager, innovation-scout

## Execution Modes

### Single Agent

```rust
let result = executor.execute_single("backend-developer", "Create API").await?;
```

### Parallel Execution

```rust
let results = executor.execute(
    ExecutionMode::Parallel(vec![
        "api-designer".to_string(),
        "database-designer".to_string(),
    ]),
    "Design e-commerce system"
).await?;
```

### Sequential Pipeline

```rust
let results = executor.execute(
    ExecutionMode::Sequential(vec![
        "api-designer".to_string(),
        "backend-developer".to_string(),
        "code-reviewer".to_string(),
    ]),
    "Create user management"
).await?;
```

### Coordinated (Auto-select)

```rust
let results = executor.execute(
    ExecutionMode::Coordinated,
    "Implement authentication with tests"
).await?;
```

## Editor Integration

```rust
use ait42_ait42::editor_integration::*;

let mut bridge = EditorAgentBridge::new(executor);

// Review code
let review = bridge.review_buffer(&buffer).await?;

// Generate tests
let tests = bridge.generate_tests(&buffer).await?;

// Refactor selection
let refactored = bridge.refactor_selection(&selection, &buffer).await?;

// Security scan
let scan = bridge.security_scan(&buffer).await?;
```

## Configuration

### Environment Variables

```bash
export AIT42_ROOT=/path/to/ait42
export ANTHROPIC_API_KEY=sk-ant-xxxxx
export AIT42_MAX_PARALLEL=3
export AIT42_TIMEOUT=600
```

### Required Directory Structure

```
AIT42/
├── .claude/
│   └── agents/
│       ├── backend-developer.md
│       └── ... (48 more)
└── scripts/
    ├── tmux-single-agent.sh
    └── tmux-parallel-agents.sh
```

## Documentation

See [AIT42_INTEGRATION_REPORT.md](../../AIT42_INTEGRATION_REPORT.md) for comprehensive documentation.

## Testing

```bash
# Run all tests
cargo test -p ait42-ait42

# Run with output
cargo test -p ait42-ait42 -- --nocapture

# Run specific test
cargo test -p ait42-ait42 test_agent_registry
```

## Dependencies

- tokio (async runtime)
- serde (serialization)
- tracing (logging)
- thiserror (error handling)

## License

MIT OR Apache-2.0
