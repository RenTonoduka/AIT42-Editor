/**
 * Diagnostics Panel Component
 *
 * Displays LSP diagnostics (errors, warnings, info, hints) from all open files.
 * Features:
 * - Grouped by file
 * - Severity filtering (all, errors only, warnings only)
 * - Click to navigate to diagnostic location
 * - Color-coded by severity
 * - Expandable/collapsible panel
 */

import React, { useMemo, useState } from 'react';
import {
  AlertCircle,
  AlertTriangle,
  Info,
  Lightbulb,
  ChevronDown,
  ChevronRight,
  X,
} from 'lucide-react';
import { useLspStore } from '@/store/lspStore';
import { useEditorStore } from '@/store/editorStore';
import type { LspDiagnostic } from '@/services/tauri';

/**
 * Diagnostic severity filter options
 */
type SeverityFilter = 'all' | 'errors' | 'warnings';

/**
 * Get severity icon and color
 */
function getSeverityInfo(severity: number) {
  switch (severity) {
    case 1: // Error
      return {
        icon: AlertCircle,
        color: 'text-red-500',
        bgColor: 'bg-red-500/10',
        label: 'Error',
      };
    case 2: // Warning
      return {
        icon: AlertTriangle,
        color: 'text-yellow-500',
        bgColor: 'bg-yellow-500/10',
        label: 'Warning',
      };
    case 3: // Information
      return {
        icon: Info,
        color: 'text-blue-500',
        bgColor: 'bg-blue-500/10',
        label: 'Info',
      };
    case 4: // Hint
      return {
        icon: Lightbulb,
        color: 'text-gray-400',
        bgColor: 'bg-gray-500/10',
        label: 'Hint',
      };
    default:
      return {
        icon: Info,
        color: 'text-gray-400',
        bgColor: 'bg-gray-500/10',
        label: 'Unknown',
      };
  }
}

/**
 * Diagnostic item component
 */
interface DiagnosticItemProps {
  diagnostic: LspDiagnostic;
  filePath: string;
  onNavigate: (filePath: string, line: number, character: number) => void;
}

const DiagnosticItem: React.FC<DiagnosticItemProps> = ({
  diagnostic,
  filePath,
  onNavigate,
}) => {
  const { icon: Icon, color, bgColor } = getSeverityInfo(diagnostic.severity);

  return (
    <div
      className={`flex items-start gap-2 px-3 py-2 cursor-pointer hover:bg-[#2A2D2E] transition-colors ${bgColor}`}
      onClick={() =>
        onNavigate(filePath, diagnostic.startLine, diagnostic.startCharacter)
      }
    >
      <Icon size={16} className={`${color} mt-0.5 flex-shrink-0`} />
      <div className="flex-1 min-w-0">
        <div className="text-[13px] text-[#CCCCCC] break-words">
          {diagnostic.message}
        </div>
        <div className="text-[11px] text-[#858585] mt-1">
          {diagnostic.source && (
            <span className="mr-2">[{diagnostic.source}]</span>
          )}
          {diagnostic.code && <span className="mr-2">({diagnostic.code})</span>}
          <span>
            Ln {diagnostic.startLine + 1}, Col {diagnostic.startCharacter + 1}
          </span>
        </div>
      </div>
    </div>
  );
};

/**
 * File group component
 */
interface FileGroupProps {
  filePath: string;
  diagnostics: LspDiagnostic[];
  errorCount: number;
  warningCount: number;
  onNavigate: (filePath: string, line: number, character: number) => void;
}

const FileGroup: React.FC<FileGroupProps> = ({
  filePath,
  diagnostics,
  errorCount,
  warningCount,
  onNavigate,
}) => {
  const [isExpanded, setIsExpanded] = useState(true);

  // Get file name from path
  const fileName = filePath.split('/').pop() || filePath;
  const folderPath = filePath.substring(0, filePath.length - fileName.length);

  return (
    <div className="border-b border-[#2A2D2E] last:border-b-0">
      {/* File header */}
      <div
        className="flex items-center gap-2 px-3 py-2 cursor-pointer hover:bg-[#2A2D2E] transition-colors"
        onClick={() => setIsExpanded(!isExpanded)}
      >
        {isExpanded ? (
          <ChevronDown size={16} className="text-[#CCCCCC] flex-shrink-0" />
        ) : (
          <ChevronRight size={16} className="text-[#CCCCCC] flex-shrink-0" />
        )}
        <div className="flex-1 min-w-0">
          <div className="text-[13px] text-[#CCCCCC] truncate font-medium">
            {fileName}
          </div>
          <div className="text-[11px] text-[#858585] truncate">
            {folderPath}
          </div>
        </div>
        <div className="flex items-center gap-2 flex-shrink-0">
          {errorCount > 0 && (
            <div className="flex items-center gap-1 text-red-500">
              <AlertCircle size={14} />
              <span className="text-[11px]">{errorCount}</span>
            </div>
          )}
          {warningCount > 0 && (
            <div className="flex items-center gap-1 text-yellow-500">
              <AlertTriangle size={14} />
              <span className="text-[11px]">{warningCount}</span>
            </div>
          )}
        </div>
      </div>

      {/* Diagnostics list */}
      {isExpanded && (
        <div className="bg-[#252526]">
          {diagnostics.map((diag, index) => (
            <DiagnosticItem
              key={index}
              diagnostic={diag}
              filePath={filePath}
              onNavigate={onNavigate}
            />
          ))}
        </div>
      )}
    </div>
  );
};

