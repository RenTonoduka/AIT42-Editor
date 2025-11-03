//! LSP Client Implementation
//!
//! Provides a client for communicating with Language Server Protocol servers.

use crate::{LspError, Result};
use lsp_types::*;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{ChildStdin, ChildStdout};
use tokio::sync::{mpsc, Mutex, RwLock};
use tracing::{debug, error, info};

/// LSP client for a single language server
pub struct LspClient {
    stdin: Arc<Mutex<ChildStdin>>,
    request_id: AtomicU64,
    capabilities: Arc<RwLock<Option<ServerCapabilities>>>,
    pending_requests: Arc<Mutex<HashMap<u64, mpsc::Sender<serde_json::Value>>>>,
    diagnostics: Arc<RwLock<HashMap<Url, Vec<Diagnostic>>>>,
    server_process: Arc<Mutex<Child>>,
}

impl LspClient {
    /// Create a new LSP client
    ///
    /// # Arguments
    /// * `server_cmd` - Command to execute the LSP server (e.g., "rust-analyzer")
    /// * `args` - Command-line arguments for the server
    /// * `root_uri` - Root URI for the workspace
    pub async fn new(server_cmd: &str, args: &[&str], root_uri: Option<Url>) -> Result<Self> {
        info!("Starting LSP server: {} {:?}", server_cmd, args);

        // Spawn the LSP server process
        let mut child = Command::new(server_cmd)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| LspError::ProcessError(format!("Failed to spawn server: {}", e)))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| LspError::ProcessError("Failed to get stdin".to_string()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| LspError::ProcessError("Failed to get stdout".to_string()))?;

        let stdin = Arc::new(Mutex::new(tokio::process::ChildStdin::from_std(stdin)?));
        let stdout = tokio::process::ChildStdout::from_std(stdout)?;

        let pending_requests = Arc::new(Mutex::new(HashMap::new()));
        let diagnostics = Arc::new(RwLock::new(HashMap::new()));
        let capabilities = Arc::new(RwLock::new(None));

        let client = Self {
            stdin: stdin.clone(),
            request_id: AtomicU64::new(1),
            capabilities: capabilities.clone(),
            pending_requests: pending_requests.clone(),
            diagnostics: diagnostics.clone(),
            server_process: Arc::new(Mutex::new(child)),
        };

        // Spawn background task to handle server responses
        tokio::spawn(Self::handle_server_output(stdout, pending_requests, diagnostics));

        // Initialize the server
        client.initialize(root_uri).await?;

