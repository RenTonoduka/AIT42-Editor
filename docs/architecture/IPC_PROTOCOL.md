# AIT42 Editor - IPC Protocol Specification

**Version**: 1.0.0
**Date**: 2025-01-06
**Status**: Design Phase

---

## Table of Contents

1. [Overview](#overview)
2. [LSP Communication Protocol](#lsp-communication-protocol)
3. [AIT42 Agent Communication](#ait42-agent-communication)
4. [Tmux Session Protocol](#tmux-session-protocol)
5. [Event Streaming Protocol](#event-streaming-protocol)
6. [File Watcher Protocol](#file-watcher-protocol)
7. [Message Format Specifications](#message-format-specifications)
8. [Error Handling](#error-handling)
9. [Sequence Diagrams](#sequence-diagrams)

---

## Overview

AIT42 Editor uses multiple IPC mechanisms for communication between components:

1. **LSP Protocol**: JSON-RPC 2.0 over stdin/stdout with LSP servers
2. **AIT42 Agent Protocol**: Command-line invocation with JSON responses
3. **Tmux Protocol**: Command-line interface for session management
4. **Event Bus**: Internal tokio mpsc channels for component communication
5. **File Watcher**: Filesystem event notifications

### Communication Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    AIT42 Editor Process                     │
│                                                             │
│  ┌──────────┐   mpsc    ┌──────────┐   mpsc   ┌─────────┐ │
│  │   Core   │ ◄────────►│   TUI    │◄────────►│   LSP   │ │
│  │  Editor  │           │  Layer   │          │ Client  │ │
│  └──────────┘           └──────────┘          └────┬────┘ │
│       │ mpsc                 │ mpsc                 │      │
│       │                      │                      │      │
│  ┌────▼─────┐           ┌────▼────┐                │      │
│  │  AIT42   │           │  File   │                │      │
│  │Integration│           │Watcher  │                │      │
│  └────┬─────┘           └─────────┘                │      │
└───────┼──────────────────────────────────────────────┼──────┘
        │                                              │
        │ Process Spawn                     JSON-RPC 2.0
        │                                    (stdin/stdout)
        ▼                                              ▼
   ┌─────────────┐                          ┌──────────────────┐
   │    tmux     │                          │   LSP Servers    │
   │  Sessions   │                          │ (rust-analyzer,  │
   │             │                          │  typescript-ls)  │
   │ ┌─────────┐ │                          └──────────────────┘
   │ │Agent 1  │ │
   │ └─────────┘ │
   │ ┌─────────┐ │
   │ │Agent 2  │ │
   │ └─────────┘ │
   └─────────────┘
```

---

## LSP Communication Protocol

### Protocol: JSON-RPC 2.0

AIT42 Editor implements the Language Server Protocol (LSP) 3.17 specification using JSON-RPC 2.0 over stdin/stdout.

### Connection Lifecycle

```
Editor                                    LSP Server
  │                                           │
  │  1. Spawn process                         │
  ├──────────────────────────────────────────►│
  │                                           │
  │  2. initialize (JSON-RPC request)         │
  ├──────────────────────────────────────────►│
  │                                           │
  │  3. InitializeResult                      │
  │◄──────────────────────────────────────────┤
  │                                           │
  │  4. initialized (notification)            │
  ├──────────────────────────────────────────►│
  │                                           │
  │  ... text synchronization ...             │
  │                                           │
  │  5. shutdown (request)                    │
  ├──────────────────────────────────────────►│
  │                                           │
  │  6. null (response)                       │
  │◄──────────────────────────────────────────┤
  │                                           │
  │  7. exit (notification)                   │
  ├──────────────────────────────────────────►│
  │                                           │
  │  8. Process terminates                    │
  │                                           X
```

### Message Format

#### Initialize Request

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {
    "processId": 12345,
    "clientInfo": {
      "name": "ait42-editor",
      "version": "1.0.0"
    },
    "rootUri": "file:///path/to/project",
    "capabilities": {
      "textDocument": {
        "completion": {
          "dynamicRegistration": true,
          "completionItem": {
            "snippetSupport": true,
            "commitCharactersSupport": true,
            "documentationFormat": ["markdown", "plaintext"]
          }
        },
        "hover": {
          "dynamicRegistration": true,
          "contentFormat": ["markdown", "plaintext"]
        },
        "definition": {
          "dynamicRegistration": true,
          "linkSupport": true
        }
      }
    }
  }
}
```

#### Text Document Synchronization

**didOpen Notification**

```json
{
  "jsonrpc": "2.0",
  "method": "textDocument/didOpen",
  "params": {
    "textDocument": {
      "uri": "file:///path/to/main.rs",
      "languageId": "rust",
      "version": 1,
      "text": "fn main() {\n    println!(\"Hello\");\n}\n"
    }
  }
}
```

**didChange Notification**

```json
{
  "jsonrpc": "2.0",
  "method": "textDocument/didChange",
  "params": {
    "textDocument": {
      "uri": "file:///path/to/main.rs",
      "version": 2
    },
    "contentChanges": [
      {
        "range": {
          "start": { "line": 1, "character": 4 },
          "end": { "line": 1, "character": 4 }
        },
        "text": "// Comment\n    "
      }
    ]
  }
}
```

#### Completion Request

```json
{
  "jsonrpc": "2.0",
  "id": 42,
  "method": "textDocument/completion",
  "params": {
    "textDocument": {
      "uri": "file:///path/to/main.rs"
    },
    "position": {
      "line": 1,
      "character": 10
    }
  }
}
```

**Completion Response**

```json
{
  "jsonrpc": "2.0",
  "id": 42,
  "result": {
    "isIncomplete": false,
    "items": [
      {
        "label": "println!",
        "kind": 3,
        "detail": "macro",
        "documentation": {
          "kind": "markdown",
          "value": "Print to stdout with newline"
        },
        "insertText": "println!(\"$1\");$0",
        "insertTextFormat": 2
      }
    ]
  }
}
```

#### Diagnostics (Server Push)

```json
{
  "jsonrpc": "2.0",
  "method": "textDocument/publishDiagnostics",
  "params": {
    "uri": "file:///path/to/main.rs",
    "version": 2,
    "diagnostics": [
      {
        "range": {
          "start": { "line": 1, "character": 4 },
          "end": { "line": 1, "character": 15 }
        },
        "severity": 1,
        "code": "E0425",
        "message": "cannot find value `printn` in this scope",
        "source": "rustc"
      }
    ]
  }
}
```

### LSP Message Transport

```rust
/// LSP message transport layer
///
/// Handles message framing for JSON-RPC over stdin/stdout.
pub struct LspTransport {
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl LspTransport {
    /// Send JSON-RPC message
    ///
    /// Format: "Content-Length: {size}\r\n\r\n{json}"
    pub async fn send(&mut self, message: &str) -> Result<()> {
        let content = format!(
            "Content-Length: {}\r\n\r\n{}",
            message.len(),
            message
        );
        self.stdin.write_all(content.as_bytes()).await?;
        self.stdin.flush().await?;
        Ok(())
    }

    /// Receive JSON-RPC message
    ///
    /// Parses Content-Length header and reads exact number of bytes.
    pub async fn receive(&mut self) -> Result<String> {
        // Read headers
        let mut headers = HashMap::new();
        loop {
            let mut line = String::new();
            self.stdout.read_line(&mut line).await?;

            if line == "\r\n" {
                break; // End of headers
            }

            if let Some((key, value)) = line.split_once(':') {
                headers.insert(
                    key.trim().to_lowercase(),
                    value.trim().to_string()
                );
            }
        }

        // Read content
        let length: usize = headers
            .get("content-length")
            .and_then(|s| s.parse().ok())
            .ok_or(Error::InvalidLspMessage)?;

        let mut buffer = vec![0u8; length];
        self.stdout.read_exact(&mut buffer).await?;

        Ok(String::from_utf8(buffer)?)
    }
}
```

---

## AIT42 Agent Communication

### Protocol: Command-Line Invocation

AIT42 agents are invoked as separate processes. Communication is via:
- **Input**: Command-line arguments
- **Output**: Structured JSON to stdout
- **Errors**: JSON error objects to stderr
- **Progress**: Real-time text to stdout (when in tmux session)

### Agent Invocation

```bash
# Direct invocation
ait42 agent <agent-name> --task "<task description>"

# With tmux session
tmux new-session -d -s "ait42-backend-dev-1735678900" \
  "ait42 agent backend-developer --task 'Implement REST API'"
```

### Message Format

#### Agent Task Request

```json
{
  "agent": "backend-developer",
  "task": "Implement REST API for user management",
  "context": {
    "project_dir": "/path/to/project",
    "files": [
      "src/main.rs",
      "Cargo.toml"
    ],
    "environment": {
      "RUST_VERSION": "1.75.0"
    }
  },
  "options": {
    "parallel": true,
    "tmux_session": "ait42-backend-dev-1735678900",
    "coordinator": false
  }
}
```

#### Agent Response (Success)

```json
{
  "status": "success",
  "agent": "backend-developer",
  "task_id": "task-1735678900-42",
  "result": {
    "summary": "Successfully implemented REST API",
    "files_modified": [
      "src/api/users.rs",
      "src/main.rs"
    ],
    "files_created": [
      "tests/api_tests.rs"
    ],
    "output": "Created 3 endpoints: GET /users, POST /users, DELETE /users/:id"
  },
  "metadata": {
    "duration_ms": 12500,
    "model": "claude-3-7-sonnet-20250219",
    "tokens_used": 8432
  }
}
```

#### Agent Response (Error)

```json
{
  "status": "error",
  "agent": "backend-developer",
  "task_id": "task-1735678900-42",
  "error": {
    "code": "COMPILATION_FAILED",
    "message": "Rust compilation failed with 3 errors",
    "details": [
      {
        "file": "src/api/users.rs",
        "line": 42,
        "error": "expected `;`, found `}`"
      }
    ]
  },
  "metadata": {
    "duration_ms": 5200,
    "partial_output": "Generated code but compilation failed"
  }
}
```

#### Agent Progress (Streaming)

When running in tmux, agents can output progress updates:

```
[00:01] Analyzing codebase...
[00:03] Generating API endpoints...
[00:05] Writing src/api/users.rs...
[00:07] Running tests...
[00:10] All tests passed!
[00:12] Done.
```

### Agent Communication API

```rust
/// Agent communication client
pub struct AgentClient {
    coordinator_enabled: bool,
}

impl AgentClient {
    /// Invoke agent directly
    ///
    /// # Examples
    /// ```rust
    /// let client = AgentClient::new(true);
    /// let result = client.invoke_agent("backend-developer", "Implement API").await?;
    /// ```
    pub async fn invoke_agent(
        &self,
        agent_name: &str,
        task: &str,
    ) -> Result<AgentResult> {
        let output = Command::new("ait42")
            .arg("agent")
            .arg(agent_name)
            .arg("--task")
            .arg(task)
            .arg("--json")
            .output()
            .await?;

        if output.status.success() {
            let result: AgentResponse = serde_json::from_slice(&output.stdout)?;
            Ok(result.into())
        } else {
            let error: AgentError = serde_json::from_slice(&output.stderr)?;
            Err(Error::AgentFailed(error.message))
        }
    }

    /// Invoke agent in tmux session
    pub async fn invoke_in_tmux(
        &self,
        session: &TmuxSession,
        agent_name: &str,
        task: &str,
    ) -> Result<()> {
        let command = format!(
            "ait42 agent {} --task '{}' --tmux-session {}",
            agent_name,
            task.replace('\'', "\\'"),
            session.name
        );

        Command::new("tmux")
            .arg("send-keys")
            .arg("-t")
            .arg(&session.name)
            .arg(&command)
            .arg("C-m")
            .output()
            .await?;

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
struct AgentResponse {
    status: String,
    agent: String,
    task_id: String,
    result: AgentResultData,
    metadata: AgentMetadata,
}

#[derive(Debug, Deserialize)]
struct AgentError {
    status: String,
    agent: String,
    error: ErrorDetails,
}
```

---

## Tmux Session Protocol

### Protocol: Command-Line Interface

Tmux communication uses the `tmux` command-line tool.

### Session Management

#### Create Session

```bash
tmux new-session -d \
  -s "ait42-backend-dev-1735678900" \
  -x 200 \
  -y 50
```

**Response**: Exit code 0 (success) or 1 (error)

#### Send Command

```bash
tmux send-keys \
  -t "ait42-backend-dev-1735678900" \
  "echo 'Hello from agent'" \
  C-m
```

#### Capture Output

```bash
tmux capture-pane \
  -t "ait42-backend-dev-1735678900" \
  -p
```

**Output**: Pane content (plain text)

```
$ echo 'Hello from agent'
Hello from agent
$ ait42 agent backend-developer --task 'Implement API'
[00:01] Analyzing codebase...
[00:03] Generating API endpoints...
```

#### Kill Session

```bash
tmux kill-session -t "ait42-backend-dev-1735678900"
```

### Tmux Protocol API

```rust
/// Tmux session protocol implementation
pub struct TmuxProtocol;

impl TmuxProtocol {
    /// Create new tmux session
    pub async fn create_session(
        name: &str,
        width: u16,
        height: u16,
    ) -> Result<()> {
        let output = Command::new("tmux")
            .arg("new-session")
            .arg("-d")
            .arg("-s").arg(name)
            .arg("-x").arg(width.to_string())
            .arg("-y").arg(height.to_string())
            .output()
            .await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Tmux(format!("Failed to create session: {}", error)));
        }

        Ok(())
    }

    /// Send keys to session
    pub async fn send_keys(session: &str, keys: &str) -> Result<()> {
        let output = Command::new("tmux")
            .arg("send-keys")
            .arg("-t").arg(session)
            .arg(keys)
            .arg("C-m")
            .output()
            .await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Tmux(format!("Failed to send keys: {}", error)));
        }

        Ok(())
    }

    /// Capture pane content
    pub async fn capture_pane(session: &str) -> Result<String> {
        let output = Command::new("tmux")
            .arg("capture-pane")
            .arg("-t").arg(session)
            .arg("-p")
            .output()
            .await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Tmux(format!("Failed to capture pane: {}", error)));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Kill session
    pub async fn kill_session(session: &str) -> Result<()> {
        let output = Command::new("tmux")
            .arg("kill-session")
            .arg("-t").arg(session)
            .output()
            .await?;

        // Ignore error if session doesn't exist
        Ok(())
    }

    /// Check if session exists
    pub async fn has_session(session: &str) -> bool {
        let output = Command::new("tmux")
            .arg("has-session")
            .arg("-t").arg(session)
            .output()
            .await;

        output.map(|o| o.status.success()).unwrap_or(false)
    }
}
```

---

## Event Streaming Protocol

### Protocol: Tokio MPSC Channels

Internal components communicate via async message passing.

### Event Bus Architecture

```rust
/// Event bus for inter-component communication
pub struct EventBus {
    tx: mpsc::Sender<EditorEvent>,
    rx: mpsc::Receiver<EditorEvent>,
    subscribers: HashMap<String, Vec<mpsc::Sender<EditorEvent>>>,
}

impl EventBus {
    /// Create new event bus with buffer size
    pub fn new(buffer_size: usize) -> Self {
        let (tx, rx) = mpsc::channel(buffer_size);
        Self {
            tx,
            rx,
            subscribers: HashMap::new(),
        }
    }

    /// Get sender handle (cloneable)
    pub fn sender(&self) -> mpsc::Sender<EditorEvent> {
        self.tx.clone()
    }

    /// Subscribe to events
    ///
    /// Returns a receiver for filtered events.
    pub fn subscribe(&mut self, filter: &str) -> mpsc::Receiver<EditorEvent> {
        let (tx, rx) = mpsc::channel(100);
        self.subscribers
            .entry(filter.to_string())
            .or_insert_with(Vec::new)
            .push(tx);
        rx
    }

    /// Dispatch events to subscribers
    pub async fn dispatch_loop(&mut self) {
        while let Some(event) = self.rx.recv().await {
            // Send to all subscribers
            for (filter, subscribers) in &mut self.subscribers {
                if self.matches_filter(&event, filter) {
                    for sub in subscribers {
                        // Non-blocking send (drop if full)
                        let _ = sub.try_send(event.clone());
                    }
                }
            }

            // Handle event internally
            self.handle_event(event).await;
        }
    }

    fn matches_filter(&self, event: &EditorEvent, filter: &str) -> bool {
        match filter {
            "all" => true,
            "buffer" => matches!(
                event,
                EditorEvent::BufferChanged { .. }
                    | EditorEvent::BufferSaved(_)
                    | EditorEvent::BufferClosed(_)
            ),
            "lsp" => matches!(
                event,
                EditorEvent::LspResponse { .. }
                    | EditorEvent::DiagnosticsUpdated { .. }
            ),
            "agent" => matches!(
                event,
                EditorEvent::AgentStarted { .. }
                    | EditorEvent::AgentCompleted { .. }
                    | EditorEvent::AgentFailed { .. }
            ),
            _ => false,
        }
    }
}
```

### Event Flow Example

```
User presses 'i' in Normal Mode
    │
    ├─► KeyPress(KeyEvent) sent to event bus
    │
    ├─► NormalMode::handle_key() receives event
    │   └─► Returns ModeTransition::Switch(InsertMode)
    │
    ├─► EditorContext::switch_mode()
    │   ├─► NormalMode::on_exit()
    │   ├─► InsertMode::on_enter()
    │   └─► ModeChanged event sent to event bus
    │
    ├─► TUI Layer receives ModeChanged
    │   └─► Updates status bar
    │
    └─► Render frame
```

---

## File Watcher Protocol

### Protocol: Filesystem Events (notify crate)

Monitor file changes and notify editor.

### File Watcher API

```rust
use notify::{Watcher, RecursiveMode, Event, EventKind};

/// File watcher for external changes
pub struct FileWatcher {
    watcher: RecommendedWatcher,
    event_tx: mpsc::Sender<EditorEvent>,
}

impl FileWatcher {
    /// Create new file watcher
    pub fn new(event_tx: mpsc::Sender<EditorEvent>) -> Result<Self> {
        let tx = event_tx.clone();
        let watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
            if let Ok(event) = res {
                if let Err(e) = Self::handle_fs_event(&event, &tx) {
                    eprintln!("File watcher error: {}", e);
                }
            }
        })?;

        Ok(Self { watcher, event_tx })
    }

    /// Watch file for changes
    pub fn watch_file(&mut self, path: &Path) -> Result<()> {
        self.watcher.watch(path, RecursiveMode::NonRecursive)?;
        Ok(())
    }

    /// Stop watching file
    pub fn unwatch_file(&mut self, path: &Path) -> Result<()> {
        self.watcher.unwatch(path)?;
        Ok(())
    }

    fn handle_fs_event(
        event: &Event,
        tx: &mpsc::Sender<EditorEvent>,
    ) -> Result<()> {
        match event.kind {
            EventKind::Modify(_) => {
                for path in &event.paths {
                    tx.try_send(EditorEvent::FileChanged(path.clone()))?;
                }
            }
            _ => {}
        }
        Ok(())
    }
}
```

### File Change Flow

```
External process modifies file
    │
    ├─► Filesystem event triggered
    │
    ├─► notify watcher callback
    │
    ├─► FileChanged event sent to event bus
    │
    ├─► EditorContext receives event
    │   └─► Checks if file is open in buffer
    │
    ├─► Prompt user: "File changed on disk. Reload?"
    │   ├─► Yes: Reload buffer from disk
    │   └─► No: Keep current buffer (mark as diverged)
    │
    └─► Update UI
```

---

## Message Format Specifications

### JSON Schema for Agent Messages

#### AgentRequest Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["agent", "task"],
  "properties": {
    "agent": {
      "type": "string",
      "description": "Agent name"
    },
    "task": {
      "type": "string",
      "description": "Task description"
    },
    "context": {
      "type": "object",
      "properties": {
        "project_dir": { "type": "string" },
        "files": {
          "type": "array",
          "items": { "type": "string" }
        },
        "environment": {
          "type": "object",
          "additionalProperties": { "type": "string" }
        }
      }
    },
    "options": {
      "type": "object",
      "properties": {
        "parallel": { "type": "boolean" },
        "tmux_session": { "type": "string" },
        "coordinator": { "type": "boolean" }
      }
    }
  }
}
```

#### AgentResponse Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["status", "agent", "task_id"],
  "properties": {
    "status": {
      "type": "string",
      "enum": ["success", "error"]
    },
    "agent": { "type": "string" },
    "task_id": { "type": "string" },
    "result": {
      "type": "object",
      "properties": {
        "summary": { "type": "string" },
        "files_modified": {
          "type": "array",
          "items": { "type": "string" }
        },
        "files_created": {
          "type": "array",
          "items": { "type": "string" }
        },
        "output": { "type": "string" }
      }
    },
    "error": {
      "type": "object",
      "properties": {
        "code": { "type": "string" },
        "message": { "type": "string" },
        "details": { "type": "array" }
      }
    },
    "metadata": {
      "type": "object",
      "properties": {
        "duration_ms": { "type": "integer" },
        "model": { "type": "string" },
        "tokens_used": { "type": "integer" }
      }
    }
  }
}
```

---

## Error Handling

### LSP Error Responses

```json
{
  "jsonrpc": "2.0",
  "id": 42,
  "error": {
    "code": -32700,
    "message": "Parse error",
    "data": {
      "details": "Invalid JSON at line 5"
    }
  }
}
```

**Standard Error Codes**:
- `-32700`: Parse error
- `-32600`: Invalid request
- `-32601`: Method not found
- `-32602`: Invalid params
- `-32603`: Internal error

### Agent Error Codes

```rust
pub enum AgentErrorCode {
    /// Agent not found
    AgentNotFound,

    /// Task parsing failed
    InvalidTask,

    /// Agent execution failed
    ExecutionFailed,

    /// Compilation error
    CompilationFailed,

    /// Test failure
    TestFailed,

    /// Timeout
    Timeout,

    /// Resource limit exceeded
    ResourceLimitExceeded,
}
```

### Retry Logic

```rust
/// Retry policy for transient failures
pub struct RetryPolicy {
    max_retries: usize,
    backoff: Duration,
}

impl RetryPolicy {
    pub async fn retry<F, T, E>(&self, mut f: F) -> Result<T, E>
    where
        F: FnMut() -> Pin<Box<dyn Future<Output = Result<T, E>> + Send>>,
        E: std::fmt::Display,
    {
        let mut attempts = 0;
        loop {
            match f().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    attempts += 1;
                    if attempts >= self.max_retries {
                        return Err(e);
                    }
                    eprintln!("Retry attempt {}/{}: {}", attempts, self.max_retries, e);
                    tokio::time::sleep(self.backoff * attempts as u32).await;
                }
            }
        }
    }
}
```

---

## Sequence Diagrams

### LSP Completion Sequence

```
User          Editor          LSP Client          rust-analyzer
 │              │                  │                     │
 │ Type 'p'     │                  │                     │
 ├─────────────►│                  │                     │
 │              │                  │                     │
 │              │ Buffer::insert() │                     │
 │              ├─────────────────►│                     │
 │              │                  │                     │
 │              │ Debounce 100ms   │                     │
 │              │◄─────────────────┤                     │
 │              │                  │                     │
 │              │                  │ textDocument/       │
 │              │                  │   completion        │
 │              │                  ├────────────────────►│
 │              │                  │                     │
 │              │                  │ CompletionList      │
 │              │                  │◄────────────────────┤
 │              │                  │                     │
 │              │ CompletionItems  │                     │
 │              │◄─────────────────┤                     │
 │              │                  │                     │
 │ Show popup   │                  │                     │
 │◄─────────────┤                  │                     │
 │              │                  │                     │
