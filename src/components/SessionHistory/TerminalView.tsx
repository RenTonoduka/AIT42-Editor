/**
 * TerminalView Component
 *
 * Real-time terminal interface for tmux sessions
 * Displays output and allows command execution
 */
import React, { useEffect, useRef, useState, useCallback } from 'react';
import { Terminal as XTerm } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import '@xterm/xterm/css/xterm.css';
import { tauriApi } from '@/services/tauri';
import { AlertCircle, AlertTriangle } from 'lucide-react';

interface TerminalViewProps {
  /**
   * Tmux session ID to connect to
   */
  tmuxSessionId: string;

  /**
   * Instance name for display
   */
  instanceName?: string;

  /**
   * Polling interval in milliseconds (default: 1000ms)
   */
  pollingInterval?: number;
}

export const TerminalView: React.FC<TerminalViewProps> = ({
  tmuxSessionId,
  instanceName = 'Terminal',
  pollingInterval = 1000,
}) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const xtermRef = useRef<XTerm | null>(null);
  const fitAddonRef = useRef<FitAddon | null>(null);
  const [isInitialized, setIsInitialized] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [sessionAlive, setSessionAlive] = useState(true);
  const lastOutputRef = useRef<string>('');
  const pollingIntervalRef = useRef<NodeJS.Timeout | null>(null);
  const currentInputRef = useRef<string>('');
  const consecutiveErrorsRef = useRef<number>(0);
  const sessionAliveRef = useRef<boolean>(true);

  /**
   * Sync sessionAlive state with ref for event handlers
   */
  useEffect(() => {
    sessionAliveRef.current = sessionAlive;
  }, [sessionAlive]);

  /**
   * Initialize xterm.js terminal
   */
  useEffect(() => {
    if (!containerRef.current) return;

    // Initialize xterm.js
    const xterm = new XTerm({
      cursorBlink: true,
      cursorStyle: 'block',
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
      rows: 30,
      cols: 100,
      scrollback: 10000,
    });

    // Initialize fit addon
    const fitAddon = new FitAddon();
    xterm.loadAddon(fitAddon);

    // Open terminal in container
    xterm.open(containerRef.current);
    fitAddon.fit();

    xtermRef.current = xterm;
    fitAddonRef.current = fitAddon;

    // Display welcome message
    xterm.writeln('\x1b[1;32m━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\x1b[0m');
    xterm.writeln(`\x1b[1;36m  Connected to: ${instanceName}\x1b[0m`);
    xterm.writeln(`\x1b[90m  Tmux Session: ${tmuxSessionId}\x1b[0m`);
    xterm.writeln('\x1b[1;32m━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\x1b[0m');
    xterm.writeln('');

    // Handle terminal input
    xterm.onData((data) => {
      // Disable input if session is not alive
      if (!sessionAliveRef.current) {
        return;
      }

      const code = data.charCodeAt(0);

      if (code === 13) {
        // Enter key - send command
        xterm.write('\r\n');
        const command = currentInputRef.current;

        if (command.trim()) {
          sendCommand(command);
        }

        currentInputRef.current = '';
        displayPrompt(xterm);
      } else if (code === 127) {
        // Backspace
        if (currentInputRef.current.length > 0) {
          currentInputRef.current = currentInputRef.current.slice(0, -1);
          xterm.write('\b \b');
        }
      } else if (code === 3) {
        // Ctrl+C
        xterm.write('^C\r\n');
        currentInputRef.current = '';
        displayPrompt(xterm);
      } else if (code >= 32 && code < 127) {
        // Printable character
        currentInputRef.current += data;
        xterm.write(data);
      }
    });

    // Handle window resize
    const handleResize = () => {
      fitAddon.fit();
    };

    window.addEventListener('resize', handleResize);

    // Mark as initialized
    setIsInitialized(true);

    // Display initial prompt
    displayPrompt(xterm);

    // Cleanup
    return () => {
      window.removeEventListener('resize', handleResize);
      if (pollingIntervalRef.current) {
        clearInterval(pollingIntervalRef.current);
      }
      xterm.dispose();
    };
  }, [tmuxSessionId, instanceName]);

  /**
   * Display command prompt
   */
  const displayPrompt = (xterm: XTerm) => {
    xterm.write('\x1b[1;32m❯\x1b[0m ');
  };

  /**
   * Check if error indicates session termination
   */
  const isSessionTerminationError = (error: any): boolean => {
    const errorMessage = error?.message?.toLowerCase() || String(error).toLowerCase();
    return (
      errorMessage.includes('session not found') ||
      errorMessage.includes('no session') ||
      errorMessage.includes('can\'t find session') ||
      errorMessage.includes('session has been deleted') ||
      errorMessage.includes('no such session')
    );
  };

  /**
   * Send command to tmux session
   */
  const sendCommand = async (command: string) => {
    try {
      await tauriApi.sendTmuxKeys(tmuxSessionId, command);

      // Wait a moment for command to execute
      await new Promise((resolve) => setTimeout(resolve, 100));

      // Force immediate output capture after command
      await captureOutput();

      // Reset error counter on successful command
      consecutiveErrorsRef.current = 0;
    } catch (err) {
      console.error('Failed to send command to tmux:', err);

      // Check if session has terminated
      if (isSessionTerminationError(err)) {
        setSessionAlive(false);
        setError('Tmux session has ended (exit command detected or session killed)');

        if (xtermRef.current) {
          xtermRef.current.writeln('');
          xtermRef.current.writeln('\x1b[1;33m━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\x1b[0m');
          xtermRef.current.writeln('\x1b[1;31m  ⚠️  Session Terminated\x1b[0m');
          xtermRef.current.writeln('\x1b[90m  The tmux session is no longer active.\x1b[0m');
          xtermRef.current.writeln('\x1b[1;33m━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\x1b[0m');
          xtermRef.current.writeln('');
        }
      } else {
        setError(err instanceof Error ? err.message : 'Failed to send command');

        if (xtermRef.current) {
          xtermRef.current.writeln(`\x1b[1;31mError: ${err}\x1b[0m`);
          displayPrompt(xtermRef.current);
        }
      }
    }
  };

  /**
   * Capture tmux output and display in terminal
   */
  const captureOutput = useCallback(async () => {
    if (!xtermRef.current || !isInitialized || !sessionAlive) return;

    try {
      const output = await tauriApi.captureTmuxOutput(tmuxSessionId);

      // Reset consecutive errors on success
      consecutiveErrorsRef.current = 0;

      if (output && output !== lastOutputRef.current) {
        // Calculate new content (diff)
        const newContent = output.startsWith(lastOutputRef.current)
          ? output.slice(lastOutputRef.current.length)
          : output;

        if (newContent.trim()) {
          // Clear current line (remove prompt)
          xtermRef.current.write('\r\x1b[K');

          // Write new output
          const lines = newContent.split('\n');
          for (let i = 0; i < lines.length; i++) {
            if (i > 0) {
              xtermRef.current.write('\r\n');
            }
            xtermRef.current.write(lines[i]);
          }

          // Move to new line and show prompt
          xtermRef.current.write('\r\n');
          displayPrompt(xtermRef.current);

          // Re-display current input if any
          if (currentInputRef.current) {
            xtermRef.current.write(currentInputRef.current);
          }
        }

        lastOutputRef.current = output;
      }
    } catch (err) {
      console.error('Failed to capture tmux output:', err);

      // Increment consecutive error counter
      consecutiveErrorsRef.current += 1;

      // Check if session has terminated (after 3 consecutive failures)
      if (consecutiveErrorsRef.current >= 3 && isSessionTerminationError(err)) {
        setSessionAlive(false);
        setError('Tmux session has ended (exit command detected or session killed)');

        if (xtermRef.current) {
          xtermRef.current.writeln('');
          xtermRef.current.writeln('\x1b[1;33m━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\x1b[0m');
          xtermRef.current.writeln('\x1b[1;31m  ⚠️  Session Terminated\x1b[0m');
          xtermRef.current.writeln('\x1b[90m  The tmux session is no longer active.\x1b[0m');
          xtermRef.current.writeln('\x1b[1;33m━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\x1b[0m');
          xtermRef.current.writeln('');
        }
      }
    }
  }, [tmuxSessionId, isInitialized, sessionAlive]);

  /**
   * Load initial tmux history
   */
  useEffect(() => {
    const loadInitialHistory = async () => {
      if (!xtermRef.current || !isInitialized) return;

      try {
        const output = await tauriApi.captureTmuxOutput(tmuxSessionId);

        if (output) {
          // Clear terminal
          xtermRef.current.clear();

          // Display welcome message again
          xtermRef.current.writeln('\x1b[1;32m━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\x1b[0m');
          xtermRef.current.writeln(`\x1b[1;36m  Connected to: ${instanceName}\x1b[0m`);
          xtermRef.current.writeln(`\x1b[90m  Tmux Session: ${tmuxSessionId}\x1b[0m`);
          xtermRef.current.writeln('\x1b[1;32m━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\x1b[0m');
          xtermRef.current.writeln('');

          // Display history
          xtermRef.current.writeln('\x1b[90m--- Session History ---\x1b[0m');
          const lines = output.split('\n');
          for (const line of lines) {
            xtermRef.current.writeln(line);
          }
          xtermRef.current.writeln('\x1b[90m--- End of History ---\x1b[0m');
          xtermRef.current.writeln('');

          lastOutputRef.current = output;
          displayPrompt(xtermRef.current);
        }
      } catch (err) {
        console.error('Failed to load tmux history:', err);
        setError(err instanceof Error ? err.message : 'Failed to load history');
      }
    };

    if (isInitialized) {
      loadInitialHistory();
    }
  }, [isInitialized, tmuxSessionId, instanceName]);

  /**
   * Start polling for tmux output
   */
  useEffect(() => {
    if (!isInitialized) return;

    // Start polling
    pollingIntervalRef.current = setInterval(() => {
      captureOutput();
    }, pollingInterval);

    return () => {
      if (pollingIntervalRef.current) {
        clearInterval(pollingIntervalRef.current);
      }
    };
  }, [isInitialized, pollingInterval, captureOutput]);

  return (
    <div className="flex flex-col h-full bg-[#1E1E1E]">
      {/* Header */}
      <div className="flex-shrink-0 bg-[#252526] border-b border-[#3E3E42] px-4 py-2">
        <div className="flex items-center gap-3">
          <div className="flex items-center gap-2">
            <div
              className={`w-2 h-2 rounded-full ${
                sessionAlive
                  ? 'bg-green-500 animate-pulse'
                  : 'bg-red-500'
              }`}
            />
            <span className="text-sm font-semibold text-gray-300">
              Terminal: {instanceName}
            </span>
          </div>
          <span className="text-xs text-gray-500">
            Session: {tmuxSessionId}
          </span>
          {!sessionAlive && (
            <span className="text-xs font-medium text-red-400 bg-red-900/30 px-2 py-1 rounded">
              Session Ended
            </span>
          )}
        </div>
      </div>

      {/* Session ended warning */}
      {!sessionAlive && (
        <div className="flex-shrink-0 bg-yellow-900/40 border-b border-yellow-700 px-4 py-3">
          <div className="flex items-start gap-3">
            <AlertTriangle className="w-5 h-5 text-yellow-400 flex-shrink-0 mt-0.5" />
            <div className="flex-1">
              <div className="flex items-center gap-2 mb-1">
                <span className="font-semibold text-yellow-300 text-sm">
                  Tmux session has ended
                </span>
              </div>
              <p className="text-xs text-yellow-200 leading-relaxed">
                The terminal session is no longer active. This typically happens when an 'exit' command is executed
                or the session is killed externally. You can close this window or the agent may restart automatically.
              </p>
            </div>
          </div>
        </div>
      )}

      {/* Error display */}
      {error && sessionAlive && (
        <div className="flex-shrink-0 flex items-center gap-2 px-4 py-2 bg-red-900/30 border-b border-red-800">
          <AlertCircle className="w-4 h-4 text-red-400" />
          <span className="text-sm text-red-300">{error}</span>
        </div>
      )}

      {/* Terminal container */}
      <div className="flex-1 overflow-hidden p-2">
        <div
          ref={containerRef}
          className="w-full h-full"
          style={{ minHeight: 0 }}
        />
      </div>

      {/* Status bar */}
      <div className="flex-shrink-0 bg-[#252526] border-t border-[#3E3E42] px-4 py-1">
        <div className="flex items-center justify-between text-xs text-gray-500">
          <span>Press Enter to execute commands</span>
          <span>Polling: {pollingInterval}ms</span>
        </div>
      </div>
    </div>
  );
};
