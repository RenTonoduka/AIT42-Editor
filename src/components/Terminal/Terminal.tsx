/**
 * Terminal Component
 *
 * Integrated terminal using xterm.js and Tauri backend
 */
import { useEffect, useRef, useState } from 'react';
import { Terminal as XTerm } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import '@xterm/xterm/css/xterm.css';
import { tauriApi } from '../../services/tauri';

export interface TerminalProps {
  /**
   * Initial working directory
   */
  initialDir?: string;

  /**
   * Terminal height (default: 200px)
   */
  height?: number;

  /**
   * Callback when terminal is ready
   */
  onReady?: (terminal: XTerm) => void;
}

export const Terminal: React.FC<TerminalProps> = ({
  initialDir,
  height = 200,
  onReady
}) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const xtermRef = useRef<XTerm | null>(null);
  const fitAddonRef = useRef<FitAddon | null>(null);
  const [currentDir, setCurrentDir] = useState<string>('');
  const [commandHistory, setCommandHistory] = useState<string[]>([]);

  useEffect(() => {
    if (!containerRef.current) return;

    // Initialize xterm.js
    const xterm = new XTerm({
      cursorBlink: true,
      cursorStyle: 'bar',
      fontSize: 14,
      fontFamily: '"Fira Code", "Monaco", "Menlo", monospace',
      theme: {
        background: '#1E1E1E',
        foreground: '#D4D4D4',
        cursor: '#AEAFAD',
        black: '#000000',
        red: '#CD3131',
        green: '#0DBC79',
        yellow: '#E5E510',
        blue: '#2472C8',
        magenta: '#BC3FBC',
        cyan: '#11A8CD',
        white: '#E5E5E5',
        brightBlack: '#666666',
        brightRed: '#F14C4C',
        brightGreen: '#23D18B',
        brightYellow: '#F5F543',
        brightBlue: '#3B8EEA',
        brightMagenta: '#D670D6',
        brightCyan: '#29B8DB',
        brightWhite: '#E5E5E5',
      },
      rows: 20,
      cols: 80,
      scrollback: 1000,
    });

    // Initialize fit addon
    const fitAddon = new FitAddon();
    xterm.loadAddon(fitAddon);

    // Open terminal in container
    xterm.open(containerRef.current);
    fitAddon.fit();

    xtermRef.current = xterm;
    fitAddonRef.current = fitAddon;

    // Initialize terminal
    const initTerminal = async () => {
      let dir = process.cwd?.() || '/';

      try {
        // Display welcome message first
        xterm.writeln('\x1b[1;32mAIT42 Editor Terminal\x1b[0m');
        xterm.writeln('\x1b[90mType commands and press Enter to execute\x1b[0m');
        xterm.writeln('');

        // Set initial directory if provided
        if (initialDir) {
          await tauriApi.setCurrentDirectory(initialDir);
        }

        // Get current directory
        dir = await tauriApi.getCurrentDirectory();
        setCurrentDir(dir);

        // Load command history
        const history = await tauriApi.getCommandHistory();
        setCommandHistory(history);

        // Notify parent that terminal is ready
        if (onReady) {
          onReady(xterm);
        }
      } catch (error) {
        console.error('Terminal initialization error:', error);
        xterm.writeln(`\x1b[1;31mWarning: ${error}\x1b[0m`);
        xterm.writeln('\x1b[90mTerminal started in fallback mode\x1b[0m');
        xterm.writeln('');
      } finally {
        // Always display prompt, even if there was an error
        displayPrompt(xterm, dir);
      }
    };

    initTerminal();

    // Handle terminal input
    let currentInput = '';
    let currentHistoryIndex = -1;

    xterm.onData((data) => {
      const code = data.charCodeAt(0);

      // Handle special keys
      if (code === 13) {
        // Enter key - execute command
        xterm.write('\r\n');

        if (currentInput.trim()) {
          executeCommand(currentInput.trim(), xterm)
            .then(() => {
              currentInput = '';
              currentHistoryIndex = -1;
            })
            .catch((error) => {
              xterm.writeln(`\x1b[1;31mError: ${error}\x1b[0m`);
              displayPrompt(xterm, currentDir);
            });
        } else {
          displayPrompt(xterm, currentDir);
        }
      } else if (code === 127) {
        // Backspace
        if (currentInput.length > 0) {
          currentInput = currentInput.slice(0, -1);
          xterm.write('\b \b');
        }
      } else if (code === 27) {
        // Escape sequences (arrow keys, etc.)
        // Handle arrow up/down for history navigation
        if (data === '\x1b[A') {
          // Arrow up - previous command
          if (commandHistory.length > 0 && currentHistoryIndex < commandHistory.length - 1) {
            currentHistoryIndex++;
            const historicalCommand = commandHistory[currentHistoryIndex];

            // Clear current line
            xterm.write('\r\x1b[K');
            displayPrompt(xterm, currentDir);
            xterm.write(historicalCommand);

            currentInput = historicalCommand;
          }
        } else if (data === '\x1b[B') {
          // Arrow down - next command
          if (currentHistoryIndex > 0) {
            currentHistoryIndex--;
            const historicalCommand = commandHistory[currentHistoryIndex];

            // Clear current line
            xterm.write('\r\x1b[K');
            displayPrompt(xterm, currentDir);
            xterm.write(historicalCommand);

            currentInput = historicalCommand;
          } else if (currentHistoryIndex === 0) {
            currentHistoryIndex = -1;

            // Clear current line
            xterm.write('\r\x1b[K');
            displayPrompt(xterm, currentDir);

            currentInput = '';
          }
        }
      } else if (code >= 32 && code < 127) {
        // Printable character
        currentInput += data;
        xterm.write(data);
      }
    });

    // Handle window resize
    const handleResize = () => {
      fitAddon.fit();
    };

    window.addEventListener('resize', handleResize);

    // Cleanup
    return () => {
      window.removeEventListener('resize', handleResize);
      xterm.dispose();
    };
  }, [initialDir, onReady]);

  /**
   * Display command prompt with full path information
   */
  const displayPrompt = (xterm: XTerm, dir: string) => {
    // Get home directory and replace with ~
    const homeDir = process.env.HOME || '/Users';
    const displayPath = dir.startsWith(homeDir)
      ? `~${dir.slice(homeDir.length)}`
      : dir;

    // Show full path with color coding
    xterm.write(`\x1b[1;34m${displayPath}\x1b[0m \x1b[1;32m❯\x1b[0m `);
  };

  /**
   * Execute a command
   */
  const executeCommand = async (command: string, xterm: XTerm) => {
    let newDir = currentDir;
    const oldDir = currentDir;

    try {
      // Execute command via Tauri backend
      const output = await tauriApi.executeCommand(command);

      // Display output
      if (output) {
        const lines = output.split('\n');
        for (const line of lines) {
          xterm.writeln(line);
        }
      }

      // Update current directory (may have changed with cd command)
      newDir = await tauriApi.getCurrentDirectory();

      // If directory changed, show a message
      if (newDir !== oldDir) {
        const homeDir = process.env.HOME || '/Users';
        const displayPath = newDir.startsWith(homeDir)
          ? `~${newDir.slice(homeDir.length)}`
          : newDir;
        xterm.writeln(`\x1b[90mChanged directory to: \x1b[34m${displayPath}\x1b[0m`);
      }

      setCurrentDir(newDir);

      // Update command history
      const history = await tauriApi.getCommandHistory();
      setCommandHistory(history);
    } catch (error) {
      console.error('Command execution error:', error);
      xterm.writeln(`\x1b[1;31mError: ${error}\x1b[0m`);
    } finally {
      // Always display prompt, even if there was an error
      displayPrompt(xterm, newDir);
    }
  };

  // Format current directory for display
  const homeDir = typeof process !== 'undefined' && process.env?.HOME || '/Users';
  const displayCurrentDir = currentDir
    ? currentDir.startsWith(homeDir)
      ? `~${currentDir.slice(homeDir.length)}` || '~'
      : currentDir
    : '~';

  return (
    <div
      className="terminal-container"
      style={{
        width: '100%',
        height: `${height}px`,
        backgroundColor: '#1E1E1E',
        display: 'flex',
        flexDirection: 'column',
        overflow: 'hidden',
      }}
    >
      {/* Directory indicator header */}
      <div
        style={{
          backgroundColor: '#252526',
          padding: '6px 12px',
          borderBottom: '1px solid #3E3E42',
          fontSize: '12px',
          fontFamily: '"Fira Code", "Monaco", "Menlo", monospace',
          color: '#9CDCFE',
          display: 'flex',
          alignItems: 'center',
          gap: '8px',
          flexShrink: 0,
        }}
      >
        <span style={{ color: '#858585' }}>📁</span>
        <span style={{ fontWeight: 600 }}>{displayCurrentDir}</span>
      </div>

      {/* Terminal content */}
      <div
        style={{
          flex: 1,
          padding: '8px',
          overflow: 'hidden',
        }}
      >
        <div
          ref={containerRef}
          style={{
            width: '100%',
            height: '100%'
          }}
        />
      </div>
    </div>
  );
};
