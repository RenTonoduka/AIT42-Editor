// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tracing::info;

mod commands;
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
            commands::list_buffers
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
