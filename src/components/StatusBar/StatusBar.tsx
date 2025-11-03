/**
 * StatusBar Component - Display editor status information
 *
 * Shows:
 * - File status (saved/unsaved)
 * - Cursor position (line, column)
 * - File encoding
 * - Line ending type
 * - Language mode
 */

import React from 'react';
import { useEditorStore } from '@/store/editorStore';
import { getFileIcon } from '@/utils/monaco';

/**
 * StatusBar component
 */
export const StatusBar: React.FC = () => {
  const { getActiveTab } = useEditorStore();
  const activeTab = getActiveTab();

  return (
    <div className="h-8 bg-[#007ACC] flex items-center justify-between px-4 text-sm text-white">
      {/* Left side - File info */}
      <div className="flex items-center space-x-4">
        {activeTab ? (
          <>
            <span className="flex items-center gap-1">
              <span>{getFileIcon(activeTab.language)}</span>
              <span>{activeTab.name}</span>
            </span>
            {activeTab.isDirty && (
              <span className="text-xs">● Modified</span>
            )}
          </>
        ) : (
          <span>Ready</span>
        )}
      </div>

      {/* Right side - Editor settings */}
      <div className="flex items-center space-x-4">
        <span>UTF-8</span>
        <span>LF</span>
        {activeTab && (
          <span className="capitalize">{activeTab.language}</span>
        )}
      </div>
    </div>
  );
};
