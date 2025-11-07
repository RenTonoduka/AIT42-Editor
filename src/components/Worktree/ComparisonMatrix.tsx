/**
 * Comparison Matrix Component
 *
 * Compare multiple worktrees side-by-side
 */
import React, { useState, useEffect, useMemo } from 'react';
import { useWorktreeStore } from '@/store/worktreeStore';
import {
  CheckCircle,
  AlertCircle,
  Loader2,
  GitBranch,
  FileText,
  Plus,
  Minus,
  ChevronDown,
  ChevronRight,
} from 'lucide-react';
import { worktreeApi, type FileDiff } from '@/services/worktree';

interface ComparisonMatrixProps {
  competitionId: string;
}

interface WorktreeDiffData {
  worktreeId: string;
  branch: string;
  path: string;
  diffs: FileDiff[];
  isLoading: boolean;
  error: string | null;
}

/**
 * Comparison Matrix Component
 */
export const ComparisonMatrix: React.FC<ComparisonMatrixProps> = ({ competitionId: _competitionId }) => {
  const { worktrees } = useWorktreeStore();
  const [selectedWorktreeIds, setSelectedWorktreeIds] = useState<string[]>([]);
  const [diffData, setDiffData] = useState<Map<string, WorktreeDiffData>>(new Map());
  const [expandedFiles, setExpandedFiles] = useState<Set<string>>(new Set());

  // Load diffs for selected worktrees
  useEffect(() => {
    selectedWorktreeIds.forEach((id) => {
      const worktree = worktrees.find((w) => w.id === id);
      if (worktree && !diffData.has(id)) {
        loadWorktreeDiff(id, worktree.path, worktree.branch);
      }
    });
  }, [selectedWorktreeIds, worktrees]);

  const loadWorktreeDiff = async (id: string, path: string, branch: string) => {
    // Set loading state
    setDiffData((prev) => {
      const newMap = new Map(prev);
      newMap.set(id, {
        worktreeId: id,
        branch,
        path,
        diffs: [],
        isLoading: true,
        error: null,
      });
      return newMap;
    });

    try {
      const diffs = await worktreeApi.getWorktreeDiff(path);

      setDiffData((prev) => {
        const newMap = new Map(prev);
        newMap.set(id, {
          worktreeId: id,
          branch,
          path,
          diffs,
          isLoading: false,
          error: null,
        });
        return newMap;
      });
    } catch (error) {
      setDiffData((prev) => {
        const newMap = new Map(prev);
        newMap.set(id, {
          worktreeId: id,
          branch,
          path,
          diffs: [],
          isLoading: false,
          error: error instanceof Error ? error.message : 'Failed to load diff',
        });
        return newMap;
      });
    }
  };

  const toggleWorktreeSelection = (id: string) => {
    setSelectedWorktreeIds((prev) => {
      if (prev.includes(id)) {
        // Remove
        const newIds = prev.filter((wid) => wid !== id);
        // Clean up diff data
        setDiffData((diffMap) => {
          const newMap = new Map(diffMap);
          newMap.delete(id);
          return newMap;
        });
        return newIds;
      } else {
        // Add (max 4 worktrees)
        if (prev.length >= 4) {
          return prev;
        }
        return [...prev, id];
      }
    });
  };

  const toggleFileExpansion = (filePath: string) => {
    setExpandedFiles((prev) => {
      const next = new Set(prev);
      if (next.has(filePath)) {
        next.delete(filePath);
      } else {
        next.add(filePath);
      }
      return next;
    });
  };

  // Get all unique file paths across selected worktrees
  const allFilePaths = useMemo(() => {
    const paths = new Set<string>();
    diffData.forEach((data) => {
      data.diffs.forEach((diff) => {
        paths.add(diff.file_path);
      });
    });
    return Array.from(paths).sort();
  }, [diffData]);

  // Get diff for a specific file in a specific worktree
  const getFileDiffForWorktree = (worktreeId: string, filePath: string): FileDiff | null => {
    const data = diffData.get(worktreeId);
    if (!data) return null;
    return data.diffs.find((d) => d.file_path === filePath) || null;
  };

  return (
    <div className="h-full flex flex-col bg-gray-900">
      {/* Header */}
      <div className="flex-shrink-0 px-4 py-3 border-b border-gray-700 bg-gray-800">
        <h3 className="text-sm font-semibold text-gray-300 mb-2">
          Worktree Comparison Matrix
        </h3>
        <p className="text-xs text-gray-500">
          Select up to 4 worktrees to compare changes side-by-side
        </p>
      </div>

      {/* Worktree Selection */}
      <div className="flex-shrink-0 border-b border-gray-700 bg-gray-850 p-4">
        <div className="flex flex-wrap gap-2">
          {worktrees.map((worktree) => {
            const isSelected = selectedWorktreeIds.includes(worktree.id);
            const canSelect = !isSelected && selectedWorktreeIds.length < 4;

            return (
              <button
                key={worktree.id}
                onClick={() => toggleWorktreeSelection(worktree.id)}
                disabled={!canSelect && !isSelected}
                className={`
                  flex items-center space-x-2 px-3 py-2 rounded-lg text-sm
                  transition-all duration-200
                  ${
                    isSelected
                      ? 'bg-blue-500/20 border-2 border-blue-400 text-blue-300'
                      : canSelect
                      ? 'bg-gray-700 border-2 border-gray-600 text-gray-300 hover:border-gray-500'
                      : 'bg-gray-800 border-2 border-gray-700 text-gray-500 cursor-not-allowed'
                  }
                `}
              >
                <GitBranch className="w-4 h-4" />
                <span className="font-mono">{worktree.branch}</span>
                {isSelected && <CheckCircle className="w-4 h-4" />}
              </button>
            );
          })}
        </div>
      </div>

      {/* Comparison Matrix */}
      <div className="flex-1 overflow-auto">
        {selectedWorktreeIds.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full text-gray-400">
            <AlertCircle className="w-16 h-16 mb-4 text-gray-600" />
            <p className="text-sm">Select worktrees to compare</p>
          </div>
        ) : (
          <div className="min-w-max">
            {/* Header Row */}
            <div className="sticky top-0 z-10 bg-gray-800 border-b border-gray-700">
              <div className="flex">
                {/* File Column Header */}
                <div className="w-80 px-4 py-3 border-r border-gray-700">
                  <span className="text-sm font-semibold text-gray-300">File Path</span>
                </div>

                {/* Worktree Column Headers */}
                {selectedWorktreeIds.map((id) => {
                  const data = diffData.get(id);
                  return (
                    <div
                      key={id}
                      className="w-64 px-4 py-3 border-r border-gray-700"
                    >
                      <div className="flex items-center space-x-2">
                        <GitBranch className="w-4 h-4 text-blue-400" />
                        <span className="text-sm font-mono text-blue-300 truncate">
                          {data?.branch || 'Loading...'}
                        </span>
                      </div>
                      {data?.isLoading && (
                        <Loader2 className="w-3 h-3 animate-spin text-gray-400 mt-1" />
                      )}
                      {data?.error && (
                        <p className="text-xs text-red-400 mt-1">{data.error}</p>
                      )}
                    </div>
                  );
                })}
              </div>
            </div>

            {/* File Rows */}
            {allFilePaths.map((filePath) => {
              const isExpanded = expandedFiles.has(filePath);

              return (
                <div key={filePath} className="border-b border-gray-700">
                  {/* File Row */}
                  <div className="flex hover:bg-gray-800/50">
                    {/* File Name */}
                    <div
                      className="w-80 px-4 py-3 border-r border-gray-700 cursor-pointer flex items-center space-x-2"
                      onClick={() => toggleFileExpansion(filePath)}
                    >
                      {isExpanded ? (
                        <ChevronDown className="w-4 h-4 text-gray-400 flex-shrink-0" />
                      ) : (
                        <ChevronRight className="w-4 h-4 text-gray-400 flex-shrink-0" />
                      )}
                      <FileText className="w-4 h-4 text-blue-400 flex-shrink-0" />
                      <span className="text-sm font-mono text-gray-300 truncate">
                        {filePath}
                      </span>
                    </div>

                    {/* Worktree Cells */}
                    {selectedWorktreeIds.map((id) => {
                      const diff = getFileDiffForWorktree(id, filePath);

                      return (
                        <div
                          key={id}
                          className="w-64 px-4 py-3 border-r border-gray-700"
                        >
                          {diff ? (
                            <div className="flex items-center space-x-4 text-xs">
                              <span className="flex items-center text-green-400">
                                <Plus className="w-3 h-3 mr-1" />
                                {diff.additions}
                              </span>
                              <span className="flex items-center text-red-400">
                                <Minus className="w-3 h-3 mr-1" />
                                {diff.deletions}
                              </span>
                              <span className="text-gray-500">{diff.change_type}</span>
                            </div>
                          ) : (
                            <span className="text-xs text-gray-500">No changes</span>
                          )}
                        </div>
                      );
                    })}
                  </div>

                  {/* Expanded Details */}
                  {isExpanded && (
                    <div className="bg-gray-850">
                      {selectedWorktreeIds.map((id) => {
                        const diff = getFileDiffForWorktree(id, filePath);
                        if (!diff) return null;

                        return (
                          <div
                            key={id}
                            className="px-4 py-2 border-t border-gray-700"
                          >
                            <div className="flex items-center space-x-2 mb-2">
                              <GitBranch className="w-3 h-3 text-blue-400" />
                              <span className="text-xs font-mono text-blue-300">
                                {diffData.get(id)?.branch}
                              </span>
                            </div>

                            {diff.hunks.map((hunk, hunkIdx) => (
                              <div key={hunkIdx} className="mb-2">
                                <div className="text-xs font-mono text-gray-400 mb-1">
                                  @@ -{hunk.old_start},{hunk.old_lines} +{hunk.new_start},
                                  {hunk.new_lines} @@
                                </div>
                                <div className="font-mono text-xs">
                                  {hunk.lines.slice(0, 5).map((line, lineIdx) => (
                                    <div
                                      key={lineIdx}
                                      className={`
                                        px-2 py-0.5
                                        ${
                                          line.line_type === 'add'
                                            ? 'bg-green-500/10 text-green-300'
                                            : line.line_type === 'delete'
                                            ? 'bg-red-500/10 text-red-300'
                                            : 'text-gray-400'
                                        }
                                      `}
                                    >
                                      {line.content}
                                    </div>
                                  ))}
                                  {hunk.lines.length > 5 && (
                                    <div className="text-xs text-gray-500 px-2 py-1">
                                      ... {hunk.lines.length - 5} more lines
                                    </div>
                                  )}
                                </div>
                              </div>
                            ))}
                          </div>
                        );
                      })}
                    </div>
                  )}
                </div>
              );
            })}

            {/* Empty State */}
            {allFilePaths.length === 0 && (
              <div className="flex flex-col items-center justify-center py-12 text-gray-400">
                <FileText className="w-12 h-12 mb-4 text-gray-600" />
                <p className="text-sm">No changes detected in selected worktrees</p>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
};

export default ComparisonMatrix;
