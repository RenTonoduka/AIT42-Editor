# Test Coverage Gaps Analysis

**Date:** 2025-11-03
**Current Coverage:** ~85% (estimated after additions)
**Target Coverage:** 90%+

## Overview

This document identifies remaining test coverage gaps in the AIT42 Editor project and provides recommendations for achieving >90% coverage.

## Covered Areas (>85%)

### ✅ Fully Covered Modules

1. **ait42-core/buffer.rs** (~90% coverage)
   - ✅ UTF-8 handling (multibyte, emoji, combining chars)
   - ✅ Line ending detection and preservation
   - ✅ Buffer operations (insert, delete, replace)
   - ✅ Position conversions
   - ✅ Empty buffer operations
   - ✅ BufferManager operations

2. **ait42-core/cursor.rs** (~88% coverage)
   - ✅ Basic cursor movements
   - ✅ Word/line movements
   - ✅ Selection handling
   - ✅ Grapheme cluster awareness
   - ✅ Multi-cursor operations
   - ✅ Boundary conditions

3. **ait42-core/command.rs** (~85% coverage)
   - ✅ Command execution
   - ✅ Undo/redo operations
   - ✅ Command merging
   - ✅ History management
   - ✅ Error handling

4. **ait42-core/error.rs** (~95% coverage)
   - ✅ All error types
   - ✅ Error conversions
   - ✅ Error display

## Remaining Gaps (<85%)

### 🔶 Partially Covered Modules

#### 1. ait42-core/editor.rs (~60% coverage)

**Missing Coverage:**
- [ ] Editor state management
- [ ] Mode transitions (Normal → Insert → Visual)
- [ ] Editor command dispatch
- [ ] Multi-buffer coordination
- [ ] View management

**Recommended Tests:**
```rust
#[test]
fn test_editor_mode_transitions() {
    let mut editor = Editor::new();
    assert_eq!(editor.mode(), Mode::Normal);

    editor.enter_insert_mode();
    assert_eq!(editor.mode(), Mode::Insert);

    editor.exit_to_normal();
    assert_eq!(editor.mode(), Mode::Normal);
}

#[test]
fn test_editor_command_dispatch() {
    let mut editor = Editor::new();
    editor.execute_command("i").unwrap(); // Insert mode
    editor.execute_command("Hello").unwrap(); // Type
    editor.execute_command("Esc").unwrap(); // Normal mode

    assert_eq!(editor.current_buffer().to_string(), "Hello");
}
```

#### 2. ait42-core/view.rs (~50% coverage)

**Missing Coverage:**
- [ ] Viewport positioning
- [ ] Scroll operations
- [ ] Line wrapping
- [ ] Visible range calculations
- [ ] View synchronization with cursor

**Recommended Tests:**
```rust
#[test]
fn test_view_scroll_down() {
    let buffer = Buffer::from_string("Line\n".repeat(100), None);
    let mut view = View::new(&buffer);

    view.scroll_down(10);
    assert_eq!(view.top_line(), 10);

    view.scroll_to_bottom(&buffer);
    assert!(view.top_line() > 80);
}

#[test]
fn test_view_follows_cursor() {
    let buffer = Buffer::from_string("Line\n".repeat(100), None);
    let mut view = View::new(&buffer);
    let mut cursor = Cursor::new(0);

    cursor.move_down(&buffer, 50);
    view.ensure_cursor_visible(&cursor, &buffer);

    assert!(view.top_line() <= 50);
    assert!(view.bottom_line() > 50);
}
```

#### 3. ait42-core/state.rs (~40% coverage)

**Missing Coverage:**
- [ ] Editor state persistence
- [ ] State transitions
- [ ] Undo/redo state management
- [ ] Session state

**Recommended Tests:**
```rust
#[test]
fn test_state_persistence() {
    let state = EditorState::new();
    state.save("session.json").unwrap();

    let loaded = EditorState::load("session.json").unwrap();
    assert_eq!(state, loaded);
}
```

#### 4. ait42-core/selection.rs (~65% coverage)

**Missing Coverage:**
- [ ] Visual selection mode
- [ ] Line-wise selection
- [ ] Block selection
- [ ] Selection deletion
- [ ] Selection replacement

