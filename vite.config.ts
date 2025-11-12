import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { resolve } from 'path';

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],

  // Prevent vite from obscuring rust errors
  clearScreen: false,

  // Tauri expects a fixed port, fail if that port is not available
  server: {
    port: 5173,
    strictPort: true,
    watch: {
      // Ignore Git worktrees to prevent hot reload during Competition execution
      ignored: ['**/src-tauri/.worktrees/**', '**/.ait42/**'],
    },
  },

  // To access the Tauri environment variables set by the CLI with information about the current target
  envPrefix: ['VITE_', 'TAURI_PLATFORM', 'TAURI_ARCH', 'TAURI_FAMILY', 'TAURI_PLATFORM_VERSION', 'TAURI_PLATFORM_TYPE', 'TAURI_DEBUG'],

  build: {
    // Tauri uses Chromium on Windows and WebKit on macOS and Linux
    target: process.env.TAURI_PLATFORM == 'windows' ? 'chrome105' : 'safari13',
    // Don't minify for debug builds
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    // Produce sourcemaps for debug builds
    sourcemap: !!process.env.TAURI_DEBUG,
    // Bundle size optimization: Split vendor chunks
    rollupOptions: {
      output: {
        manualChunks: {
          // Split Monaco Editor (large dependency)
          'monaco': ['monaco-editor'],
          // Split React libraries
          'react-vendor': ['react', 'react-dom'],
          // Split UI components
          'ui': ['lucide-react'],
          // Split Zustand state management
          'store': ['zustand'],
        },
      },
    },
  },

  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
    },
  },
});
