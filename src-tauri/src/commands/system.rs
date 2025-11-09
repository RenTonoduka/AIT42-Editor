/**
 * System integration commands
 *
 * Provides OS-level integrations like opening VS Code, terminals, and file managers
 */

use std::process::Command;

/// Open a directory in VS Code
#[tauri::command]
pub async fn open_in_vscode(path: String) -> Result<String, String> {
    tracing::info!("🚀 Opening VS Code at: {}", path);

    #[cfg(target_os = "macos")]
    let result = Command::new("open")
        .args(&["-a", "Visual Studio Code", &path])
        .output();

    #[cfg(target_os = "windows")]
    let result = Command::new("cmd")
        .args(&["/C", "code", &path])
        .output();

    #[cfg(target_os = "linux")]
    let result = Command::new("code")
        .arg(&path)
        .output();

    match result {
        Ok(output) => {
            if output.status.success() {
                Ok(format!("VS Code opened: {}", path))
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(format!("Failed to open VS Code: {}", stderr))
            }
        }
        Err(e) => Err(format!("Failed to execute VS Code command: {}. Make sure VS Code is installed and 'code' command is available.", e)),
    }
}

/// Open a terminal in a directory
#[tauri::command]
pub async fn open_terminal(path: String) -> Result<String, String> {
    tracing::info!("💻 Opening terminal at: {}", path);

    #[cfg(target_os = "macos")]
    let result = Command::new("open")
        .args(&["-a", "Terminal", &path])
        .output();

    #[cfg(target_os = "windows")]
    let result = Command::new("cmd")
        .args(&["/C", "start", "cmd.exe", "/K", "cd", &path])
        .output();

    #[cfg(target_os = "linux")]
    let result = Command::new("x-terminal-emulator")
        .args(&["--working-directory", &path])
        .output();

    match result {
        Ok(output) => {
            if output.status.success() {
                Ok(format!("Terminal opened: {}", path))
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(format!("Failed to open terminal: {}", stderr))
            }
        }
        Err(e) => Err(format!("Failed to execute terminal command: {}", e)),
    }
}

/// Open a directory in the system file manager
#[tauri::command]
pub async fn open_in_finder(path: String) -> Result<String, String> {
    tracing::info!("📂 Opening Finder at: {}", path);

    #[cfg(target_os = "macos")]
    let result = Command::new("open")
        .arg(&path)
        .output();

    #[cfg(target_os = "windows")]
    let result = Command::new("explorer")
        .arg(&path)
        .output();

    #[cfg(target_os = "linux")]
    let result = Command::new("xdg-open")
        .arg(&path)
        .output();

    match result {
        Ok(output) => {
            if output.status.success() {
                Ok(format!("File manager opened: {}", path))
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(format!("Failed to open file manager: {}", stderr))
            }
        }
        Err(e) => Err(format!("Failed to execute file manager command: {}", e)),
    }
}

/// Copy path to clipboard
#[tauri::command]
pub async fn copy_to_clipboard(text: String) -> Result<String, String> {
    use clipboard::{ClipboardProvider, ClipboardContext};

    let mut ctx: ClipboardContext = ClipboardProvider::new()
        .map_err(|e| format!("Failed to access clipboard: {}", e))?;

    ctx.set_contents(text.clone())
        .map_err(|e| format!("Failed to copy to clipboard: {}", e))?;

    Ok(format!("Copied to clipboard: {}", text))
}