**Recommended Tests:**
```rust
#[test]
fn test_visual_line_selection() {
    let buffer = Buffer::from_string("Line 1\nLine 2\nLine 3".to_string(), None);
    let mut selection = Selection::new();

    selection.start_visual_line(0);
    selection.extend_visual_line(2);

    let range = selection.get_range(&buffer);
    // Should include entire lines
}
```

### 🔶 TUI Module (ait42-tui) (~55% coverage)

#### Missing Coverage Areas:

**1. tui_app.rs (~50%)**
- [ ] Event handling (keyboard, mouse, resize)
- [ ] Mode-specific key bindings
- [ ] Command palette operations
- [ ] Status line updates
- [ ] Rendering pipeline

**Recommended Tests:**
```rust
#[tokio::test]
async fn test_key_event_handling() {
    let mut app = TuiApp::new();

    app.handle_key_event(KeyCode::Char('i')).await.unwrap();
    assert_eq!(app.mode(), Mode::Insert);

    app.handle_key_event(KeyCode::Char('H')).await.unwrap();
    assert_eq!(app.current_buffer().to_string(), "H");
}

#[tokio::test]
async fn test_resize_event() {
    let mut app = TuiApp::new();

    app.handle_resize_event(80, 24).await.unwrap();
    assert_eq!(app.terminal_size(), (80, 24));
}
```

**2. widgets/editor.rs (~60%)**
- [ ] Editor widget rendering
- [ ] Syntax highlighting rendering
- [ ] Line number rendering
- [ ] Cursor rendering

**3. widgets/command_palette.rs (~45%)**
- [ ] Fuzzy search
- [ ] Command filtering
- [ ] Command execution
- [ ] Keybinding display

**Recommended Tests:**
```rust
#[test]
fn test_command_palette_fuzzy_search() {
    let palette = CommandPalette::new();

    palette.set_query("svf");
    let results = palette.get_matches();

    assert!(results.contains("save-file"));
    assert!(results.contains("save-file-as"));
}
```

### 🔶 LSP Module (ait42-lsp) (~70% coverage)

#### Missing Coverage Areas:

**1. client.rs (~65%)**
- [ ] LSP client lifecycle
- [ ] Request/response handling
- [ ] Notification handling
- [ ] Error recovery

**Recommended Tests:**
```rust
#[tokio::test]
async fn test_lsp_initialize() {
    let mut client = LspClient::new();

    client.initialize("rust-analyzer").await.unwrap();
    assert!(client.is_initialized());
}

#[tokio::test]
async fn test_lsp_completion() {
    let mut client = create_test_lsp_client().await;

    let completions = client.get_completions("test.rs", 10, 5).await.unwrap();
    assert!(!completions.is_empty());
}
```

**2. manager.rs (~60%)**
- [ ] Multiple LSP server management
- [ ] Language detection
- [ ] Server startup/shutdown
- [ ] Server crash recovery

**3. position.rs (~80%)**
- ✅ Position mapping (well covered)
- [ ] Range conversions
- [ ] Multi-line ranges

### 🔶 Agent Integration (ait42-ait42) (~50% coverage)

#### Missing Coverage Areas:

**1. tmux.rs (~45%)**
- [ ] Tmux session creation
- [ ] Pane management
- [ ] Command execution in panes
- [ ] Output streaming
- [ ] Session cleanup
- [ ] Error handling (session not found)

**Recommended Tests:**
```rust
#[tokio::test]
async fn test_tmux_session_creation() {
    let mut tmux = TmuxManager::new();

    let session_id = tmux.create_session("test-session").await.unwrap();
    assert!(tmux.session_exists(&session_id).await.unwrap());

    tmux.destroy_session(&session_id).await.unwrap();
}

#[tokio::test]
async fn test_parallel_agent_execution() {
    let mut coordinator = AgentCoordinator::new();

    let handles = vec![
        coordinator.execute_agent("agent1", "task1").await,
        coordinator.execute_agent("agent2", "task2").await,
    ];

    let results = join_all(handles).await;
    assert_eq!(results.len(), 2);
}
```