        Ok(client)
    }

    /// Initialize the LSP server
    async fn initialize(&self, root_uri: Option<Url>) -> Result<()> {
        debug!("Initializing LSP server");

        let workspace_folders = root_uri.as_ref().map(|uri| {
            vec![WorkspaceFolder {
                uri: uri.clone(),
                name: uri
                    .path()
                    .rsplit('/')
                    .next()
                    .unwrap_or("workspace")
                    .to_string(),
            }]
        });

        let init_params = InitializeParams {
            process_id: Some(std::process::id()),
            workspace_folders,
            capabilities: ClientCapabilities {
                text_document: Some(TextDocumentClientCapabilities {
                    completion: Some(CompletionClientCapabilities {
                        completion_item: Some(CompletionItemCapability {
                            snippet_support: Some(true),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }),
                    hover: Some(HoverClientCapabilities {
                        content_format: Some(vec![MarkupKind::Markdown, MarkupKind::PlainText]),
                        ..Default::default()
                    }),
                    definition: Some(GotoCapability {
                        link_support: Some(true),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        };

        let response: InitializeResult = self.send_request("initialize", init_params).await?;

        // Store server capabilities
        *self.capabilities.write().await = Some(response.capabilities);

        // Send initialized notification
        self.send_notification("initialized", InitializedParams {})
            .await?;

        info!("LSP server initialized successfully");
        Ok(())
    }

    /// Send a request and wait for response
    async fn send_request<P: Serialize, R: DeserializeOwned>(
        &self,
        method: &str,
        params: P,
    ) -> Result<R> {
        let id = self.request_id.fetch_add(1, Ordering::SeqCst);

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });

        // Create channel for response
        let (tx, mut rx) = mpsc::channel(1);
        self.pending_requests.lock().await.insert(id, tx);

        // Send request
        self.send_message(&request).await?;

        // Wait for response
        let response = rx
            .recv()
            .await
            .ok_or_else(|| LspError::CommunicationError("No response received".to_string()))?;

        // Check for error
        if let Some(error) = response.get("error") {
            return Err(LspError::CommunicationError(format!("Server error: {}", error)));
        }

        // Extract result
        let result = response
            .get("result")
            .ok_or_else(|| LspError::InvalidResponse("Missing result field".to_string()))?;

        serde_json::from_value(result.clone()).map_err(Into::into)
    }

    /// Send a notification (no response expected)
    async fn send_notification<P: Serialize>(&self, method: &str, params: P) -> Result<()> {
        let notification = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        });

        self.send_message(&notification).await
    }

    /// Send raw JSON-RPC message
    async fn send_message(&self, message: &serde_json::Value) -> Result<()> {
        let content = serde_json::to_string(message)?;
        let header = format!("Content-Length: {}\r\n\r\n", content.len());

        let mut stdin = self.stdin.lock().await;
        stdin.write_all(header.as_bytes()).await?;
        stdin.write_all(content.as_bytes()).await?;
        stdin.flush().await?;

        debug!("Sent LSP message: {}", method_from_message(message));
        Ok(())
    }

    /// Handle server output in background
    async fn handle_server_output(
        stdout: ChildStdout,
        pending_requests: Arc<Mutex<HashMap<u64, mpsc::Sender<serde_json::Value>>>>,
        diagnostics: Arc<RwLock<HashMap<Url, Vec<Diagnostic>>>>,
    ) {
        let mut reader = BufReader::new(stdout);
        let mut buffer = String::new();

        loop {
            buffer.clear();

            // Read headers
            let mut content_length = 0;
            loop {
                if reader.read_line(&mut buffer).await.is_err() {
                    return;
                }

                if buffer == "\r\n" {
                    break;
                }

                if let Some(len) = buffer.strip_prefix("Content-Length: ") {
                    content_length = len.trim().parse().unwrap_or(0);
                }

                buffer.clear();
            }

            if content_length == 0 {
                continue;
            }

            // Read content
            let mut content = vec![0u8; content_length];
            if reader.get_mut().read_exact(&mut content).await.is_err() {
                return;
            }

            let message: serde_json::Value = match serde_json::from_slice(&content) {
                Ok(msg) => msg,
                Err(e) => {
                    error!("Failed to parse LSP message: {}", e);
                    continue;
                }
            };

            debug!("Received LSP message: {:?}", message);

            // Handle response
            if let Some(id) = message.get("id").and_then(|v| v.as_u64()) {
                if let Some(tx) = pending_requests.lock().await.remove(&id) {
                    let _ = tx.send(message.clone()).await;
                }
            }

            // Handle notifications
            if let Some(method) = message.get("method").and_then(|v| v.as_str()) {
                if method == "textDocument/publishDiagnostics" {
                    if let Some(params) = message.get("params") {
                        Self::handle_diagnostics(params, &diagnostics).await;
                    }
                }
            }
        }
    }

    /// Handle diagnostic notifications
    async fn handle_diagnostics(
        params: &serde_json::Value,
        diagnostics: &Arc<RwLock<HashMap<Url, Vec<Diagnostic>>>>,
    ) {
        if let Ok(publish_params) =
            serde_json::from_value::<PublishDiagnosticsParams>(params.clone())
        {
            debug!(
                "Received {} diagnostics for {}",
                publish_params.diagnostics.len(),
                publish_params.uri
            );
            diagnostics
                .write()
                .await
                .insert(publish_params.uri, publish_params.diagnostics);
        }
    }

    // === Text Document Synchronization ===

    /// Notify server that a document was opened
    pub async fn did_open(&self, uri: Url, text: String, language_id: String) -> Result<()> {
        let params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri,
                language_id,
                version: 1,
                text,
            },
        };

        self.send_notification("textDocument/didOpen", params).await
    }

    /// Notify server of document changes
    pub async fn did_change(
        &self,
        uri: Url,
        version: i32,
        changes: Vec<TextDocumentContentChangeEvent>,
    ) -> Result<()> {
        let params = DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier { uri, version },
            content_changes: changes,
        };

        self.send_notification("textDocument/didChange", params)
            .await
    }

    /// Notify server that a document was saved
    pub async fn did_save(&self, uri: Url, text: Option<String>) -> Result<()> {
        let params = DidSaveTextDocumentParams {
            text_document: TextDocumentIdentifier { uri },
            text,
        };

        self.send_notification("textDocument/didSave", params).await
    }

    /// Notify server that a document was closed
    pub async fn did_close(&self, uri: Url) -> Result<()> {
        let params = DidCloseTextDocumentParams {
            text_document: TextDocumentIdentifier { uri },
        };

        self.send_notification("textDocument/didClose", params)
            .await
    }

    // === Language Features ===

    /// Request completion suggestions
    pub async fn completion(&self, uri: Url, position: Position) -> Result<Vec<CompletionItem>> {
        let params = CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position,
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
            context: None,
        };

        let response: Option<CompletionResponse> =
            self.send_request("textDocument/completion", params).await?;

        Ok(match response {
            Some(CompletionResponse::Array(items)) => items,
            Some(CompletionResponse::List(list)) => list.items,
            None => Vec::new(),
        })
    }

    /// Request hover information
    pub async fn hover(&self, uri: Url, position: Position) -> Result<Option<Hover>> {
        let params = HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position,
            },
            work_done_progress_params: Default::default(),
        };

        self.send_request("textDocument/hover", params).await
    }

    /// Go to definition
    pub async fn goto_definition(
        &self,
        uri: Url,
        position: Position,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let params = GotoDefinitionParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position,
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        };

        self.send_request("textDocument/definition", params).await
    }

    /// Get diagnostics for a document
    pub async fn diagnostics(&self, uri: &Url) -> Result<Vec<Diagnostic>> {
        Ok(self
            .diagnostics
            .read()
            .await
            .get(uri)
            .cloned()
            .unwrap_or_default())
    }

    /// Get server capabilities
    pub async fn capabilities(&self) -> Option<ServerCapabilities> {
        self.capabilities.read().await.clone()
    }

    /// Shutdown the server
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down LSP server");

        // Send shutdown request
        let _: serde_json::Value = self.send_request("shutdown", ()).await?;

        // Send exit notification
        self.send_notification("exit", ()).await?;

        // Kill the process if still running
        let mut process = self.server_process.lock().await;
        let _ = process.kill();

        Ok(())
    }
}

