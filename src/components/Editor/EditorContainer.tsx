/**
 * EditorContainer - Main editor container with tabs
 *
 * Manages the relationship between TabBar and EditorPane,
 * handling tab switching and content updates.
 */

import React, { useRef, useCallback } from 'react';
import { FileText } from 'lucide-react';
import { TabBar } from './TabBar';
import { EditorPane } from './EditorPane';
import { Terminal } from '@/components/Terminal';
import { DiagnosticsPanel } from '@/components/Diagnostics';
import { GitPanel } from '@/components/Git';
import { useEditorStore } from '@/store/editorStore';
import { useTerminalStore, MIN_TERMINAL_HEIGHT } from '@/store/terminalStore';
import { useLspStore } from '@/store/lspStore';
import { useGitStore } from '@/store/gitStore';
import { open } from '@tauri-apps/api/dialog';

interface EmptyStateProps {
  onFileOpen?: (path: string) => void;
}

/**
 * Empty state when no files are open
 */
const EmptyState: React.FC<EmptyStateProps> = ({ onFileOpen }) => {
  const handleOpenFile = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [
          {
            name: 'All Files',
            extensions: ['*']
          }
        ]
      });

      if (selected && typeof selected === 'string' && onFileOpen) {
        await onFileOpen(selected);
      }
    } catch (error) {
      console.error('Failed to open file:', error);
    }
  };

  const handleNewFile = () => {
    // Create a new untitled file
    const timestamp = Date.now();
    const newTab = {
      id: `untitled-${timestamp}`,
      path: `Untitled-${timestamp}`,
      name: 'Untitled',
      content: '',
      language: 'plaintext',
      isDirty: false,
      isActive: true,
    };

    useEditorStore.setState((state) => ({
      tabs: [...state.tabs.map(t => ({ ...t, isActive: false })), newTab],
      activeTabId: newTab.id,
    }));
  };
  return (
    <div className="flex items-center justify-center h-full bg-editor-bg text-text-primary animate-fade-in">
      <div className="text-center space-y-6 max-w-md px-8">
        {/* Icon with glow effect */}
        <div className="relative mb-8">
          <div className="absolute inset-0 bg-gradient-to-br from-accent-primary/30 to-accent-secondary/30 rounded-full blur-3xl animate-glow" />
          <FileText size={72} className="relative mx-auto text-text-tertiary drop-shadow-lg" />
        </div>

        {/* Title and description */}
        <div className="space-y-3">
          <h2 className="text-3xl font-bold bg-gradient-to-r from-text-primary to-text-secondary bg-clip-text text-transparent">
            No File Open
          </h2>
          <p className="text-text-secondary leading-relaxed">
            Start by opening an existing file or create a new one
          </p>
          <p className="text-xs text-text-tertiary font-mono">
            <kbd className="px-2 py-1 bg-editor-elevated/50 rounded border border-editor-border/30">⌘</kbd> +
            <kbd className="px-2 py-1 bg-editor-elevated/50 rounded border border-editor-border/30">O</kbd> to open file
          </p>
        </div>

        {/* Action buttons - Modern gradient design */}
        <div className="flex gap-3 justify-center mt-8">
          <button
            onClick={handleOpenFile}
            className="group relative px-6 py-3 bg-gradient-to-r from-accent-primary to-accent-secondary rounded-xl text-white font-semibold transition-all duration-300 hover:shadow-glow-lg hover:scale-105 overflow-hidden"
            title="Open file (Cmd+O)"
          >
            <span className="relative z-10 flex items-center gap-2">
              <FileText size={18} />
              Open File
            </span>
            <div className="absolute inset-0 bg-gradient-to-r from-accent-secondary to-accent-primary opacity-0 group-hover:opacity-100 transition-opacity duration-300" />
          </button>
          <button
            onClick={handleNewFile}
            className="group px-6 py-3 bg-editor-elevated/50 hover:bg-editor-elevated border border-editor-border/30 hover:border-accent-primary/50 text-text-primary rounded-xl font-semibold transition-all duration-300 hover:shadow-glow-sm"
            title="New file (Cmd+N)"
          >
            <span className="flex items-center gap-2">
              <FileText size={18} />
              New File
            </span>
          </button>
        </div>
      </div>
    </div>
  );
};

/**
 * Terminal resize handle component
 */
const ResizeHandle: React.FC<{
  onMouseDown: (e: React.MouseEvent) => void;
}> = ({ onMouseDown }) => {
  return (
    <div
      className="h-1 bg-[#2D2D30] hover:bg-[#007ACC] cursor-ns-resize transition-colors"
      onMouseDown={onMouseDown}
    />
  );
};

