# Security Architecture - AIT42 Editor

**Version**: 1.0.0
**Date**: 2025-11-03
**Status**: Design Phase - Security Review
**Classification**: Internal - Security Design Document

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Security Requirements](#security-requirements)
3. [Security Architecture Overview](#security-architecture-overview)
4. [Component Security Design](#component-security-design)
5. [Defense-in-Depth Strategy](#defense-in-depth-strategy)
6. [Secure Coding Guidelines](#secure-coding-guidelines)
7. [Security Testing Plan](#security-testing-plan)
8. [Incident Response](#incident-response)

---

## Executive Summary

### Security Posture

AIT42 Editor is a **macOS-native terminal-based code editor** with integrated AI agent execution capabilities. The security architecture follows **Defense-in-Depth** principles with multiple layers of protection:

- **Trust Model**: Zero Trust for external inputs, controlled trust for AIT42 agents
- **Attack Surface**: File system operations, LSP servers, AIT42 agent execution, configuration
- **Security Classification**: Medium Risk (local execution, handles sensitive code)
- **Compliance**: No regulatory requirements (local-only, no data collection)

### Key Security Principles

1. **Least Privilege**: Minimal file system permissions, no root/sudo operations
2. **Fail Secure**: Default-deny configuration, secure defaults
3. **Input Validation**: Sanitize all external inputs (user, LSP, file system)
4. **Process Isolation**: Tmux session isolation for agent execution
5. **Memory Safety**: Rust's compile-time guarantees + runtime validation

### Security Goals

| Goal | Target | Verification Method |
|------|--------|---------------------|
| **No arbitrary code execution** | Zero exploitable paths | Security audit + fuzzing |
| **File permission enforcement** | 100% respect macOS permissions | Integration tests |
| **Agent isolation** | Separate tmux sessions | Process inspection |
| **Safe configuration** | Validated schemas | Fuzz testing |
| **Dependency security** | Zero critical CVEs | `cargo audit` |

---

## Security Requirements

### SR-1: File System Security

**Requirement**: Editor SHALL respect macOS file permissions and SHALL NOT attempt privilege escalation.

**Controls**:
- Check file permissions before read/write operations
- Atomic file writes with temporary file + rename
- Secure temporary file creation with restrictive permissions (0600)
- Prevent directory traversal attacks
- Validate symlink resolution

**Verification**:
```rust
// Example test case
#[test]
fn test_respects_readonly_files() {
    let readonly_file = create_readonly_test_file();
    let result = editor.save(&readonly_file, "content");
    assert!(matches!(result, Err(Error::PermissionDenied(_))));
}
```

---

### SR-2: LSP Server Security

**Requirement**: Editor SHALL validate all LSP responses and SHALL isolate LSP server processes.

**Controls**:
- Schema validation for all LSP messages
- Timeout enforcement (5 second default)
- Resource limits on LSP server processes
- Sanitize LSP-provided URIs and file paths
- Rate limiting for LSP requests

**Attack Scenarios**:
- **Malicious LSP server sends crafted responses**: Mitigated by schema validation
- **LSP server consumes excessive resources**: Mitigated by process limits
- **LSP server attempts file system access**: Mitigated by process isolation

---

### SR-3: AIT42 Agent Execution Security

**Requirement**: AIT42 agents SHALL execute in isolated environments with auditable execution logs.

**Controls**:
- Tmux session isolation (one session per agent)
- Execution audit logs (`~/.ait42-editor/audit/`)
- User confirmation for destructive operations
- Resource limits per agent (CPU, memory, time)
- Agent output sanitization before display

**Security Model**:
```
User Intent → Explicit Confirmation → Agent Execution → Audit Log
                                    ↓
                              Tmux Session (Isolated)
                                    ↓
                              Monitor & Timeout
```

---

### SR-4: Configuration Security

**Requirement**: Configuration files SHALL be validated and SHALL provide secure defaults.

**Controls**:
- TOML schema validation
- Allowlist-based configuration options
- Secure defaults (e.g., `auto_execute = false`)
- Configuration file permission check (warn if world-writable)
- No secret storage in plain text (use macOS Keychain for API keys)

**Secure Configuration Example**:
```toml
[editor]
theme = "monokai"              # Validated against built-in themes
tab_size = 4                   # Range: 1-8
auto_save = false              # Default: off for safety

[ait42]
coordinator_enabled = false    # Default: explicit opt-in
tmux_parallel_max = 5          # Range: 1-10
require_confirmation = true    # Default: always confirm

[lsp]
# Only allow known-good LSP servers
rust = "rust-analyzer"         # Validated against allowlist
timeout_ms = 5000              # Max: 30000
```

---

### SR-5: Input Validation

**Requirement**: All external inputs SHALL be validated and sanitized.

**Input Sources**:
1. **Keyboard input**: UTF-8 validation, control character filtering
2. **File content**: UTF-8 validation, BOM handling, size limits
3. **LSP responses**: JSON schema validation, size limits
4. **Configuration files**: TOML parsing + schema validation
5. **Command-line arguments**: Argument parsing validation

**Validation Strategy**:
```rust
pub trait ValidatedInput {
    type Output;
    type Error;

    fn validate(&self) -> Result<Self::Output, Self::Error>;
}

impl ValidatedInput for FileContent {
    type Output = String;
    type Error = ValidationError;

    fn validate(&self) -> Result<String, ValidationError> {
        // 1. Check size limit (default: 100MB)
        if self.size() > MAX_FILE_SIZE {
            return Err(ValidationError::FileTooLarge);
        }

        // 2. Validate UTF-8
        let content = std::str::from_utf8(&self.bytes)
            .map_err(|_| ValidationError::InvalidUtf8)?;

        // 3. Sanitize control characters (keep \n, \t, \r)
        let sanitized = sanitize_control_chars(content);

        Ok(sanitized)
    }
}
```

---

### SR-6: Dependency Management

**Requirement**: All dependencies SHALL be audited for security vulnerabilities.

**Controls**:
- `Cargo.lock` version pinning
- Weekly `cargo audit` runs in CI/CD
- Dependency review for new additions
- Prefer well-maintained crates with active security response
- Use minimal feature flags to reduce attack surface

**Dependency Risk Assessment**:

| Dependency | Risk Level | Justification | Mitigation |
|------------|------------|---------------|------------|
| **tokio** | Low | Mature, widely audited | Lock version, monitor CVEs |
| **ropey** | Low | Simple, well-tested | Fork available if needed |
| **ratatui** | Low | Active maintenance | Regular updates |
| **tower-lsp** | Medium | Complex, network handling | Input validation, timeouts |
| **serde** | Low | Battle-tested | Use derive macros only |
| **notify** | Medium | File system monitoring | Validate paths |

---

## Security Architecture Overview

### Security Domains

```
┌────────────────────────────────────────────────────────────────┐
│                      UNTRUSTED ZONE                            │
│                                                                │
│  User Input  │  File System  │  LSP Servers  │  AIT42 Agents │
└───────┬──────┴───────┬───────┴───────┬───────┴────────┬───────┘
        │              │               │                │
        │    ┌─────────▼───────────────▼────────────────▼──────┐
        │    │         VALIDATION LAYER                        │
        │    │  - Input sanitization                           │
        │    │  - Schema validation                            │
        │    │  - Permission checks                            │
        │    └─────────┬───────────────┬────────────────┬──────┘
        │              │               │                │
┌───────▼──────────────▼───────────────▼────────────────▼───────┐
│                    TRUSTED CORE                               │
│                                                               │
│  ait42-core  │  ait42-tui  │  ait42-lsp  │  ait42-ait42    │
│                                                               │
│  Security Guarantees:                                        │
│  - Rust memory safety                                        │
│  - Type safety                                               │
│  - Validated inputs only                                     │
└───────────────────────────────────────────────────────────────┘
```

### Trust Boundaries

1. **User → Editor**: Trusted after initial validation
2. **File System → Editor**: Untrusted, validate all reads
3. **LSP Server → Editor**: Untrusted, validate all responses
4. **AIT42 Agent → Editor**: Controlled trust, audit all actions
5. **Configuration → Editor**: Untrusted, validate schema

---

## Component Security Design

### 1. ait42-core: Core Editor Security

**Threat Model**:
- Buffer overflow attacks → **Mitigated by Rust**
- Invalid UTF-8 sequences → **Validated on input**
- Undo/redo history exhaustion → **Size limits**

**Security Controls**:

```rust
// Buffer security wrapper
pub struct SecureBuffer {
    inner: Rope,
    max_size: usize,
    change_limit: usize,
    undo_history_max: usize,
}

impl SecureBuffer {
    pub fn new(max_size: usize) -> Self {
        Self {
            inner: Rope::new(),
            max_size,
            change_limit: 10_000, // Max changes per operation
            undo_history_max: 1_000,
        }
    }

    pub fn insert(&mut self, pos: usize, text: &str) -> Result<()> {
        // 1. Validate position
        if pos > self.inner.len_chars() {
            return Err(Error::InvalidPosition);
        }

        // 2. Check size limit
        if self.inner.len_bytes() + text.len() > self.max_size {
            return Err(Error::BufferSizeLimitExceeded);
        }

        // 3. Validate UTF-8
        if !text.is_char_boundary(0) {
            return Err(Error::InvalidUtf8);
        }

        // 4. Apply change
        self.inner.insert(pos, text);

        Ok(())
    }
}
```

**Key Security Features**:
- ✅ Memory-safe by design (Rust)
- ✅ Bounds checking on all buffer operations
- ✅ UTF-8 validation
- ✅ Size limits to prevent DoS

---

### 2. ait42-fs: File System Security

**Threat Model**:
- Path traversal attacks (e.g., `../../etc/passwd`)
- Symlink attacks
- Race conditions (TOCTOU)
- Permission bypass attempts

**Security Controls**:

```rust
use std::path::{Path, PathBuf};
use std::os::unix::fs::PermissionsExt;

pub struct SecureFileSystem {
    allowed_roots: Vec<PathBuf>,
    follow_symlinks: bool,
}

impl SecureFileSystem {
    /// Validate and canonicalize path
    pub fn validate_path(&self, path: &Path) -> Result<PathBuf> {
        // 1. Canonicalize (resolve symlinks, remove ..)
        let canonical = path.canonicalize()
            .map_err(|_| Error::InvalidPath)?;

        // 2. Check against allowed roots (if configured)
        if !self.allowed_roots.is_empty() {
            let is_allowed = self.allowed_roots.iter()
                .any(|root| canonical.starts_with(root));

            if !is_allowed {
                return Err(Error::PathNotAllowed);
            }
        }

        // 3. Check for symlinks (if not following)
        if !self.follow_symlinks && self.is_symlink(&canonical)? {
            return Err(Error::SymlinkNotAllowed);
        }

        Ok(canonical)
    }

    /// Safe file write with atomic rename
    pub async fn safe_write(&self, path: &Path, content: &[u8])
        -> Result<()>
    {
        // 1. Validate path
        let path = self.validate_path(path)?;

        // 2. Check write permission
        if path.exists() {
            let metadata = tokio::fs::metadata(&path).await?;
            let permissions = metadata.permissions();

            if permissions.mode() & 0o200 == 0 {
                return Err(Error::PermissionDenied);
            }
        }

        // 3. Write to temporary file
        let tmp_path = path.with_extension(".tmp");
        tokio::fs::write(&tmp_path, content).await?;

        // 4. Set restrictive permissions (0644)
        let mut perms = tokio::fs::metadata(&tmp_path).await?
            .permissions();
        perms.set_mode(0o644);
        tokio::fs::set_permissions(&tmp_path, perms).await?;

        // 5. Atomic rename
        tokio::fs::rename(&tmp_path, &path).await?;

        Ok(())
    }
}
```

**Key Security Features**:
- ✅ Path canonicalization to prevent traversal
- ✅ Permission checks before operations
- ✅ Atomic writes to prevent corruption
- ✅ Restrictive file permissions (0644)
- ✅ Optional root directory restriction

---

### 3. ait42-lsp: LSP Server Security

**Threat Model**:
- Malicious LSP server sends crafted responses
- LSP server attempts unauthorized file access
- Resource exhaustion (memory, CPU)
- Protocol confusion attacks

**Security Controls**:

```rust
use tower_lsp::jsonrpc::Result as LspResult;
use serde_json::Value;

pub struct SecureLspClient {
    client: LspClient,
    timeout: Duration,
    response_size_limit: usize,
    rate_limiter: RateLimiter,
}

impl SecureLspClient {
    pub async fn request_completion(&mut self, params: CompletionParams)
        -> Result<Vec<CompletionItem>>
    {
        // 1. Rate limiting
        self.rate_limiter.check_rate()?;

        // 2. Send request with timeout
        let response = tokio::time::timeout(
            self.timeout,
            self.client.completion(params)
        ).await
            .map_err(|_| Error::LspTimeout)?
            .map_err(|e| Error::LspError(e))?;

        // 3. Validate response size
        let response_json = serde_json::to_string(&response)?;
        if response_json.len() > self.response_size_limit {
            return Err(Error::LspResponseTooLarge);
        }

        // 4. Validate response schema
        let items = match response {
            Some(CompletionResponse::Array(items)) => items,
            Some(CompletionResponse::List(list)) => list.items,
            None => vec![],
        };

        // 5. Sanitize completion items
        let sanitized = items.into_iter()
            .map(|item| self.sanitize_completion_item(item))
            .collect::<Result<Vec<_>>>()?;

        Ok(sanitized)
    }

    fn sanitize_completion_item(&self, item: CompletionItem)
        -> Result<CompletionItem>
    {
        // Sanitize label (remove control characters)
        let label = sanitize_string(&item.label)?;

        // Validate text edit positions
        if let Some(edit) = &item.text_edit {
            self.validate_text_edit(edit)?;
        }

        Ok(CompletionItem {
            label,
            ..item
        })
    }
}
```

**Key Security Features**:
- ✅ Timeout enforcement (5 seconds default)
- ✅ Response size limits (1MB default)
- ✅ Schema validation for all messages
- ✅ Rate limiting to prevent abuse
- ✅ URI and path sanitization

---

### 4. ait42-ait42: Agent Execution Security

**Threat Model**:
- Command injection in agent parameters
- Runaway agents consuming resources
- Agent output exploiting terminal vulnerabilities
- Multiple agents interfering with each other

**Security Controls**:

```rust
use std::process::Command;
use std::time::Duration;

pub struct SecureAgentExecutor {
    sessions: HashMap<AgentId, TmuxSession>,
    max_parallel: usize,
    execution_timeout: Duration,
    audit_logger: AuditLogger,
}

impl SecureAgentExecutor {
    pub async fn execute_agent(&mut self, agent: Agent, params: AgentParams)
        -> Result<AgentExecution>
    {
        // 1. Check parallel limit
        if self.sessions.len() >= self.max_parallel {
            return Err(Error::TooManyAgents);
        }

        // 2. Validate agent parameters (no shell metacharacters)
        self.validate_params(&params)?;

        // 3. Create isolated tmux session
        let session_name = self.create_secure_session_name(&agent);
        let session = self.create_tmux_session(&session_name).await?;

        // 4. Audit log
        self.audit_logger.log_agent_start(
            &agent.name,
            &params,
            &session_name
        ).await?;

        // 5. Execute with timeout
        let execution = tokio::time::timeout(
            self.execution_timeout,
            self.run_agent_in_session(&session, &agent, &params)
        ).await
            .map_err(|_| Error::AgentTimeout)?;

        // 6. Audit log completion
        self.audit_logger.log_agent_complete(
            &agent.name,
            execution.status,
            execution.duration
        ).await?;

        Ok(execution)
    }

    fn validate_params(&self, params: &AgentParams) -> Result<()> {
        // Disallow shell metacharacters
        const FORBIDDEN_CHARS: &[char] = &[';', '|', '&', '`', '$', '(', ')'];

        for (key, value) in &params.0 {
            if value.chars().any(|c| FORBIDDEN_CHARS.contains(&c)) {
                return Err(Error::UnsafeParameter(key.clone()));
            }
        }

        Ok(())
    }

    async fn create_tmux_session(&self, name: &str) -> Result<TmuxSession> {
        // Execute tmux with validated arguments only
        let output = Command::new("tmux")
            .arg("new-session")
            .arg("-d")  // Detached
            .arg("-s")
            .arg(name)  // Already validated
            .output()
            .await?;

        if !output.status.success() {
            return Err(Error::TmuxSessionFailed);
        }

        Ok(TmuxSession {
            name: name.to_string(),
            created_at: Instant::now(),
        })
    }

    fn sanitize_agent_output(&self, output: &str) -> String {
        // Remove ANSI escape sequences that could exploit terminal
        const DANGEROUS_SEQUENCES: &[&str] = &[
            "\x1b]",   // OSC (Operating System Command)
            "\x1b[>",  // Private mode
            "\x1b_",   // APC (Application Program Command)
        ];

        let mut sanitized = output.to_string();
        for seq in DANGEROUS_SEQUENCES {
            sanitized = sanitized.replace(seq, "");
        }

        sanitized
    }
}
```

**Key Security Features**:
- ✅ Tmux session isolation
- ✅ Command injection prevention
- ✅ Execution timeouts (default: 30 minutes)
- ✅ Resource limits via cgroups (Phase 2)
- ✅ Audit logging for all executions
- ✅ Terminal output sanitization

---

### 5. ait42-config: Configuration Security

**Threat Model**:
- Malicious configuration injection
- Secrets in plain text
- Configuration tampering

**Security Controls**:

```rust
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]  // Reject unknown keys
pub struct Config {
    #[serde(default = "default_editor_config")]
    pub editor: EditorConfig,

    #[serde(default)]
    pub ait42: AIT42Config,

    #[serde(default)]
    pub lsp: LspConfig,
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        // 1. Check file permissions (warn if world-writable)
        let metadata = std::fs::metadata(path)?;
        let permissions = metadata.permissions();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if permissions.mode() & 0o002 != 0 {
                eprintln!("WARNING: Config file is world-writable!");
            }
        }

        // 2. Read and parse
        let content = std::fs::read_to_string(path)?;
        let mut config: Config = toml::from_str(&content)
            .map_err(|e| Error::InvalidConfig(e.to_string()))?;

        // 3. Validate configuration
        config.validate()?;

        // 4. Load secrets from keychain (not from config file)
        config.load_secrets()?;

        Ok(config)
    }

    fn validate(&self) -> Result<()> {
        // Validate editor config
        if self.editor.tab_size == 0 || self.editor.tab_size > 8 {
            return Err(Error::InvalidConfig("tab_size must be 1-8".into()));
        }

        // Validate AIT42 config
        if self.ait42.tmux_parallel_max > 10 {
            return Err(Error::InvalidConfig(
                "tmux_parallel_max must be <= 10".into()
            ));
        }

        // Validate LSP config (allowlist only)
        for (lang, server) in &self.lsp.servers {
            if !ALLOWED_LSP_SERVERS.contains(&server.as_str()) {
                return Err(Error::InvalidConfig(
                    format!("Unknown LSP server: {}", server)
                ));
            }
        }

        Ok(())
    }

    fn load_secrets(&mut self) -> Result<()> {
        // Load API keys from macOS Keychain (Phase 2)
        // For MVP: use environment variables

        if let Ok(api_key) = std::env::var("ANTHROPIC_API_KEY") {
            self.ait42.api_key = Some(api_key);
        }

        Ok(())
    }
}

const ALLOWED_LSP_SERVERS: &[&str] = &[
    "rust-analyzer",
    "typescript-language-server",
    "pyright",
    "gopls",
    "clangd",
];
```

**Key Security Features**:
- ✅ Schema validation with `serde`
- ✅ Allowlist for LSP servers
- ✅ No secrets in configuration files
- ✅ File permission warnings
- ✅ Secure defaults

---

## Defense-in-Depth Strategy

### Layer 1: Perimeter Defense

**Goal**: Prevent malicious inputs from entering the system

**Controls**:
- Input validation at all entry points
- UTF-8 validation
- Size limits
- Sanitization of control characters

**Implementation**:
```rust
pub fn validate_user_input(input: &str) -> Result<String> {
    // 1. Check length
    if input.len() > MAX_INPUT_LENGTH {
        return Err(Error::InputTooLarge);
    }

    // 2. Validate UTF-8
    if !input.is_char_boundary(0) {
        return Err(Error::InvalidUtf8);
    }

    // 3. Remove dangerous control characters
    let sanitized = input.chars()
        .filter(|c| !c.is_control() || matches!(c, '\n' | '\t' | '\r'))
        .collect();

    Ok(sanitized)
}
```

---

### Layer 2: Application Security

**Goal**: Secure processing within the application

**Controls**:
- Type safety (Rust)
- Memory safety (Rust)
- Bounds checking
- Error handling (no panics in production)

**Implementation**:
```rust
// Use Result types everywhere, never panic
pub fn process_buffer_edit(edit: Edit) -> Result<()> {
    // Validate edit
    edit.validate()?;

    // Apply with bounds checking
    buffer.apply_edit(edit)
        .map_err(|e| Error::BufferError(e))?;

    Ok(())
}

// Set custom panic handler for graceful degradation
std::panic::set_hook(Box::new(|info| {
    error!("PANIC: {:?}", info);
    // Log to file, show user-friendly error
}));
```

---

### Layer 3: Process Isolation

**Goal**: Isolate untrusted components

**Controls**:
- Tmux sessions for agent execution
- Separate processes for LSP servers
- Resource limits (CPU, memory, time)

**Architecture**:
```
┌─────────────────┐
│  Editor Process │  ← Main application
└────────┬────────┘
         │
    ┌────┴────────────────────────┐
    │                             │
┌───▼───────┐          ┌──────────▼──────┐
│ LSP Server│          │  Tmux Session   │
│ (Isolated)│          │  (Agent Exec)   │
└───────────┘          └─────────────────┘
```

---

### Layer 4: Audit & Monitoring

**Goal**: Detect and respond to security events

**Controls**:
- Audit logging for all agent executions
- Error logging with context
- Security event monitoring

**Audit Log Format**:
```json
{
  "timestamp": "2025-11-03T10:30:00Z",
  "event_type": "agent_execution",
  "agent_name": "backend-developer",
  "user": "username",
  "session_id": "ait42-backend-1234567890",
  "status": "started",
  "parameters": {
    "task": "implement authentication"
  }
}
```

**Log Storage**: `~/.ait42-editor/audit/YYYY-MM-DD.log`

---

## Secure Coding Guidelines

### Rule 1: Never Use `unwrap()` or `expect()` in Production Code

**Bad**:
```rust
let file = std::fs::read_to_string(path).unwrap();  // ❌ Can panic
```

**Good**:
```rust
let file = std::fs::read_to_string(path)
    .map_err(|e| Error::FileReadFailed(path.to_path_buf(), e))?;  // ✅ Proper error handling
```

---

### Rule 2: Validate All External Inputs

**Bad**:
```rust
fn open_file(path: &str) {
    std::fs::read_to_string(path);  // ❌ No validation
}
```

**Good**:
```rust
fn open_file(path: &Path) -> Result<String> {
    // 1. Validate path
    let canonical = path.canonicalize()
        .map_err(|_| Error::InvalidPath)?;

    // 2. Check permissions
    if !canonical.exists() {
        return Err(Error::FileNotFound);
    }

    // 3. Read with error handling
    std::fs::read_to_string(&canonical)
        .map_err(|e| Error::FileReadFailed(canonical, e))
}
```

---

### Rule 3: Use Timeouts for All External Operations

**Bad**:
```rust
let response = lsp_client.completion(params).await;  // ❌ No timeout
```

**Good**:
```rust
let response = tokio::time::timeout(
    Duration::from_secs(5),
    lsp_client.completion(params)
).await
    .map_err(|_| Error::Timeout)?
    .map_err(|e| Error::LspError(e))?;  // ✅ With timeout
```

---

### Rule 4: Sanitize Before Display

**Bad**:
```rust
println!("{}", agent_output);  // ❌ May contain ANSI exploits
```

**Good**:
```rust
let sanitized = sanitize_ansi_escapes(&agent_output);
println!("{}", sanitized);  // ✅ Sanitized
```

---

### Rule 5: Use Secure Defaults

**Bad**:
```rust
#[derive(Default)]
struct Config {
    auto_execute_agents: bool,  // Defaults to false, but unclear
}
```

**Good**:
```rust
struct Config {
    #[serde(default = "default_false")]
    auto_execute_agents: bool,
}

fn default_false() -> bool { false }  // ✅ Explicit secure default
```

---

### Rule 6: Avoid Command Injection

**Bad**:
```rust
Command::new("sh")
    .arg("-c")
    .arg(format!("tmux new-session -s {}", user_input))  // ❌ Injection risk
    .spawn();
```

**Good**:
```rust
// Use individual arguments, never shell interpretation
Command::new("tmux")
    .arg("new-session")
    .arg("-s")
    .arg(&session_name)  // ✅ No shell interpretation
    .spawn();
```

---

## Security Testing Plan

### 1. Static Analysis

**Tools**:
- `cargo clippy` (lints)
- `cargo audit` (dependency vulnerabilities)
- `cargo deny` (license and security policy enforcement)

**CI/CD Integration**:
```yaml
# .github/workflows/security.yml
name: Security Checks
on: [push, pull_request]

jobs:
  security:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Install Rust
        uses: actions-rs/toolchain@v1
      - name: Clippy
        run: cargo clippy --all-targets -- -D warnings
      - name: Audit
        run: cargo audit
      - name: Deny check
        run: cargo deny check
```

---

### 2. Dynamic Analysis

**Fuzzing**:
```rust
// fuzz/fuzz_targets/buffer_operations.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use ait42_core::Buffer;

fuzz_target!(|data: &[u8]| {
    if let Ok(text) = std::str::from_utf8(data) {
        let mut buffer = Buffer::new();
        let _ = buffer.insert(0, text);
    }
});
```

**Run Fuzzing**:
```bash
cargo +nightly fuzz run buffer_operations -- -max_len=1048576
```

---

### 3. Integration Tests

**Security Test Cases**:

```rust
#[cfg(test)]
mod security_tests {
    use super::*;

    #[test]
    fn test_path_traversal_prevention() {
        let fs = SecureFileSystem::new();

        // Should reject path traversal
        assert!(fs.validate_path(Path::new("../../etc/passwd")).is_err());
        assert!(fs.validate_path(Path::new("/etc/passwd")).is_err());
    }

    #[test]
    fn test_lsp_response_size_limit() {
        let mut client = SecureLspClient::new();

        // Create oversized response
        let large_response = "x".repeat(2_000_000);

        assert!(matches!(
            client.process_response(&large_response),
            Err(Error::LspResponseTooLarge)
        ));
    }

    #[test]
    fn test_command_injection_prevention() {
        let executor = SecureAgentExecutor::new();

        let malicious_params = AgentParams::from_iter([
            ("task", "normal; rm -rf /")
        ]);

        assert!(matches!(
            executor.validate_params(&malicious_params),
            Err(Error::UnsafeParameter(_))
        ));
    }
}
```

---

### 4. Penetration Testing

**Test Scenarios**:

1. **Path Traversal**:
   - Try opening `../../etc/passwd`
   - Try opening `/etc/passwd`
   - Try symlink attacks

2. **LSP Exploitation**:
   - Send malformed JSON responses
   - Send oversized responses
   - Test timeout handling

3. **Agent Command Injection**:
   - Pass parameters with shell metacharacters
   - Test ANSI escape sequence exploits

4. **Configuration Tampering**:
   - Inject malicious TOML
   - Test unknown field handling

**Penetration Test Report Template**:
```markdown
## Test: Path Traversal Attack
- **Date**: 2025-11-03
- **Tester**: Security Team
- **Result**: PASS (Attack prevented)
- **Details**: Attempted to open `../../etc/passwd`, got `Error::InvalidPath`
```

---

## Incident Response

### Security Incident Classification

| Severity | Criteria | Response Time | Escalation |
|----------|----------|---------------|------------|
| **Critical** | Remote code execution, data loss | Immediate | CEO + CTO |
| **High** | Local privilege escalation, DoS | < 4 hours | CTO + Security Lead |
| **Medium** | Information disclosure, limited DoS | < 24 hours | Security Lead |
| **Low** | Minor information leak | < 1 week | Development Team |

---

### Incident Response Plan

**Phase 1: Detection & Triage** (0-1 hour)
1. Security event detected (automated or reported)
2. Classify severity
3. Assemble response team
4. Initial impact assessment

**Phase 2: Containment** (1-4 hours)
1. Disable affected features (if applicable)
2. Deploy emergency patch (if available)
3. Notify users (if user action required)
4. Document incident timeline

**Phase 3: Eradication** (4-24 hours)
1. Develop comprehensive fix
2. Test fix thoroughly
3. Prepare security advisory
4. Update documentation

**Phase 4: Recovery** (24-48 hours)
1. Deploy fix to all users
2. Verify fix effectiveness
3. Monitor for recurrence
4. Restore normal operations

**Phase 5: Post-Mortem** (1-2 weeks)
1. Root cause analysis
2. Lessons learned documentation
3. Update security procedures
4. Improve detection mechanisms

---

### Security Advisory Template

```markdown
# Security Advisory: AIT42-EDITOR-2025-001

## Summary
[Brief description of vulnerability]

## Impact
- **Severity**: [Critical/High/Medium/Low]
- **Affected Versions**: [Version range]
- **CVE ID**: CVE-2025-XXXXX (if assigned)

## Description
[Technical details of vulnerability]

## Mitigation
[Immediate steps users can take]

## Fix
[Version containing fix]

## Credits
[Researcher who reported issue]

## Timeline
- **Reported**: YYYY-MM-DD
- **Fixed**: YYYY-MM-DD
- **Disclosed**: YYYY-MM-DD
```

---

## Appendix A: Security Checklist

**Pre-Release Security Review**:

- [ ] All `unwrap()`/`expect()` calls reviewed
- [ ] `cargo audit` shows zero critical/high vulnerabilities
- [ ] `cargo clippy` passes with zero warnings
- [ ] All integration tests pass
- [ ] Fuzzing completed (24 hours minimum)
- [ ] Penetration testing completed
- [ ] Security documentation updated
- [ ] Incident response plan tested
- [ ] Dependency licenses reviewed
- [ ] Code signing configured (macOS Gatekeeper)

---

## Appendix B: Security Contacts

**Security Team**:
- **Security Lead**: [Name] <security-lead@example.com>
- **On-Call**: <security-oncall@example.com>

**Vulnerability Reporting**:
- **Email**: security@ait42-editor.com
- **PGP Key**: [Key ID]
- **Response Time**: < 24 hours

**Security Mailing List**:
- Subscribe: security-announce@ait42-editor.com

---

**End of Security Architecture Document**

**Version**: 1.0.0
**Date**: 2025-11-03
**Author**: Security Architect
**Next Review**: 2025-12-03
