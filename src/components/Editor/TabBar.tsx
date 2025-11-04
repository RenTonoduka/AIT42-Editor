/**
 * TabBar Component - Display and manage editor tabs
 *
 * Features:
 * - Active tab highlighting
 * - Close button with hover effect
 * - Dirty indicator (unsaved changes)
 * - Horizontal scrolling for overflow
 * - Keyboard shortcuts (Cmd+1-9)
 */

import React, { useEffect, useRef } from 'react';
import { X } from 'lucide-react';
import { useEditorStore } from '@/store/editorStore';
import type { EditorTab } from '@/store/editorStore';

interface TabItemProps {
  tab: EditorTab;
  onActivate: (id: string) => void;
  onClose: (id: string) => void;
}

/**
 * Individual tab component
 */
const TabItem: React.FC<TabItemProps> = ({ tab, onActivate, onClose }) => {
  const handleClose = (e: React.MouseEvent) => {
    e.stopPropagation();
    onClose(tab.id);
  };

  return (
    <div
      onClick={() => onActivate(tab.id)}
      className={`
        group relative flex items-center gap-2.5 px-4 py-2.5 min-w-[140px] max-w-[220px]
        cursor-pointer select-none transition-all duration-200
        ${
          tab.isActive
            ? 'bg-editor-elevated/80 text-text-primary shadow-glow-sm'
            : 'bg-editor-surface/20 text-text-secondary hover:bg-editor-hover/50 hover:text-text-primary'
        }
      `}
      role="tab"
      aria-selected={tab.isActive}
      tabIndex={tab.isActive ? 0 : -1}
    >
      {/* Active indicator line */}
      {tab.isActive && (
        <div className="absolute top-0 left-0 right-0 h-0.5 bg-gradient-to-r from-accent-primary via-accent-secondary to-accent-primary animate-fade-in" />
      )}

      {/* Dirty indicator (unsaved changes) - Modern pulsing dot */}
      {tab.isDirty && (
        <div className="relative">
          <div
            className="w-2 h-2 rounded-full bg-accent-primary animate-glow"
            title="Unsaved changes"
          />
          <div className="absolute inset-0 w-2 h-2 rounded-full bg-accent-primary blur-sm opacity-50" />
        </div>
      )}

      {/* File name with font styling */}
      <span className="flex-1 truncate text-sm font-medium">{tab.name}</span>

      {/* Close button - Elegant hover effect */}
      <button
        onClick={handleClose}
        className={`
          p-1 rounded-lg transition-all duration-200
          ${tab.isActive ? 'opacity-60 hover:opacity-100' : 'opacity-0 group-hover:opacity-60 hover:opacity-100'}
          hover:bg-editor-hover/80 hover:text-accent-danger
        `}
        aria-label={`Close ${tab.name}`}
        title="Close"
      >
        <X size={14} className="transition-transform hover:scale-110" />
      </button>

      {/* Subtle gradient overlay on active tab */}
      {tab.isActive && (
        <div className="absolute inset-0 bg-gradient-to-b from-accent-primary/5 to-transparent pointer-events-none" />
      )}
    </div>
  );
};

/**
 * TabBar component - displays all open tabs
 */
export const TabBar: React.FC = () => {
  const { tabs, setActiveTab, closeTab } = useEditorStore();
  const scrollContainerRef = useRef<HTMLDivElement>(null);

  // Keyboard shortcuts (Cmd/Ctrl + 1-9)
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      const isCmdOrCtrl = e.metaKey || e.ctrlKey;

      if (isCmdOrCtrl && e.key >= '1' && e.key <= '9') {
        e.preventDefault();
        const index = parseInt(e.key, 10) - 1;

        if (tabs[index]) {
          setActiveTab(tabs[index].id);
        }
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [tabs, setActiveTab]);

  // Auto-scroll to active tab
  useEffect(() => {
    const activeTab = tabs.find((t) => t.isActive);
    if (!activeTab || !scrollContainerRef.current) return;

    const container = scrollContainerRef.current;
    const activeElement = container.querySelector('[aria-selected="true"]');

    if (activeElement) {
      activeElement.scrollIntoView({
        behavior: 'smooth',
        block: 'nearest',
        inline: 'nearest',
      });
    }
  }, [tabs]);

  if (tabs.length === 0) {
    return (
      <div className="h-11 bg-editor-surface/50 backdrop-blur-sm border-b border-editor-border/30 flex items-center px-5">
        <div className="text-sm text-text-tertiary font-medium">No files open</div>
      </div>
    );
  }

  return (
    <div className="h-11 bg-editor-surface/80 border-b border-editor-border/30 shadow-sm" style={{ willChange: 'transform' }}>
      <div
        ref={scrollContainerRef}
        className="flex overflow-x-auto scrollbar-thin scrollbar-thumb-editor-border/50 scrollbar-track-transparent hover:scrollbar-thumb-editor-border"
        role="tablist"
        aria-label="Editor tabs"
      >
        {tabs.map((tab) => (
          <TabItem
            key={tab.id}
            tab={tab}
            onActivate={setActiveTab}
            onClose={closeTab}
          />
        ))}
      </div>
    </div>
  );
};
