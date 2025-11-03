//! LSP Commands
//!
//! Tauri commands for Language Server Protocol integration.

use crate::state::AppState;
use lsp_types::{Position, Url};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tauri::State;

/// LSP diagnostic information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspDiagnostic {
    pub message: String,
    pub severity: u8, // 1=Error, 2=Warning, 3=Information, 4=Hint
    pub start_line: u32,
    pub start_character: u32,
    pub end_line: u32,
    pub end_character: u32,
    pub code: Option<String>,
    pub source: Option<String>,
}

/// Completion item information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspCompletionItem {
    pub label: String,
    pub kind: Option<u8>,
    pub detail: Option<String>,
    pub documentation: Option<String>,
    pub insert_text: Option<String>,
    pub sort_text: Option<String>,
}

/// Hover information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspHoverInfo {
    pub contents: String,
}

/// Location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspLocation {
    pub uri: String,
    pub start_line: u32,
    pub start_character: u32,
    pub end_line: u32,
    pub end_character: u32,
}

/// Start LSP server for a specific language
#[tauri::command]
pub async fn start_lsp_server(
    language: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .lsp_manager
        .start_server(&language)
        .await
        .map_err(|e| format!("Failed to start LSP server for {}: {}", language, e))
}

/// Stop LSP server for a specific language
#[tauri::command]
pub async fn stop_lsp_server(language: String, state: State<'_, AppState>) -> Result<(), String> {
    state
        .lsp_manager
        .stop_server(&language)
        .await
        .map_err(|e| format!("Failed to stop LSP server for {}: {}", language, e))
}

/// Get list of running LSP servers
#[tauri::command]
pub async fn get_running_lsp_servers(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    Ok(state.lsp_manager.running_servers().await)
}

/// Notify LSP server that a document was opened
#[tauri::command]
pub async fn lsp_did_open(
    file_path: String,
    content: String,
    language_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let path = Path::new(&file_path);

    // Ensure server is running for this language
    let client = state
        .lsp_manager
        .ensure_server(&language_id)
        .await
        .map_err(|e| format!("Failed to ensure LSP server: {}", e))?;

    // Convert file path to URI
    let uri = Url::from_file_path(path)
        .map_err(|_| format!("Invalid file path: {}", file_path))?;

    // Notify server
    client
        .did_open(uri, content, language_id)
        .await
        .map_err(|e| format!("Failed to notify LSP server: {}", e))
}

/// Notify LSP server of document changes
#[tauri::command]
pub async fn lsp_did_change(
    file_path: String,
    content: String,
    version: i32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let path = Path::new(&file_path);

    // Get language from file extension
    let language = state
        .lsp_manager
        .detect_language(path)
        .ok_or_else(|| format!("Could not detect language for {}", file_path))?;

    // Get client
    let client = state
        .lsp_manager
        .get_client(&language)
        .await
        .ok_or_else(|| format!("No LSP server running for {}", language))?;

    // Convert file path to URI
    let uri = Url::from_file_path(path)
        .map_err(|_| format!("Invalid file path: {}", file_path))?;

    // Create change event
    let changes = vec![lsp_types::TextDocumentContentChangeEvent {
        range: None,
        range_length: None,
        text: content,
    }];

    // Notify server
    client
        .did_change(uri, version, changes)
        .await
        .map_err(|e| format!("Failed to notify LSP server of changes: {}", e))
}

/// Notify LSP server that a document was saved
#[tauri::command]
pub async fn lsp_did_save(
    file_path: String,
    content: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let path = Path::new(&file_path);

    // Get language from file extension
    let language = state
        .lsp_manager
        .detect_language(path)
        .ok_or_else(|| format!("Could not detect language for {}", file_path))?;

    // Get client
    let client = state
        .lsp_manager
        .get_client(&language)
        .await
        .ok_or_else(|| format!("No LSP server running for {}", language))?;

    // Convert file path to URI
    let uri = Url::from_file_path(path)
        .map_err(|_| format!("Invalid file path: {}", file_path))?;

    // Notify server
    client
        .did_save(uri, content)
        .await
        .map_err(|e| format!("Failed to notify LSP server of save: {}", e))
}

/// Notify LSP server that a document was closed
#[tauri::command]
pub async fn lsp_did_close(file_path: String, state: State<'_, AppState>) -> Result<(), String> {
    let path = Path::new(&file_path);

    // Get language from file extension
    let language = state
        .lsp_manager
        .detect_language(path)
        .ok_or_else(|| format!("Could not detect language for {}", file_path))?;

    // Get client
    let client = state
        .lsp_manager
        .get_client(&language)
        .await
        .ok_or_else(|| format!("No LSP server running for {}", language))?;

    // Convert file path to URI
    let uri = Url::from_file_path(path)
        .map_err(|_| format!("Invalid file path: {}", file_path))?;

    // Notify server
    client
        .did_close(uri)
        .await
        .map_err(|e| format!("Failed to notify LSP server of close: {}", e))
}

