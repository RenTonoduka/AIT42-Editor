/**
 * FileTree Component - Displays recursive file tree
 *
 * Features:
 * - Recursive tree rendering
 * - Lazy loading for directories
 * - File icons
 * - Expand/collapse animation
 * - Selection highlighting
 * - Double-click to open files
 */
import { useState } from 'react';
import { ChevronRight, ChevronDown, Loader2 } from 'lucide-react';
import { FileIcon } from './FileIcon';
import { useFileTreeStore, type FileNode } from '@/store/fileTreeStore';
import { tauriApi } from '@/services/tauri';

interface FileTreeItemProps {
  node: FileNode;
  level: number;
  onFileOpen?: (path: string) => void;
}

function FileTreeItem({ node, level, onFileOpen }: FileTreeItemProps) {
  const { expandedPaths, selectedPath, toggleExpand, selectPath } = useFileTreeStore();
  const [loading, setLoading] = useState(false);
  const [children, setChildren] = useState<FileNode[]>(node.children || []);

  const isExpanded = expandedPaths.has(node.path);
  const isSelected = selectedPath === node.path;

  /**
   * Load directory contents on expand
   */
  const loadDirectory = async () => {
    if (loading || children.length > 0) return;

    setLoading(true);
    try {
      const entries = await tauriApi.readDirectory(node.path);
      setChildren(entries);
    } catch (error) {
      console.error('Failed to load directory:', error);
    } finally {
      setLoading(false);
    }
  };

  /**
   * Handle expand/collapse
   */
  const handleToggle = async (e: React.MouseEvent) => {
    e.stopPropagation();

    if (node.isDirectory) {
      toggleExpand(node.path);

      if (!isExpanded && children.length === 0) {
        await loadDirectory();
      }
    }
  };

  /**
   * Handle file/directory click
   */
  const handleClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    selectPath(node.path);

    if (node.isDirectory) {
      handleToggle(e);
    }
  };

  /**
   * Handle double-click to open file
   */
  const handleDoubleClick = (e: React.MouseEvent) => {
    e.stopPropagation();

    if (!node.isDirectory && onFileOpen) {
      onFileOpen(node.path);
    }
  };

  return (
    <div className="select-none">
      <div
        className={`
          flex items-center px-2 py-1 hover:bg-gray-700 cursor-pointer
          transition-colors duration-150 group
          ${isSelected ? 'bg-gray-700' : ''}
        `}
        style={{ paddingLeft: `${level * 12 + 8}px` }}
        onClick={handleClick}
        onDoubleClick={handleDoubleClick}
      >
        {/* Expand/Collapse Icon */}
        <div className="w-4 h-4 mr-1 flex items-center justify-center">
          {node.isDirectory && (
            <button
              onClick={handleToggle}
              className="hover:bg-gray-600 rounded p-0.5 transition-colors"
            >
              {loading ? (
                <Loader2 size={14} className="animate-spin text-gray-400" />
              ) : isExpanded ? (
                <ChevronDown size={14} className="text-gray-400" />
              ) : (
                <ChevronRight size={14} className="text-gray-400" />
              )}
            </button>
          )}
        </div>

        {/* File Icon */}
        <FileIcon
          name={node.name}
          isDirectory={node.isDirectory}
          isExpanded={isExpanded}
          className="mr-2 flex-shrink-0"
        />

        {/* File Name */}
        <span className="text-sm text-gray-200 truncate">{node.name}</span>
      </div>

      {/* Children (recursive) */}
      {node.isDirectory && isExpanded && children.length > 0 && (
        <div className="transition-all duration-200">
          {children.map((child) => (
            <FileTreeItem
              key={child.path}
              node={child}
              level={level + 1}
              onFileOpen={onFileOpen}
            />
          ))}
        </div>
      )}
    </div>
  );
}

interface FileTreeProps {
  onFileOpen?: (path: string) => void;
}

export function FileTree({ onFileOpen }: FileTreeProps) {
  const { tree, loading, error } = useFileTreeStore();

  if (loading) {
    return (
      <div className="flex items-center justify-center py-8">
        <Loader2 className="animate-spin text-gray-400" size={24} />
      </div>
    );
  }

  if (error) {
    return (
      <div className="px-4 py-2 text-red-400 text-sm">
        <p>Error loading file tree:</p>
        <p className="text-xs mt-1">{error}</p>
      </div>
    );
  }

  if (tree.length === 0) {
    return (
      <div className="px-4 py-2 text-gray-400 text-sm">
        <p>No files found</p>
      </div>
    );
  }

  return (
    <div className="overflow-auto h-full">
      {tree.map((node) => (
        <FileTreeItem key={node.path} node={node} level={0} onFileOpen={onFileOpen} />
      ))}
    </div>
  );
}
