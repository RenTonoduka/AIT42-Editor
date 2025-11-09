# Security Architecture Diagrams - AIT42 Editor

**Version**: 1.0.0
**Date**: 2025-11-03
**Purpose**: Visual representation of security architecture

---

## 1. Trust Boundaries & Attack Surface

```
┌────────────────────────────────────────────────────────────────┐
│                      UNTRUSTED ZONE                            │
│                   (External Inputs)                            │
├────────────────────────────────────────────────────────────────┤
│                                                                │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐       │
│  │ User Input   │  │ File System  │  │ LSP Servers  │       │
│  │              │  │              │  │              │       │
│  │ • Keyboard   │  │ • Files      │  │ • Rust LSP   │       │
│  │ • Mouse      │  │ • Directories│  │ • TypeScript │       │
│  │ • CLI args   │  │ • Symlinks   │  │ • Python     │       │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘       │
│         │                  │                  │               │
│  ┌──────▼──────┐  ┌────────▼────────┐  ┌─────▼─────┐       │
│  │ AIT42 Agents│  │  Configuration  │  │  Tmux     │       │
│  │             │  │                 │  │           │       │
│  │ • 49 agents │  │  • config.toml  │  │  • Sess.  │       │
│  │ • Coordin.  │  │  • Keybindings  │  │  • Cmd    │       │
│  └──────┬──────┘  └────────┬────────┘  └─────┬─────┘       │
│         │                   │                  │             │
└─────────┼───────────────────┼──────────────────┼─────────────┘
          │                   │                  │
     ═════▼═══════════════════▼══════════════════▼═════════
     ║          VALIDATION LAYER (Trust Boundary)        ║
     ║                                                    ║
     ║  • Input sanitization                             ║
     ║  • Schema validation                              ║
     ║  • Permission checks                              ║
     ║  • Path canonicalization                          ║
     ║  • ANSI sanitization                              ║
     ╚════════════════════════════════════════════════════╝
          │                   │                  │
┌─────────┼───────────────────┼──────────────────┼─────────────┐
│         ▼                   ▼                  ▼             │
│  ┌────────────────────────────────────────────────────┐     │
│  │            TRUSTED CORE                            │     │
│  │         (AIT42 Editor Application)                 │     │
│  ├────────────────────────────────────────────────────┤     │
│  │                                                    │     │
│  │  ┌────────┐  ┌────────┐  ┌────────┐  ┌────────┐ │     │
│  │  │ Core   │  │  TUI   │  │  LSP   │  │ AIT42  │ │     │
│  │  │        │  │        │  │        │  │        │ │     │
│  │  │ Buffer │  │ Render │  │ Client │  │ Agents │ │     │
│  │  │ Cursor │  │ Widget │  │ Handler│  │ Tmux   │ │     │
│  │  │ Modes  │  │ Layout │  │ Timeout│  │ Audit  │ │     │
│  │  └────────┘  └────────┘  └────────┘  └────────┘ │     │
│  │                                                    │     │
│  │  Security Guarantees:                             │     │
│  │  ✓ Rust memory safety                             │     │
│  │  ✓ Type safety                                    │     │
│  │  ✓ Validated inputs only                          │     │
│  │  ✓ No unsafe operations (or documented)           │     │
│  └────────────────────────────────────────────────────┘     │
│                                                              │
│                      TRUSTED ZONE                            │
└──────────────────────────────────────────────────────────────┘
          │                   │                  │
          ▼                   ▼                  ▼
┌──────────────────────────────────────────────────────────────┐
│               PROTECTED RESOURCES                             │
│                                                               │
│  • User source code                                          │
│  • Configuration data                                        │
│  • Audit logs                                                │
│  • Session state                                             │
└──────────────────────────────────────────────────────────────┘
```

---

## 2. Defense-in-Depth Layers

