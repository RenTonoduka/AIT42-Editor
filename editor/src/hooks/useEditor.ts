/**
 * AIT42 Editor - Editor Operations Hook
 */

import { useCallback } from 'react';
import { useEditorStore } from '../store/editorStore';
import { useFileSystem } from './useFileSystem';

/**
 * Custom hook for editor-specific operations
 *
 * Combines editor store with file system operations
 */
export function useEditor() {
  const { readFile, writeFile } = useFileSystem();
  const {
    tabs,
    activeTabId,
    addTab,
    removeTab,
    setActiveTab,
    updateTabContent,
    markTabClean,
  } = useEditorStore();

  /**
   * Get currently active tab
   */
  const activeTab = tabs.find(tab => tab.id === activeTabId);

  /**
   * Open a file in the editor
   */
  const openFile = useCallback(async (path: string, name: string) => {
    try {
      const content = await readFile(path);
      const language = detectLanguage(path);

      addTab({
        path,
        name,
        content,
        language,
      });
    } catch (error) {
      console.error('Failed to open file:', error);
      throw error;
    }
  }, [readFile, addTab]);

  /**
   * Save current file
   */
  const saveFile = useCallback(async (tabId?: string) => {
    const tab = tabId
      ? tabs.find(t => t.id === tabId)
      : activeTab;

    if (!tab) {
      throw new Error('No tab to save');
    }

    try {
      await writeFile(tab.path, tab.content);
      markTabClean(tab.id);
    } catch (error) {
      console.error('Failed to save file:', error);
      throw error;
    }
  }, [tabs, activeTab, writeFile, markTabClean]);

  /**
   * Save all open files
   */
  const saveAllFiles = useCallback(async () => {
    const dirtyTabs = tabs.filter(tab => tab.isDirty);

    await Promise.all(
      dirtyTabs.map(tab => writeFile(tab.path, tab.content))
    );

    dirtyTabs.forEach(tab => markTabClean(tab.id));
  }, [tabs, writeFile, markTabClean]);

  /**
   * Close file with dirty check
   */
  const closeFile = useCallback(async (tabId: string, force = false) => {
    const tab = tabs.find(t => t.id === tabId);

    if (!tab) {
      return;
    }

    // If file is dirty and not forcing, prompt user
    if (tab.isDirty && !force) {
      const shouldSave = window.confirm(
        `${tab.name} has unsaved changes. Save before closing?`
      );

      if (shouldSave) {
        await saveFile(tabId);
      }
    }

    removeTab(tabId);
  }, [tabs, saveFile, removeTab]);

  /**
   * Update editor content
   */
  const updateContent = useCallback((content: string, tabId?: string) => {
    const targetTabId = tabId || activeTabId;
    if (targetTabId) {
      updateTabContent(targetTabId, content);
    }
  }, [activeTabId, updateTabContent]);

  return {
    tabs,
    activeTab,
    activeTabId,
    openFile,
    saveFile,
    saveAllFiles,
    closeFile,
    updateContent,
    setActiveTab,
  };
}

/**
 * Detect programming language from file extension
 */
function detectLanguage(path: string): string {
  const ext = path.split('.').pop()?.toLowerCase();

  const languageMap: Record<string, string> = {
    'ts': 'typescript',
    'tsx': 'typescript',
    'js': 'javascript',
    'jsx': 'javascript',
    'json': 'json',
    'rs': 'rust',
    'py': 'python',
    'go': 'go',
    'java': 'java',
    'c': 'c',
    'cpp': 'cpp',
    'h': 'c',
    'hpp': 'cpp',
    'cs': 'csharp',
    'html': 'html',
    'css': 'css',
    'scss': 'scss',
    'sass': 'sass',
    'md': 'markdown',
    'yaml': 'yaml',
    'yml': 'yaml',
    'toml': 'toml',
    'xml': 'xml',
    'sh': 'shell',
    'bash': 'shell',
    'zsh': 'shell',
    'fish': 'shell',
  };

  return languageMap[ext || ''] || 'plaintext';
}