/**
 * Diagnostics Panel Component
 */
export const DiagnosticsPanel: React.FC = () => {
  const {
    diagnostics,
    showDiagnosticsPanel,
    hideDiagnostics,
    getTotalErrorCount,
    getTotalWarningCount,
  } = useLspStore();
  const { addTab } = useEditorStore();

  const [severityFilter, setSeverityFilter] = useState<SeverityFilter>('all');

  /**
   * Filter diagnostics by severity
   */
  const filteredDiagnostics = useMemo(() => {
    const allDiagnostics = Array.from(diagnostics.values());

    if (severityFilter === 'errors') {
      return allDiagnostics
        .map((file) => ({
          ...file,
          diagnostics: file.diagnostics.filter((d) => d.severity === 1),
        }))
        .filter((file) => file.diagnostics.length > 0);
    }

    if (severityFilter === 'warnings') {
      return allDiagnostics
        .map((file) => ({
          ...file,
          diagnostics: file.diagnostics.filter((d) => d.severity === 2),
        }))
        .filter((file) => file.diagnostics.length > 0);
    }

    return allDiagnostics;
  }, [diagnostics, severityFilter]);

  /**
   * Handle navigation to diagnostic location
   */
  const handleNavigate = async (
    filePath: string,
    line: number,
    character: number
  ) => {
    try {
      // Open the file in editor
      await addTab(filePath);

      // TODO: Navigate to specific line/column in Monaco Editor
      // This will be implemented when we add editor cursor positioning API
      console.log(`Navigate to ${filePath}:${line + 1}:${character + 1}`);
    } catch (error) {
      console.error('Failed to navigate to diagnostic:', error);
    }
  };

  if (!showDiagnosticsPanel) {
    return null;
  }

  const totalErrors = getTotalErrorCount();
  const totalWarnings = getTotalWarningCount();
  const totalDiagnostics = filteredDiagnostics.reduce(
    (sum, file) => sum + file.diagnostics.length,
    0
  );

  return (
    <div className="h-full flex flex-col bg-[#1E1E1E] text-[#CCCCCC] border-t border-[#2A2D2E]">
      {/* Header */}
      <div className="flex items-center justify-between px-3 py-2 border-b border-[#2A2D2E]">
        <div className="flex items-center gap-3">
          <h3 className="text-[13px] font-semibold">PROBLEMS</h3>
          <div className="flex items-center gap-2 text-[11px]">
            {totalErrors > 0 && (
              <div className="flex items-center gap-1 text-red-500">
                <AlertCircle size={14} />
                <span>{totalErrors}</span>
              </div>
            )}
            {totalWarnings > 0 && (
              <div className="flex items-center gap-1 text-yellow-500">
                <AlertTriangle size={14} />
                <span>{totalWarnings}</span>
              </div>
            )}
            <span className="text-[#858585]">
              {totalDiagnostics} {totalDiagnostics === 1 ? 'item' : 'items'}
            </span>
          </div>
        </div>

        <div className="flex items-center gap-2">
          {/* Severity filter */}
          <div className="flex items-center gap-1">
            <button
              className={`px-2 py-1 text-[11px] rounded transition-colors ${
                severityFilter === 'all'
                  ? 'bg-[#007ACC] text-white'
                  : 'bg-[#3E3E42] hover:bg-[#505050] text-[#CCCCCC]'
              }`}
              onClick={() => setSeverityFilter('all')}
            >
              All
            </button>
            <button
              className={`px-2 py-1 text-[11px] rounded transition-colors ${
                severityFilter === 'errors'
                  ? 'bg-[#007ACC] text-white'
                  : 'bg-[#3E3E42] hover:bg-[#505050] text-[#CCCCCC]'
              }`}
              onClick={() => setSeverityFilter('errors')}
            >
              Errors
            </button>
            <button
              className={`px-2 py-1 text-[11px] rounded transition-colors ${
                severityFilter === 'warnings'
                  ? 'bg-[#007ACC] text-white'
                  : 'bg-[#3E3E42] hover:bg-[#505050] text-[#CCCCCC]'
              }`}
              onClick={() => setSeverityFilter('warnings')}
            >
              Warnings
            </button>
          </div>

          {/* Close button */}
          <button
            className="p-1 hover:bg-[#3E3E42] rounded transition-colors"
            onClick={hideDiagnostics}
            title="Close diagnostics panel"
          >
            <X size={16} />
          </button>
        </div>
      </div>

      {/* Diagnostics list */}
      <div className="flex-1 overflow-y-auto">
        {filteredDiagnostics.length === 0 ? (
          <div className="flex items-center justify-center h-full text-[#858585] text-[13px]">
            No problems detected
          </div>
        ) : (
          filteredDiagnostics.map((file) => (
            <FileGroup
              key={file.filePath}
              filePath={file.filePath}
              diagnostics={file.diagnostics}
              errorCount={file.errorCount}
              warningCount={file.warningCount}
              onNavigate={handleNavigate}
            />
          ))
        )}
      </div>
    </div>
  );
};
