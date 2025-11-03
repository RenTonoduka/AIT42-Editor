/**
 * AIT42 Editor - Terminal Component
 */

import React, { useEffect, useRef } from 'react';
import { Terminal as XTerm } from 'xterm';
import { FitAddon } from 'xterm-addon-fit';
import { WebLinksAddon } from 'xterm-addon-web-links';
import 'xterm/css/xterm.css';
import styles from './Terminal.module.css';

/**
 * Terminal component using xterm.js
 *
 * Features:
 * - Shell integration via Tauri
 * - Resizable terminal
 * - Copy/paste support
 * - Link detection
 */
export const Terminal: React.FC = () => {
  const terminalRef = useRef<HTMLDivElement>(null);
  const xtermRef = useRef<XTerm | null>(null);
  const fitAddonRef = useRef<FitAddon | null>(null);

  useEffect(() => {
    if (!terminalRef.current) return;

    // Create xterm instance with Cursor dark theme
    const xterm = new XTerm({
      cursorBlink: true,
      cursorStyle: 'block',
      fontFamily: "'Fira Code', 'Monaco', 'Menlo', 'Consolas', monospace",
      fontSize: 14,
      lineHeight: 1.2,
      theme: {
        background: '#1E1E1E',
        foreground: '#CCCCCC',
        cursor: '#AEAFAD',
        cursorAccent: '#1E1E1E',
        selectionBackground: '#264F78',
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
    });

    // Add fit addon for responsive sizing
    const fitAddon = new FitAddon();
    xterm.loadAddon(fitAddon);

    // Add web links addon
    const webLinksAddon = new WebLinksAddon();
    xterm.loadAddon(webLinksAddon);

    // Open terminal
    xterm.open(terminalRef.current);
    fitAddon.fit();

    // Store references
    xtermRef.current = xterm;
    fitAddonRef.current = fitAddon;

    // Welcome message
    xterm.writeln('\x1b[1;36mAIT42 Terminal\x1b[0m');
    xterm.writeln('');
    xterm.writeln('Type your commands here...');
    xterm.writeln('');
    xterm.write('$ ');

    // Handle terminal input
    xterm.onData((data) => {
      // Echo input (in real implementation, send to Tauri backend)
      if (data === '\r') {
        xterm.write('\r\n$ ');
      } else if (data === '\u007F') {
        // Backspace
        xterm.write('\b \b');
      } else {
        xterm.write(data);
      }
    });

    // Handle resize
    const handleResize = () => {
      fitAddon.fit();
    };

    window.addEventListener('resize', handleResize);

    // Cleanup
    return () => {
      window.removeEventListener('resize', handleResize);
      xterm.dispose();
    };
  }, []);

  // Handle panel resize
  useEffect(() => {
    const resizeObserver = new ResizeObserver(() => {
      if (fitAddonRef.current) {
        fitAddonRef.current.fit();
      }
    });

    if (terminalRef.current) {
      resizeObserver.observe(terminalRef.current);
    }

    return () => {
      resizeObserver.disconnect();
    };
  }, []);

  return (
    <div className={styles.terminal}>
      {/* Terminal Header */}
      <div className={styles.header}>
        <span className={styles.title}>Terminal</span>
        <div className={styles.actions}>
          <button
            className={styles.iconButton}
            title="New Terminal"
            aria-label="New Terminal"
          >
            <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
              <path d="M13 16h-1v-3H9v-1h3V9h1v3h3v1h-3v3zM3 3v10h6v1H2.5l-.5-.5v-11l.5-.5h9.793l.353.146L15.854 5.5l.146.354V8h-1V6h-4.5l-.5-.5V1H3zm7 0v3h3L10 3z" />
            </svg>
          </button>
          <button
            className={styles.iconButton}
            title="Split Terminal"
            aria-label="Split Terminal"
          >
            <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
              <path d="M1 1v14h14V1H1zm1 1h5v12H2V2zm6 0h5v12H8V2z" />
            </svg>
          </button>
          <button
            className={styles.iconButton}
            title="Clear Terminal"
            aria-label="Clear Terminal"
          >
            <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
              <path d="M4.5 1l-.5.5v4l.5.5h4l.5-.5v-4L8.5 1h-4zM2 7v1h5V7H2zm0 2v1h5V9H2zm0 2v1h5v-1H2zm0 2v1h5v-1H2z" />
            </svg>
          </button>
        </div>
      </div>

      {/* Terminal Content */}
      <div ref={terminalRef} className={styles.content} />
    </div>
  );
};
