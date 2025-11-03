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

import React, { useEffect } from 'react';
import { AlertCircle, AlertTriangle, GitBranch, Upload, Download } from 'lucide-react';
import { useEditorStore } from '@/store/editorStore';
import { useLspStore } from '@/store/lspStore';
import { useGitStore } from '@/store/gitStore';
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

  const {
    status: gitStatus,
    fetchStatus,
    toggleGitPanel,
    showGitPanel,
  } = useGitStore();

  // Fetch Git status on mount
  useEffect(() => {
    fetchStatus();
  }, [fetchStatus]);

  const errorCount = getTotalErrorCount();
  const warningCount = getTotalWarningCount();
  const hasProblems = errorCount > 0 || warningCount > 0;

  return (
    <div className="h-7 bg-editor-surface/60 backdrop-blur-md border-t border-editor-border/20 flex items-center justify-between px-4 text-xs font-medium shadow-sm">
      {/* Left side - File info with elegant badges */}
      <div className="flex items-center space-x-3">
        {activeTab ? (
          <>
            <div className="flex items-center gap-2 px-2.5 py-1 rounded-lg bg-editor-elevated/50 border border-editor-border/20">
              <span>{getFileIcon(activeTab.language)}</span>
              <span className="text-text-primary font-semibold">{activeTab.name}</span>
            </div>
            {activeTab.isDirty && (
              <div className="flex items-center gap-1.5 px-2 py-0.5 rounded-full bg-accent-primary/10 text-accent-primary border border-accent-primary/20">
                <div className="w-1.5 h-1.5 rounded-full bg-accent-primary animate-glow" />
                <span className="text-xs font-medium">Modified</span>
              </div>
            )}
          </>
        ) : (
          <span className="text-text-secondary">Ready</span>
        )}

        {/* Diagnostics button - Modern pill design */}
        {hasProblems && (
          <button
            className={`flex items-center gap-2 px-2.5 py-1 rounded-lg transition-all duration-200 ${
              showDiagnosticsPanel
                ? 'bg-accent-primary/20 text-accent-primary border border-accent-primary/30'
                : 'bg-editor-hover/30 text-text-secondary hover:bg-editor-hover/50 hover:text-text-primary border border-transparent'
            }`}
            onClick={toggleDiagnosticsPanel}
            title="Toggle Problems Panel"
          >
            {errorCount > 0 && (
              <div className="flex items-center gap-1">
                <AlertCircle size={13} className="text-accent-danger" />
                <span className="font-semibold">{errorCount}</span>
              </div>
            )}
            {warningCount > 0 && (
              <div className="flex items-center gap-1">
                <AlertTriangle size={13} className="text-accent-warning" />
                <span className="font-semibold">{warningCount}</span>
              </div>
            )}
          </button>
        )}

        {/* Git status button - Sleek design */}
        {gitStatus && (
          <button
            className={`flex items-center gap-2 px-2.5 py-1 rounded-lg transition-all duration-200 ${
              showGitPanel
                ? 'bg-accent-primary/20 text-accent-primary border border-accent-primary/30'
                : 'bg-editor-hover/30 text-text-secondary hover:bg-editor-hover/50 hover:text-text-primary border border-transparent'
            }`}
            onClick={toggleGitPanel}
            title="Toggle Source Control"
          >
            <GitBranch size={13} />
            <span className="font-semibold">{gitStatus.branch}</span>
            {gitStatus.files.length > 0 && (
              <span className="text-xs opacity-70">({gitStatus.files.length})</span>
            )}
            {gitStatus.ahead > 0 && (
              <div className="flex items-center gap-0.5">
                <Upload size={11} />
                <span>{gitStatus.ahead}</span>
              </div>
            )}
            {gitStatus.behind > 0 && (
              <div className="flex items-center gap-0.5">
                <Download size={11} />
                <span>{gitStatus.behind}</span>
              </div>
            )}
          </button>
        )}
      </div>

      {/* Right side - Editor settings with subtle badges */}
      <div className="flex items-center space-x-2">
        <div className="px-2 py-0.5 rounded bg-editor-elevated/30 text-text-tertiary">UTF-8</div>
        <div className="w-px h-3 bg-editor-border/30" />
        <div className="px-2 py-0.5 rounded bg-editor-elevated/30 text-text-tertiary">LF</div>
        {activeTab && (
          <>
            <div className="w-px h-3 bg-editor-border/30" />
            <div className="px-2 py-0.5 rounded bg-editor-elevated/30 text-text-tertiary capitalize">{activeTab.language}</div>
          </>
        )}
      </div>
    </div>
  );
};
