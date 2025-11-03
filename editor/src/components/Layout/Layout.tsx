/**
 * AIT42 Editor - Main Layout Component
 */

import React, { useState } from 'react';
import { Panel, PanelGroup, PanelResizeHandle } from 'react-resizable-panels';
import { Sidebar } from '../Sidebar/Sidebar';
import { EditorArea } from '../Editor/EditorArea';
import { Terminal } from '../Terminal/Terminal';
import { CommandPalette } from '../CommandPalette/CommandPalette';
import styles from './Layout.module.css';

/**
 * Main layout component with resizable panels
 *
 * Layout structure:
 * - Sidebar (file tree)
 * - Editor area (tabs + monaco)
 * - Terminal (bottom panel)
 * - Command palette (overlay)
 */
export const Layout: React.FC = () => {
  const [sidebarVisible, setSidebarVisible] = useState(true);
  const [terminalVisible, setTerminalVisible] = useState(true);
  const [commandPaletteOpen, setCommandPaletteOpen] = useState(false);

  // Handle keyboard shortcuts
  React.useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Cmd+P / Ctrl+P - Open command palette
      if ((e.metaKey || e.ctrlKey) && e.key === 'p') {
        e.preventDefault();
        setCommandPaletteOpen(true);
      }

      // Cmd+B / Ctrl+B - Toggle sidebar
      if ((e.metaKey || e.ctrlKey) && e.key === 'b') {
        e.preventDefault();
        setSidebarVisible(prev => !prev);
      }

      // Cmd+J / Ctrl+J - Toggle terminal
      if ((e.metaKey || e.ctrlKey) && e.key === 'j') {
        e.preventDefault();
        setTerminalVisible(prev => !prev);
      }

      // Cmd+S / Ctrl+S - Save file (handled in editor)
      if ((e.metaKey || e.ctrlKey) && e.key === 's') {
        e.preventDefault();
        // Save will be handled by editor component
      }

      // Escape - Close command palette
      if (e.key === 'Escape' && commandPaletteOpen) {
        setCommandPaletteOpen(false);
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [commandPaletteOpen]);

  return (
    <div className={styles.layout}>
      {/* Main content area */}
      <PanelGroup direction="horizontal">
        {/* Sidebar */}
        {sidebarVisible && (
          <>
            <Panel
              defaultSize={20}
              minSize={15}
              maxSize={40}
              className={styles.sidebarPanel}
            >
              <Sidebar />
            </Panel>
            <PanelResizeHandle className={styles.resizeHandle} />
          </>
        )}

        {/* Editor + Terminal */}
        <Panel minSize={30}>
          <PanelGroup direction="vertical">
            {/* Editor area */}
            <Panel minSize={20} defaultSize={terminalVisible ? 70 : 100}>
              <EditorArea />
            </Panel>

            {/* Terminal */}
            {terminalVisible && (
              <>
                <PanelResizeHandle className={styles.resizeHandle} />
                <Panel minSize={10} defaultSize={30}>
                  <Terminal />
                </Panel>
              </>
            )}
          </PanelGroup>
        </Panel>
      </PanelGroup>

      {/* Command Palette Overlay */}
      {commandPaletteOpen && (
        <CommandPalette onClose={() => setCommandPaletteOpen(false)} />
      )}
    </div>
  );
};
