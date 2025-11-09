# AIT42 Editor - Plugin API Specification (Phase 2)

**Version**: 1.0.0 (Draft)
**Date**: 2025-01-06
**Status**: Phase 2 Planning
**Target Release**: Q3 2025

---

## Table of Contents

1. [Overview](#overview)
2. [Plugin System Architecture](#plugin-system-architecture)
3. [Plugin Lifecycle API](#plugin-lifecycle-api)
4. [Hook System API](#hook-system-api)
5. [Command Registration API](#command-registration-api)
6. [UI Extension API](#ui-extension-api)
7. [Wasm Integration](#wasm-integration)
8. [Security and Sandboxing](#security-and-sandboxing)
9. [Plugin Manifest Format](#plugin-manifest-format)
10. [Example Plugins](#example-plugins)

---

## Overview

The AIT42 Editor plugin system enables third-party extensions while maintaining security and stability. Plugins can:

- Add custom commands
- Register key bindings
- Extend UI with custom widgets
- Hook into editor events
- Provide language support
- Integrate external tools

### Design Goals

1. **Security First**: Sandboxed execution via WebAssembly
2. **Performance**: Minimal overhead, async-friendly
3. **Stability**: Plugin crashes don't affect editor
4. **Developer Experience**: Simple API, good documentation
5. **Hot Reload**: Update plugins without restarting editor

### Plugin Types

| Type | Language | Sandbox | Use Case |
|------|----------|---------|----------|
| **Wasm Plugin** | Rust, AssemblyScript | Yes | Complex logic, safe execution |
| **Lua Script** | Lua | Limited | Simple automation, config |
| **Native Plugin** | Rust | No | High performance, system access |

---

## Plugin System Architecture

### Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                      AIT42 Editor Core                      │
│                                                             │
│  ┌──────────────────────────────────────────────────────┐  │
│  │               Plugin Manager                         │  │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐    │  │
│  │  │   Plugin   │  │   Plugin   │  │   Plugin   │    │  │
│  │  │  Registry  │  │   Loader   │  │   Sandbox  │    │  │
│  │  └────────────┘  └────────────┘  └────────────┘    │  │
│  └──────────────────────────────────────────────────────┘  │
│                           │                                 │
│  ┌────────────────────────┼─────────────────────────────┐  │
│  │         Hook System    │                             │  │
│  │  ┌─────────────────────▼──────────────────────────┐  │  │
│  │  │  Event Hooks (Buffer, Mode, LSP, Agent, UI)   │  │  │
│  │  └────────────────────────────────────────────────┘  │  │
│  └─────────────────────────────────────────────────────┘  │
│                           │                                 │
└───────────────────────────┼─────────────────────────────────┘
                            │
            ┌───────────────┴───────────────┐
            │                               │
       ┌────▼────┐                    ┌────▼────┐
       │  Wasm   │                    │  Lua    │
       │ Runtime │                    │ Runtime │
       │(wasmtime│                    │(mlua)   │
       └────┬────┘                    └────┬────┘
            │                               │
       ┌────▼────────┐               ┌─────▼──────┐
       │ Plugin A    │               │ Plugin B   │
       │ (git.wasm)  │               │(format.lua)│
       └─────────────┘               └────────────┘
```

### Component Responsibilities

#### Plugin Manager

- Load/unload plugins
- Manage plugin lifecycle
- Dispatch hooks to plugins
- Handle plugin crashes gracefully

#### Plugin Registry

- Store plugin metadata
- Track installed plugins
- Version management
- Dependency resolution

#### Plugin Sandbox

- Isolate plugin execution
- Enforce resource limits (CPU, memory)
- Control system access (filesystem, network)

---

## Plugin Lifecycle API

### Plugin Trait

```rust
/// Core plugin trait
///
/// All plugins must implement this trait.
pub trait Plugin: Send + Sync {
    /// Plugin metadata
    fn metadata(&self) -> &PluginMetadata;

    /// Initialize plugin
    ///
    /// Called once when plugin is loaded.
    /// Use for setup, loading configuration, etc.
    ///
    /// # Errors
    /// Return error if initialization fails.
    fn init(&mut self, ctx: &mut PluginContext) -> Result<()>;

    /// Shutdown plugin
    ///
    /// Called when plugin is unloaded.
    /// Use for cleanup, saving state, etc.
    fn shutdown(&mut self, ctx: &mut PluginContext) -> Result<()> {
        Ok(())
    }

    /// Plugin configuration changed
    ///
    /// Called when user updates plugin config.
    fn on_config_changed(&mut self, config: &Config) -> Result<()> {
        Ok(())
    }
}

/// Plugin metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Unique plugin identifier (e.g., "com.example.git-plugin")
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Plugin version (semver)
    pub version: String,

    /// Author information
    pub author: String,

    /// Short description
    pub description: String,

    /// Plugin homepage
    pub homepage: Option<String>,

    /// Minimum editor version required
    pub min_editor_version: String,

    /// Plugin dependencies
    pub dependencies: Vec<PluginDependency>,

    /// Requested permissions
    pub permissions: Vec<Permission>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependency {
    pub plugin_id: String,
    pub version_req: String, // semver requirement
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Permission {
    /// Read files in workspace
    ReadFiles,

    /// Write files in workspace
    WriteFiles,

    /// Execute commands
    ExecuteCommands,

    /// Network access
    Network,

    /// System clipboard access
    Clipboard,

    /// Register keybindings
    Keybindings,

    /// Modify UI
    ModifyUI,
}
```

### Plugin Context

```rust
/// Plugin execution context
///
/// Provides access to editor APIs.
pub struct PluginContext {
    /// Plugin ID
    plugin_id: String,

    /// Editor API facade
    editor: EditorApi,

    /// Event bus sender
    event_tx: mpsc::Sender<EditorEvent>,

    /// Plugin configuration
    config: Config,

    /// Resource limits
    limits: ResourceLimits,
}

impl PluginContext {
    /// Get editor API
    pub fn editor(&self) -> &EditorApi;

    /// Send event to editor
    pub fn send_event(&self, event: EditorEvent) -> Result<()>;

    /// Get plugin configuration
    pub fn config(&self) -> &Config;

    /// Log message (respects log level)
    pub fn log(&self, level: LogLevel, message: &str);

    /// Get plugin data directory
    ///
    /// Use for storing plugin-specific data.
    /// Path: `~/.local/share/ait42-editor/plugins/<plugin-id>/`
    pub fn data_dir(&self) -> PathBuf;
}

/// Editor API facade for plugins
///
/// Provides safe, sandboxed access to editor functionality.
pub struct EditorApi {
    inner: Arc<EditorContext>,
}

impl EditorApi {
    /// Get active buffer
    pub fn active_buffer(&self) -> Result<BufferHandle>;

    /// Open file
    pub fn open_file(&self, path: &Path) -> Result<BufferHandle>;

    /// Get all open buffers
    pub fn buffers(&self) -> Vec<BufferHandle>;

    /// Execute command
    pub fn execute_command(&self, cmd: Box<dyn Command>) -> Result<()>;

    /// Register command
    pub fn register_command(&self, name: &str, handler: CommandHandler) -> Result<()>;

    /// Get cursor position
    pub fn cursor_position(&self) -> Position;

    /// Show notification
    pub fn notify(&self, message: &str, level: NotificationLevel);

    /// Prompt user for input
    pub async fn prompt(&self, message: &str) -> Result<String>;

    /// Show confirmation dialog
    pub async fn confirm(&self, message: &str) -> Result<bool>;
}

/// Buffer handle (safe reference)
pub struct BufferHandle {
    id: BufferId,
    api: Arc<EditorApi>,
}

impl BufferHandle {
    /// Get buffer content
    pub fn content(&self) -> Result<String>;

    /// Get buffer path
    pub fn path(&self) -> Result<Option<PathBuf>>;

    /// Get buffer language
    pub fn language(&self) -> Result<Language>;

    /// Insert text
    pub fn insert(&self, offset: usize, text: &str) -> Result<()>;

    /// Delete range
    pub fn delete(&self, range: Range<usize>) -> Result<()>;

    /// Get text slice
    pub fn slice(&self, range: Range<usize>) -> Result<String>;
}
```

### Plugin Manager

```rust
/// Plugin manager
///
/// Manages plugin lifecycle, loading, and unloading.
pub struct PluginManager {
    plugins: HashMap<String, Box<dyn Plugin>>,
    registry: PluginRegistry,
    wasm_runtime: WasmRuntime,
    lua_runtime: LuaRuntime,
}

impl PluginManager {
    /// Create new plugin manager
    pub fn new() -> Self;

    /// Load plugin from file
    ///
    /// # Examples
    /// ```rust
    /// let manager = PluginManager::new();
    /// manager.load_plugin(Path::new("plugins/git.wasm")).await?;
    /// ```
    pub async fn load_plugin(&mut self, path: &Path) -> Result<String>;

    /// Unload plugin
    pub async fn unload_plugin(&mut self, plugin_id: &str) -> Result<()>;

    /// Reload plugin (hot reload)
    pub async fn reload_plugin(&mut self, plugin_id: &str) -> Result<()>;

    /// Get plugin by ID
    pub fn get_plugin(&self, plugin_id: &str) -> Option<&dyn Plugin>;

    /// List all loaded plugins
    pub fn plugins(&self) -> Vec<&PluginMetadata>;

    /// Check if plugin is loaded
    pub fn is_loaded(&self, plugin_id: &str) -> bool;

    /// Enable plugin
    pub fn enable_plugin(&mut self, plugin_id: &str) -> Result<()>;

    /// Disable plugin
    pub fn disable_plugin(&mut self, plugin_id: &str) -> Result<()>;
}
```

---

## Hook System API

### Hook Types

```rust
/// Hook types for plugin integration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HookType {
    /// Before buffer change
    BufferPreChange,

    /// After buffer change
    BufferPostChange,

    /// Buffer opened
    BufferOpened,

    /// Buffer closed
    BufferClosed,

    /// Buffer saved
    BufferSaved,

    /// Mode changed
    ModeChanged,

    /// Key pressed (can be intercepted)
    KeyPressed,

    /// LSP completion requested
    LspCompletion,

    /// LSP diagnostics received
    LspDiagnostics,

    /// Agent started
    AgentStarted,

    /// Agent completed
    AgentCompleted,

    /// UI render (can add widgets)
    UiRender,

    /// Editor started
    EditorStarted,

    /// Editor shutting down
    EditorShutdown,
}

/// Hook handler trait
pub trait HookHandler: Send + Sync {
    /// Handle hook event
    ///
    /// Returns `HookResult` indicating whether to continue or stop.
    fn handle(&mut self, event: &HookEvent, ctx: &mut PluginContext) -> HookResult;
}

/// Hook event data
#[derive(Debug, Clone)]
pub enum HookEvent {
    BufferPreChange {
        buffer_id: BufferId,
        change: TextChange,
    },
    BufferPostChange {
        buffer_id: BufferId,
        change: TextChange,
    },
    BufferOpened {
        buffer_id: BufferId,
        path: PathBuf,
    },
    BufferClosed {
        buffer_id: BufferId,
    },
    BufferSaved {
        buffer_id: BufferId,
        path: PathBuf,
    },
    ModeChanged {
        from: String,
        to: String,
    },
    KeyPressed {
        key: KeyEvent,
    },
    LspCompletion {
        buffer_id: BufferId,
        position: Position,
        items: Vec<CompletionItem>,
    },
    LspDiagnostics {
        buffer_id: BufferId,
        diagnostics: Vec<Diagnostic>,
    },
    AgentStarted {
        agent_id: AgentId,
        name: String,
    },
    AgentCompleted {
        agent_id: AgentId,
        result: AgentResult,
    },
    UiRender {
        widgets: Vec<Box<dyn Widget>>,
    },
    EditorStarted,
    EditorShutdown,
}

/// Hook result
#[derive(Debug, Clone)]
pub enum HookResult {
    /// Continue to next hook
    Continue,

    /// Stop hook chain (event handled)
    Stop,

    /// Modify event and continue
    Modify(HookEvent),

    /// Cancel event (e.g., prevent buffer change)
    Cancel,
}
```

### Hook Registration

```rust
impl PluginContext {
    /// Register hook handler
    ///
    /// # Examples
    /// ```rust
    /// ctx.register_hook(HookType::BufferPostChange, MyHookHandler)?;
    /// ```
    pub fn register_hook(
        &self,
        hook_type: HookType,
        handler: Box<dyn HookHandler>,
    ) -> Result<HookId>;

    /// Unregister hook
    pub fn unregister_hook(&self, hook_id: HookId) -> Result<()>;
}

pub type HookId = u64;
```

### Example: Auto-format on Save

```rust
struct AutoFormatHook;

impl HookHandler for AutoFormatHook {
    fn handle(&mut self, event: &HookEvent, ctx: &mut PluginContext) -> HookResult {
        if let HookEvent::BufferPreSave { buffer_id, .. } = event {
            // Format buffer before saving
            match ctx.editor().format_buffer(*buffer_id) {
                Ok(_) => {
                    ctx.log(LogLevel::Info, "Auto-formatted buffer");
                    HookResult::Continue
                }
                Err(e) => {
                    ctx.log(LogLevel::Error, &format!("Format failed: {}", e));
                    HookResult::Continue
                }
            }
        } else {
            HookResult::Continue
        }
    }
}

// In plugin init:
ctx.register_hook(HookType::BufferPreSave, Box::new(AutoFormatHook))?;
```

---

## Command Registration API

### Command Handler

```rust
/// Command handler trait
pub trait CommandHandler: Send + Sync {
    /// Execute command
    fn execute(&mut self, args: &[String], ctx: &mut PluginContext) -> Result<()>;

    /// Get command metadata
    fn metadata(&self) -> CommandMetadata;
}

/// Command metadata
#[derive(Debug, Clone)]
pub struct CommandMetadata {
    /// Command name (e.g., "git-commit")
    pub name: String,

    /// Short description
    pub description: String,

    /// Usage example
    pub usage: String,

    /// Command arguments
    pub args: Vec<CommandArg>,
}

#[derive(Debug, Clone)]
pub struct CommandArg {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub default: Option<String>,
}
```

### Example: Git Commit Command

```rust
struct GitCommitCommand;

impl CommandHandler for GitCommitCommand {
    fn execute(&mut self, args: &[String], ctx: &mut PluginContext) -> Result<()> {
        let message = args.get(0)
            .ok_or_else(|| Error::InvalidArgs("Missing commit message"))?;

        // Execute git commit
        let output = std::process::Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg(message)
            .output()?;

        if output.status.success() {
            ctx.notify("Committed successfully", NotificationLevel::Success);
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            ctx.notify(&error, NotificationLevel::Error);
        }

        Ok(())
    }

    fn metadata(&self) -> CommandMetadata {
        CommandMetadata {
            name: "git-commit".into(),
            description: "Commit changes with message".into(),
            usage: ":git-commit <message>".into(),
            args: vec![
                CommandArg {
                    name: "message".into(),
                    description: "Commit message".into(),
                    required: true,
                    default: None,
                }
            ],
        }
    }
}

// In plugin init:
ctx.register_command("git-commit", Box::new(GitCommitCommand))?;
```

---

## UI Extension API

### Widget Extension

```rust
/// Custom widget trait for plugins
pub trait PluginWidget: Widget {
    /// Widget ID
    fn id(&self) -> &str;

    /// Widget position in UI
    fn position(&self) -> WidgetPosition;

    /// Update widget state
    fn update(&mut self, ctx: &PluginContext);
}

#[derive(Debug, Clone, Copy)]
pub enum WidgetPosition {
    /// Sidebar (left)
    SidebarLeft,

    /// Sidebar (right)
    SidebarRight,

    /// Bottom panel
    BottomPanel,

    /// Status bar
    StatusBar,

    /// Floating (overlay)
    Floating { x: u16, y: u16 },
}
```

### Example: Git Status Widget

```rust
struct GitStatusWidget {
    branch: String,
    changes: usize,
}

impl PluginWidget for GitStatusWidget {
    fn id(&self) -> &str {
        "git-status"
    }

    fn position(&self) -> WidgetPosition {
        WidgetPosition::StatusBar
    }

    fn update(&mut self, ctx: &PluginContext) {
        // Get git status
        let output = std::process::Command::new("git")
            .arg("status")
            .arg("--porcelain")
            .output()
            .unwrap();

        self.changes = output.stdout.lines().count();

        let branch_output = std::process::Command::new("git")
            .arg("branch")
            .arg("--show-current")
            .output()
            .unwrap();

        self.branch = String::from_utf8_lossy(&branch_output.stdout)
            .trim()
            .to_string();
    }
}

impl Widget for GitStatusWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let text = format!(" {} ({} changes)", self.branch, self.changes);
        let span = Span::styled(text, Style::default().fg(Color::Green));
        span.render(area, buf);
    }
}
```

---

## Wasm Integration

### Wasm Plugin Runtime

```rust
/// Wasm runtime using wasmtime
pub struct WasmRuntime {
    engine: wasmtime::Engine,
    linker: wasmtime::Linker<PluginState>,
}

impl WasmRuntime {
    /// Create new Wasm runtime
    pub fn new() -> Result<Self> {
        let mut config = wasmtime::Config::new();
        config.wasm_multi_memory(true);
        config.wasm_bulk_memory(true);

        let engine = wasmtime::Engine::new(&config)?;
        let mut linker = wasmtime::Linker::new(&engine);

        // Register host functions
        Self::register_host_functions(&mut linker)?;

        Ok(Self { engine, linker })
    }

    /// Load Wasm plugin
    pub fn load_plugin(&mut self, wasm_bytes: &[u8]) -> Result<WasmPlugin> {
        let module = wasmtime::Module::new(&self.engine, wasm_bytes)?;

        let mut store = wasmtime::Store::new(&self.engine, PluginState::new());

        let instance = self.linker.instantiate(&mut store, &module)?;

        Ok(WasmPlugin {
            store,
            instance,
        })
    }

    fn register_host_functions(linker: &mut wasmtime::Linker<PluginState>) -> Result<()> {
        // Register editor API functions
        linker.func_wrap("env", "log", |caller: Caller<PluginState>, ptr: i32, len: i32| {
            let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
            let data = &memory.data(&caller)[ptr as usize..(ptr + len) as usize];
            let message = std::str::from_utf8(data).unwrap();
            println!("[Plugin] {}", message);
        })?;

        linker.func_wrap("env", "buffer_insert",
            |mut caller: Caller<PluginState>, buffer_id: i32, offset: i32, ptr: i32, len: i32| -> i32 {
                // Implementation
                0 // Success
            }
        )?;

        // More host functions...

        Ok(())
    }
}

/// Plugin state (passed to Wasm functions)
struct PluginState {
    context: Arc<PluginContext>,
}
```

### Wasm Plugin Interface (Guest Side)

```rust
// In plugin (compiled to Wasm)

#[no_mangle]
pub extern "C" fn plugin_init() -> i32 {
    log("Plugin initialized");
    0 // Success
}

#[no_mangle]
pub extern "C" fn plugin_on_buffer_change(buffer_id: i32, offset: i32, len: i32) -> i32 {
    log(&format!("Buffer {} changed at {}", buffer_id, offset));
    0
}

// Host function imports
extern "C" {
    fn log(ptr: *const u8, len: usize);
    fn buffer_insert(buffer_id: i32, offset: i32, ptr: *const u8, len: usize) -> i32;
    fn buffer_get_content(buffer_id: i32, out_ptr: *mut u8, out_len: *mut usize) -> i32;
}

fn log(message: &str) {
    unsafe {
        log(message.as_ptr(), message.len());
    }
}
```

---

## Security and Sandboxing

### Resource Limits

```rust
/// Resource limits for plugins
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    /// Max memory usage (bytes)
    pub max_memory: usize,

    /// Max CPU time per operation (milliseconds)
    pub max_cpu_time: u64,

    /// Max file size read/write (bytes)
    pub max_file_size: usize,

    /// Max network bandwidth (bytes/sec)
    pub max_network_bandwidth: usize,

    /// Allowed filesystem paths
    pub allowed_paths: Vec<PathBuf>,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory: 100 * 1024 * 1024, // 100 MB
            max_cpu_time: 5000, // 5 seconds
            max_file_size: 10 * 1024 * 1024, // 10 MB
            max_network_bandwidth: 1024 * 1024, // 1 MB/s
            allowed_paths: vec![],
        }
    }
}
```

### Permission System

```rust
/// Permission manager
pub struct PermissionManager {
    grants: HashMap<String, HashSet<Permission>>,
}

impl PermissionManager {
    /// Grant permission to plugin
    pub fn grant(&mut self, plugin_id: &str, permission: Permission) {
        self.grants
            .entry(plugin_id.to_string())
            .or_insert_with(HashSet::new)
            .insert(permission);
    }

    /// Check if plugin has permission
    pub fn check(&self, plugin_id: &str, permission: &Permission) -> bool {
        self.grants
            .get(plugin_id)
            .map(|perms| perms.contains(permission))
            .unwrap_or(false)
    }

    /// Revoke permission
    pub fn revoke(&mut self, plugin_id: &str, permission: &Permission) {
        if let Some(perms) = self.grants.get_mut(plugin_id) {
            perms.remove(permission);
        }
    }
}

/// Check permission before operation
fn check_permission(ctx: &PluginContext, permission: Permission) -> Result<()> {
    if !ctx.permissions().check(&ctx.plugin_id, &permission) {
        return Err(Error::PermissionDenied(permission));
    }
    Ok(())
}
```

### Crash Isolation

```rust
impl PluginManager {
    /// Execute plugin function with crash isolation
    async fn safe_execute<F, T>(&self, plugin_id: &str, f: F) -> Result<T>
    where
        F: FnOnce() -> Result<T> + Send + 'static,
        T: Send + 'static,
    {
        let (tx, rx) = oneshot::channel();

        // Spawn plugin execution in separate task
        tokio::spawn(async move {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
            let _ = tx.send(result);
        });

        // Wait with timeout
        match tokio::time::timeout(Duration::from_secs(5), rx).await {
            Ok(Ok(Ok(result))) => Ok(result),
            Ok(Ok(Err(panic))) => {
                eprintln!("Plugin {} panicked: {:?}", plugin_id, panic);
                Err(Error::PluginCrashed(plugin_id.to_string()))
            }
            Ok(Err(e)) => Err(Error::PluginError(e.to_string())),
            Err(_) => Err(Error::PluginTimeout(plugin_id.to_string())),
        }
    }
}
```

---

## Plugin Manifest Format

### plugin.toml

```toml
[plugin]
id = "com.example.git-plugin"
name = "Git Integration"
version = "1.0.0"
author = "John Doe <john@example.com>"
description = "Git integration for AIT42 Editor"
homepage = "https://github.com/example/git-plugin"
license = "MIT"

[requires]
editor_version = ">=1.0.0"

[dependencies]
# Other plugin dependencies
"com.example.utils" = "^0.5.0"

[permissions]
# Requested permissions
read_files = true
write_files = true
execute_commands = true
network = false

[resources]
# Resource limits
max_memory = "100MB"
max_cpu_time = "5s"

[commands]
# Registered commands
[[commands.entries]]
name = "git-commit"
description = "Commit changes"
usage = ":git-commit <message>"

[[commands.entries]]
name = "git-push"
description = "Push to remote"
usage = ":git-push"

[hooks]
# Registered hooks
buffer_saved = true
buffer_opened = true

[ui]
# UI extensions
[[ui.widgets]]
id = "git-status"
position = "status-bar"

[config]
# Default configuration
[config.default]
auto_fetch = true
push_on_commit = false
```

---

## Example Plugins

### Example 1: Simple Lua Plugin (Word Count)

```lua
-- word-count.lua

function init()
    log("Word count plugin initialized")
    register_command("word-count", word_count_command)
    register_hook("buffer_changed", on_buffer_changed)
end

function word_count_command(args)
    local buffer = get_active_buffer()
    if buffer == nil then
        notify("No active buffer", "error")
        return
    end

    local content = buffer:get_content()
    local word_count = count_words(content)
    local char_count = #content

    notify(string.format("Words: %d, Characters: %d", word_count, char_count), "info")
end

function count_words(text)
    local count = 0
    for word in text:gmatch("%S+") do
        count = count + 1
    end
    return count
end

function on_buffer_changed(event)
    -- Update word count in status bar
    local buffer = get_buffer(event.buffer_id)
    local content = buffer:get_content()
    local count = count_words(content)

    set_status_bar_item("word-count", string.format("%d words", count))
end
```

### Example 2: Wasm Plugin (Auto-format)

```rust
// auto-format/src/lib.rs

use ait42_plugin_sdk::*;

#[plugin_main]
fn init(ctx: &mut PluginContext) -> Result<()> {
    ctx.log(LogLevel::Info, "Auto-format plugin initialized");

    ctx.register_hook(HookType::BufferPreSave, Box::new(FormatHook))?;

    Ok(())
}

struct FormatHook;

impl HookHandler for FormatHook {
    fn handle(&mut self, event: &HookEvent, ctx: &mut PluginContext) -> HookResult {
        if let HookEvent::BufferPreSave { buffer_id, .. } = event {
            match format_buffer(ctx, *buffer_id) {
                Ok(_) => {
                    ctx.log(LogLevel::Info, "Formatted buffer");
                    HookResult::Continue
                }
                Err(e) => {
                    ctx.log(LogLevel::Error, &format!("Format failed: {}", e));
                    HookResult::Continue
                }
            }
        } else {
            HookResult::Continue
        }
    }
}

fn format_buffer(ctx: &PluginContext, buffer_id: BufferId) -> Result<()> {
    let buffer = ctx.editor().get_buffer(buffer_id)?;
    let content = buffer.content()?;

    // Format with rustfmt (example)
    let formatted = format_with_rustfmt(&content)?;

    if formatted != content {
        buffer.set_content(&formatted)?;
    }

    Ok(())
}

fn format_with_rustfmt(code: &str) -> Result<String> {
    // Call rustfmt or other formatter
    // ...
    Ok(code.to_string())
}
```

### Example 3: Native Plugin (Performance Monitor)

```rust
// perf-monitor/src/lib.rs

use ait42_plugin_sdk::*;
use std::time::Instant;

pub struct PerfMonitorPlugin {
    start_time: Instant,
    operation_times: Vec<Duration>,
}

impl Plugin for PerfMonitorPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &PluginMetadata {
            id: "com.ait42.perf-monitor".into(),
            name: "Performance Monitor".into(),
            version: "1.0.0".into(),
            author: "AIT42 Team".into(),
            description: "Monitor editor performance".into(),
            // ...
        }
    }

    fn init(&mut self, ctx: &mut PluginContext) -> Result<()> {
        ctx.register_hook(HookType::BufferPostChange, Box::new(BufferChangeTimer))?;
        ctx.register_hook(HookType::UiRender, Box::new(RenderTimeWidget))?;
        Ok(())
    }
}

struct BufferChangeTimer;

impl HookHandler for BufferChangeTimer {
    fn handle(&mut self, event: &HookEvent, ctx: &mut PluginContext) -> HookResult {
        if let HookEvent::BufferPostChange { .. } = event {
            // Measure time taken
            let elapsed = /* measure */;
            ctx.log(LogLevel::Debug, &format!("Buffer change: {:?}", elapsed));
        }
        HookResult::Continue
    }
}
```

---

**End of Plugin API Specification**

Generated by: AIT42 Coordinator
Date: 2025-01-06
Version: 1.0.0
Status: Phase 2 Planning (Draft)
