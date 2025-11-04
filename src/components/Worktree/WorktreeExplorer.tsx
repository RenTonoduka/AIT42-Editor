/**
 * Worktree Explorer Component
 *
 * Displays worktrees and their file trees in a 2-column layout
 */
import React, { useEffect, useState } from 'react';
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
} from 'lucide-react';
import type { FileNode } from '@/services/worktree';

interface WorktreeExplorerProps {
  competitionId: string;
}

/**
 * File Tree Item Component (Recursive)
 */
interface FileTreeItemProps {
  node: FileNode;
  level: number;
}

const FileTreeItem: React.FC<FileTreeItemProps> = ({ node, level }) => {
  const [isExpanded, setIsExpanded] = useState(false);

  const toggleExpand = () => {
    if (node.is_directory) {
      setIsExpanded(!isExpanded);
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

  return (
    <div>
      <div
        className={`
          flex items-center px-2 py-1 hover:bg-gray-700 cursor-pointer
          transition-colors duration-150
          ${gitStatusColor(node.git_status)}
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
            <FileTreeItem key={`${child.path}-${idx}`} node={child} level={level + 1} />
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
    isLoading,
    error,
    fetchWorktrees,
    selectWorktree,
    deleteWorktree,
  } = useWorktreeStore();

  useEffect(() => {
    if (competitionId) {
      fetchWorktrees(competitionId);
    }
  }, [competitionId, fetchWorktrees]);

  const handleDelete = async (id: string, e: React.MouseEvent) => {
    e.stopPropagation();

    if (confirm('Are you sure you want to delete this worktree?')) {
      await deleteWorktree(id);
    }
  };

  return (
    <div className="h-full flex bg-gray-900">
      {/* Left Panel: Worktree List */}
      <div className="w-1/3 border-r border-gray-700 overflow-y-auto">
        <div className="sticky top-0 bg-gray-800 border-b border-gray-700 px-4 py-3">
          <h3 className="text-sm font-semibold text-gray-300 flex items-center">
            <GitBranch className="w-4 h-4 mr-2 text-blue-400" />
            Worktrees ({worktrees.length})
          </h3>
        </div>

        {/* Loading State */}
        {isLoading && worktrees.length === 0 && (
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
        {!isLoading && worktrees.length === 0 && !error && (
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
          {worktrees.map((worktree) => (
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
              <div className="flex items-start space-x-2 mb-2">
                <Folder className="w-4 h-4 text-yellow-400 mt-0.5 flex-shrink-0" />
                <span className="text-xs font-mono text-gray-400 break-all">
                  {worktree.path}
                </span>
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

      {/* Right Panel: File Tree */}
      <div className="flex-1 overflow-y-auto">
        <div className="sticky top-0 bg-gray-800 border-b border-gray-700 px-4 py-3">
          <h3 className="text-sm font-semibold text-gray-300 flex items-center">
            <FolderOpen className="w-4 h-4 mr-2 text-yellow-400" />
            Files
            {selectedWorktree && (
              <span className="ml-2 text-xs text-gray-500">
                ({fileTree.length} items)
              </span>
            )}
          </h3>
        </div>

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
              <FileTreeItem key={`${node.path}-${idx}`} node={node} level={0} />
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
      </div>
    </div>
  );
};

export default WorktreeExplorer;
