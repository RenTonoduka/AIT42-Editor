/**
 * Settings Store
 *
 * Manages application settings and preferences
 */
import { create } from 'zustand';
import { persist } from 'zustand/middleware';

export interface EditorSettings {
  fontSize: number;
  tabSize: number;
  lineNumbers: boolean;
  wordWrap: boolean;
  minimap: boolean;
}

export interface ApplicationSettings {
  autoSave: boolean;
  autoSaveDelay: number;
  confirmDelete: boolean;
}

interface SettingsStore {
  editor: EditorSettings;
  application: ApplicationSettings;
  showSettingsPanel: boolean;

  // Editor settings
  setFontSize: (size: number) => void;
  setTabSize: (size: number) => void;
  toggleLineNumbers: () => void;
  toggleWordWrap: () => void;
  toggleMinimap: () => void;

  // Application settings
  toggleAutoSave: () => void;
  setAutoSaveDelay: (delay: number) => void;
  toggleConfirmDelete: () => void;

  // UI
  toggleSettingsPanel: () => void;
  resetSettings: () => void;
}

const defaultEditorSettings: EditorSettings = {
  fontSize: 14,
  tabSize: 2,
  lineNumbers: true,
  wordWrap: false,
  minimap: true,
};

const defaultApplicationSettings: ApplicationSettings = {
  autoSave: true,
  autoSaveDelay: 1000,
  confirmDelete: true,
};

export const useSettingsStore = create<SettingsStore>()(
  persist(
    (set) => ({
      editor: defaultEditorSettings,
      application: defaultApplicationSettings,
      showSettingsPanel: false,

      // Editor settings
      setFontSize: (size: number) =>
        set((state) => ({
          editor: { ...state.editor, fontSize: Math.max(8, Math.min(32, size)) },
        })),

      setTabSize: (size: number) =>
        set((state) => ({
          editor: { ...state.editor, tabSize: Math.max(2, Math.min(8, size)) },
        })),

      toggleLineNumbers: () =>
        set((state) => ({
          editor: { ...state.editor, lineNumbers: !state.editor.lineNumbers },
        })),

      toggleWordWrap: () =>
        set((state) => ({
          editor: { ...state.editor, wordWrap: !state.editor.wordWrap },
        })),

      toggleMinimap: () =>
        set((state) => ({
          editor: { ...state.editor, minimap: !state.editor.minimap },
        })),

      // Application settings
      toggleAutoSave: () =>
        set((state) => ({
          application: { ...state.application, autoSave: !state.application.autoSave },
        })),

      setAutoSaveDelay: (delay: number) =>
        set((state) => ({
          application: {
            ...state.application,
            autoSaveDelay: Math.max(500, Math.min(5000, delay)),
          },
        })),

      toggleConfirmDelete: () =>
        set((state) => ({
          application: {
            ...state.application,
            confirmDelete: !state.application.confirmDelete,
          },
        })),

      // UI
      toggleSettingsPanel: () =>
        set((state) => ({ showSettingsPanel: !state.showSettingsPanel })),

      resetSettings: () =>
        set({
          editor: defaultEditorSettings,
          application: defaultApplicationSettings,
        }),
    }),
    {
      name: 'settings-storage',
    }
  )
);