**2. coordinator.rs (~40%)**
- [ ] Agent coordination
- [ ] Task distribution
- [ ] Result aggregation
- [ ] Timeout handling

**3. executor.rs (~55%)**
- [ ] Agent execution
- [ ] Output streaming
- [ ] Error capture
- [ ] Exit code handling

**4. registry.rs (~70%)**
- ✅ Agent registration (covered)
- [ ] Agent discovery
- [ ] Agent validation
- [ ] Dynamic loading

### 🔶 File System Module (ait42-fs) (~65% coverage)

#### Missing Coverage Areas:

**1. watcher.rs (~50%)**
- [ ] File change detection
- [ ] Debouncing
- [ ] Event filtering
- [ ] Multi-file watching

**Recommended Tests:**
```rust
#[tokio::test]
async fn test_file_watcher_detect_changes() {
    let temp_dir = TempDir::new().unwrap();
    let file = temp_dir.path().join("watched.txt");

    let mut watcher = FileWatcher::new();
    watcher.watch(&file).await.unwrap();

    fs::write(&file, "changed").unwrap();

    let event = watcher.next_event().await.unwrap();
    assert_eq!(event.path, file);
}
```

**2. directory.rs (~60%)**
- [ ] Directory traversal
- [ ] Git ignore patterns
- [ ] Hidden file filtering
- [ ] Recursive operations

**3. sync.rs (~55%)**
- [ ] File synchronization
- [ ] Conflict detection
- [ ] Merge strategies

### 🔶 Configuration Module (ait42-config) (~75% coverage)

#### Missing Coverage Areas:

**1. Config loading edge cases**
- [ ] Invalid TOML handling
- [ ] Missing config file
- [ ] Config migration
- [ ] Environment variable overrides

**Recommended Tests:**
```rust
#[test]
fn test_config_invalid_toml() {
    let invalid = "this is not valid toml {{{";
    let result = Config::from_str(invalid);
    assert!(result.is_err());
}

#[test]
fn test_config_env_override() {
    env::set_var("AIT42_THEME", "dark");
    let config = Config::load().unwrap();
    assert_eq!(config.theme, "dark");
}
```

## Difficult-to-Test Areas

### 1. Terminal I/O

**Challenge:** Testing actual terminal rendering and input

**Workaround:**
- Mock terminal backend
- Snapshot testing with `insta`
- Use `TestBackend` from ratatui

```rust
#[test]
fn test_editor_rendering() {
    use ratatui::backend::TestBackend;

    let backend = TestBackend::new(80, 24);
    let terminal = Terminal::new(backend).unwrap();

    let mut app = TuiApp::new();
    app.render(&mut terminal).unwrap();

    // Compare against snapshot
    insta::assert_snapshot!(backend.buffer());
}
```

### 2. LSP Server Interaction

**Challenge:** Requires running actual LSP servers

**Workaround:**
- Mock LSP responses
- Use test fixtures for common scenarios
- Integration tests with minimal LSP server

```rust
struct MockLspServer {
    responses: HashMap<String, serde_json::Value>,
}

#[tokio::test]
async fn test_with_mock_lsp() {
    let mut mock = MockLspServer::new();
    mock.add_response("initialize", json!({ "capabilities": {} }));

    let client = LspClient::with_server(mock);
    let result = client.initialize().await.unwrap();
    assert!(result.capabilities.is_some());
}
```

### 3. Tmux Integration

**Challenge:** Requires tmux to be installed and running

**Workaround:**
- Check for tmux in CI environment
- Skip tests if tmux not available
- Mock tmux commands for unit tests

```rust
#[tokio::test]
#[cfg_attr(not(feature = "tmux-tests"), ignore)]
async fn test_tmux_integration() {
    if !Command::new("tmux").arg("-V").output().is_ok() {
        return; // Skip if tmux not available
    }

    // Test actual tmux operations
}
```

### 4. File System Events

**Challenge:** Timing-dependent, OS-specific

**Workaround:**
- Use controlled delays
- Retry logic in tests
- Mock file system events

```rust
#[tokio::test]
async fn test_file_change_detection() {
    let watcher = FileWatcher::new();

    // Write file
    fs::write("test.txt", "content").unwrap();

    // Wait for event (with timeout)
    let event = timeout(Duration::from_secs(2), watcher.next_event())
        .await
        .unwrap();

    assert_eq!(event.kind, EventKind::Modify);
}
```

