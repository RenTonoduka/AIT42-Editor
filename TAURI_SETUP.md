# AIT42 Editor - Tauri GUI Setup

## Overview

This document describes the Tauri project structure for AIT42 Editor's GUI version.

## Project Structure

```
AIT42-Editor/
├── src-tauri/                 # Rust backend (Tauri)
│   ├── Cargo.toml            # Rust dependencies
│   ├── tauri.conf.json       # Tauri configuration
│   ├── build.rs              # Build script
│   ├── icons/                # Application icons
│   └── src/
│       ├── main.rs           # Main application entry
│       ├── state.rs          # Application state management
│       └── commands.rs       # Tauri commands (IPC)
├── src/                       # React frontend
│   ├── components/           # React components
│   ├── hooks/                # Custom hooks
│   ├── services/             # API services
│   ├── types/                # TypeScript types
│   ├── App.tsx               # Main app component
│   ├── main.tsx              # React entry point
│   └── index.css             # Global styles
├── public/                    # Static assets
├── index.html                 # HTML template
├── package.json               # Node dependencies
├── vite.config.ts            # Vite configuration
└── tsconfig.json             # TypeScript configuration
```

## Technology Stack

### Backend (Rust + Tauri)
- **Tauri v1.5**: Native desktop application framework
- **ait42-core**: Core editor functionality
- **ait42-lsp**: Language Server Protocol integration
- **ait42-fs**: File system operations
- **ait42-config**: Configuration management
- **tokio**: Async runtime

### Frontend (React + TypeScript)
- **React 18**: UI framework
- **TypeScript**: Type-safe JavaScript
- **Vite**: Fast build tool and dev server
- **Tailwind CSS**: Utility-first CSS framework
- **Monaco Editor**: VS Code's editor component
- **xterm.js**: Terminal emulator
- **Lucide React**: Icon library

## Installation

### Prerequisites

1. **Rust** (1.91+)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Node.js** (18+)
   ```bash
   # macOS (using Homebrew)
   brew install node

   # Or download from https://nodejs.org
   ```

3. **Tauri Prerequisites**

   **macOS**:
   ```bash
   xcode-select --install
   ```

   **Linux (Debian/Ubuntu)**:
   ```bash
   sudo apt update
   sudo apt install libwebkit2gtk-4.0-dev \
     build-essential \
     curl \
     wget \
     file \
     libssl-dev \
     libgtk-3-dev \
     libayatana-appindicator3-dev \
     librsvg2-dev
   ```

   **Windows**:
   - Install [Microsoft Visual Studio C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
   - Install [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)

### Setup Steps

1. **Install Node dependencies**:
   ```bash
   cd /Users/tonodukaren/Programming/AI/02_Workspace/05_Client/03_Sun/AIT42-Editor
   npm install
   ```

2. **Install Tauri CLI** (optional, already in package.json):
   ```bash
   cargo install tauri-cli --version ^1.5
   ```

## Development

### Running in Development Mode

```bash
# Start the Tauri dev server (runs both frontend and backend)
npm run tauri:dev

# Or use cargo directly
cargo tauri dev
```

This will:
1. Start Vite dev server on http://localhost:5173
2. Compile the Rust backend
3. Open the application window with hot-reload enabled

### Frontend Only Development

```bash
npm run dev
```

This runs only the Vite dev server without Tauri, useful for UI development.

## Building

### Development Build

```bash
npm run tauri:build
```

### Release Build

```bash
# Set release mode
cargo tauri build --release
```

The built application will be in:
- **macOS**: `src-tauri/target/release/bundle/macos/`
- **Windows**: `src-tauri/target/release/bundle/msi/`
- **Linux**: `src-tauri/target/release/bundle/deb/` or `appimage/`

## Tauri Commands (IPC)

The following commands are available for frontend-backend communication:

### File Operations
- `open_file(path: string)`: Read file content
- `save_file(path: string, content: string)`: Save file
- `create_file(path: string)`: Create new file
- `get_file_tree(rootPath: string)`: Get directory tree

### LSP Integration
- `format_document(path, content)`: Format code
- `get_diagnostics(path)`: Get errors/warnings
- `get_completions(path, line, character)`: Get code completions
- `goto_definition(path, line, character)`: Navigate to definition

### Search
- `search_files(rootPath, pattern)`: Search files by pattern

## Configuration

### Tauri Configuration (`src-tauri/tauri.conf.json`)

Key settings:
- **App identifier**: `com.ait42.editor`
- **Window size**: 1400x900 (default)
- **Minimum size**: 800x600
- **Dev URL**: http://localhost:5173
- **Build output**: `../dist`

### Vite Configuration (`vite.config.ts`)

- **Port**: 5173 (fixed for Tauri)
- **Path alias**: `@` maps to `./src`
- **Target**: Chrome 105 (Windows) / Safari 13 (macOS/Linux)

## Security

### Content Security Policy

The app uses a strict CSP defined in `tauri.conf.json`:
```
default-src 'self';
script-src 'self' 'unsafe-inline';
style-src 'self' 'unsafe-inline';
img-src 'self' data: https:;
font-src 'self' data:;
```

### IPC Security

All Tauri commands are explicitly whitelisted in the `invoke_handler`.

## Icons

Application icons should be placed in `src-tauri/icons/`:
- `32x32.png`
- `128x128.png`
- `128x128@2x.png`
- `icon.icns` (macOS)
- `icon.ico` (Windows)

Generate icons from a source image:
```bash
cargo tauri icon /path/to/icon.png
```

## Debugging

### Frontend Debugging

1. **Open DevTools**: Enabled automatically in debug mode
2. **React DevTools**: Install browser extension

### Backend Debugging

1. **Rust logs**: Set `RUST_LOG` environment variable
   ```bash
   RUST_LOG=debug npm run tauri:dev
   ```

2. **Tracing**: The app uses `tracing` for structured logging

### Common Issues

**Issue**: `cargo: command not found`
- **Solution**: Add `~/.cargo/bin` to PATH
  ```bash
  export PATH="$HOME/.cargo/bin:$PATH"
  ```

**Issue**: `WebView2 not found` (Windows)
- **Solution**: Install [WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)

**Issue**: Port 5173 already in use
- **Solution**: Kill process on port 5173 or change port in `vite.config.ts`

## Next Steps

See [NEXT_STEPS.md](./NEXT_STEPS.md) for implementation roadmap.

## References

- [Tauri Documentation](https://tauri.app/)
- [Vite Documentation](https://vitejs.dev/)
- [React Documentation](https://react.dev/)
- [Monaco Editor](https://microsoft.github.io/monaco-editor/)