/// Get completion suggestions at a specific position
#[tauri::command]
pub async fn lsp_completion(
    file_path: String,
    line: u32,
    character: u32,
    state: State<'_, AppState>,
) -> Result<Vec<LspCompletionItem>, String> {
    let path = Path::new(&file_path);

    // Get language from file extension
    let language = state
        .lsp_manager
        .detect_language(path)
        .ok_or_else(|| format!("Could not detect language for {}", file_path))?;

    // Get client
    let client = state
        .lsp_manager
        .get_client(&language)
        .await
        .ok_or_else(|| format!("No LSP server running for {}", language))?;

    // Convert file path to URI
    let uri = Url::from_file_path(path)
        .map_err(|_| format!("Invalid file path: {}", file_path))?;

    // Get completions
    let position = Position { line, character };
    let completions = client
        .completion(uri, position)
        .await
        .map_err(|e| format!("Failed to get completions: {}", e))?;

    // Convert to simplified format
    Ok(completions
        .into_iter()
        .map(|item| LspCompletionItem {
            label: item.label,
            kind: item.kind.map(|k| {
                use lsp_types::CompletionItemKind;
                match k {
                    CompletionItemKind::TEXT => 1,
                    CompletionItemKind::METHOD => 2,
                    CompletionItemKind::FUNCTION => 3,
                    CompletionItemKind::CONSTRUCTOR => 4,
                    CompletionItemKind::FIELD => 5,
                    CompletionItemKind::VARIABLE => 6,
                    CompletionItemKind::CLASS => 7,
                    CompletionItemKind::INTERFACE => 8,
                    CompletionItemKind::MODULE => 9,
                    CompletionItemKind::PROPERTY => 10,
                    CompletionItemKind::UNIT => 11,
                    CompletionItemKind::VALUE => 12,
                    CompletionItemKind::ENUM => 13,
                    CompletionItemKind::KEYWORD => 14,
                    CompletionItemKind::SNIPPET => 15,
                    CompletionItemKind::COLOR => 16,
                    CompletionItemKind::FILE => 17,
                    CompletionItemKind::REFERENCE => 18,
                    CompletionItemKind::FOLDER => 19,
                    CompletionItemKind::ENUM_MEMBER => 20,
                    CompletionItemKind::CONSTANT => 21,
                    CompletionItemKind::STRUCT => 22,
                    CompletionItemKind::EVENT => 23,
                    CompletionItemKind::OPERATOR => 24,
                    CompletionItemKind::TYPE_PARAMETER => 25,
                    _ => 1, // Default to TEXT
                }
            }),
            detail: item.detail,
            documentation: item.documentation.and_then(|doc| match doc {
                lsp_types::Documentation::String(s) => Some(s),
                lsp_types::Documentation::MarkupContent(markup) => Some(markup.value),
            }),
            insert_text: item.insert_text,
            sort_text: item.sort_text,
        })
        .collect())
}

/// Get hover information at a specific position
#[tauri::command]
pub async fn lsp_hover(
    file_path: String,
    line: u32,
    character: u32,
    state: State<'_, AppState>,
) -> Result<Option<LspHoverInfo>, String> {
    let path = Path::new(&file_path);

    // Get language from file extension
    let language = state
        .lsp_manager
        .detect_language(path)
        .ok_or_else(|| format!("Could not detect language for {}", file_path))?;

    // Get client
    let client = state
        .lsp_manager
        .get_client(&language)
        .await
        .ok_or_else(|| format!("No LSP server running for {}", language))?;

    // Convert file path to URI
    let uri = Url::from_file_path(path)
        .map_err(|_| format!("Invalid file path: {}", file_path))?;

    // Get hover info
    let position = Position { line, character };
    let hover = client
        .hover(uri, position)
        .await
        .map_err(|e| format!("Failed to get hover info: {}", e))?;

    // Convert to simplified format
    Ok(hover.map(|h| {
        let contents = match h.contents {
            lsp_types::HoverContents::Scalar(markup) => match markup {
                lsp_types::MarkedString::String(s) => s,
                lsp_types::MarkedString::LanguageString(ls) => ls.value,
            },
            lsp_types::HoverContents::Array(arr) => arr
                .into_iter()
                .map(|markup| match markup {
                    lsp_types::MarkedString::String(s) => s,
                    lsp_types::MarkedString::LanguageString(ls) => ls.value,
                })
                .collect::<Vec<_>>()
                .join("\n\n"),
            lsp_types::HoverContents::Markup(markup) => markup.value,
        };

        LspHoverInfo { contents }
    }))
}

