//! AIT42 TUI - Terminal User Interface
//!
//! Provides the terminal-based UI layer using ratatui.

use ait42_core::Editor;
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::{io, path::PathBuf};
use tracing::info;

pub mod app;
pub mod ui;

/// Run the TUI application
pub async fn run(editor: Editor, path: PathBuf) -> Result<()> {
    info!("Initializing TUI");

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run app loop
    let result = run_app(&mut terminal, editor, path).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

/// Main application loop
async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    mut editor: Editor,
    _path: PathBuf,
) -> Result<()> {
    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Min(0),
                    Constraint::Length(1),
                ])
                .split(f.area());

            // Title bar
            let title = Paragraph::new("AIT42 Editor - Press 'q' to quit")
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(title, chunks[0]);

            // Editor area
            let editor_block = Block::default()
                .title("Editor")
                .borders(Borders::ALL);
            f.render_widget(editor_block, chunks[1]);

            // Status bar
            let status = Paragraph::new("Ready")
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(status, chunks[2]);
        })?;

        // Handle input
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => {
                        info!("User quit");
                        return Ok(());
                    }
                    _ => {}
                }
            }
        }
    }
}
