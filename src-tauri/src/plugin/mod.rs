/**
 * Plugin System
 *
 * Provides extensibility through a plugin architecture:
 * - Plugin metadata and manifest
 * - Plugin lifecycle management
 * - Plugin API and hooks
 * - Plugin discovery and loading
 */

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/**
 * Plugin metadata
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub entry_point: String, // Path to main plugin file
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(default)]
    pub permissions: Vec<String>,
}

/**
 * Plugin state
 */
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginState {
    Installed,
    Enabled,
    Disabled,
    Error,
}

/**
 * Plugin information
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub manifest: PluginManifest,
    pub state: PluginState,
    pub install_path: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/**
 * Plugin manager
 */
#[derive(Debug)]
pub struct PluginManager {
    plugins: HashMap<String, PluginInfo>,
    plugins_dir: PathBuf,
}

impl PluginManager {
    /**
     * Create a new plugin manager
     */
    pub fn new(plugins_dir: PathBuf) -> Self {
        Self {
            plugins: HashMap::new(),
            plugins_dir,
        }
    }

    /**
     * Initialize plugin manager and discover plugins
     */
    pub fn initialize(&mut self) -> Result<(), String> {
        // Create plugins directory if it doesn't exist
        if !self.plugins_dir.exists() {
            std::fs::create_dir_all(&self.plugins_dir)
                .map_err(|e| format!("Failed to create plugins directory: {}", e))?;
        }

        // Discover plugins
        self.discover_plugins()?;

        Ok(())
    }