impl Drop for LspClient {
    fn drop(&mut self) {
        // Attempt graceful shutdown (fire and forget)
        let stdin = self.stdin.clone();
        tokio::spawn(async move {
            let exit = serde_json::json!({
                "jsonrpc": "2.0",
                "method": "exit",
            });
            let content = serde_json::to_string(&exit).unwrap();
            let header = format!("Content-Length: {}\r\n\r\n", content.len());

            let mut stdin = stdin.lock().await;
            let _ = stdin.write_all(header.as_bytes()).await;
            let _ = stdin.write_all(content.as_bytes()).await;
            let _ = stdin.flush().await;
        });
    }
}

/// Helper to extract method from message
fn method_from_message(message: &serde_json::Value) -> &str {
    message
        .get("method")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
}

/// Builder for LSP client
pub struct LspClientBuilder {
    command: String,
    args: Vec<String>,
    root_uri: Option<Url>,
}

impl LspClientBuilder {
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            args: Vec::new(),
            root_uri: None,
        }
    }

    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.args.extend(args.into_iter().map(Into::into));
        self
    }

    pub fn root_uri(mut self, uri: Url) -> Self {
        self.root_uri = Some(uri);
        self
    }

    pub async fn build(self) -> Result<LspClient> {
        let args: Vec<&str> = self.args.iter().map(|s| s.as_str()).collect();
        LspClient::new(&self.command, &args, self.root_uri).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder() {
        let builder = LspClientBuilder::new("rust-analyzer")
            .arg("--stdio")
            .root_uri(Url::parse("file:///tmp/test").unwrap());

        assert_eq!(builder.command, "rust-analyzer");
        assert_eq!(builder.args, vec!["--stdio"]);
    }

    #[test]
    fn test_method_extraction() {
        let msg = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
        });
        assert_eq!(method_from_message(&msg), "textDocument/didOpen");
    }
}
