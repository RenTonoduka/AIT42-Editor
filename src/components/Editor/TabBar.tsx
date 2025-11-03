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
        group flex items-center gap-2 px-4 py-2 min-w-[120px] max-w-[200px]
        cursor-pointer select-none transition-colors border-r
        ${
          tab.isActive
            ? 'bg-[#2D2D2D] text-white border-[#3E3E42]'
            : 'bg-[#252525] text-[#858585] hover:bg-[#2A2A2A] border-[#3E3E42]'
        }
      `}
      role="tab"
      aria-selected={tab.isActive}
      tabIndex={tab.isActive ? 0 : -1}
    >
      {/* Dirty indicator (unsaved changes) */}
      {tab.isDirty && (
        <div
          className="w-2 h-2 rounded-full bg-[#007ACC]"
          title="Unsaved changes"
        />
      )}

      {/* File name */}
      <span className="flex-1 truncate text-sm">{tab.name}</span>

      {/* Close button */}
      <button
        onClick={handleClose}
        className={`
          p-0.5 rounded opacity-0 group-hover:opacity-100 transition-opacity
          hover:bg-[#3E3E42]
          ${tab.isActive ? 'opacity-100' : ''}
        `}
        aria-label={`Close ${tab.name}`}
        title="Close"
      >
        <X size={14} />
      </button>
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
      <div className="h-10 bg-[#252525] border-b border-[#3E3E42] flex items-center px-4">
        <div className="text-sm text-[#858585]">No files open</div>
      </div>
    );
  }

  return (
    <div className="h-10 bg-[#252525] border-b border-[#3E3E42]">
      <div
        ref={scrollContainerRef}
        className="flex overflow-x-auto scrollbar-thin scrollbar-thumb-[#79797966] scrollbar-track-transparent hover:scrollbar-thumb-[#646464B3]"
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
