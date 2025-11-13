/**
 * Worktree Explorer Component (Phase 2 - Enhanced)
 *
 * Displays worktrees, file trees, and file preview in a 3-column layout
 */
import React, { useEffect, useState, useMemo } from 'react';
import { useWorktreeStore } from '@/store/worktreeStore';
import {
  Folder,
  FolderOpen,
  FileText,
  Loader2,
  GitBranch,
  Calendar,
  FileCode,
  ChevronRight,
  ChevronDown,
  Trash2,
  AlertCircle,
  Eye,
  GitCompare,
  Grid,
  Code,
  Terminal,
  Copy,
  ExternalLink,
} from 'lucide-react';
import type { FileNode } from '@/services/worktree';
import { FileSearch } from './FileSearch';
import { FilePreview } from './FilePreview';
import { DiffViewer } from './DiffViewer';
import { ComparisonMatrix } from './ComparisonMatrix';
import { tauriApi } from '@/services/tauri';

interface WorktreeExplorerProps {
  competitionId: string;
}

/**
 * File Tree Item Component (Recursive)
 */
interface FileTreeItemProps {
  node: FileNode;
  level: number;
  searchQuery: string;
}

const FileTreeItem: React.FC<FileTreeItemProps> = ({ node, level, searchQuery }) => {
  const [isExpanded, setIsExpanded] = useState(false);
  const { selectFile, selectedFile } = useWorktreeStore();

  // Filter logic: check if node or any child matches search
  const matchesSearch = useMemo(() => {
    if (!searchQuery) return true;
    const query = searchQuery.toLowerCase();

    const nodeMatches = node.name.toLowerCase().includes(query);

    // If directory, check children recursively
    if (node.is_directory && node.children) {
      const childMatches = (children: FileNode[]): boolean => {
        return children.some(child => {
          if (child.name.toLowerCase().includes(query)) return true;
          if (child.is_directory && child.children) {
            return childMatches(child.children);
          }
          return false;
        });
      };
      return nodeMatches || childMatches(node.children);
    }

    return nodeMatches;
  }, [node, searchQuery]);

  // Auto-expand if search matches children
  useEffect(() => {
    if (searchQuery && node.is_directory && matchesSearch) {
      setIsExpanded(true);
    }
  }, [searchQuery, node.is_directory, matchesSearch]);

  if (!matchesSearch) return null;

  const toggleExpand = () => {
    if (node.is_directory) {
      setIsExpanded(!isExpanded);
    } else {
      // File clicked - load preview
      selectFile(node.path);
    }
  };

  const gitStatusColor = (status?: string) => {
    switch (status) {
      case 'modified':
        return 'text-yellow-400';
      case 'added':
        return 'text-green-400';
      case 'deleted':
        return 'text-red-400';
      default:
        return 'text-gray-300';
    }
  };

  const isSelected = !node.is_directory && selectedFile === node.path;

  return (
    <div>
      <div
        className={`
          flex items-center px-2 py-1 cursor-pointer
          transition-all duration-150
          ${gitStatusColor(node.git_status)}
          ${isSelected ? 'bg-blue-500/20 border-l-2 border-blue-400' : 'hover:bg-gray-700'}
        `}
        style={{ paddingLeft: `${level * 16 + 8}px` }}
        onClick={toggleExpand}
      >
        {/* Expand/Collapse Icon */}
        <div className="w-4 h-4 mr-1 flex items-center justify-center">
          {node.is_directory && (
            isExpanded ? (
              <ChevronDown size={14} className="text-gray-400" />
            ) : (
              <ChevronRight size={14} className="text-gray-400" />
            )
          )}
        </div>

        {/* File/Folder Icon */}
        {node.is_directory ? (
          isExpanded ? (
            <FolderOpen size={16} className="mr-2 text-blue-400" />
          ) : (
            <Folder size={16} className="mr-2 text-blue-400" />
          )
        ) : (
          <FileText size={16} className="mr-2 text-gray-400" />
        )}

        {/* Name */}
        <span className="text-sm truncate">{node.name}</span>

        {/* Git Status Badge */}
        {node.git_status && (
          <span className="ml-2 text-xs bg-gray-700 px-1.5 py-0.5 rounded">
            {node.git_status[0].toUpperCase()}
          </span>
        )}
      </div>

      {/* Children (recursive) */}
      {node.is_directory && isExpanded && node.children && (
        <div>
          {node.children.map((child, idx) => (
            <FileTreeItem
              key={`${child.path}-${idx}`}
              node={child}
              level={level + 1}
              searchQuery={searchQuery}
            />
          ))}
        </div>
      )}
    </div>
  );
};

/**
 * Worktree Explorer Component
 */