/// Go to definition of symbol at a specific position
#[tauri::command]
pub async fn lsp_goto_definition(
    file_path: String,
    line: u32,
    character: u32,
    state: State<'_, AppState>,
) -> Result<Vec<LspLocation>, String> {
    let path = Path::new(&file_path);

    // Get language from file extension
    let language = state
        .lsp_manager
        .detect_language(path)
        .ok_or_else(|| format!("Could not detect language for {}", file_path))?;

    // Get client
    let client = state
        .lsp_manager
        .get_client(&language)
        .await
        .ok_or_else(|| format!("No LSP server running for {}", language))?;

    // Convert file path to URI
    let uri = Url::from_file_path(path)
        .map_err(|_| format!("Invalid file path: {}", file_path))?;

    // Get definition
    let position = Position { line, character };
    let definition = client
        .goto_definition(uri, position)
        .await
        .map_err(|e| format!("Failed to get definition: {}", e))?;

    // Convert to simplified format
    let locations = match definition {
        Some(lsp_types::GotoDefinitionResponse::Scalar(location)) => {
            vec![location]
        }
        Some(lsp_types::GotoDefinitionResponse::Array(locations)) => locations,
        Some(lsp_types::GotoDefinitionResponse::Link(links)) => {
            links.into_iter().map(|link| link.target_uri).map(|uri| {
                lsp_types::Location {
                    uri,
                    range: lsp_types::Range::default(),
                }
            }).collect()
        }
        None => Vec::new(),
    };

    Ok(locations
        .into_iter()
        .map(|loc| LspLocation {
            uri: loc.uri.to_string(),
            start_line: loc.range.start.line,
            start_character: loc.range.start.character,
            end_line: loc.range.end.line,
            end_character: loc.range.end.character,
        })
        .collect())
}

/// Get diagnostics for a specific file
#[tauri::command]
pub async fn lsp_diagnostics(
    file_path: String,
    state: State<'_, AppState>,
) -> Result<Vec<LspDiagnostic>, String> {
    let path = Path::new(&file_path);

    // Get language from file extension
    let language = state
        .lsp_manager
        .detect_language(path)
        .ok_or_else(|| format!("Could not detect language for {}", file_path))?;

    // Get client
    let client = state
        .lsp_manager
        .get_client(&language)
        .await
        .ok_or_else(|| format!("No LSP server running for {}", language))?;

    // Convert file path to URI
    let uri = Url::from_file_path(path)
        .map_err(|_| format!("Invalid file path: {}", file_path))?;

    // Get diagnostics
    let diagnostics = client
        .diagnostics(&uri)
        .await
        .map_err(|e| format!("Failed to get diagnostics: {}", e))?;

    // Convert to simplified format
    Ok(diagnostics
        .into_iter()
        .map(|diag| {
            use lsp_types::DiagnosticSeverity;
            let severity = diag.severity.map(|s| match s {
                DiagnosticSeverity::ERROR => 1,
                DiagnosticSeverity::WARNING => 2,
                DiagnosticSeverity::INFORMATION => 3,
                DiagnosticSeverity::HINT => 4,
                _ => 3, // Default to INFORMATION
            }).unwrap_or(3);

            LspDiagnostic {
                message: diag.message,
                severity,
                start_line: diag.range.start.line,
                start_character: diag.range.start.character,
                end_line: diag.range.end.line,
                end_character: diag.range.end.character,
                code: diag.code.and_then(|c| match c {
                    lsp_types::NumberOrString::Number(n) => Some(n.to_string()),
                    lsp_types::NumberOrString::String(s) => Some(s),
                }),
                source: diag.source,
            }
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lsp_diagnostic_creation() {
        let diag = LspDiagnostic {
            message: "Test error".to_string(),
            severity: 1,
            start_line: 0,
            start_character: 0,
            end_line: 0,
            end_character: 10,
            code: Some("E001".to_string()),
            source: Some("test".to_string()),
        };

        assert_eq!(diag.message, "Test error");
        assert_eq!(diag.severity, 1);
    }

    #[test]
    fn test_completion_item_creation() {
        let item = LspCompletionItem {
            label: "test_function".to_string(),
            kind: Some(3),
            detail: Some("fn() -> String".to_string()),
            documentation: None,
            insert_text: None,
            sort_text: None,
        };

        assert_eq!(item.label, "test_function");
        assert_eq!(item.kind, Some(3));
    }
}
