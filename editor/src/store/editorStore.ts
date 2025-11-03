/**
 * AIT42 Editor - Editor State Management (Zustand)
 */

import { create } from 'zustand';
import { EditorState, EditorTab } from '../types';

/**
 * Editor store for managing tabs and content
 *
 * Features:
 * - Tab management (add, remove, switch)
 * - Content tracking with dirty state
 * - Active tab selection
 */
export const useEditorStore = create<EditorState>((set) => ({
  tabs: [],
  activeTabId: null,

  /**
   * Add a new tab to the editor
   */
  addTab: (tabData) => set((state) => {
    const id = `tab-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
    const newTab: EditorTab = {
      ...tabData,
      id,
      isDirty: false,
      isActive: true,
    };

    // Check if file is already open
    const existingTab = state.tabs.find(tab => tab.path === tabData.path);
    if (existingTab) {
      return {
        ...state,
        activeTabId: existingTab.id,
        tabs: state.tabs.map(tab => ({
          ...tab,
          isActive: tab.id === existingTab.id,
        })),
      };
    }

    return {
      ...state,
      tabs: [
        ...state.tabs.map(tab => ({ ...tab, isActive: false })),
        newTab,
      ],
      activeTabId: id,
    };
  }),

  /**
   * Remove a tab from the editor
   */
  removeTab: (id) => set((state) => {
    const tabIndex = state.tabs.findIndex(tab => tab.id === id);
    const newTabs = state.tabs.filter(tab => tab.id !== id);

    // If closing active tab, activate next tab
    let newActiveTabId = state.activeTabId;
    if (state.activeTabId === id && newTabs.length > 0) {
      const nextTab = newTabs[tabIndex] || newTabs[tabIndex - 1] || newTabs[0];
      newActiveTabId = nextTab.id;
    } else if (newTabs.length === 0) {
      newActiveTabId = null;
    }

    return {
      ...state,
      tabs: newTabs.map(tab => ({
        ...tab,
        isActive: tab.id === newActiveTabId,
      })),
      activeTabId: newActiveTabId,
    };
  }),

  /**
   * Set active tab
   */
  setActiveTab: (id) => set((state) => ({
    ...state,
    tabs: state.tabs.map(tab => ({
      ...tab,
      isActive: tab.id === id,
    })),
    activeTabId: id,
  })),

  /**
   * Update tab content and mark as dirty
   */
  updateTabContent: (id, content) => set((state) => ({
    ...state,
    tabs: state.tabs.map(tab =>
      tab.id === id
        ? { ...tab, content, isDirty: true }
        : tab
    ),
  })),

  /**
   * Mark tab as clean (after save)
   */
  markTabClean: (id) => set((state) => ({
    ...state,
    tabs: state.tabs.map(tab =>
      tab.id === id
        ? { ...tab, isDirty: false }
        : tab
    ),
  })),
}));