```
┌─────────────────────────────────────────────────────────────┐
│ Layer 4: AUDIT & MONITORING                                 │
│                                                             │
│  • Execution audit logs (all agents)                       │
│  • Security event logging                                  │
│  • Anomaly detection                                       │
│  • Incident response triggers                              │
│                                                             │
│  └─► ~/.ait42-editor/audit/YYYY-MM-DD.log                 │
└─────────────────────────────────────────────────────────────┘
         ▲
         │ Monitoring
         │
┌─────────────────────────────────────────────────────────────┐
│ Layer 3: PROCESS ISOLATION                                  │
│                                                             │
│  ┌────────────────┐  ┌────────────────┐  ┌──────────────┐ │
│  │ Tmux Session 1 │  │ LSP Server     │  │ File Watcher │ │
│  │                │  │ (Separate PID) │  │              │ │
│  │ Agent:         │  │                │  │ (notify)     │ │
│  │ backend-dev    │  │ rust-analyzer  │  │              │ │
│  └────────────────┘  └────────────────┘  └──────────────┘ │
│                                                             │
│  ┌────────────────┐  ┌────────────────┐                   │
│  │ Tmux Session 2 │  │ Tmux Session 3 │  ...             │
│  │                │  │                │                   │
│  │ Agent:         │  │ Agent:         │                   │
│  │ frontend-dev   │  │ test-generator │                   │
│  └────────────────┘  └────────────────┘                   │
│                                                             │
│  Resource Limits:                                          │
│  • Max 5 parallel agents                                   │
│  • 30 min timeout per agent                                │
│  • Session cleanup on completion                           │
└─────────────────────────────────────────────────────────────┘
         ▲
         │ Isolated Execution
         │
┌─────────────────────────────────────────────────────────────┐
│ Layer 2: APPLICATION SECURITY                               │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ Rust Memory Safety                                  │   │
│  │  • No buffer overflows                              │   │
│  │  • No use-after-free                                │   │
│  │  • No data races                                    │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ Bounds Checking                                     │   │
│  │  • Buffer operations validated                      │   │
│  │  • Array access checked                             │   │
│  │  • Cursor position validated                        │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ Error Handling                                      │   │
│  │  • No unwrap() in production                        │   │
│  │  • Result types everywhere                          │   │
│  │  • Graceful degradation                             │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
         ▲
         │ Type-Safe Processing
         │
┌─────────────────────────────────────────────────────────────┐
│ Layer 1: PERIMETER DEFENSE                                  │
│                                                             │
│  ┌────────────────┐  ┌────────────────┐  ┌──────────────┐ │
│  │ Input          │  │ Schema         │  │ Permission   │ │
│  │ Validation     │  │ Validation     │  │ Checks       │ │
│  │                │  │                │  │              │ │
│  │ • UTF-8 check  │  │ • LSP schema   │  │ • File read  │ │
│  │ • Size limits  │  │ • Config schema│  │ • File write │ │
│  │ • Sanitization │  │ • JSON valid   │  │ • Directory  │ │
│  └────────────────┘  └────────────────┘  └──────────────┘ │
│                                                             │
│  ┌────────────────┐  ┌────────────────┐  ┌──────────────┐ │
│  │ Path           │  │ Rate           │  │ Timeout      │ │
│  │ Canonicalize   │  │ Limiting       │  │ Enforcement  │ │
│  │                │  │                │  │              │ │
│  │ • Resolve ..   │  │ • LSP requests │  │ • LSP: 5s    │ │
│  │ • Symlinks     │  │ • Agent spawn  │  │ • Agent: 30m │ │
│  │ • Traversal    │  │ • File ops     │  │ • File: 10s  │ │
│  └────────────────┘  └────────────────┘  └──────────────┘ │
└─────────────────────────────────────────────────────────────┘

         External Inputs (Untrusted)
         ─────────────────────────────►
```

---

## 3. Threat Flow & Mitigation

### Scenario: Command Injection Attempt

