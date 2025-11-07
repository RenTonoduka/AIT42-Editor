/**
 * Diff Viewer Component
 *
 * Display git diff with syntax highlighting
 */
import React, { useState, useEffect } from 'react';
import { FileText, Plus, Minus, AlertCircle, Loader2, ChevronRight, ChevronDown } from 'lucide-react';
import { worktreeApi, type FileDiff, type DiffLine } from '@/services/worktree';

interface DiffViewerProps {
  worktreePath: string;
  filePath?: string; // If provided, show single file diff
}

export const DiffViewer: React.FC<DiffViewerProps> = ({ worktreePath, filePath }) => {
  const [diffs, setDiffs] = useState<FileDiff[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [expandedFiles, setExpandedFiles] = useState<Set<string>>(new Set());

  useEffect(() => {
    loadDiff();
  }, [worktreePath, filePath]);

  const loadDiff = async () => {
    setIsLoading(true);
    setError(null);

    try {
      if (filePath) {
        // Single file diff
        const diff = await worktreeApi.getFileDiff(worktreePath, filePath);
        setDiffs([diff]);
        setExpandedFiles(new Set([filePath]));
      } else {
        // All files diff
        const allDiffs = await worktreeApi.getWorktreeDiff(worktreePath);
        setDiffs(allDiffs);
        // Auto-expand first file
        if (allDiffs.length > 0) {
          setExpandedFiles(new Set([allDiffs[0].file_path]));
        }
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load diff');
    } finally {
      setIsLoading(false);
    }
  };

  const toggleFileExpansion = (path: string) => {
    setExpandedFiles((prev) => {
      const next = new Set(prev);
      if (next.has(path)) {
        next.delete(path);
      } else {
        next.add(path);
      }
      return next;
    });
  };

  const getLineColor = (lineType: DiffLine['line_type']) => {
    switch (lineType) {
      case 'add':
        return 'bg-green-500/20 border-l-2 border-green-400';
      case 'delete':
        return 'bg-red-500/20 border-l-2 border-red-400';
      default:
        return 'bg-gray-900';
    }
  };

  const getLineIcon = (lineType: DiffLine['line_type']) => {
    switch (lineType) {
      case 'add':
        return <Plus className="w-3 h-3 text-green-400" />;
      case 'delete':
        return <Minus className="w-3 h-3 text-red-400" />;
      default:
        return null;
    }
  };

  // Loading state
  if (isLoading) {
    return (
      <div className="flex flex-col items-center justify-center h-full">
        <Loader2 className="w-8 h-8 animate-spin text-blue-400 mb-4" />
        <p className="text-sm text-gray-400">Loading diff...</p>
      </div>
    );
  }

  // Error state
  if (error) {
    return (
      <div className="flex flex-col items-center justify-center h-full">
        <AlertCircle className="w-12 h-12 text-red-400 mb-4" />
        <p className="text-sm text-red-400">{error}</p>
        <button
          onClick={loadDiff}
          className="mt-4 px-4 py-2 bg-blue-500 hover:bg-blue-600 rounded text-sm"
        >
          Retry
        </button>
      </div>
    );
  }

  // Empty state
  if (diffs.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center h-full text-gray-400">
        <FileText className="w-16 h-16 mb-4 text-gray-600" />
        <p className="text-sm">No changes detected</p>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col bg-gray-900">
      {/* Header */}
      <div className="flex-shrink-0 px-4 py-3 border-b border-gray-700 bg-gray-800">
        <h3 className="text-sm font-semibold text-gray-300">
          Git Diff ({diffs.length} {diffs.length === 1 ? 'file' : 'files'})
        </h3>
      </div>

      {/* Diff Content */}
      <div className="flex-1 overflow-y-auto">
        {diffs.map((fileDiff) => {
          const isExpanded = expandedFiles.has(fileDiff.file_path);

          return (
            <div key={fileDiff.file_path} className="border-b border-gray-700">
              {/* File Header */}
              <div
                className="flex items-center justify-between px-4 py-3 bg-gray-800 cursor-pointer hover:bg-gray-750"
                onClick={() => toggleFileExpansion(fileDiff.file_path)}
              >
                <div className="flex items-center space-x-2">
                  {isExpanded ? (
                    <ChevronDown className="w-4 h-4 text-gray-400" />
                  ) : (
                    <ChevronRight className="w-4 h-4 text-gray-400" />
                  )}
                  <FileText className="w-4 h-4 text-blue-400" />
                  <span className="text-sm font-mono text-gray-200">{fileDiff.file_path}</span>
                </div>

                <div className="flex items-center space-x-4 text-xs">
                  <span className="flex items-center text-green-400">
                    <Plus className="w-3 h-3 mr-1" />
                    {fileDiff.additions}
                  </span>
                  <span className="flex items-center text-red-400">
                    <Minus className="w-3 h-3 mr-1" />
                    {fileDiff.deletions}
                  </span>
                  <span className="text-gray-500">{fileDiff.change_type}</span>
                </div>
              </div>

              {/* Hunks */}
              {isExpanded && fileDiff.hunks.map((hunk, hunkIdx) => (
                <div key={hunkIdx} className="border-t border-gray-700">
                  {/* Hunk Header */}
                  <div className="px-4 py-2 bg-gray-850 text-xs font-mono text-gray-400">
                    @@ -{hunk.old_start},{hunk.old_lines} +{hunk.new_start},{hunk.new_lines} @@
                  </div>

                  {/* Hunk Lines */}
                  <div className="font-mono text-xs">
                    {hunk.lines.map((line, lineIdx) => (
                      <div
                        key={lineIdx}
                        className={`flex items-start ${getLineColor(line.line_type)} transition-colors`}
                      >
                        {/* Line Numbers */}
                        <div className="flex-shrink-0 flex items-center space-x-1 px-2 py-1 text-gray-500 select-none">
                          <span className="w-10 text-right">
                            {line.old_line_num || ''}
                          </span>
                          <span className="w-10 text-right">
                            {line.new_line_num || ''}
                          </span>
                        </div>

                        {/* Icon */}
                        <div className="flex-shrink-0 w-6 flex items-center justify-center py-1">
                          {getLineIcon(line.line_type)}
                        </div>

                        {/* Content */}
                        <div className="flex-1 py-1 pr-4 overflow-x-auto whitespace-pre text-gray-200">
                          {line.content}
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              ))}
            </div>
          );
        })}
      </div>
    </div>
  );
};
