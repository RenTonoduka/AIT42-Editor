/**
 * Theme Store
 *
 * Manages application theme and color schemes
 */
import { create } from 'zustand';
import { persist } from 'zustand/middleware';

export type ThemeName = 'dark' | 'light' | 'high-contrast';

export interface Theme {
  name: ThemeName;
  label: string;
  colors: {
    background: string;
    foreground: string;
    primary: string;
    secondary: string;
    accent: string;
    border: string;
    hover: string;
  };
}

const themes: Record<ThemeName, Theme> = {
  dark: {
    name: 'dark',
    label: 'Dark',
    colors: {
      background: '#1E1E1E',
      foreground: '#CCCCCC',
      primary: '#007ACC',
      secondary: '#3E3E42',
      accent: '#4EC9B0',
      border: '#2A2D2E',
      hover: '#2A2D2E',
    },
  },
  light: {
    name: 'light',
    label: 'Light',
    colors: {
      background: '#FFFFFF',
      foreground: '#333333',
      primary: '#0066CC',
      secondary: '#F0F0F0',
      accent: '#00A896',
      border: '#E0E0E0',
      hover: '#F5F5F5',
    },
  },
  'high-contrast': {
    name: 'high-contrast',
    label: 'High Contrast',
    colors: {
      background: '#000000',
      foreground: '#FFFFFF',
      primary: '#FFFF00',
      secondary: '#1A1A1A',
      accent: '#00FF00',
      border: '#FFFFFF',
      hover: '#333333',
    },
  },
};

interface ThemeStore {
  currentTheme: ThemeName;
  theme: Theme;

  setTheme: (themeName: ThemeName) => void;
  getAvailableThemes: () => Theme[];
}

export const useThemeStore = create<ThemeStore>()(
  persist(
    (set) => ({
      currentTheme: 'dark',
      theme: themes.dark,

      setTheme: (themeName: ThemeName) => {
        set({ currentTheme: themeName, theme: themes[themeName] });
      },

      getAvailableThemes: () => Object.values(themes),
    }),
    {
      name: 'theme-storage',
    }
  )
);
