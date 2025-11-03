/**
 * AIT42 Editor - File Tree Component
 */

import React, { useEffect, useState } from 'react';
import { useFileTreeStore } from '../../store/fileTreeStore';
import { useFileSystem } from '../../hooks/useFileSystem';
import { useEditor } from '../../hooks/useEditor';
import { FileNode } from '../../types';
import styles from './Sidebar.module.css';

/**
 * File tree component for navigating the project structure
 *
 * Features:
 * - Recursive directory rendering
 * - Expand/collapse folders
 * - Open files in editor
 * - File type icons
 */
export const FileTree: React.FC = () => {
  const { nodes, setNodes, rootPath, setRootPath } = useFileTreeStore();
  const { readDirectory } = useFileSystem();
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    // Load root directory on mount
    const loadRootDirectory = async () => {
      setLoading(true);
      try {
        // Default to current working directory
        const cwd = await import('@tauri-apps/api/path').then(m => m.appDir());
        setRootPath(cwd);
        const rootNodes = await readDirectory(cwd);
        setNodes(rootNodes);
      } catch (error) {
        console.error('Failed to load root directory:', error);
      } finally {
        setLoading(false);
      }
    };

    loadRootDirectory();
  }, [readDirectory, setNodes, setRootPath]);

  if (loading) {
    return (
      <div className={styles.loading}>
        <div className={styles.spinner} />
        <p>Loading files...</p>
      </div>
    );
  }

  return (
    <div className={styles.fileTree}>
      {nodes.map(node => (
        <FileTreeItem key={node.path} node={node} level={0} />
      ))}
    </div>
  );
};

/**
 * Individual file tree item component
 */
interface FileTreeItemProps {
  node: FileNode;
  level: number;
}

const FileTreeItem: React.FC<FileTreeItemProps> = ({ node, level }) => {
  const [expanded, setExpanded] = useState(false);
  const [children, setChildren] = useState<FileNode[]>([]);
  const { selectedPath, setSelectedPath } = useFileTreeStore();
  const { readDirectory } = useFileSystem();
  const { openFile } = useEditor();

  const isSelected = selectedPath === node.path;
  const hasChildren = node.is_dir;

  const handleClick = async () => {
    setSelectedPath(node.path);

    if (node.is_dir) {
      // Toggle directory expansion
      if (!expanded && children.length === 0) {
        try {
          const childNodes = await readDirectory(node.path);
          setChildren(childNodes);
        } catch (error) {
          console.error('Failed to load directory:', error);
        }
      }
      setExpanded(!expanded);
    } else {
      // Open file in editor
      try {
        await openFile(node.path, node.name);
      } catch (error) {
        console.error('Failed to open file:', error);
      }
    }
  };

  return (
    <div className={styles.treeItem}>
      <div
        className={`${styles.treeItemContent} ${isSelected ? styles.selected : ''}`}
        style={{ paddingLeft: `${level * 12 + 8}px` }}
        onClick={handleClick}
      >
        {/* Expand/collapse icon for directories */}
        {hasChildren && (
          <span className={styles.expandIcon}>
            {expanded ? (
              <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
                <path d="M3 5.5L8 10.5L13 5.5H3Z" />
              </svg>
            ) : (
              <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
                <path d="M5.5 3L10.5 8L5.5 13V3Z" />
              </svg>
            )}
          </span>
        )}

        {/* File/folder icon */}
        <span className={styles.fileIcon}>
          {node.is_dir ? (
            expanded ? '📂' : '📁'
          ) : (
            getFileIcon(node.name)
          )}
        </span>

        {/* File/folder name */}
        <span className={styles.fileName}>{node.name}</span>
      </div>

      {/* Children (for directories) */}
      {expanded && children.length > 0 && (
        <div className={styles.treeItemChildren}>
          {children.map(child => (
            <FileTreeItem key={child.path} node={child} level={level + 1} />
          ))}
        </div>
      )}
    </div>
  );
};

/**
 * Get icon for file based on extension
 */
function getFileIcon(filename: string): string {
  const ext = filename.split('.').pop()?.toLowerCase();

  const iconMap: Record<string, string> = {
    'ts': '📘',
    'tsx': '⚛️',
    'js': '📜',
    'jsx': '⚛️',
    'json': '📋',
    'rs': '🦀',
    'py': '🐍',
    'go': '🐹',
    'java': '☕',
    'html': '🌐',
    'css': '🎨',
    'scss': '🎨',
    'md': '📝',
    'yaml': '⚙️',
    'yml': '⚙️',
    'toml': '⚙️',
    'sh': '🐚',
    'bash': '🐚',
    'git': '🌿',
  };

  return iconMap[ext || ''] || '📄';
}
