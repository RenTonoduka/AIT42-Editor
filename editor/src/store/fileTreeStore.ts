/**
 * AIT42 Editor - File Tree State Management (Zustand)
 */

import { create } from 'zustand';
import { FileNode } from '../types';

interface FileTreeState {
  rootPath: string;
  nodes: FileNode[];
  selectedPath: string | null;
  expandedPaths: Set<string>;
  setRootPath: (path: string) => void;
  setNodes: (nodes: FileNode[]) => void;
  setSelectedPath: (path: string | null) => void;
  toggleExpanded: (path: string) => void;
  updateNodeChildren: (path: string, children: FileNode[]) => void;
}

/**
 * File tree store for managing file system navigation
 *
 * Features:
 * - Root path management
 * - Node expansion state
 * - File/folder selection
 * - Dynamic loading of subdirectories
 */
export const useFileTreeStore = create<FileTreeState>((set) => ({
  rootPath: '',
  nodes: [],
  selectedPath: null,
  expandedPaths: new Set(),

  /**
   * Set the root path for file tree
   */
  setRootPath: (path) => set({ rootPath: path }),

  /**
   * Set the file tree nodes
   */
  setNodes: (nodes) => set({ nodes }),

  /**
   * Set selected file/folder path
   */
  setSelectedPath: (path) => set({ selectedPath: path }),

  /**
   * Toggle expansion state of a directory
   */
  toggleExpanded: (path) => set((state) => {
    const newExpandedPaths = new Set(state.expandedPaths);
    if (newExpandedPaths.has(path)) {
      newExpandedPaths.delete(path);
    } else {
      newExpandedPaths.add(path);
    }
    return { expandedPaths: newExpandedPaths };
  }),

  /**
   * Update children of a specific node
   */
  updateNodeChildren: (path, children) => set((state) => {
    const updateNode = (nodes: FileNode[]): FileNode[] => {
      return nodes.map(node => {
        if (node.path === path) {
          return { ...node, children };
        }
        if (node.children) {
          return { ...node, children: updateNode(node.children) };
        }
        return node;
      });
    };

    return { nodes: updateNode(state.nodes) };
  }),
}));