```
┌─────────────────────────────────────────────────────────────┐
│ ATTACKER                                                    │
│                                                             │
│  Crafts malicious agent parameter:                         │
│  task = "build project; rm -rf /"                          │
│                                                             │
└─────────────────┬───────────────────────────────────────────┘
                  │
                  ▼
         ┌─────────────────┐
         │  User invokes   │
         │  agent with     │
         │  parameter      │
         └────────┬────────┘
                  │
                  ▼
┌─────────────────────────────────────────────────────────────┐
│ VALIDATION LAYER                                            │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ AgentExecutor::validate_params()                    │   │
│  │                                                     │   │
│  │  const FORBIDDEN_CHARS: &[char] =                  │   │
│  │      &[';', '|', '&', '`', '$', '(', ')'];        │   │
│  │                                                     │   │
│  │  for (key, value) in &params.0 {                   │   │
│  │      if value.chars().any(|c|                      │   │
│  │          FORBIDDEN_CHARS.contains(&c))             │   │
│  │      {                                              │   │
│  │          return Err(Error::UnsafeParameter);       │   │
│  │      }                                              │   │
│  │  }                                                  │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  Detection: Semicolon (;) found in parameter              │
│                                                             │
└─────────────────┬───────────────────────────────────────────┘
                  │
                  ▼
         ┌─────────────────┐
         │  ❌ BLOCKED     │
         │                 │
         │  Error returned:│
         │  "Unsafe param" │
         └────────┬────────┘
                  │
                  ▼
┌─────────────────────────────────────────────────────────────┐
│ FALLBACK LAYER (if validation bypassed)                    │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ Command::new("tmux")                                │   │
│  │     .arg("new-session")                             │   │
│  │     .arg("-s")                                      │   │
│  │     .arg(&session_name)  // Literal string         │   │
│  │     .spawn()                                        │   │
│  │                                                     │   │
│  │ NO shell interpretation - parameters passed as-is  │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  Even if validation fails, no shell execution occurs       │
│                                                             │
└─────────────────┬───────────────────────────────────────────┘
                  │
                  ▼
         ┌─────────────────┐
         │  ✅ SAFE        │
         │                 │
         │  Attack failed  │
         │  at 2 layers    │
         └─────────────────┘
