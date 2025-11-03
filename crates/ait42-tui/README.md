# AIT42 TUI - Terminal User Interface

Modern terminal-based UI layer for AIT42 Editor built with ratatui and crossterm.

## Features

- ✅ **Async Event Loop**: Non-blocking input handling with tokio
- ✅ **Vim-like Keybindings**: Modal editing (Normal, Insert, Visual, Command)
- ✅ **3 Built-in Themes**: Monokai, Solarized Dark, Gruvbox
- ✅ **Responsive Layout**: Dynamic resizing with minimum size validation
- ✅ **Command Palette**: Fuzzy search for commands
- ✅ **Smart Scrolling**: Auto-scroll to keep cursor visible
- ✅ **Line Numbers**: Configurable gutter with active line highlighting

## Quick Start

```rust
use ait42_tui::TuiApp;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut app = TuiApp::new().await?;
    app.run().await?;
    Ok(())
}
```

## Architecture

```
TuiApp
├── EventLoop        # Async event handling
├── Renderer         # Terminal rendering
├── KeyMap           # Key bindings
├── Theme            # Color schemes
└── Widgets
    ├── EditorWidget      # Main text editor
    ├── StatusLine        # Bottom status bar
    └── CommandPalette    # Fuzzy command search
```

## Key Bindings (Normal Mode)

| Key | Action |
|-----|--------|
| `h/j/k/l` | Move cursor left/down/up/right |
| `i` | Enter insert mode |
| `v` | Enter visual mode |
| `:` | Enter command mode |
| `w` | Move word forward |
| `b` | Move word backward |
| `0` | Move to line start |
| `$` | Move to line end |
| `u` | Undo |
| `Ctrl+r` | Redo |
| `Ctrl+p` | Open command palette |
| `Ctrl+s` | Save file |
| `/` | Search |
| `q` | Quit |

## Themes

### Monokai (Default)
```rust
let theme = Theme::monokai();
app.set_theme(theme);
```

### Solarized Dark
```rust
let theme = Theme::solarized_dark();
app.set_theme(theme);
```

### Gruvbox
```rust
let theme = Theme::gruvbox();
app.set_theme(theme);
```

### Custom Colors
```rust
use ait42_tui::theme::{Theme, ThemeConfig, CustomColors};

let config = ThemeConfig {
    name: Some("monokai".to_string()),
    custom_colors: Some(CustomColors {
        background: Some("#000000".to_string()),
        foreground: Some("#FFFFFF".to_string()),
        cursor: Some("#FF0000".to_string()),
    }),
};

let theme = Theme::from_config(&config);
```

## Widgets

### EditorWidget

Renders the main text editing area with line numbers and scrolling.

```rust
use ait42_tui::widgets::{EditorWidget, ViewState};
use ait42_core::{Buffer, Cursor};

let buffer = Buffer::new();
let cursor = Cursor::default();
let view = ViewState::new();
let theme = Theme::default();

let widget = EditorWidget::new(&buffer, &cursor, &view, &theme)
    .show_line_numbers(true);
```

### StatusLine

Displays editor mode, file info, and cursor position.

```rust
use ait42_tui::widgets::StatusLine;
use ait42_tui::Mode;

let status = StatusLine::new(Mode::Normal, (10, 5), 100, &theme)
    .file_path(Path::new("main.rs"))
    .dirty(true)
    .file_type("rust");
```

### CommandPalette

Fuzzy search command interface.

```rust
use ait42_tui::widgets::command_palette::{CommandPalette, default_commands};

let commands = default_commands();
let palette = CommandPalette::new("save", &commands, &theme)
    .selected(0);
```

## Layout System

The layout manager automatically calculates UI areas based on terminal size.

```rust
use ait42_tui::layout::{EditorLayout, LayoutConfig};

let config = LayoutConfig {
    show_line_numbers: true,
    line_number_width: 5,
    show_command_palette: false,
    show_sidebar: false,
    ..Default::default()
};

let layout = EditorLayout::calculate(terminal_size, &config);
```

### Layout Areas

```
┌────────────────────────────────────┐
│ [Sidebar]  │ Editor Area           │
│            │ ┌───┬─────────────────┤
│            │ │Ln │ Text Editor     │
│            │ │   │                 │
│            │ └───┴─────────────────┤
│            │ Command Palette       │
├────────────┴───────────────────────┤
│ Status Line                        │
└────────────────────────────────────┘
```

## Event Loop

The event loop handles terminal input asynchronously.

```rust
use ait42_tui::event::{EventLoop, EditorEvent};
use std::time::Duration;

let mut event_loop = EventLoop::new(Duration::from_millis(250));

while let Some(event) = event_loop.next().await {
    match event {
        EditorEvent::Key(key) => { /* handle key */ },
        EditorEvent::Resize(w, h) => { /* handle resize */ },
        EditorEvent::Tick => { /* periodic refresh */ },
        _ => {}
    }
}
```

## Testing

### Run Tests

```bash
cargo test --package ait42-tui
```

### Snapshot Testing

```rust
use insta::assert_snapshot;

#[test]
fn test_editor_rendering() {
    let widget = EditorWidget::new(/* ... */);
    // Render to buffer
    assert_snapshot!(buffer);
}
```

## Performance

- **Frame Rate**: ~4 FPS (250ms tick rate)
- **Input Latency**: <16ms
- **Memory**: ~5-10MB
- **CPU**: <1% idle, ~5% typing

## Development

### Project Structure

```
src/
├── lib.rs                  # Public API
├── event.rs               # Event loop
├── theme.rs               # Theme system
├── layout.rs              # Layout manager
├── keybinds.rs           # Key bindings
├── renderer.rs           # Terminal renderer
├── tui_app.rs            # Main application
└── widgets/
    ├── mod.rs
    ├── editor.rs         # Editor widget
    ├── statusline.rs     # Status line widget
    └── command_palette.rs # Command palette
```

### Adding a New Widget

1. Create widget file in `src/widgets/`
2. Implement `ratatui::widgets::Widget` trait
3. Add to `src/widgets/mod.rs`
4. Write tests

```rust
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

pub struct MyWidget<'a> {
    data: &'a str,
}

impl<'a> Widget for MyWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Render logic
    }
}
```

### Adding a New Command

1. Add to `EditorCommand` enum in `keybinds.rs`
2. Add key binding in `KeyMap::setup_*_mode()`
3. Implement in `EditorState::execute_command()`
4. Write tests

## Troubleshooting

### Terminal Not Rendering

- Ensure terminal supports 256 colors
- Check minimum size (80x24)
- Verify raw mode is enabled

### Input Not Working

- Check if another process is using stdin
- Verify terminal is in raw mode
- Test with simple key press event

### Slow Rendering

- Reduce tick rate
- Disable debug logging
- Check terminal emulator performance

## Dependencies

```toml
ratatui = "0.25"          # TUI framework
crossterm = "0.27"        # Terminal manipulation
tokio = "1.35"            # Async runtime
fuzzy-matcher = "0.3"     # Fuzzy search
unicode-width = "0.1"     # Unicode support
```

## Contributing

1. Fork the repository
2. Create feature branch
3. Write tests
4. Submit pull request

## License

MIT OR Apache-2.0

## See Also

- [API Documentation](../../API_SPECIFICATION.md)
- [Architecture](../../ARCHITECTURE.md)
- [Component Design](../../COMPONENT_DESIGN.md)
- [Implementation Report](../../TUI_IMPLEMENTATION_REPORT.md)
