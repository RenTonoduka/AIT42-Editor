//! Advanced usage example demonstrating parallel and sequential execution

use ait42_ait42::prelude::*;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    println!("=== AIT42 Advanced Usage Examples ===\n");

    // Setup
    let config = AIT42Config::load()?;
    let coordinator = Coordinator::new(config)?;
    let mut executor = AgentExecutor::new(coordinator);

    // Example 1: Single Agent Execution
    println!("Example 1: Single Agent Execution");
    println!("----------------------------------");
    example_single_agent(&mut executor).await?;

    // Example 2: Parallel Execution
    println!("\nExample 2: Parallel Execution");
    println!("-----------------------------");
    example_parallel_execution(&mut executor).await?;

    // Example 3: Sequential Pipeline
    println!("\nExample 3: Sequential Pipeline");
    println!("------------------------------");
    example_sequential_pipeline(&mut executor).await?;

    // Example 4: Coordinated Auto-Selection
    println!("\nExample 4: Coordinated Auto-Selection");
    println!("--------------------------------------");
    example_coordinated(&mut executor).await?;

    // Example 5: Editor Integration
    println!("\nExample 5: Editor Integration");
    println!("-----------------------------");
    example_editor_integration(&mut executor).await?;

    println!("\n=== All Examples Complete ===");

    Ok(())
}

async fn example_single_agent(executor: &mut AgentExecutor) -> Result<()> {
    println!("Task: Review code quality\n");

    let mode = ExecutionMode::Single("code-reviewer".to_string());
    let task = "Review this authentication implementation for security issues";

    println!("Selected: code-reviewer");
    println!("Mode: Single execution");
    println!("Status: Would execute in tmux session (skipped in example)\n");

    // In real usage:
    // let results = executor.execute(mode, task).await?;
    // for result in results {
    //     println!("Output: {}", result.output);
    // }

    Ok(())
}

async fn example_parallel_execution(executor: &mut AgentExecutor) -> Result<()> {
    println!("Task: Design e-commerce system\n");

    let agents = vec![
        "api-designer".to_string(),
        "database-designer".to_string(),
        "ui-ux-designer".to_string(),
    ];

    println!("Selected agents (parallel):");
    for agent in &agents {
        println!("  - {}", agent);
    }

    println!("\nExecution flow:");
    println!("  1. Start all 3 agents simultaneously in separate tmux sessions");
    println!("  2. Each agent works independently");
    println!("  3. Wait for all to complete");
    println!("  4. Aggregate results\n");

    println!("Status: Would execute in tmux sessions (skipped in example)\n");

    // In real usage:
    // let mode = ExecutionMode::Parallel(agents);
    // let results = executor.execute(mode, "Design e-commerce system").await?;
    // for result in results {
    //     println!("{}: completed in {:?}", result.agent_name, result.duration);
    // }

    Ok(())
}

async fn example_sequential_pipeline(executor: &mut AgentExecutor) -> Result<()> {
    println!("Task: Create user management system with full QA\n");

    let pipeline = vec![
        "api-designer".to_string(),
        "backend-developer".to_string(),
        "test-generator".to_string(),
        "code-reviewer".to_string(),
    ];

    println!("Pipeline stages:");
    for (i, agent) in pipeline.iter().enumerate() {
        println!("  {}. {}", i + 1, agent);
    }

    println!("\nExecution flow:");
    println!("  1. api-designer: Design API specification");
    println!("  2. backend-developer: Implement based on design (uses step 1 output)");
    println!("  3. test-generator: Generate tests (uses step 2 code)");
    println!("  4. code-reviewer: Review everything (uses all previous outputs)\n");

    println!("Each stage uses output from previous stage as context");
    println!("Status: Would execute in tmux sessions (skipped in example)\n");

    // In real usage:
    // let mode = ExecutionMode::Sequential(pipeline);
    // let results = executor.execute(mode, "Create user management").await?;
    // for (i, result) in results.iter().enumerate() {
    //     println!("Stage {}: {} - {:?}", i + 1, result.agent_name, result.status);
    // }

    Ok(())
}

async fn example_coordinated(executor: &mut AgentExecutor) -> Result<()> {
    println!("Task: Implement authentication with security testing\n");

    println!("Auto-selection process:");
    println!("  1. Analyze task keywords: 'authentication', 'security', 'testing'");
    println!("  2. Match to agents:");
    println!("     - 'authentication' → backend-developer");
    println!("     - 'security' → security-tester");
    println!("     - 'testing' → test-generator");
    println!("  3. Determine execution mode: Parallel (3 independent tasks)\n");

    let selected = executor
        .coordinator()
        .auto_select_agents("Implement authentication with security testing")?;

    println!("Auto-selected agents:");
    for agent in &selected {
        println!("  - {}", agent);
    }

    println!("\nStatus: Would execute in coordinated mode (skipped in example)\n");

    // In real usage:
    // let results = executor.execute(
    //     ExecutionMode::Coordinated,
    //     "Implement authentication with security testing"
    // ).await?;

    Ok(())
}

async fn example_editor_integration(executor: &mut AgentExecutor) -> Result<()> {
    use ait42_ait42::editor_integration::*;

    println!("Simulating editor operations\n");

    // Note: EditorAgentBridge requires AgentExecutor to implement Clone
    // let bridge = EditorAgentBridge::new(executor.clone());

    // Mock buffer
    let buffer = Buffer {
        content: r#"
fn authenticate(username: &str, password: &str) -> Result<Token> {
    // TODO: implement authentication
    unimplemented!()
}
"#
        .to_string(),
        file_path: Some("src/auth.rs".to_string()),
        language: Some("rust".to_string()),
    };

    println!("Buffer: src/auth.rs");
    println!("Language: Rust");
    println!("Content: Authentication function\n");

    println!("Available operations:");
    println!("  1. review_buffer() - Code review");
    println!("  2. generate_tests() - Generate unit tests");
    println!("  3. security_scan() - Security analysis");
    println!("  4. document_code() - Add documentation");
    println!("  5. refactor_selection() - Refactor selected code\n");

    println!("Example: bridge.review_buffer(&buffer).await");
    println!("  → Sends buffer to code-reviewer agent");
    println!("  → Returns detailed code review with suggestions\n");

    println!("Status: Would execute agents on buffer (skipped in example)\n");

    // In real usage:
    // let review = bridge.review_buffer(&buffer).await?;
    // let tests = bridge.generate_tests(&buffer).await?;
    // let security = bridge.security_scan(&buffer).await?;

    Ok(())
}