    /**
     * Discover plugins in the plugins directory
     */
    fn discover_plugins(&mut self) -> Result<(), String> {
        let entries = std::fs::read_dir(&self.plugins_dir)
            .map_err(|e| format!("Failed to read plugins directory: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();

            if path.is_dir() {
                if let Err(e) = self.load_plugin_manifest(&path) {
                    eprintln!("[Plugin Manager] Failed to load plugin at {:?}: {}", path, e);
                }
            }
        }

        Ok(())
    }

    /**
     * Load plugin manifest from directory
     */
    fn load_plugin_manifest(&mut self, plugin_dir: &Path) -> Result<(), String> {
        let manifest_path = plugin_dir.join("plugin.json");

        if !manifest_path.exists() {
            return Err("No plugin.json found".to_string());
        }

        let manifest_data = std::fs::read_to_string(&manifest_path)
            .map_err(|e| format!("Failed to read manifest: {}", e))?;

        let manifest: PluginManifest = serde_json::from_str(&manifest_data)
            .map_err(|e| format!("Failed to parse manifest: {}", e))?;

        let plugin_info = PluginInfo {
            manifest: manifest.clone(),
            state: PluginState::Installed,
            install_path: plugin_dir.to_path_buf(),
            error: None,
        };

        self.plugins.insert(manifest.id.clone(), plugin_info);

        println!("[Plugin Manager] Discovered plugin: {} ({})", manifest.name, manifest.id);

        Ok(())
    }

    /**
     * Get list of all plugins
     */
    pub fn list_plugins(&self) -> Vec<PluginInfo> {
        self.plugins.values().cloned().collect()
    }

    /**
     * Get plugin by ID
     */
    pub fn get_plugin(&self, plugin_id: &str) -> Option<&PluginInfo> {
        self.plugins.get(plugin_id)
    }

    /**
     * Enable a plugin
     */
    pub fn enable_plugin(&mut self, plugin_id: &str) -> Result<(), String> {
        let plugin = self.plugins.get_mut(plugin_id)
            .ok_or_else(|| format!("Plugin not found: {}", plugin_id))?;

        if plugin.state == PluginState::Enabled {
            return Ok(()); // Already enabled
        }

        // TODO: Load and initialize plugin
        plugin.state = PluginState::Enabled;

        println!("[Plugin Manager] Enabled plugin: {}", plugin_id);

        Ok(())
    }

    /**
     * Disable a plugin
     */
    pub fn disable_plugin(&mut self, plugin_id: &str) -> Result<(), String> {
        let plugin = self.plugins.get_mut(plugin_id)
            .ok_or_else(|| format!("Plugin not found: {}", plugin_id))?;

        if plugin.state == PluginState::Disabled {
            return Ok(()); // Already disabled
        }

        // TODO: Cleanup and unload plugin
        plugin.state = PluginState::Disabled;

        println!("[Plugin Manager] Disabled plugin: {}", plugin_id);

        Ok(())
    }

    /**
     * Install a plugin from a directory
     */
    pub fn install_plugin(&mut self, source_path: &Path) -> Result<String, String> {
        // Load manifest from source
        let manifest_path = source_path.join("plugin.json");
        if !manifest_path.exists() {
            return Err("No plugin.json found in source directory".to_string());
        }

        let manifest_data = std::fs::read_to_string(&manifest_path)
            .map_err(|e| format!("Failed to read manifest: {}", e))?;

        let manifest: PluginManifest = serde_json::from_str(&manifest_data)
            .map_err(|e| format!("Failed to parse manifest: {}", e))?;

        // Check if plugin is already installed
        if self.plugins.contains_key(&manifest.id) {
            return Err(format!("Plugin already installed: {}", manifest.id));
        }

        // Copy plugin to plugins directory
        let dest_path = self.plugins_dir.join(&manifest.id);
        if dest_path.exists() {
            std::fs::remove_dir_all(&dest_path)
                .map_err(|e| format!("Failed to remove existing plugin directory: {}", e))?;
        }

        copy_dir_recursive(source_path, &dest_path)?;

        // Add to plugins map
        let plugin_info = PluginInfo {
            manifest: manifest.clone(),
            state: PluginState::Installed,
            install_path: dest_path,
            error: None,
        };

        self.plugins.insert(manifest.id.clone(), plugin_info);

        println!("[Plugin Manager] Installed plugin: {} ({})", manifest.name, manifest.id);

        Ok(manifest.id)
    }

    /**
     * Uninstall a plugin
     */
    pub fn uninstall_plugin(&mut self, plugin_id: &str) -> Result<(), String> {
        let plugin = self.plugins.remove(plugin_id)
            .ok_or_else(|| format!("Plugin not found: {}", plugin_id))?;

        // Remove plugin directory
        if plugin.install_path.exists() {
            std::fs::remove_dir_all(&plugin.install_path)
                .map_err(|e| format!("Failed to remove plugin directory: {}", e))?;
        }

        println!("[Plugin Manager] Uninstalled plugin: {}", plugin_id);

        Ok(())
    }
}

/**
 * Recursively copy directory
 */
fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dest)
        .map_err(|e| format!("Failed to create destination directory: {}", e))?;

    for entry in std::fs::read_dir(src)
        .map_err(|e| format!("Failed to read source directory: {}", e))? {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();
        let file_name = path.file_name().ok_or("Invalid file name")?;
        let dest_path = dest.join(file_name);

        if path.is_dir() {
            copy_dir_recursive(&path, &dest_path)?;
        } else {
            std::fs::copy(&path, &dest_path)
                .map_err(|e| format!("Failed to copy file: {}", e))?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_plugin_manager_creation() {
        let temp_dir = env::temp_dir().join("ait42-test-plugins");
        let manager = PluginManager::new(temp_dir.clone());
        assert_eq!(manager.plugins.len(), 0);
        assert_eq!(manager.plugins_dir, temp_dir);
    }

    #[test]
    fn test_plugin_manifest_serialization() {
        let manifest = PluginManifest {
            id: String::from("test-plugin"),
            name: String::from("Test Plugin"),
            version: String::from("1.0.0"),
            author: String::from("Test Author"),
            description: String::from("A test plugin"),
            entry_point: String::from("main.js"),
            dependencies: vec![],
            permissions: vec![],
        };

        let json = serde_json::to_string(&manifest).unwrap();
        let deserialized: PluginManifest = serde_json::from_str(&json).unwrap();

        assert_eq!(manifest.id, deserialized.id);
        assert_eq!(manifest.name, deserialized.name);
    }
}
