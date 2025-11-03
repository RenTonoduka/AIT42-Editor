/**
 * Plugin Commands
 *
 * Tauri commands for plugin management
 */

use tauri::State;
use crate::state::AppState;
use crate::plugin::PluginInfo;

/**
 * List all plugins
 */
#[tauri::command]
pub async fn list_plugins(state: State<'_, AppState>) -> Result<Vec<PluginInfo>, String> {
    let manager = state.plugin_manager.lock()
        .map_err(|e| format!("Failed to lock plugin manager: {}", e))?;

    Ok(manager.list_plugins())
}

/**
 * Get plugin by ID
 */
#[tauri::command]
pub async fn get_plugin(
    plugin_id: String,
    state: State<'_, AppState>,
) -> Result<Option<PluginInfo>, String> {
    let manager = state.plugin_manager.lock()
        .map_err(|e| format!("Failed to lock plugin manager: {}", e))?;

    Ok(manager.get_plugin(&plugin_id).cloned())
}

/**
 * Enable a plugin
 */
#[tauri::command]
pub async fn enable_plugin(
    plugin_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut manager = state.plugin_manager.lock()
        .map_err(|e| format!("Failed to lock plugin manager: {}", e))?;

    manager.enable_plugin(&plugin_id)
}

/**
 * Disable a plugin
 */
#[tauri::command]
pub async fn disable_plugin(
    plugin_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut manager = state.plugin_manager.lock()
        .map_err(|e| format!("Failed to lock plugin manager: {}", e))?;

    manager.disable_plugin(&plugin_id)
}

/**
 * Install a plugin from path
 */
#[tauri::command]
pub async fn install_plugin(
    source_path: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let mut manager = state.plugin_manager.lock()
        .map_err(|e| format!("Failed to lock plugin manager: {}", e))?;

    let path = std::path::PathBuf::from(source_path);
    manager.install_plugin(&path)
}

/**
 * Uninstall a plugin
 */
#[tauri::command]
pub async fn uninstall_plugin(
    plugin_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut manager = state.plugin_manager.lock()
        .map_err(|e| format!("Failed to lock plugin manager: {}", e))?;

    manager.uninstall_plugin(&plugin_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_commands_compile() {
        // This test just ensures the commands compile correctly
        // Actual testing would require a full Tauri runtime
    }
}
