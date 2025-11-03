/**
 * Plugin Store
 *
 * Manages plugin state and operations
 */
import { create } from 'zustand';

export interface PluginInfo {
  manifest: {
    id: string;
    name: string;
    version: string;
    author: string;
    description: string;
    entry_point: string;
    dependencies: string[];
    permissions: string[];
  };
  state: 'Installed' | 'Enabled' | 'Disabled' | 'Error';
  install_path: string;
  error?: string;
}

interface PluginStore {
  plugins: PluginInfo[];
  selectedPlugin: PluginInfo | null;
  showPluginPanel: boolean;
  isLoading: boolean;
  error: string | null;

  // Actions
  fetchPlugins: () => Promise<void>;
  enablePlugin: (pluginId: string) => Promise<void>;
  disablePlugin: (pluginId: string) => Promise<void>;
  uninstallPlugin: (pluginId: string) => Promise<void>;
  selectPlugin: (plugin: PluginInfo | null) => void;
  togglePluginPanel: () => void;
  setError: (error: string | null) => void;
}

export const usePluginStore = create<PluginStore>((set, get) => ({
  plugins: [],
  selectedPlugin: null,
  showPluginPanel: false,
  isLoading: false,
  error: null,

  fetchPlugins: async () => {
    set({ isLoading: true, error: null });
    try {
      const { tauriApi } = await import('@/services/tauri');
      const plugins = await tauriApi.listPlugins();
      set({ plugins, isLoading: false });
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : 'Failed to fetch plugins',
        isLoading: false
      });
    }
  },

  enablePlugin: async (pluginId: string) => {
    set({ isLoading: true, error: null });
    try {
      const { tauriApi } = await import('@/services/tauri');
      await tauriApi.enablePlugin(pluginId);
      await get().fetchPlugins();
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : 'Failed to enable plugin',
        isLoading: false
      });
    }
  },

  disablePlugin: async (pluginId: string) => {
    set({ isLoading: true, error: null });
    try {
      const { tauriApi } = await import('@/services/tauri');
      await tauriApi.disablePlugin(pluginId);
      await get().fetchPlugins();
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : 'Failed to disable plugin',
        isLoading: false
      });
    }
  },

  uninstallPlugin: async (pluginId: string) => {
    set({ isLoading: true, error: null });
    try {
      const { tauriApi } = await import('@/services/tauri');
      await tauriApi.uninstallPlugin(pluginId);
      await get().fetchPlugins();
      if (get().selectedPlugin?.manifest.id === pluginId) {
        set({ selectedPlugin: null });
      }
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : 'Failed to uninstall plugin',
        isLoading: false
      });
    }
  },

  selectPlugin: (plugin: PluginInfo | null) => {
    set({ selectedPlugin: plugin });
  },

  togglePluginPanel: () => {
    set(state => ({ showPluginPanel: !state.showPluginPanel }));
  },

  setError: (error: string | null) => {
    set({ error });
  },
}));
