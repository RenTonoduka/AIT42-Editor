//! Basic usage example for ait42-ait42 crate

use ait42_ait42::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("=== AIT42 Agent Integration Example ===\n");

    // Step 1: Load configuration
    println!("1. Loading configuration...");
    let config = match AIT42Config::load() {
        Ok(config) => {
            println!("   ✓ Loaded from: {}", config.ait42_root.display());
            config
        }
        Err(e) => {
            eprintln!("   ✗ Failed to load config: {}", e);
            eprintln!("   Set AIT42_ROOT environment variable to your AIT42 installation");
            return Err(e);
        }
    };

    // Step 2: Create coordinator
    println!("\n2. Creating coordinator...");
    let coordinator = Coordinator::new(config)?;
    println!("   ✓ Loaded {} agents", coordinator.agent_count());

    // Step 3: List available agents
    println!("\n3. Available agents:");
    let agents = coordinator.list_agents();
    for (i, agent) in agents.iter().take(10).enumerate() {
        println!(
            "   {}. {} - {}",
            i + 1,
            agent.name,
            agent.description.chars().take(60).collect::<String>()
        );
    }
    println!("   ... and {} more agents", agents.len().saturating_sub(10));

    // Step 4: Search for agents
    println!("\n4. Searching for 'backend' agents:");
    let backend_agents = coordinator.search_agents("backend");
    for agent in backend_agents.iter().take(3) {
        println!("   - {}: {}", agent.name, agent.description);
    }

    // Step 5: Auto-select agents for a task
    println!("\n5. Auto-selecting agents for 'Implement user authentication API':");
    let selected = coordinator.auto_select_agents("Implement user authentication API")?;
    for agent in &selected {
        println!("   → {}", agent);
    }

    // Step 6: Create executor
    println!("\n6. Creating executor...");
    let mut executor = AgentExecutor::new(coordinator);
    println!("   ✓ Executor ready");

    // Example: Execute coordinated mode (auto-select)
    println!("\n7. Example execution (coordinated mode):");
    println!("   Task: 'Review TypeScript code quality'");
    println!("   This would execute the selected agent(s) in tmux sessions");
    println!("   (Skipping actual execution in this example)");

    // Example: List execution modes
    println!("\n8. Available execution modes:");
    println!("   - Single: Execute one specific agent");
    println!("   - Parallel: Execute multiple agents concurrently");
    println!("   - Sequential: Execute agents in pipeline (output → input)");
    println!("   - Coordinated: Auto-select and execute optimal agent(s)");

    // Example: Show command palette commands
    println!("\n9. Command palette examples:");
    let commands = vec![
        ("RunAgent", "Run specific agent with task"),
        ("RunCoordinator", "Auto-select and run agent"),
        ("RunParallel", "Run multiple agents in parallel"),
        ("ListAgents", "List all available agents"),
        ("ListSessions", "Show active tmux sessions"),
        ("ViewSessionOutput", "View session output"),
        ("KillSession", "Terminate a session"),
    ];

    for (cmd, desc) in commands {
        println!("   - {}: {}", cmd, desc);
    }

    println!("\n=== Example Complete ===");
    println!("\nTo actually run agents, ensure:");
    println!("1. tmux is installed: brew install tmux");
    println!("2. AIT42_ROOT is set to your AIT42 installation");
    println!("3. Scripts directory contains tmux-single-agent.sh");

    Ok(())
}
