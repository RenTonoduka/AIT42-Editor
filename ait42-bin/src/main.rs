//! AIT42 Editor - Main Entry Point
//!
//! A modern TUI code editor with integrated AI agents for development automation.
//!
//! # Features
//! - TUI-based code editing with syntax highlighting
//! - LSP integration for intelligent code completion
//! - 49 AI agents for development tasks
//! - Tmux session management
//! - File system operations
//!
//! # Usage
//! ```bash
//! # Open current directory
//! ait42
//!
//! # Open specific file
//! ait42 src/main.rs
//!
//! # Open directory
//! ait42 /path/to/project
//! ```

use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;
use tracing::{info, Level};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// AIT42 Editor CLI Arguments
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// File or directory to open
    #[arg(value_name = "PATH")]
    path: Option<PathBuf>,

    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Configuration file path
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Log file path
    #[arg(long, value_name = "FILE")]
    log_file: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    setup_logging(&args)?;

    info!("Starting AIT42 Editor v{}", env!("CARGO_PKG_VERSION"));
    info!("Rust version: {}", env!("CARGO_PKG_RUST_VERSION"));

    // Load configuration
    let _config = load_config(&args).await?;
    info!("Configuration loaded successfully");

    // Determine target path
    let target_path = resolve_target_path(args.path)?;
    info!("Target path: {}", target_path.display());

    // Start TUI application
    info!("Starting TUI application...");
    ait42_tui::run_with_file(target_path)
        .await
        .context("TUI application error")?;

    info!("AIT42 Editor shutdown complete");
    Ok(())
}

/// Setup logging based on CLI arguments
fn setup_logging(args: &Args) -> Result<()> {
    let log_level = if args.debug {
        Level::DEBUG
    } else if args.verbose {
        Level::INFO
    } else {
        Level::WARN
    };

    let filter = EnvFilter::from_default_env()
        .add_directive(log_level.into())
        .add_directive("ait42=trace".parse()?);

    let subscriber = tracing_subscriber::registry().with(filter);

    // Console logging
    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(false)
        .with_file(true)
        .with_line_number(true);

    // File logging (optional)
    if let Some(log_path) = &args.log_file {
        let file_appender = tracing_appender::rolling::daily(
            log_path.parent().unwrap_or(std::path::Path::new(".")),
            log_path
                .file_name()
                .unwrap_or(std::ffi::OsStr::new("ait42.log")),
        );
        let file_layer = fmt::layer().json().with_writer(file_appender);

        subscriber.with(fmt_layer).with(file_layer).init();
    } else {
        subscriber.with(fmt_layer).init();
    }

    Ok(())
}

/// Load configuration from file or defaults
async fn load_config(args: &Args) -> Result<ait42_config::Config> {
    use ait42_config::{default_config, ConfigLoader};

    let config = if let Some(config_path) = &args.config {
        ConfigLoader::with_path(config_path.clone())
            .load()
            .await
            .context("Failed to load config file")?
    } else {
        let loader = ConfigLoader::new().context("Failed to create config loader")?;
        loader
            .load_or_create()
            .await
            .unwrap_or_else(|_| default_config())
    };

    Ok(config)
}

/// Resolve target path from arguments or current directory
fn resolve_target_path(path: Option<PathBuf>) -> Result<PathBuf> {
    let target = path.unwrap_or_else(|| PathBuf::from("."));

    let canonical = target
        .canonicalize()
        .with_context(|| format!("Path does not exist: {}", target.display()))?;

    // If it's a directory, try to find a suitable file to open
    if canonical.is_dir() {
        info!("Target is a directory, looking for a file to open...");

        // Try common project files
        let candidates = [
            "README.md",
            "Cargo.toml",
            "package.json",
            "main.rs",
            "src/main.rs",
            "src/lib.rs",
        ];

        for candidate in &candidates {
            let file_path = canonical.join(candidate);
            if file_path.exists() && file_path.is_file() {
                info!("Opening file: {}", file_path.display());
                return Ok(file_path);
            }
        }

        // If no suitable file found, return error message
        anyhow::bail!(
            "Target is a directory. Please specify a file to open.\n\
             Directory: {}\n\
             \n\
             Usage: ait42 <file>",
            canonical.display()
        );
    }

    Ok(canonical)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_current_dir() {
        let result = resolve_target_path(None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_resolve_invalid_path() {
        let result = resolve_target_path(Some(PathBuf::from("/nonexistent/path")));
        assert!(result.is_err());
    }
}
