/**
 * useKeyboardShortcuts - Global keyboard shortcuts
 *
 * Manages application-wide keyboard shortcuts including:
 * - File operations (Cmd+O, Cmd+N, Cmd+S, Cmd+Shift+S)
 * - Tab navigation (Cmd+1-9, Cmd+W, Cmd+Tab)
 * - Editor actions (Cmd+F, Cmd+H, Cmd+P)
 */

import { useEffect } from 'react';
import { useEditorStore } from '@/store/editorStore';

export interface KeyboardShortcutHandlers {
  /** Open file dialog */
  onOpenFile?: () => void;
  /** Create new file */
  onNewFile?: () => void;
  /** Save current file */
  onSave?: () => void;
  /** Save all files */
  onSaveAll?: () => void;
  /** Find in file */
  onFind?: () => void;
  /** Find and replace */
  onFindReplace?: () => void;
  /** Command palette */
  onCommandPalette?: () => void;
}

/**
 * Check if key combination matches
 */
function matchesShortcut(
  e: KeyboardEvent,
  key: string,
  modifiers: { cmd?: boolean; ctrl?: boolean; shift?: boolean; alt?: boolean }
): boolean {
  const isCmdOrCtrl = e.metaKey || e.ctrlKey;

  return (
    e.key === key &&
    (modifiers.cmd ? isCmdOrCtrl : true) &&
    (modifiers.shift ? e.shiftKey : !e.shiftKey) &&
    (modifiers.alt ? e.altKey : !e.altKey)
  );
}

/**
 * Global keyboard shortcuts hook
 */
export function useKeyboardShortcuts(handlers: KeyboardShortcutHandlers = {}) {
  const { tabs, activeTabId, closeTab, setActiveTab, saveTab, saveAllTabs } =
    useEditorStore();

  useEffect(() => {
    const handleKeyDown = async (e: KeyboardEvent) => {
      const isCmdOrCtrl = e.metaKey || e.ctrlKey;

      // Cmd+O - Open file
      if (matchesShortcut(e, 'o', { cmd: true })) {
        e.preventDefault();
        handlers.onOpenFile?.();
        return;
      }

      // Cmd+N - New file
      if (matchesShortcut(e, 'n', { cmd: true })) {
        e.preventDefault();
        handlers.onNewFile?.();
        return;
      }

      // Cmd+S - Save current file
      if (matchesShortcut(e, 's', { cmd: true })) {
        e.preventDefault();
        if (activeTabId) {
          await saveTab(activeTabId);
          handlers.onSave?.();
        }
        return;
      }

      // Cmd+Shift+S - Save all files
      if (matchesShortcut(e, 'S', { cmd: true, shift: true })) {
        e.preventDefault();
        await saveAllTabs();
        handlers.onSaveAll?.();
        return;
      }

      // Cmd+W - Close current tab
      if (matchesShortcut(e, 'w', { cmd: true })) {
        e.preventDefault();
        if (activeTabId) {
          closeTab(activeTabId);
        }
        return;
      }

      // Cmd+F - Find in file
      if (matchesShortcut(e, 'f', { cmd: true })) {
        // Monaco handles this internally, but we can add custom behavior
        handlers.onFind?.();
        return;
      }

      // Cmd+H - Find and replace
      if (matchesShortcut(e, 'h', { cmd: true })) {
        e.preventDefault();
        handlers.onFindReplace?.();
        return;
      }

      // Cmd+P - Command palette
      if (matchesShortcut(e, 'p', { cmd: true })) {
        e.preventDefault();
        handlers.onCommandPalette?.();
        return;
      }

      // Cmd+1 through Cmd+9 - Switch to tab 1-9
      if (isCmdOrCtrl && e.key >= '1' && e.key <= '9') {
        e.preventDefault();
        const index = parseInt(e.key, 10) - 1;
        if (tabs[index]) {
          setActiveTab(tabs[index].id);
        }
        return;
      }

      // Cmd+Tab - Next tab
      if (isCmdOrCtrl && e.key === 'Tab' && !e.shiftKey) {
        e.preventDefault();
        const currentIndex = tabs.findIndex((t) => t.id === activeTabId);
        if (currentIndex !== -1 && tabs.length > 1) {
          const nextIndex = (currentIndex + 1) % tabs.length;
          setActiveTab(tabs[nextIndex].id);
        }
        return;
      }

      // Cmd+Shift+Tab - Previous tab
      if (isCmdOrCtrl && e.key === 'Tab' && e.shiftKey) {
        e.preventDefault();
        const currentIndex = tabs.findIndex((t) => t.id === activeTabId);
        if (currentIndex !== -1 && tabs.length > 1) {
          const prevIndex = currentIndex === 0 ? tabs.length - 1 : currentIndex - 1;
          setActiveTab(tabs[prevIndex].id);
        }
        return;
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [
    tabs,
    activeTabId,
    handlers,
    closeTab,
    setActiveTab,
    saveTab,
    saveAllTabs,
  ]);
}