export const WorktreeExplorer: React.FC<WorktreeExplorerProps> = ({ competitionId }) => {
  const {
    worktrees,
    selectedWorktree,
    fileTree,
    searchQuery,
    isLoading,
    error,
    fetchWorktrees,
    selectWorktree,
    deleteWorktree,
  } = useWorktreeStore();

  // Tab state for right panel
  const [activeTab, setActiveTab] = useState<'preview' | 'diff' | 'compare'>('preview');

  // Filter worktrees by competitionId
  const filteredWorktrees = useMemo(() => {
    return worktrees.filter((w) => w.competition_id === competitionId);
  }, [worktrees, competitionId]);

  useEffect(() => {
    // Only fetch from backend if no worktrees exist for this competition
    // This prevents overwriting data set by setWorktrees (e.g., from SessionDetailView)
    if (competitionId && filteredWorktrees.length === 0 && !isLoading) {
      console.log('[WorktreeExplorer] No worktrees found, fetching from backend:', competitionId);
      fetchWorktrees(competitionId);
    } else {
      console.log('[WorktreeExplorer] Using existing worktrees:', filteredWorktrees.length);
    }
  }, [competitionId, filteredWorktrees.length, isLoading]); // Removed fetchWorktrees from deps to avoid infinite loop

  const handleDelete = async (id: string, e: React.MouseEvent) => {
    e.stopPropagation();

    if (confirm('Are you sure you want to delete this worktree?')) {
      await deleteWorktree(id);
    }
  };

  // Quick action handlers
  const handleOpenVSCode = async (path: string, e: React.MouseEvent) => {
    e.stopPropagation();
    try {
      await tauriApi.openInVscode(path);
    } catch (error) {
      alert(`Failed to open VS Code: ${error}`);
    }
  };

  const handleOpenTerminal = async (path: string, e: React.MouseEvent) => {
    e.stopPropagation();
    try {
      await tauriApi.openTerminal(path);
    } catch (error) {
      alert(`Failed to open terminal: ${error}`);
    }
  };

  const handleOpenFinder = async (path: string, e: React.MouseEvent) => {
    e.stopPropagation();
    try {
      await tauriApi.openInFinder(path);
    } catch (error) {
      alert(`Failed to open file manager: ${error}`);
    }
  };

  const handleCopyPath = async (path: string, e: React.MouseEvent) => {
    e.stopPropagation();
    try {
      await tauriApi.copyToClipboard(path);
      // Show temporary success message
      alert(`Path copied to clipboard: ${path}`);
    } catch (error) {
      alert(`Failed to copy path: ${error}`);
    }
  };

  // Get selected worktree path for DiffViewer
  const selectedWorktreePath = useMemo(() => {
    if (!selectedWorktree) return null;
    const worktree = worktrees.find(w => w.id === selectedWorktree);
    return worktree?.path || null;
  }, [selectedWorktree, worktrees]);

  // Count filtered files
  const filteredFileCount = useMemo(() => {
    if (!searchQuery) return fileTree.length;

    const countMatches = (nodes: FileNode[]): number => {
      return nodes.reduce((count, node) => {
        if (node.name.toLowerCase().includes(searchQuery.toLowerCase())) {
          count++;
        }
        if (node.is_directory && node.children) {
          count += countMatches(node.children);
        }
        return count;
      }, 0);
    };

    return countMatches(fileTree);
  }, [fileTree, searchQuery]);

  return (
    <div className="h-full flex bg-gray-900">
      {/* LEFT PANEL: Worktree List (1/4) */}
      <div className="w-1/4 min-w-[250px] border-r border-gray-700 overflow-y-auto flex-shrink-0">
        <div className="sticky top-0 bg-gray-800 border-b border-gray-700 px-4 py-3 z-10">
          <h3 className="text-sm font-semibold text-gray-300 flex items-center">
            <GitBranch className="w-4 h-4 mr-2 text-blue-400" />
            Worktrees ({filteredWorktrees.length})
          </h3>
        </div>

        {/* Loading State */}
        {isLoading && filteredWorktrees.length === 0 && (
          <div className="flex items-center justify-center py-8">
            <Loader2 className="animate-spin text-gray-400" size={24} />
          </div>
        )}

        {/* Error State */}
        {error && (
          <div className="px-4 py-3 bg-red-500/10 border-l-4 border-red-500 mx-2 mt-2">
            <div className="flex items-center">
              <AlertCircle className="w-4 h-4 text-red-400 mr-2 flex-shrink-0" />
              <p className="text-sm text-red-400">{error}</p>
            </div>
          </div>
        )}

        {/* Empty State */}
        {!isLoading && filteredWorktrees.length === 0 && !error && (
          <div className="px-4 py-8 text-center">
            <Folder className="w-12 h-12 text-gray-600 mx-auto mb-3" />
            <p className="text-sm text-gray-400">No worktrees found</p>
            <p className="text-xs text-gray-500 mt-1">
              Run a competition to create worktrees
            </p>
          </div>
        )}

        {/* Worktree Cards */}
        <div className="p-2 space-y-2">
          {filteredWorktrees.map((worktree) => (
            <div
              key={worktree.id}
              onClick={() => selectWorktree(worktree.id)}
              className={`
                p-3 rounded-lg border-2 cursor-pointer transition-all duration-200
                hover:shadow-lg hover:scale-[1.02]
                ${
                  selectedWorktree === worktree.id
                    ? 'bg-blue-500/10 border-blue-500/50'
                    : 'bg-gray-800 border-gray-700 hover:border-gray-600'
                }
              `}
            >
              {/* Branch */}
              <div className="flex items-center justify-between mb-2">
                <div className="flex items-center space-x-2">
                  <GitBranch className="w-4 h-4 text-green-400" />
                  <span className="text-sm font-mono text-green-300 truncate">
                    {worktree.branch}
                  </span>
                </div>

                {/* Delete Button */}
                <button
                  onClick={(e) => handleDelete(worktree.id, e)}
                  className="p-1 hover:bg-red-500/20 rounded transition-colors"
                  title="Delete worktree"
                >
                  <Trash2 size={14} className="text-gray-400 hover:text-red-400" />
                </button>
              </div>

              {/* Path */}
              <div className="flex items-start space-x-2 mb-3">
                <Folder className="w-4 h-4 text-yellow-400 mt-0.5 flex-shrink-0" />
                <span className="text-xs font-mono text-gray-400 break-all">
                  {worktree.path}
                </span>
              </div>

              {/* Quick Action Buttons */}
              <div className="flex items-center space-x-2 mb-2">
                <button
                  onClick={(e) => handleOpenVSCode(worktree.path, e)}
                  className="flex items-center space-x-1 px-2 py-1 rounded bg-blue-500/10 hover:bg-blue-500/20 border border-blue-500/30 transition-colors"
                  title="Open in VS Code"
                >
                  <Code size={12} className="text-blue-400" />
                  <span className="text-xs text-blue-300">Code</span>
                </button>

                <button
                  onClick={(e) => handleOpenTerminal(worktree.path, e)}
                  className="flex items-center space-x-1 px-2 py-1 rounded bg-purple-500/10 hover:bg-purple-500/20 border border-purple-500/30 transition-colors"
                  title="Open Terminal"
                >
                  <Terminal size={12} className="text-purple-400" />
                  <span className="text-xs text-purple-300">Terminal</span>
                </button>

                <button
                  onClick={(e) => handleOpenFinder(worktree.path, e)}
                  className="flex items-center space-x-1 px-2 py-1 rounded bg-green-500/10 hover:bg-green-500/20 border border-green-500/30 transition-colors"
                  title="Open in Finder"
                >
                  <ExternalLink size={12} className="text-green-400" />
                  <span className="text-xs text-green-300">Finder</span>
                </button>

                <button
                  onClick={(e) => handleCopyPath(worktree.path, e)}
                  className="flex items-center space-x-1 px-2 py-1 rounded bg-gray-500/10 hover:bg-gray-500/20 border border-gray-500/30 transition-colors"
                  title="Copy path to clipboard"
                >
                  <Copy size={12} className="text-gray-400" />
                </button>
              </div>

              {/* Metadata */}
              <div className="flex items-center justify-between text-xs text-gray-500">
                <div className="flex items-center space-x-1">
                  <Calendar className="w-3 h-3" />
                  <span>{new Date(worktree.created_at).toLocaleString()}</span>
                </div>

                <div className="flex items-center space-x-1">
                  <FileCode className="w-3 h-3" />
                  <span>{worktree.changed_files} files</span>
                </div>
              </div>

              {/* Status Badge */}
              <div className="mt-2">
                <span
                  className={`
                    inline-block px-2 py-0.5 rounded text-xs font-semibold
                    ${
                      worktree.status === 'completed'
                        ? 'bg-green-500/10 text-green-400'
                        : worktree.status === 'running'
                        ? 'bg-blue-500/10 text-blue-400'
                        : 'bg-gray-500/10 text-gray-400'
                    }
                  `}
                >
                  {worktree.status}
                </span>
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* MIDDLE PANEL: File Tree (1/3) */}
      <div className="w-1/3 min-w-[300px] border-r border-gray-700 flex flex-col flex-shrink-0">
        <div className="flex-shrink-0 sticky top-0 bg-gray-800 border-b border-gray-700 z-10">
          <div className="px-4 py-3">
            <h3 className="text-sm font-semibold text-gray-300 flex items-center">
              <FolderOpen className="w-4 h-4 mr-2 text-yellow-400" />
              Files
              {selectedWorktree && (
                <span className="ml-2 text-xs text-gray-500">
                  ({searchQuery ? filteredFileCount : fileTree.length} items)
                </span>
              )}
            </h3>
          </div>

          {/* Search Bar */}
          {selectedWorktree && <FileSearch />}
        </div>

        <div className="flex-1 overflow-y-auto">
          {/* Loading State */}
          {isLoading && selectedWorktree && (
            <div className="flex items-center justify-center py-8">
              <Loader2 className="animate-spin text-gray-400" size={24} />
            </div>
          )}

          {/* No Selection */}
          {!selectedWorktree && (
            <div className="px-4 py-8 text-center">
              <Folder className="w-12 h-12 text-gray-600 mx-auto mb-3" />
              <p className="text-sm text-gray-400">Select a worktree to view files</p>
            </div>
          )}

          {/* File Tree */}
          {selectedWorktree && !isLoading && fileTree.length > 0 && (
            <div className="py-2">
              {fileTree.map((node, idx) => (
                <FileTreeItem
                  key={`${node.path}-${idx}`}
                  node={node}
                  level={0}
                  searchQuery={searchQuery}
                />
              ))}
            </div>
          )}

          {/* Empty File Tree */}
          {selectedWorktree && !isLoading && fileTree.length === 0 && (
            <div className="px-4 py-8 text-center">
              <FileText className="w-12 h-12 text-gray-600 mx-auto mb-3" />
              <p className="text-sm text-gray-400">No files found</p>
            </div>
          )}

          {/* No Search Results */}
          {selectedWorktree && !isLoading && fileTree.length > 0 && filteredFileCount === 0 && searchQuery && (
            <div className="px-4 py-8 text-center">
              <AlertCircle className="w-12 h-12 text-gray-600 mx-auto mb-3" />
              <p className="text-sm text-gray-400">No files match "{searchQuery}"</p>
            </div>
          )}
        </div>
      </div>

      {/* RIGHT PANEL: Tabbed View (remaining space) */}
      <div className="flex-1 min-w-0 flex flex-col">
        {/* Tab Header */}
        <div className="flex-shrink-0 border-b border-gray-700 bg-gray-800">
          <div className="flex items-center space-x-1 px-2 py-2">
            <button
              onClick={() => setActiveTab('preview')}
              className={`
                flex items-center space-x-2 px-4 py-2 rounded-lg text-sm font-medium
                transition-all duration-200
                ${
                  activeTab === 'preview'
                    ? 'bg-blue-500/20 text-blue-300 shadow-sm'
                    : 'text-gray-400 hover:text-gray-200 hover:bg-gray-700/50'
                }
              `}
            >
              <Eye className="w-4 h-4" />
              <span>Preview</span>
            </button>

            <button
              onClick={() => setActiveTab('diff')}
              className={`
                flex items-center space-x-2 px-4 py-2 rounded-lg text-sm font-medium
                transition-all duration-200
                ${
                  activeTab === 'diff'
                    ? 'bg-blue-500/20 text-blue-300 shadow-sm'
                    : 'text-gray-400 hover:text-gray-200 hover:bg-gray-700/50'
                }
              `}
            >
              <GitCompare className="w-4 h-4" />
              <span>Diff</span>
            </button>

            <button
              onClick={() => setActiveTab('compare')}
              className={`
                flex items-center space-x-2 px-4 py-2 rounded-lg text-sm font-medium
                transition-all duration-200
                ${
                  activeTab === 'compare'
                    ? 'bg-blue-500/20 text-blue-300 shadow-sm'
                    : 'text-gray-400 hover:text-gray-200 hover:bg-gray-700/50'
                }
              `}
            >
              <Grid className="w-4 h-4" />
              <span>Compare</span>
            </button>
          </div>
        </div>

        {/* Tab Content */}
        <div className="flex-1 min-h-0">
          {activeTab === 'preview' ? (
            <FilePreview />
          ) : activeTab === 'diff' ? (
            selectedWorktreePath ? (
              <DiffViewer worktreePath={selectedWorktreePath} />
            ) : (
              <div className="flex flex-col items-center justify-center h-full text-gray-400">
                <GitCompare className="w-16 h-16 mb-4 text-gray-600" />
                <p className="text-sm">Select a worktree to view diff</p>
              </div>
            )
          ) : (
            <ComparisonMatrix competitionId={competitionId} />
          )}
        </div>
      </div>
    </div>
  );
};

export default WorktreeExplorer;