```

### Agent Execution in Tmux

```
User       Editor       AgentExecutor    TmuxManager      tmux       Agent
 │            │              │               │             │           │
 │ Ctrl+P     │              │               │             │           │
 ├───────────►│              │               │             │           │
 │            │              │               │             │           │
 │ Select     │              │               │             │           │
 │ "backend-  │              │               │             │           │
 │ developer" │              │               │             │           │
 ├───────────►│              │               │             │           │
 │            │              │               │             │           │
 │            │ execute()    │               │             │           │
 │            ├─────────────►│               │             │           │
 │            │              │               │             │           │
 │            │              │ create_       │             │           │
 │            │              │   session()   │             │           │
 │            │              ├──────────────►│             │           │
 │            │              │               │             │           │
 │            │              │               │ new-session │           │
 │            │              │               ├────────────►│           │
 │            │              │               │             │           │
 │            │              │               │ OK          │           │
 │            │              │               │◄────────────┤           │
 │            │              │               │             │           │
 │            │              │ execute_in_   │             │           │
 │            │              │   session()   │             │           │
 │            │              ├──────────────►│             │           │
 │            │              │               │             │           │
 │            │              │               │ send-keys   │           │
 │            │              │               ├────────────►│           │
 │            │              │               │             │           │
 │            │              │               │             │ ait42     │
 │            │              │               │             │   agent   │
 │            │              │               │             ├──────────►│
 │            │              │               │             │           │
 │            │              │               │             │ Running   │
 │            │              │               │             │           │
 │            │ Poll output  │               │             │           │
 │            │◄─────────────┤               │             │           │
 │            │              │               │             │           │
 │            │              │ capture_      │             │           │
 │            │              │   output()    │             │           │
 │            │              ├──────────────►│             │           │
 │            │              │               │             │           │
 │            │              │               │ capture-    │           │
 │            │              │               │   pane      │           │
 │            │              │               ├────────────►│           │
 │            │              │               │             │           │
 │            │              │               │ Output      │           │
 │            │              │               │◄────────────┤           │
 │            │              │               │             │           │
 │            │              │ Output        │             │           │
 │            │              │◄──────────────┤             │           │
 │            │              │               │             │           │
 │ Display    │ Output       │               │             │           │
 │   in UI    │◄─────────────┤               │             │           │
 │◄───────────┤              │               │             │           │
 │            │              │               │             │           │
```

---

**End of IPC Protocol Specification**

Generated by: AIT42 Coordinator
Date: 2025-01-06
Version: 1.0.0
