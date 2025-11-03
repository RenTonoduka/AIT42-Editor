# Threat Model - AIT42 Editor

**Version**: 1.0.0
**Date**: 2025-11-03
**Methodology**: STRIDE + DREAD
**Classification**: Internal - Security Analysis

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [System Overview](#system-overview)
3. [STRIDE Threat Analysis](#stride-threat-analysis)
4. [Attack Scenarios](#attack-scenarios)
5. [Risk Assessment Matrix](#risk-assessment-matrix)
6. [Security Controls Mapping](#security-controls-mapping)
7. [Threat Intelligence](#threat-intelligence)

---

## Executive Summary

### Threat Modeling Scope

**System**: AIT42 Editor - macOS terminal-based code editor with AI agent integration
**Version**: 1.0.0 (MVP)
**Environment**: Local macOS system
**Attack Surface**: File system, LSP servers, AIT42 agents, configuration, user input

### Key Findings

| Threat Category | High Risk | Medium Risk | Low Risk | Total |
|-----------------|-----------|-------------|----------|-------|
| **Spoofing** | 0 | 1 | 2 | 3 |
| **Tampering** | 1 | 3 | 1 | 5 |
| **Repudiation** | 0 | 1 | 1 | 2 |
| **Information Disclosure** | 0 | 2 | 3 | 5 |
| **Denial of Service** | 0 | 3 | 2 | 5 |
| **Elevation of Privilege** | 0 | 1 | 2 | 3 |
| **TOTAL** | 1 | 11 | 11 | 23 |

**Overall Risk Level**: **MEDIUM**

### Critical Threats (Risk Score > 7)

1. **T-02: Malicious File Modification** - DREAD: 8.0
   - Impact: Code injection, data corruption
   - Mitigation: Atomic writes, permission validation

---

## System Overview

### Architecture Diagram

```
┌────────────────────────────────────────────────────────────┐
│                      Attack Surface                        │
└────────────────────────────────────────────────────────────┘
         │                  │                  │
    User Input       File System         LSP Servers
         │                  │                  │
         ▼                  ▼                  ▼
┌──────────────────────────────────────────────────────────────┐
│                    AIT42 Editor Core                         │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐   │
│  │  Buffer  │  │   LSP    │  │  AIT42   │  │  Config  │   │
│  │ Manager  │  │  Client  │  │  Agent   │  │  Parser  │   │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘   │
└──────────────────────────────────────────────────────────────┘
         │                  │                  │
         ▼                  ▼                  ▼
┌────────────────────────────────────────────────────────────┐
│                 Protected Resources                         │
│  - File system (user files)                                │
│  - Process memory (buffers)                                │
│  - Configuration data                                      │
│  - Audit logs                                              │
└────────────────────────────────────────────────────────────┘
```

### Trust Boundaries

| Boundary | Trust Level | Description |
|----------|-------------|-------------|
| **User → Editor** | High | Trusted user, validated input |
| **File System → Editor** | Low | Untrusted, full validation required |
| **LSP Server → Editor** | Medium | Validated protocol, untrusted content |
| **AIT42 Agent → Editor** | Medium | Controlled execution, audited |
| **Config File → Editor** | Low | Untrusted, schema validation required |

### Assets & Criticality

| Asset | Criticality | Confidentiality | Integrity | Availability |
|-------|-------------|-----------------|-----------|--------------|
| **User source code** | Critical | High | Critical | High |
| **Configuration** | High | Medium | High | Medium |
| **Audit logs** | Medium | Low | High | Medium |
| **LSP credentials** | High | High | Medium | Low |
| **Editor state** | Medium | Low | Medium | High |

---

## STRIDE Threat Analysis

### S - Spoofing Identity

#### S-01: Malicious LSP Server Impersonation

**Description**: Attacker replaces legitimate LSP server with malicious version

**Attack Vector**:
1. User configuration points to attacker-controlled LSP server
2. Malicious server responds with crafted completions
3. Code injection via completion items

**DREAD Score**:
- **Damage**: 6/10 (Code injection, limited scope)
- **Reproducibility**: 3/10 (Requires user misconfiguration)
- **Exploitability**: 4/10 (Requires setup)
- **Affected Users**: 2/10 (Individual user only)
- **Discoverability**: 7/10 (LSP protocol well-known)
- **TOTAL**: 4.4/10 (MEDIUM)

**Mitigations**:
- ✅ Allowlist of known-good LSP servers in default config
- ✅ Schema validation of LSP responses
- ✅ Sanitization of completion items
- ✅ User warning when using non-default LSP server

**Status**: Mitigated (Medium confidence)

---

#### S-02: Tmux Session Hijacking

**Description**: Attacker attaches to existing tmux session to intercept agent execution

**Attack Vector**:
1. Attacker gains shell access on victim's system
2. Lists tmux sessions: `tmux list-sessions`
3. Attaches to AIT42 agent session
4. Injects commands or reads sensitive data

**DREAD Score**:
- **Damage**: 7/10 (Full agent control)
- **Reproducibility**: 8/10 (Easy if shell access obtained)
- **Exploitability**: 2/10 (Requires prior system compromise)
- **Affected Users**: 2/10 (Individual user)
- **Discoverability**: 5/10 (Tmux basics well-known)
- **TOTAL**: 4.8/10 (MEDIUM-LOW)

**Mitigations**:
- ✅ Tmux session permissions (user-only)
- ⚠️ Consider tmux socket permission hardening (Phase 2)
- ✅ Audit logging of session creation/access

**Status**: Partially mitigated (Out of scope - requires prior compromise)

---

#### S-03: Configuration File Substitution

**Description**: Attacker replaces user configuration with malicious version

**Attack Vector**:
1. Attacker gains write access to `~/.config/ait42-editor/`
2. Replaces `config.toml` with malicious version
3. Editor loads malicious config on next startup

**DREAD Score**:
- **Damage**: 5/10 (Config-level changes only)
- **Reproducibility**: 8/10 (Trivial if write access)
- **Exploitability**: 2/10 (Requires file system compromise)
- **Affected Users**: 2/10 (Individual user)
- **Discoverability**: 3/10 (Obvious)
- **TOTAL**: 4.0/10 (LOW)

**Mitigations**:
- ✅ Configuration schema validation
- ✅ Warning on world-writable config files
- ✅ Secure defaults (require explicit opt-in for dangerous features)
- ⚠️ Consider config file integrity checking (checksum)

**Status**: Mitigated (Requires prior file system compromise)

---

### T - Tampering

#### T-01: LSP Response Manipulation

**Description**: Malicious LSP server sends crafted responses to exploit editor

**Attack Vector**:
1. User connects to malicious/compromised LSP server
2. Server sends responses with:
   - Malformed JSON
   - Oversized payloads
   - Exploited file paths (path traversal)
   - XSS-style attacks (terminal control codes)

**DREAD Score**:
- **Damage**: 7/10 (Potential code execution via terminal exploits)
- **Reproducibility**: 7/10 (Repeatable)
- **Exploitability**: 5/10 (Requires malicious LSP server)
- **Affected Users**: 3/10 (Users of specific LSP server)
- **Discoverability**: 6/10 (LSP protocol documented)
- **TOTAL**: 5.6/10 (MEDIUM)

**Mitigations**:
- ✅ JSON schema validation for all LSP messages
- ✅ Response size limits (1MB default)
- ✅ Path sanitization and canonicalization
- ✅ ANSI escape sequence sanitization
- ✅ Timeout enforcement (5 seconds)

**Status**: Mitigated

---

#### T-02: Malicious File Modification

**Description**: Race condition in file write operations (TOCTOU attack)

**Attack Vector**:
1. Editor checks file permissions (Time Of Check)
2. Attacker modifies file or swaps symlink
3. Editor writes to wrong file (Time Of Use)
4. Results in unauthorized file modification

**DREAD Score**:
- **Damage**: 9/10 (Arbitrary file write)
- **Reproducibility**: 4/10 (Timing-dependent)
- **Exploitability**: 7/10 (Well-known technique)
- **Affected Users**: 5/10 (Common use case)
- **Discoverability**: 8/10 (TOCTOU well-documented)
- **TOTAL**: 6.6/10 (MEDIUM-HIGH)

**Mitigations**:
- ✅ Path canonicalization before operations
- ✅ Atomic write (temp file + rename)
- ✅ File descriptor-based operations (open once, use fd)
- ✅ Symlink validation (optional follow)

**Status**: Mitigated

---

#### T-03: Buffer Overflow via Large File

**Description**: Loading extremely large file causes memory exhaustion

**Attack Vector**:
1. Attacker tricks user into opening 10GB+ file
2. Editor attempts to load entire file into memory
3. Out-of-memory condition causes crash or undefined behavior

**DREAD Score**:
- **Damage**: 4/10 (DoS only)
- **Reproducibility**: 9/10 (Trivial)
- **Exploitability**: 8/10 (Easy)
- **Affected Users**: 8/10 (Any user)
- **Discoverability**: 3/10 (Obvious)
- **TOTAL**: 6.4/10 (MEDIUM)

**Mitigations**:
- ✅ File size limits (100MB default for full load)
- ✅ Memory-mapped files for large files
- ✅ Lazy loading (load only visible region)
- ✅ User confirmation for files > 10MB

**Status**: Mitigated

---

#### T-04: Agent Parameter Injection

**Description**: Command injection via malicious agent parameters

**Attack Vector**:
1. User (or malicious script) invokes agent with crafted parameters
2. Parameters contain shell metacharacters: `; rm -rf /`
3. Agent executor passes unsanitized params to shell
4. Arbitrary command execution

**DREAD Score**:
- **Damage**: 10/10 (Full system compromise)
- **Reproducibility**: 8/10 (Easy)
- **Exploitability**: 7/10 (Requires user action)
- **Affected Users**: 8/10 (Common use case)
- **Discoverability**: 9/10 (Classic vulnerability)
- **TOTAL**: 8.4/10 (HIGH) ⚠️

**Mitigations**:
- ✅ **CRITICAL**: Never use shell interpretation (`sh -c`)
- ✅ Parameter validation (reject shell metacharacters)
- ✅ Use `Command::arg()` (no shell)
- ✅ Audit logging of all agent invocations

**Status**: Mitigated (requires ongoing vigilance)

---

#### T-05: Configuration Injection

**Description**: Malicious TOML configuration causes unintended behavior

**Attack Vector**:
1. Attacker provides crafted `config.toml`
2. Configuration exploits parser vulnerabilities or contains malicious settings
3. Examples:
   - Path to malicious LSP server
   - Extremely high resource limits
   - Disabled security features

**DREAD Score**:
- **Damage**: 6/10 (Config-level compromise)
- **Reproducibility**: 7/10 (Repeatable)
- **Exploitability**: 5/10 (Requires user to use malicious config)
- **Affected Users**: 3/10 (Limited)
- **Discoverability**: 6/10 (Config format documented)
- **TOTAL**: 5.4/10 (MEDIUM)

**Mitigations**:
- ✅ Schema validation with `serde` (`deny_unknown_fields`)
- ✅ Range validation for numeric values
- ✅ Allowlist for sensitive settings (LSP servers)
- ✅ Secure defaults (no auto-execute)

**Status**: Mitigated

---

### R - Repudiation

#### R-01: Agent Execution Denial

**Description**: User denies executing malicious actions via AIT42 agent

**Attack Vector**:
1. User (or compromised system) executes destructive agent
2. Agent deletes files or modifies critical system components
3. User claims they didn't execute the agent

**DREAD Score**:
- **Damage**: 5/10 (Reputational, forensic issues)
- **Reproducibility**: 6/10 (Repeatable)
- **Exploitability**: 4/10 (Requires intent or compromise)
- **Affected Users**: 2/10 (Edge case)
- **Discoverability**: 3/10 (Unusual scenario)
- **TOTAL**: 4.0/10 (LOW-MEDIUM)

**Mitigations**:
- ✅ **Audit logging** for all agent executions
- ✅ Logs include: timestamp, user, agent name, parameters, session ID
- ✅ Tamper-evident logs (append-only, restrictive permissions)
- ✅ User confirmation prompts for destructive operations

**Status**: Mitigated

---

#### R-02: Configuration Change Denial

**Description**: Configuration changes lack audit trail

**Attack Vector**:
1. Configuration is modified (by user or attacker)
2. Malicious behavior occurs
3. No record of who/when changed configuration

**DREAD Score**:
- **Damage**: 3/10 (Limited impact)
- **Reproducibility**: 8/10 (Trivial)
- **Exploitability**: 2/10 (Low motivation)
- **Affected Users**: 1/10 (Rare)
- **Discoverability**: 2/10 (Not obvious)
- **TOTAL**: 3.2/10 (LOW)

**Mitigations**:
- ⚠️ Log configuration changes (Phase 2)
- ✅ Version control recommendation in documentation
- ✅ Config file last-modified timestamp

**Status**: Partially mitigated (acceptable risk for MVP)

---

### I - Information Disclosure

#### I-01: Sensitive Data in Swap Files

**Description**: Auto-save swap files contain sensitive code/credentials

**Attack Vector**:
1. User edits file containing API keys or secrets
2. Editor creates `.swp` file with same content
3. Attacker with file system access reads swap file
4. Secrets leaked

**DREAD Score**:
- **Damage**: 7/10 (Credential theft)
- **Reproducibility**: 9/10 (Always happens with auto-save)
- **Exploitability**: 3/10 (Requires file system access)
- **Affected Users**: 6/10 (Common scenario)
- **Discoverability**: 5/10 (Swap files well-known)
- **TOTAL**: 6.0/10 (MEDIUM)

**Mitigations**:
- ✅ Restrictive permissions on swap files (0600 - user-only)
- ✅ Swap file cleanup on successful save
- ✅ Swap file cleanup on editor exit
- ⚠️ Consider encrypted swap files (Phase 2)

**Status**: Partially mitigated

---

#### I-02: LSP Server Information Leakage

**Description**: LSP server logs or caches contain sensitive information

**Attack Vector**:
1. LSP server processes code with secrets
2. Server logs full file content or creates cache
3. Attacker accesses LSP server data directory
4. Secrets leaked

**DREAD Score**:
- **Damage**: 7/10 (Credential theft)
- **Reproducibility**: 5/10 (Depends on LSP implementation)
- **Exploitability**: 4/10 (Requires file system + knowledge)
- **Affected Users**: 5/10 (Common)
- **Discoverability**: 4/10 (Requires investigation)
- **TOTAL**: 5.0/10 (MEDIUM)

**Mitigations**:
- ⚠️ Out of scope (LSP server responsibility)
- ✅ Documentation: Warn users about LSP server risks
- ✅ Recommend trusted LSP servers only

**Status**: Documented (out of editor control)

---

#### I-03: Audit Log Information Disclosure

**Description**: Audit logs readable by unauthorized users

**Attack Vector**:
1. Audit logs created with overly permissive permissions
2. Attacker reads logs containing sensitive task descriptions
3. Information leakage about system architecture or vulnerabilities

**DREAD Score**:
- **Damage**: 4/10 (Limited info disclosure)
- **Reproducibility**: 8/10 (Consistent)
- **Exploitability**: 3/10 (Requires file system access)
- **Affected Users**: 4/10 (Moderate)
- **Discoverability**: 3/10 (Not obvious)
- **TOTAL**: 4.4/10 (MEDIUM-LOW)

**Mitigations**:
- ✅ Restrictive log file permissions (0600)
- ✅ Log file location: `~/.ait42-editor/audit/` (user home)
- ✅ Sanitize sensitive data before logging (Phase 2)

**Status**: Mitigated

---

#### I-04: Memory Dump Exposure

**Description**: Editor crash dumps contain sensitive buffer content

**Attack Vector**:
1. Editor crashes while editing sensitive files
2. macOS creates crash report with memory dump
3. Attacker accesses crash reports
4. Secrets extracted from memory dump

**DREAD Score**:
- **Damage**: 8/10 (Full buffer content)
- **Reproducibility**: 2/10 (Rare crashes)
- **Exploitability**: 5/10 (Requires crash + file access)
- **Affected Users**: 3/10 (Infrequent)
- **Discoverability**: 6/10 (Crash reports well-known)
- **TOTAL**: 4.8/10 (MEDIUM-LOW)

**Mitigations**:
- ⚠️ Limited control (macOS system behavior)
- ✅ Minimize crashes through testing
- ✅ Consider memory zeroing on buffer close (Phase 2)
- ✅ Recommend users encrypt crash reports

**Status**: Documented risk (limited mitigation available)

---

#### I-05: Configuration File Exposure

**Description**: Configuration file contains sensitive information

**Attack Vector**:
1. User stores API keys in `config.toml` (despite warnings)
2. Attacker gains read access to config file
3. Credentials stolen

**DREAD Score**:
- **Damage**: 8/10 (Credential theft)
- **Reproducibility**: 6/10 (If users ignore best practices)
- **Exploitability**: 3/10 (Requires file system access)
- **Affected Users**: 4/10 (Some users)
- **Discoverability**: 5/10 (Common mistake)
- **TOTAL**: 5.2/10 (MEDIUM)

**Mitigations**:
- ✅ Documentation: **Never store secrets in config files**
- ✅ Use environment variables for API keys
- ⚠️ Integrate with macOS Keychain (Phase 2)
- ✅ Config parser warnings for detected secrets

**Status**: Documented (user education critical)

---

### D - Denial of Service

#### D-01: Resource Exhaustion via Large Files

**Description**: Opening extremely large files causes memory/CPU exhaustion

**DREAD Score**: 6.4/10 (MEDIUM) - See T-03 for details

**Status**: Mitigated (file size limits, lazy loading)

---

#### D-02: LSP Server Flooding

**Description**: Rapid edits flood LSP server with requests

**Attack Vector**:
1. User (or script) makes rapid buffer changes
2. Each change triggers LSP `textDocument/didChange`
3. LSP server overwhelmed, becomes unresponsive
4. Editor hangs waiting for LSP responses

**DREAD Score**:
- **Damage**: 4/10 (Editor unresponsive)
- **Reproducibility**: 8/10 (Easy)
- **Exploitability**: 6/10 (Trivial to automate)
- **Affected Users**: 7/10 (Common)
- **Discoverability**: 5/10 (Known issue)
- **TOTAL**: 6.0/10 (MEDIUM)

**Mitigations**:
- ✅ **Debouncing** (300ms delay before LSP update)
- ✅ Request coalescing (combine rapid changes)
- ✅ Timeout enforcement (5 seconds)
- ✅ LSP request queue limit (max 100 pending)

**Status**: Mitigated

---

#### D-03: Tmux Session Exhaustion

**Description**: Creating excessive tmux sessions exhausts system resources

**Attack Vector**:
1. Script spawns hundreds of AIT42 agents rapidly
2. Each agent creates tmux session
3. System runs out of PTYs or process slots
4. Editor and system become unresponsive

**DREAD Score**:
- **Damage**: 6/10 (System-wide impact)
- **Reproducibility**: 9/10 (Trivial)
- **Exploitability**: 7/10 (Easy to automate)
- **Affected Users**: 3/10 (Requires malicious script)
- **Discoverability**: 4/10 (Unusual)
- **TOTAL**: 5.8/10 (MEDIUM)

**Mitigations**:
- ✅ **Parallel agent limit** (max 5 simultaneous)
- ✅ Agent queue (beyond limit)
- ✅ Session cleanup on agent completion
- ✅ Timeout enforcement (30 minutes default)

**Status**: Mitigated

---

#### D-04: Malicious Syntax Highlighting

**Description**: Crafted file causes tree-sitter to hang or consume excessive CPU

**Attack Vector**:
1. Attacker creates file with pathological syntax
2. User opens file
3. tree-sitter parser enters infinite loop or exponential time complexity
4. Editor freezes

**DREAD Score**:
- **Damage**: 5/10 (Editor freeze)
- **Reproducibility**: 6/10 (Depends on tree-sitter version)
- **Exploitability**: 7/10 (Grammar fuzzing can find cases)
- **Affected Users**: 5/10 (Affects specific file types)
- **Discoverability**: 6/10 (Known class of vulnerabilities)
- **TOTAL**: 5.8/10 (MEDIUM)

**Mitigations**:
- ✅ tree-sitter timeout (1 second per parse)
- ✅ Parse only visible region (lazy parsing)
- ✅ Fallback to basic highlighting on parse failure
- ✅ Keep tree-sitter updated (security patches)

**Status**: Mitigated

---

#### D-05: Agent Resource Consumption

**Description**: Malicious agent consumes excessive CPU/memory

**Attack Vector**:
1. User executes agent with malicious intent
2. Agent spawns resource-intensive process
3. System becomes unresponsive
4. Other agents and editor impacted

**DREAD Score**:
- **Damage**: 6/10 (System slowdown)
- **Reproducibility**: 8/10 (Easy)
- **Exploitability**: 5/10 (Requires agent execution)
- **Affected Users**: 4/10 (Moderate)
- **Discoverability**: 3/10 (Not obvious)
- **TOTAL**: 5.2/10 (MEDIUM)

**Mitigations**:
- ✅ Execution timeout (30 minutes default)
- ⚠️ CPU/memory limits via cgroups (Phase 2 - macOS limitations)
- ✅ User can manually kill tmux session
- ✅ Monitoring and warning when agent consumes excessive resources

**Status**: Partially mitigated

---

### E - Elevation of Privilege

#### E-01: File Permission Bypass

**Description**: Editor bypasses macOS file permissions

**Attack Vector**:
1. User attempts to edit read-only file
2. Editor exploits race condition or permission check bug
3. File modified despite lack of write permission
4. Privilege escalation

**DREAD Score**:
- **Damage**: 8/10 (Unauthorized file modification)
- **Reproducibility**: 2/10 (Requires bug)
- **Exploitability**: 5/10 (Requires finding vulnerability)
- **Affected Users**: 8/10 (Impacts core functionality)
- **Discoverability**: 7/10 (Common attack vector)
- **TOTAL**: 6.0/10 (MEDIUM)

**Mitigations**:
- ✅ Always check permissions before operations
- ✅ Use file descriptor-based operations (single open)
- ✅ Comprehensive permission testing
- ✅ Fail secure (reject on permission error)

**Status**: Mitigated (requires testing validation)

---

#### E-02: Symlink Privilege Escalation

**Description**: Following symlink leads to unauthorized file access

**Attack Vector**:
1. Attacker creates symlink: `user-file.txt -> /etc/passwd`
2. User opens `user-file.txt` in editor
3. Editor follows symlink
4. System file modified with user privileges
5. Privilege escalation if editor has elevated permissions (should not)

**DREAD Score**:
- **Damage**: 9/10 (Critical system file modification)
- **Reproducibility**: 6/10 (Repeatable if symlinks followed)
- **Exploitability**: 4/10 (Requires symlink creation)
- **Affected Users**: 6/10 (Common)
- **Discoverability**: 8/10 (Classic attack)
- **TOTAL**: 6.6/10 (MEDIUM-HIGH)

**Mitigations**:
- ✅ Path canonicalization (resolves symlinks)
- ✅ Optional symlink following (user confirmation)
- ✅ Validate target path is within allowed scope
- ✅ Never run editor with elevated permissions

**Status**: Mitigated

---

#### E-03: Agent Code Injection

**Description**: Malicious agent gains elevated system access

**Attack Vector**:
1. User executes compromised AIT42 agent
2. Agent exploits editor vulnerabilities
3. Agent gains higher privileges than editor process
4. System compromise

**DREAD Score**:
- **Damage**: 10/10 (Full system compromise)
- **Reproducibility**: 3/10 (Requires compromised agent + vulnerability)
- **Exploitability**: 4/10 (Complex multi-stage attack)
- **Affected Users**: 5/10 (Depends on agent source)
- **Discoverability**: 5/10 (Requires research)
- **TOTAL**: 5.4/10 (MEDIUM)

**Mitigations**:
- ✅ Tmux session isolation (no privilege elevation)
- ✅ Agents run with same privileges as user
- ✅ No setuid/setgid on editor binary
- ✅ Audit logging of agent executions
- ⚠️ Consider agent code signing (Phase 2)

**Status**: Mitigated (depends on agent source trust)

---

## Attack Scenarios

### Scenario 1: Malicious LSP Server Attack

**Attacker Profile**: Advanced attacker with knowledge of LSP protocol
**Motivation**: Code injection, information theft
**Attack Chain**:

```
1. Attacker creates malicious LSP server
   └─> Server advertises as "rust-analyzer-enhanced"

2. Social engineering: Convince user to install
   └─> Blog post: "Faster Rust completions with enhanced analyzer"

3. User adds to config.toml:
   [lsp]
   rust = "/path/to/malicious-lsp"

4. User opens Rust file
   └─> Editor connects to malicious LSP

5. Malicious responses:
   ├─> Oversized JSON (DoS attempt)
   ├─> Crafted file paths (path traversal)
   └─> Terminal escape sequences (terminal exploits)

6. Editor processes responses
   ├─> ✅ BLOCKED: JSON size limit exceeded
   ├─> ✅ BLOCKED: Path canonicalization fails
   └─> ✅ BLOCKED: ANSI sanitization removes exploits

7. Attack fails due to defense-in-depth
```

**Outcome**: Attack mitigated
**Lessons**: Multi-layer validation critical

---

### Scenario 2: TOCTOU File Write Attack

**Attacker Profile**: Local attacker with file system access
**Motivation**: Inject malicious code into user's project
**Attack Chain**:

```
1. User edits important file: `src/main.rs`

2. Attacker monitors with file watcher

3. User saves file (Ctrl+S)
   └─> Editor checks: Can write to src/main.rs? ✓

4. Race window: Between check and write
   └─> Attacker swaps symlink:
       src/main.rs -> /tmp/malicious-code.rs

5. Editor writes to symlinked file
   └─> ✅ BLOCKED: Canonicalization detects change
   └─> ✅ Uses file descriptor from original open()

6. Attack fails: Write goes to original file

7. Atomic rename ensures integrity
```

**Outcome**: Attack mitigated
**Lessons**: Use file descriptors, atomic operations

---

### Scenario 3: Command Injection via Agent

**Attacker Profile**: Insider or compromised system
**Motivation**: Execute arbitrary commands
**Attack Chain**:

```
1. Attacker crafts malicious agent invocation
   └─> Task: "build project; rm -rf /"

2. Agent executor receives parameters:
   params = {
     "task": "build project; rm -rf /"
   }

3. Validation phase:
   └─> Check for shell metacharacters: ; | & ` $ ( )
   └─> ✅ BLOCKED: Semicolon detected

4. Error returned to user:
   "Unsafe parameter: task contains forbidden characters"

5. If validation bypassed (hypothetical):
   └─> Agent execution uses Command::arg()
   └─> No shell interpretation occurs
   └─> ✅ Parameters passed as literal strings

6. Attack fails at multiple layers
```

**Outcome**: Attack mitigated
**Lessons**: Never use shell, validate inputs

---

### Scenario 4: Resource Exhaustion Attack

**Attacker Profile**: Malicious script or insider
**Motivation**: Denial of service
**Attack Chain**:

```
1. Attacker script spawns 100 agents rapidly
   for i in 1..100 {
     spawn_agent("backend-developer", "task-{i}")
   }

2. First 5 agents execute immediately
   └─> Tmux sessions created: ait42-backend-1 through ait42-backend-5

3. Remaining 95 agents queued
   └─> Queue limit: 50 (default)
   └─> ✅ BLOCKED: 45 agents rejected

4. Monitor detects high resource usage
   └─> Warning shown to user

5. Existing agents hit timeout (30 min)
   └─> Auto-terminated, sessions cleaned up

6. System remains responsive
```

**Outcome**: Attack mitigated (partial service degradation)
**Lessons**: Rate limiting, resource limits essential

---

## Risk Assessment Matrix

### DREAD Scoring Methodology

**Damage Potential** (D):
- 10: Complete system compromise
- 7-9: Significant data loss or corruption
- 4-6: Limited data loss or functionality impact
- 1-3: Minimal impact

**Reproducibility** (R):
- 10: Always reproducible
- 7-9: Easy to reproduce
- 4-6: Requires specific conditions
- 1-3: Difficult to reproduce

**Exploitability** (E):
- 10: Script kiddie (automated tools)
- 7-9: Advanced user with research
- 4-6: Skilled attacker
- 1-3: Requires expert + resources

**Affected Users** (A):
- 10: All users
- 7-9: Majority of users
- 4-6: Some users
- 1-3: Individual users

**Discoverability** (D):
- 10: Publicly known
- 7-9: Easy to discover
- 4-6: Requires investigation
- 1-3: Obscure

**Total Risk Score** = Average of 5 factors

---

### Risk Matrix

| Risk Score | Severity | Action Required |
|------------|----------|-----------------|
| **8.0-10.0** | CRITICAL | Fix before release |
| **6.0-7.9** | HIGH | Fix in current sprint |
| **4.0-5.9** | MEDIUM | Fix in next release |
| **2.0-3.9** | LOW | Backlog |
| **0.0-1.9** | MINIMAL | Monitor |

---

### Threat Risk Summary

| Threat ID | Threat Name | DREAD Score | Severity | Status |
|-----------|-------------|-------------|----------|--------|
| **T-04** | Agent Parameter Injection | 8.4 | HIGH | ✅ Mitigated |
| **T-02** | Malicious File Modification | 8.0 | HIGH | ✅ Mitigated |
| **E-02** | Symlink Privilege Escalation | 6.6 | MEDIUM-HIGH | ✅ Mitigated |
| **T-03** | Buffer Overflow via Large File | 6.4 | MEDIUM | ✅ Mitigated |
| **E-01** | File Permission Bypass | 6.0 | MEDIUM | ⚠️ Testing required |
| **D-02** | LSP Server Flooding | 6.0 | MEDIUM | ✅ Mitigated |
| **I-01** | Sensitive Data in Swap Files | 6.0 | MEDIUM | ⚠️ Partial |
| **D-03** | Tmux Session Exhaustion | 5.8 | MEDIUM | ✅ Mitigated |
| **D-04** | Malicious Syntax Highlighting | 5.8 | MEDIUM | ✅ Mitigated |
| **T-01** | LSP Response Manipulation | 5.6 | MEDIUM | ✅ Mitigated |
| **T-05** | Configuration Injection | 5.4 | MEDIUM | ✅ Mitigated |
| **E-03** | Agent Code Injection | 5.4 | MEDIUM | ⚠️ Depends on agent trust |
| **I-05** | Configuration File Exposure | 5.2 | MEDIUM | ⚠️ User education |
| **D-05** | Agent Resource Consumption | 5.2 | MEDIUM | ⚠️ Partial |
| **I-02** | LSP Server Information Leakage | 5.0 | MEDIUM | ⚠️ Out of scope |
| **S-02** | Tmux Session Hijacking | 4.8 | MEDIUM-LOW | ⚠️ Requires compromise |
| **I-04** | Memory Dump Exposure | 4.8 | MEDIUM-LOW | ⚠️ Limited mitigation |
| **S-01** | Malicious LSP Server | 4.4 | MEDIUM-LOW | ✅ Mitigated |
| **I-03** | Audit Log Disclosure | 4.4 | MEDIUM-LOW | ✅ Mitigated |
| **R-01** | Agent Execution Denial | 4.0 | LOW-MEDIUM | ✅ Mitigated |
| **S-03** | Config File Substitution | 4.0 | LOW | ⚠️ Requires compromise |
| **R-02** | Configuration Change Denial | 3.2 | LOW | ⚠️ Phase 2 |

---

## Security Controls Mapping

### Control Effectiveness Matrix

| Control | Threats Mitigated | Effectiveness | Implementation Status |
|---------|-------------------|---------------|----------------------|
| **Input Validation** | S-01, T-01, T-04, T-05 | High | ✅ Implemented |
| **Path Canonicalization** | T-02, E-01, E-02 | High | ✅ Implemented |
| **Atomic File Operations** | T-02 | High | ✅ Implemented |
| **Schema Validation** | T-01, T-05 | High | ✅ Implemented |
| **Resource Limits** | T-03, D-01, D-03, D-05 | Medium | ⚠️ Partial |
| **Timeout Enforcement** | D-02, D-04 | High | ✅ Implemented |
| **Audit Logging** | R-01 | Medium | ✅ Implemented |
| **Process Isolation** | E-03 | Medium | ✅ Implemented |
| **Permission Checks** | E-01, E-02, I-01 | High | ✅ Implemented |
| **Sanitization** | T-01, T-04 | High | ✅ Implemented |
| **Rate Limiting** | D-02 | Medium | ✅ Implemented |
| **File Permissions** | I-01, I-03, I-05 | Medium | ✅ Implemented |
| **Debouncing** | D-02 | Medium | ✅ Implemented |

---

## Threat Intelligence

### Known Vulnerabilities in Similar Systems

#### Vim/Neovim Vulnerabilities
- **CVE-2019-12735**: Arbitrary command execution via modelines
  - **Relevance**: We don't support modelines in MVP
  - **Mitigation**: N/A (feature not implemented)

- **CVE-2021-3968**: Heap buffer overflow in syntax highlighting
  - **Relevance**: We use tree-sitter (different implementation)
  - **Mitigation**: Keep tree-sitter updated

#### LSP Vulnerabilities
- **CVE-2022-xxxxx**: Path traversal in LSP workspace operations
  - **Relevance**: Direct - LSP path handling
  - **Mitigation**: Path canonicalization + validation

#### Terminal Emulator Vulnerabilities
- **OSC Command Injection**: Malicious ANSI escape sequences
  - **Relevance**: Agent output displayed in terminal
  - **Mitigation**: ANSI sanitization before display

---

### Threat Actor Profiles

#### 1. Script Kiddie
- **Skill Level**: Low
- **Tools**: Automated scanners, public exploits
- **Motivation**: Curiosity, reputation
- **Threat Level**: Low (automated defenses sufficient)

#### 2. Insider Threat
- **Skill Level**: Medium
- **Access**: File system, configuration
- **Motivation**: Sabotage, data theft
- **Threat Level**: Medium (hardest to defend against)

#### 3. Advanced Persistent Threat (APT)
- **Skill Level**: High
- **Resources**: Custom exploits, zero-days
- **Motivation**: Espionage, targeted attacks
- **Threat Level**: Low (editor unlikely target, but consider supply chain)

#### 4. Supply Chain Attacker
- **Skill Level**: High
- **Attack Vector**: Compromised dependencies
- **Motivation**: Mass exploitation
- **Threat Level**: Medium (mitigated by dependency auditing)

---

## Recommendations

### Immediate Actions (Before MVP Release)

1. **Validate T-04 mitigation**: Comprehensive testing of command injection prevention
2. **Enhance E-01 testing**: Permission bypass scenarios
3. **Document I-05**: Clear guidance on secret management
4. **Review E-03**: Agent source verification strategy

### Short-Term (Phase 1.1)

1. **I-01**: Implement encrypted swap files
2. **D-05**: macOS-compatible resource limits
3. **R-02**: Configuration change logging

### Long-Term (Phase 2)

1. **Agent code signing**: Verify agent integrity
2. **Keychain integration**: Secure secret storage
3. **Sandboxing**: Evaluate macOS App Sandbox
4. **Security audit**: Third-party penetration testing

---

## Appendix A: Threat Model Maintenance

### Review Schedule
- **Quarterly**: Full threat model review
- **Per Release**: New feature threat assessment
- **Ad-Hoc**: Upon discovery of new threat intelligence

### Update Triggers
- New features added
- Vulnerabilities discovered
- Dependency changes
- Security incidents
- User feedback

---

## Appendix B: References

### Standards & Frameworks
- STRIDE Threat Modeling (Microsoft)
- DREAD Risk Assessment (Microsoft)
- OWASP Top 10
- CWE Top 25

### Tools
- `cargo audit` - Dependency vulnerability scanning
- `cargo fuzz` - Fuzz testing
- `cargo clippy` - Static analysis

---

**End of Threat Model Document**

**Version**: 1.0.0
**Date**: 2025-11-03
**Next Review**: 2025-12-03
**Approved By**: Security Architect