interface EditorContainerProps {
  onFileOpen?: (path: string) => void;
}

/**
 * EditorContainer - Main editor UI
 */
export const EditorContainer: React.FC<EditorContainerProps> = ({ onFileOpen }) => {
  const { tabs, activeTabId, updateTabContent, saveTab } = useEditorStore();
  const {
    isVisible: isTerminalVisible,
    height: terminalHeight,
    setHeight: setTerminalHeight,
    setResizing,
    setXTermInstance,
  } = useTerminalStore();
  const { showDiagnosticsPanel } = useLspStore();
  const { showGitPanel } = useGitStore();

  const containerRef = useRef<HTMLDivElement>(null);
  const isResizing = useRef(false);

  // Get active tab
  const activeTab = tabs.find((t) => t.id === activeTabId);

  /**
   * Handle content change in editor
   */
  const handleContentChange = (content: string) => {
    if (activeTab) {
      updateTabContent(activeTab.id, content);
    }
  };

  /**
   * Handle save action (Cmd+S)
   */
  const handleSave = async () => {
    if (activeTab) {
      try {
        await saveTab(activeTab.id);
        console.log(`Saved: ${activeTab.path}`);
      } catch (error) {
        console.error('Save failed:', error);
      }
    }
  };

  /**
   * Handle terminal resize start
   */
  const handleResizeStart = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    isResizing.current = true;
    setResizing(true);

    const handleMouseMove = (moveEvent: MouseEvent) => {
      if (!isResizing.current || !containerRef.current) return;

      const containerRect = containerRef.current.getBoundingClientRect();
      const containerBottom = containerRect.bottom;
      const mouseY = moveEvent.clientY;

      // Calculate new height (distance from mouse to container bottom)
      const newHeight = Math.max(
        MIN_TERMINAL_HEIGHT,
        containerBottom - mouseY
      );

      setTerminalHeight(newHeight);
    };

    const handleMouseUp = () => {
      isResizing.current = false;
      setResizing(false);
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };

    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
  }, [setTerminalHeight, setResizing]);

  // Calculate panel heights
  const diagnosticsPanelHeight = 200; // Fixed height for diagnostics panel
  const gitPanelHeight = 300; // Fixed height for git panel

  // Calculate editor height accounting for all visible panels
  let editorHeight = '100%';
  let totalBottomPanelHeight = 0;
  let separatorCount = 0;

  if (showDiagnosticsPanel) {
    totalBottomPanelHeight += diagnosticsPanelHeight;
    separatorCount += 1;
  }
  if (showGitPanel) {
    totalBottomPanelHeight += gitPanelHeight;
    separatorCount += 1;
  }
  if (isTerminalVisible) {
    totalBottomPanelHeight += terminalHeight;
    separatorCount += 1;
  }

  if (totalBottomPanelHeight > 0) {
    editorHeight = `calc(100% - ${totalBottomPanelHeight}px - ${separatorCount}px)`; // -1px per separator
  }

  return (
    <div ref={containerRef} className="flex flex-col h-full">
      {/* Tab bar */}
      <TabBar />

      {/* Editor pane */}
      <div className="overflow-hidden" style={{ height: editorHeight }}>
        {activeTab ? (
          <EditorPane
            key={activeTab.id}
            bufferId={activeTab.id}
            filePath={activeTab.path}
            content={activeTab.content}
            language={activeTab.language}
            onChange={handleContentChange}
            onSave={handleSave}
          />
        ) : (
          <EmptyState onFileOpen={onFileOpen} />
        )}
      </div>

      {/* Diagnostics Panel section */}
      {showDiagnosticsPanel && (
        <>
          <div className="h-1 bg-[#2D2D30]" />
          <div style={{ height: diagnosticsPanelHeight }}>
            <DiagnosticsPanel />
          </div>
        </>
      )}

      {/* Git Panel section */}
      {showGitPanel && (
        <>
          <div className="h-1 bg-[#2D2D30]" />
          <div style={{ height: gitPanelHeight }}>
            <GitPanel />
          </div>
        </>
      )}

      {/* Terminal section */}
      {isTerminalVisible && (
        <>
          <ResizeHandle onMouseDown={handleResizeStart} />
          <Terminal
            height={terminalHeight}
            onReady={(xterm) => {
              setXTermInstance(xterm);
            }}
          />
        </>
      )}
    </div>
  );
};
