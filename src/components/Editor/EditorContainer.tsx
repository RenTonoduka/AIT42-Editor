/**
 * EditorContainer - Main editor container with tabs
 *
 * Manages the relationship between TabBar and EditorPane,
 * handling tab switching and content updates.
 */

import React from 'react';
import { FileText } from 'lucide-react';
import { TabBar } from './TabBar';
import { EditorPane } from './EditorPane';
import { useEditorStore } from '@/store/editorStore';

/**
 * Empty state when no files are open
 */
const EmptyState: React.FC = () => {
  return (
    <div className="flex items-center justify-center h-full bg-[#1E1E1E] text-[#CCCCCC]">
      <div className="text-center space-y-4">
        <FileText size={64} className="mx-auto text-[#858585]" />
        <h2 className="text-2xl font-semibold text-[#CCCCCC]">No file open</h2>
        <p className="text-[#858585]">
          Open a file from the explorer or use Cmd+O
        </p>
        <div className="flex gap-4 justify-center mt-6">
          <button
            className="px-4 py-2 bg-[#007ACC] hover:bg-[#148EE0] text-white rounded transition-colors"
            title="Open file (Cmd+O)"
          >
            Open File
          </button>
          <button
            className="px-4 py-2 bg-[#3E3E42] hover:bg-[#505050] text-white rounded transition-colors"
            title="New file (Cmd+N)"
          >
            New File
          </button>
        </div>
      </div>
    </div>
  );
};

/**
 * EditorContainer - Main editor UI
 */
export const EditorContainer: React.FC = () => {
  const { tabs, activeTabId, updateTabContent, saveTab } = useEditorStore();

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

  return (
    <div className="flex flex-col h-full">
      {/* Tab bar */}
      <TabBar />

      {/* Editor pane */}
      <div className="flex-1 overflow-hidden">
        {activeTab ? (
          <EditorPane
            key={activeTab.id}
            bufferId={activeTab.id}
            content={activeTab.content}
            language={activeTab.language}
            onChange={handleContentChange}
            onSave={handleSave}
          />
        ) : (
          <EmptyState />
        )}
      </div>
    </div>
  );
};
