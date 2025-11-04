// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tracing::info;

mod commands;
mod plugin;
mod state;

use state::AppState;

/// Generate invoke handler with conditional terminal commands
fn generate_handler() -> impl Fn(tauri::Invoke) + Send + Sync + 'static {
    #[cfg(feature = "terminal")]
    {
        tauri::generate_handler![
            // File operations
            commands::open_file,
            commands::save_file,
            commands::create_file,
            commands::create_directory,
            commands::delete_path,
            commands::rename_path,
            commands::read_directory,
            // Editor operations
            commands::insert_text,
            commands::delete_text,
            commands::replace_text,
            commands::undo,
            commands::redo,
            commands::get_buffer_content,
            commands::get_buffer_info,
            commands::close_buffer,
            commands::list_buffers,
            // LSP operations
            commands::start_lsp_server,
            commands::stop_lsp_server,
            commands::get_running_lsp_servers,
            commands::lsp_did_open,
            commands::lsp_did_change,
            commands::lsp_did_save,
            commands::lsp_did_close,
            commands::lsp_completion,
            commands::lsp_hover,
            commands::lsp_goto_definition,
            commands::lsp_diagnostics,
            // Git operations
            commands::git_status,
            commands::git_add,
            commands::git_reset,
            commands::git_commit,
            commands::git_push,
            commands::git_pull,
            commands::git_log,
            commands::git_branches,
            commands::git_checkout,
            commands::git_create_branch,
            // Git worktree operations
            commands::git_list_worktrees,
            commands::git_create_worktree,
            commands::git_remove_worktree,
            commands::git_prune_worktrees,
            // Plugin operations
            commands::list_plugins,
            commands::get_plugin,
            commands::enable_plugin,
            commands::disable_plugin,
            commands::install_plugin,
            commands::uninstall_plugin,
            // AIT42 Agent operations
            commands::list_agents,
            commands::get_agent_info,
            commands::execute_agent,
            commands::execute_parallel,
            commands::get_agent_output,
            commands::cancel_agent_execution,
            // AIT42 Tmux operations
            commands::create_tmux_session,
            commands::list_tmux_sessions,
            commands::capture_tmux_output,
            commands::send_tmux_keys,
            commands::kill_tmux_session,
            // AIT42 Competition operations
            commands::execute_claude_code_competition,
            commands::get_competition_status,
            commands::cancel_competition,
            // Terminal operations
            commands::execute_command,
            commands::get_terminal_output,
            commands::get_terminal_tail,
            commands::clear_terminal,
            commands::get_current_directory,
            commands::set_current_directory,
            commands::get_command_history,
            commands::get_terminal_info
        ]
    }

    #[cfg(not(feature = "terminal"))]
    {
        tauri::generate_handler![
            // File operations
            commands::open_file,
            commands::save_file,
            commands::create_file,
            commands::create_directory,
            commands::delete_path,
            commands::rename_path,
            commands::read_directory,
            // Editor operations
            commands::insert_text,
            commands::delete_text,
            commands::replace_text,
            commands::undo,
            commands::redo,
            commands::get_buffer_content,
            commands::get_buffer_info,
            commands::close_buffer,
            commands::list_buffers,
            // LSP operations
            commands::start_lsp_server,
            commands::stop_lsp_server,
            commands::get_running_lsp_servers,
            commands::lsp_did_open,
            commands::lsp_did_change,
            commands::lsp_did_save,
            commands::lsp_did_close,
            commands::lsp_completion,
            commands::lsp_hover,
            commands::lsp_goto_definition,
            commands::lsp_diagnostics,
            // Git operations
            commands::git_status,
            commands::git_add,
            commands::git_reset,
            commands::git_commit,
            commands::git_push,
            commands::git_pull,
            commands::git_log,
            commands::git_branches,
            commands::git_checkout,
            commands::git_create_branch,
            // Git worktree operations
            commands::git_list_worktrees,
            commands::git_create_worktree,
            commands::git_remove_worktree,
            commands::git_prune_worktrees,
            // Plugin operations
            commands::list_plugins,
            commands::get_plugin,
            commands::enable_plugin,
            commands::disable_plugin,
            commands::install_plugin,
            commands::uninstall_plugin,
            // AIT42 Agent operations
            commands::list_agents,
            commands::get_agent_info,
            commands::execute_agent,
            commands::execute_parallel,
            commands::get_agent_output,
            commands::cancel_agent_execution,
            // AIT42 Tmux operations
            commands::create_tmux_session,
            commands::list_tmux_sessions,
            commands::capture_tmux_output,
            commands::send_tmux_keys,
            commands::kill_tmux_session,
            // AIT42 Competition operations
            commands::execute_claude_code_competition,
            commands::get_competition_status,
            commands::cancel_competition
        ]
    }
}

fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    info!("Starting AIT42 Editor GUI");

    // Initialize application state
    let working_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let app_state = AppState::new(working_dir).expect("Failed to initialize application state");

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(generate_handler())
        .setup(|_app| {
            info!("AIT42 Editor GUI initialized successfully");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