```

---

## 4. File System Security Flow

```
┌─────────────────────────────────────────────────────────────┐
│ USER ACTION: Save file                                      │
│  Path: "../../etc/passwd"  (malicious input)               │
└─────────────────┬───────────────────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────────────────────┐
│ STEP 1: Path Validation                                    │
│                                                             │
│  SecureFileSystem::validate_path()                         │
│  ├─► Canonicalize path (resolve .., symlinks)             │
│  │    /Users/user/../../etc/passwd                        │
│  │    → /etc/passwd                                        │
│  │                                                          │
│  ├─► Check against allowed roots (if configured)           │
│  │    Allowed: /Users/user/projects/*                     │
│  │    Actual:  /etc/passwd                                │
│  │    ❌ NOT ALLOWED                                       │
│  │                                                          │
│  └─► Return Error::PathNotAllowed                          │
└─────────────────┬───────────────────────────────────────────┘
                  │
                  ▼ Error returned to user
┌─────────────────────────────────────────────────────────────┐
│ ALTERNATIVE: Valid path                                    │
│  Path: "src/main.rs"                                       │
└─────────────────┬───────────────────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────────────────────┐
│ STEP 2: Permission Check                                   │
│                                                             │
│  let metadata = fs::metadata(&path)?;                      │
│  let permissions = metadata.permissions();                 │
│                                                             │
│  if permissions.mode() & 0o200 == 0 {  // Write bit       │
│      return Err(Error::PermissionDenied);                  │
│  }                                                          │
│                                                             │
│  ✅ Write permission verified                              │
└─────────────────┬───────────────────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────────────────────┐
│ STEP 3: Atomic Write                                       │
│                                                             │
│  let tmp_path = path.with_extension(".tmp");               │
│                                                             │
│  // Write to temporary file                                │
│  fs::write(&tmp_path, content)?;                           │
│                                                             │
│  // Set restrictive permissions (0644)                     │
│  let mut perms = fs::metadata(&tmp_path)?.permissions();   │
│  perms.set_mode(0o644);                                    │
│  fs::set_permissions(&tmp_path, perms)?;                   │
│                                                             │
│  // Atomic rename (TOCTOU safe)                            │
│  fs::rename(&tmp_path, &path)?;                            │
│                                                             │
│  ✅ File saved safely                                      │
└─────────────────────────────────────────────────────────────┘

Defense-in-Depth:
├─► Layer 1: Path validation (directory traversal)
├─► Layer 2: Permission enforcement (unauthorized write)
└─► Layer 3: Atomic operations (TOCTOU prevention)
```

---

## 5. LSP Security Architecture

```
┌─────────────────────────────────────────────────────────────┐
│ LSP SERVER (External Process)                              │
│                                                             │
│  Examples:                                                  │
│  • rust-analyzer                                           │
│  • typescript-language-server                              │
│  • pyright                                                 │
│                                                             │
│  Trust Level: UNTRUSTED                                    │
└─────────────────┬───────────────────────────────────────────┘
                  │ JSON-RPC over stdio
                  │
                  ▼
┌─────────────────────────────────────────────────────────────┐
│ LSP CLIENT (AIT42 Editor)                                  │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ SecureLspClient                                     │   │
│  │                                                     │   │
│  │  1. Rate Limiter                                   │   │
│  │     └─► Max 100 pending requests                   │   │
│  │                                                     │   │
│  │  2. Timeout Enforcement                            │   │
│  │     └─► tokio::time::timeout(5 seconds)           │   │
│  │                                                     │   │
│  │  3. Response Size Limit                            │   │
│  │     └─► Max 1MB JSON response                      │   │
│  │                                                     │   │
│  │  4. Schema Validation                              │   │
│  │     └─► Validate CompletionResponse structure      │   │
│  │                                                     │   │
│  │  5. Content Sanitization                           │   │
│  │     └─► Remove ANSI escape sequences               │   │
│  │     └─► Validate file URIs                         │   │
│  │     └─► Canonicalize paths                         │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────┬───────────────────────────────────────────┘
                  │ Validated & sanitized data
                  │
                  ▼
┌─────────────────────────────────────────────────────────────┐
│ EDITOR CORE                                                 │
│                                                             │
│  • Display completions in UI                               │
│  • Jump to definitions                                     │
│  • Show diagnostics                                        │
│                                                             │
│  Trust Level: TRUSTED                                      │
└─────────────────────────────────────────────────────────────┘

Protection Mechanisms:

┌──────────────────┬──────────────────┬─────────────────────┐
│ Threat           │ Control          │ Status              │
├──────────────────┼──────────────────┼─────────────────────┤
│ DoS (flood)      │ Rate limiting    │ ✅ Implemented      │
│ DoS (hang)       │ Timeout          │ ✅ Implemented      │
│ DoS (memory)     │ Size limits      │ ✅ Implemented      │
│ Injection        │ Schema validation│ ✅ Implemented      │
│ Path traversal   │ Canonicalization │ ✅ Implemented      │
│ Terminal exploit │ ANSI sanitization│ ✅ Implemented      │
└──────────────────┴──────────────────┴─────────────────────┘
```

---

## 6. Agent Execution Security Model

```
┌─────────────────────────────────────────────────────────────┐
│ USER                                                        │
│                                                             │
│  Invokes agent: "backend-developer"                        │
│  With task: "Implement user authentication"                │
└─────────────────┬───────────────────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────────────────────┐
│ AGENT SELECTOR                                              │
│                                                             │
│  • Loads agent metadata from .claude/agents/               │
│  • Validates agent exists                                  │
│  • Checks parallel execution limit (max 5)                 │
│  • Determines execution mode (direct vs tmux)              │
└─────────────────┬───────────────────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────────────────────┐
│ PARAMETER VALIDATION                                        │
│                                                             │
│  validate_params():                                        │
│  ├─► Check for shell metacharacters                        │
│  │    Forbidden: ; | & ` $ ( )                            │
│  ├─► Check parameter length (max 1024 chars)               │
│  └─► Validate UTF-8 encoding                               │
│                                                             │
│  ✅ Parameters safe                                        │
└─────────────────┬───────────────────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────────────────────┐
│ AUDIT LOG (Start)                                          │
│                                                             │
│  {                                                          │
│    "timestamp": "2025-11-03T10:30:00Z",                    │
│    "event": "agent_start",                                 │
│    "agent": "backend-developer",                           │
│    "user": "username",                                     │
│    "task": "Implement user authentication",                │
│    "session": "ait42-backend-1730630400"                   │
│  }                                                          │
└─────────────────┬───────────────────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────────────────────┐
│ TMUX SESSION CREATION                                       │
│                                                             │
│  Command::new("tmux")                                      │
│      .arg("new-session")                                   │
│      .arg("-d")              // Detached                   │
│      .arg("-s")                                            │
│      .arg("ait42-backend-1730630400")  // Unique name     │
│      .spawn()?;                                            │
│                                                             │
│  ✅ Isolated session created                               │
└─────────────────┬───────────────────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────────────────────┐
│ AGENT EXECUTION (Isolated)                                  │
│                                                             │
│  ┌───────────────────────────────────────────────────────┐ │
│  │ Tmux Session: ait42-backend-1730630400              │ │
│  │                                                       │ │
│  │  • Runs Task tool with validated parameters         │ │
│  │  • Monitors resource usage                           │ │
│  │  • Enforces 30-minute timeout                        │ │
│  │  • Captures all output                               │ │
│  │                                                       │ │
│  │  Isolation guarantees:                               │ │
│  │  ✓ Separate process tree                            │ │
│  │  ✓ Cannot access other sessions                     │ │
│  │  ✓ Survives editor crash                            │ │
│  └───────────────────────────────────────────────────────┘ │
└─────────────────┬───────────────────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────────────────────┐
│ OUTPUT SANITIZATION                                         │
│                                                             │
│  sanitize_agent_output():                                  │
│  ├─► Remove dangerous ANSI sequences                       │
│  │    • OSC (Operating System Command): \x1b]            │
│  │    • Private mode: \x1b[>                             │
│  │    • APC (Application Program Command): \x1b_         │
│  └─► Validate UTF-8                                        │
│                                                             │
│  ✅ Safe output ready for display                          │
└─────────────────┬───────────────────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────────────────────┐
│ AUDIT LOG (Complete)                                       │
│                                                             │
│  {                                                          │
│    "timestamp": "2025-11-03T10:35:00Z",                    │
│    "event": "agent_complete",                              │
│    "agent": "backend-developer",                           │
│    "session": "ait42-backend-1730630400",                  │
│    "duration_sec": 300,                                    │
│    "status": "success"                                     │
│  }                                                          │
└─────────────────┬───────────────────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────────────────────┐
│ SESSION CLEANUP                                             │
│                                                             │
│  Command::new("tmux")                                      │
│      .arg("kill-session")                                  │
│      .arg("-t")                                            │
│      .arg("ait42-backend-1730630400")                      │
│      .spawn()?;                                            │
│                                                             │
│  ✅ Resources released                                     │
└─────────────────────────────────────────────────────────────┘

Security Controls:
├─► Parameter validation (injection prevention)
├─► Process isolation (tmux sessions)
├─► Resource limits (timeout, parallelism)
├─► Audit logging (non-repudiation)
├─► Output sanitization (terminal exploits)
└─► Cleanup (resource management)
```

---

## 7. Configuration Security

```
┌─────────────────────────────────────────────────────────────┐
│ CONFIGURATION FILE                                          │
│  ~/.config/ait42-editor/config.toml                        │
│                                                             │
│  Trust Level: UNTRUSTED (user-modifiable)                  │
└─────────────────┬───────────────────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────────────────────┐
│ LOAD & VALIDATE                                             │
│                                                             │
│  1. Check File Permissions                                 │
│     ├─► Warning if world-writable (0o002)                  │
│     └─► Recommend 0o644 permissions                        │
│                                                             │
│  2. Parse TOML                                             │
│     ├─► Use serde with strict schema                       │
│     ├─► deny_unknown_fields attribute                      │
│     └─► Reject malformed TOML                              │
│                                                             │
│  3. Schema Validation                                      │
│     ├─► Range check numeric values                         │
│     │    • tab_size: 1-8                                  │
│     │    • tmux_parallel_max: 1-10                        │
│     ├─► Allowlist for LSP servers                          │
│     │    • rust-analyzer ✅                               │
│     │    • typescript-language-server ✅                  │
│     │    • unknown-server ❌                              │
│     └─► Validate file paths (if any)                       │
│                                                             │
│  4. Secret Detection                                       │
│     ├─► Scan for patterns: "api_key", "password", "token" │
│     ├─► Warning if secrets detected                        │
│     └─► Recommend environment variables                    │
│                                                             │
│  5. Apply Secure Defaults                                  │
│     ├─► auto_execute = false                              │
│     ├─► require_confirmation = true                        │
│     └─► coordinator_enabled = false                        │
└─────────────────┬───────────────────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────────────────────┐
│ VALIDATED CONFIGURATION                                     │
│                                                             │
│  Loaded into memory with guarantees:                       │
│  ✓ All values within valid ranges                         │
│  ✓ No unknown/malicious fields                            │
│  ✓ Secure defaults applied                                │
│  ✓ No secrets in config (use env vars)                    │
└─────────────────────────────────────────────────────────────┘

Example Validation:

[editor]
tab_size = 100        ❌ INVALID (range: 1-8)
theme = "monokai"     ✅ VALID (built-in theme)
auto_save = true      ✅ VALID (boolean)

[ait42]
tmux_parallel_max = 20    ❌ INVALID (range: 1-10)
api_key = "sk-ant-..."    ⚠️  WARNING: Use env var instead

[lsp]
rust = "malicious-lsp"    ❌ INVALID (not in allowlist)
typescript = "typescript-language-server"  ✅ VALID
```

---

## 8. Incident Response Flow

```
┌─────────────────────────────────────────────────────────────┐
│ SECURITY EVENT DETECTED                                     │
│                                                             │
│  Sources:                                                   │
│  • Automated monitoring                                    │
│  • User report                                             │
│  • Security researcher disclosure                          │
│  • Dependency audit (cargo audit)                          │
└─────────────────┬───────────────────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────────────────────┐
│ PHASE 1: DETECTION & TRIAGE (0-1 hour)                     │
│                                                             │
│  ┌───────────────────────────────────────────────────────┐ │
│  │ 1. Classify Severity (DREAD)                         │ │
│  │    ├─► Critical (8-10): RCE, data loss              │ │
│  │    ├─► High (6-8): Privilege escalation, DoS        │ │
│  │    ├─► Medium (4-6): Info disclosure                │ │
│  │    └─► Low (2-4): Minor issues                      │ │
│  │                                                       │ │
│  │ 2. Assemble Response Team                            │ │
│  │    ├─► Security Lead                                 │ │
│  │    ├─► Development Lead                              │ │
│  │    └─► CTO (if critical)                            │ │
│  │                                                       │ │
│  │ 3. Initial Impact Assessment                         │ │
│  │    ├─► Affected versions                             │ │
│  │    ├─► Exploitation likelihood                       │ │
│  │    └─► Number of users at risk                      │ │
│  └───────────────────────────────────────────────────────┘ │
└─────────────────┬───────────────────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────────────────────┐
│ PHASE 2: CONTAINMENT (1-4 hours)                           │
│                                                             │
│  ┌───────────────────────────────────────────────────────┐ │
│  │ 1. Disable Affected Feature (if possible)            │ │
│  │    └─► Feature flag or config option                 │ │
│  │                                                       │ │
│  │ 2. Develop Emergency Patch                           │ │
│  │    ├─► Minimal fix for immediate deployment         │ │
│  │    ├─► Test thoroughly                               │ │
│  │    └─► Prepare hotfix release                       │ │
│  │                                                       │ │
│  │ 3. Notify Users (if action required)                 │ │
│  │    ├─► Security advisory draft                       │ │
│  │    ├─► Mitigation steps                              │ │
│  │    └─► ETA for fix                                  │ │
│  └───────────────────────────────────────────────────────┘ │
└─────────────────┬───────────────────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────────────────────┐
│ PHASE 3: ERADICATION (4-24 hours)                          │
│                                                             │
│  ┌───────────────────────────────────────────────────────┐ │
│  │ 1. Comprehensive Fix                                 │ │
│  │    ├─► Root cause addressed                          │ │
│  │    ├─► Related issues checked                        │ │
│  │    └─► Defense-in-depth improvements                │ │
│  │                                                       │ │
│  │ 2. Security Testing                                  │ │
│  │    ├─► Regression test written                       │ │
│  │    ├─► Penetration test for specific vulnerability  │ │
│  │    └─► Full security test suite                     │ │
│  │                                                       │ │
│  │ 3. Documentation                                      │ │
│  │    ├─► Update threat model                           │ │
│  │    ├─► Update security architecture                  │ │
│  │    └─► Prepare security advisory                    │ │
│  └───────────────────────────────────────────────────────┘ │
└─────────────────┬───────────────────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────────────────────┐
│ PHASE 4: RECOVERY (24-48 hours)                            │
│                                                             │
│  ┌───────────────────────────────────────────────────────┐ │
│  │ 1. Deploy Fix                                        │ │
│  │    ├─► Release patched version                       │ │
│  │    ├─► Update all distribution channels             │ │
│  │    └─► Notify users to update                       │ │
│  │                                                       │ │
│  │ 2. Verify Effectiveness                              │ │
│  │    ├─► Confirm vulnerability closed                  │ │
│  │    ├─► Monitor for exploitation attempts            │ │
│  │    └─► Check for similar issues                     │ │
│  │                                                       │ │
│  │ 3. Public Disclosure                                 │ │
│  │    ├─► Publish security advisory                     │ │
│  │    ├─► CVE assignment (if applicable)               │ │
│  │    └─► Credit researcher                            │ │
│  └───────────────────────────────────────────────────────┘ │
└─────────────────┬───────────────────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────────────────────┐
│ PHASE 5: POST-MORTEM (1-2 weeks)                           │
│                                                             │
│  ┌───────────────────────────────────────────────────────┐ │
│  │ 1. Root Cause Analysis                               │ │
│  │    ├─► What went wrong?                              │ │
│  │    ├─► Why wasn't it caught earlier?                │ │
│  │    └─► What were the contributing factors?          │ │
│  │                                                       │ │
│  │ 2. Lessons Learned                                   │ │
│  │    ├─► Update security procedures                    │ │
│  │    ├─► Improve detection mechanisms                  │ │
│  │    ├─► Add automated checks                         │ │
│  │    └─► Team training needs                          │ │
│  │                                                       │ │
│  │ 3. Process Improvements                              │ │
│  │    ├─► Update security checklist                     │ │
│  │    ├─► New test cases added                         │ │
│  │    ├─► Code review guidelines updated               │ │
│  │    └─► Documentation improved                       │ │
│  └───────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘

Timeline Example (Critical Vulnerability):

00:00  │ Vulnerability reported
00:30  │ Triage complete, team assembled
02:00  │ Emergency patch developed
04:00  │ Hotfix released, users notified
12:00  │ Comprehensive fix ready
24:00  │ Full patch deployed
48:00  │ Verification complete
1 week │ Post-mortem conducted
```

---

**End of Security Architecture Diagrams**

**Version**: 1.0.0
**Date**: 2025-11-03
**Purpose**: Visual companion to security documentation
