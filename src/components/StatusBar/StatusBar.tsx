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
import { AlertCircle, AlertTriangle } from 'lucide-react';
import { useEditorStore } from '@/store/editorStore';
import { useLspStore } from '@/store/lspStore';
import { getFileIcon } from '@/utils/monaco';

/**
 * StatusBar component
 */
export const StatusBar: React.FC = () => {
  const { getActiveTab } = useEditorStore();
  const activeTab = getActiveTab();

  const {
    getTotalErrorCount,
    getTotalWarningCount,
    toggleDiagnosticsPanel,
    showDiagnosticsPanel,
  } = useLspStore();

  const errorCount = getTotalErrorCount();
  const warningCount = getTotalWarningCount();
  const hasProblems = errorCount > 0 || warningCount > 0;

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

        {/* Diagnostics button */}
        {hasProblems && (
          <button
            className={`flex items-center gap-2 px-2 py-1 rounded hover:bg-white/20 transition-colors ${
              showDiagnosticsPanel ? 'bg-white/10' : ''
            }`}
            onClick={toggleDiagnosticsPanel}
            title="Toggle Problems Panel"
          >
            {errorCount > 0 && (
              <div className="flex items-center gap-1">
                <AlertCircle size={14} />
                <span className="text-xs">{errorCount}</span>
              </div>
            )}
            {warningCount > 0 && (
              <div className="flex items-center gap-1">
                <AlertTriangle size={14} />
                <span className="text-xs">{warningCount}</span>
              </div>
            )}
          </button>
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