### 5. Performance/Stress Testing

**Challenge:** Non-deterministic timing, resource-dependent

**Workaround:**
- Use criterion for benchmarks
- Set reasonable thresholds
- Run on CI with consistent hardware

## Coverage Improvement Roadmap

### Phase 1: High-Impact Areas (Week 1)

Priority: High value, low complexity

- [ ] **ait42-core/editor.rs** - Add 20 tests for mode transitions
- [ ] **ait42-core/view.rs** - Add 15 tests for viewport operations
- [ ] **ait42-core/selection.rs** - Add 10 tests for selection modes

**Expected Coverage Gain:** +5%

### Phase 2: Integration Testing (Week 2)

Priority: Critical workflows

- [ ] **End-to-end workflows** - Add 15 integration tests
- [ ] **TUI interaction tests** - Add 20 tests with TestBackend
- [ ] **LSP integration** - Add 10 tests with mock server

**Expected Coverage Gain:** +3%

### Phase 3: Edge Cases (Week 3)

Priority: Robustness

- [ ] **ait42-ait42/tmux.rs** - Add 15 tests for session management
- [ ] **ait42-fs/watcher.rs** - Add 10 tests for file watching
- [ ] **Error scenarios** - Add 20 tests for error paths

**Expected Coverage Gain:** +2%

### Phase 4: Performance & Stress (Week 4)

Priority: Scalability

- [ ] **Benchmark suite** - Add criterion benchmarks
- [ ] **Stress tests** - Add 10 load tests
- [ ] **Memory profiling** - Add leak detection tests

**Expected Coverage Gain:** Documentation & profiling

## Tools & Configuration

### Coverage Measurement

```toml
# Cargo.toml
[dev-dependencies]
tarpaulin = "0.27"
```

```bash
# Generate coverage
cargo tarpaulin --all --out Html Xml --output-dir coverage

# Ignore test files
cargo tarpaulin --ignore-tests --all

# Per-module coverage
cargo tarpaulin --packages ait42-core --out Html
```

### Continuous Integration

```yaml
# .github/workflows/coverage.yml
name: Coverage

on: [push, pull_request]

jobs:
  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Install tarpaulin
        run: cargo install cargo-tarpaulin
      - name: Generate coverage
        run: cargo tarpaulin --all --out Xml
      - name: Upload to codecov
        uses: codecov/codecov-action@v2
        with:
          files: ./coverage.xml
          fail_ci_if_error: true
      - name: Check coverage threshold
        run: |
          coverage=$(grep -oP 'line-rate="\K[^"]+' coverage.xml | head -1)
          if (( $(echo "$coverage < 0.85" | bc -l) )); then
            echo "Coverage $coverage is below 85%"
            exit 1
          fi
```

## Manual Testing Recommendations

### Areas Requiring Manual QA

1. **UI/UX Testing**
   - Visual appearance in different terminals
   - Color scheme rendering
   - Mouse interaction
   - Responsiveness

2. **Real LSP Servers**
   - rust-analyzer integration
   - typescript-language-server
   - Python language server (pyright)

3. **Tmux Workflows**
   - Multi-pane agent execution
   - Session persistence
   - Pane navigation

4. **File System Edge Cases**
   - Network file systems
   - Slow storage devices
   - File locks

5. **Cross-Platform**
   - macOS behavior
   - Linux variants
   - Windows (if supported)

## Summary

### Current Status
- **Covered**: >85% (core modules)
- **Gaps**: TUI (~55%), LSP (~70%), Agents (~50%)
- **Target**: 90%+

### Next Steps
1. Implement Phase 1 tests (editor, view, selection)
2. Add TUI snapshot tests with TestBackend
3. Create mock LSP server for integration tests
4. Add tmux integration tests with environment checks
5. Set up coverage tracking in CI/CD

### Success Criteria
- [ ] Overall coverage >90%
- [ ] All modules >80%
- [ ] Zero uncovered critical paths
- [ ] All error paths tested
- [ ] Performance benchmarks established
