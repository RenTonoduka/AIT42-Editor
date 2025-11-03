/**
 * File Operations Integration Tests
 *
 * Tests the complete workflow of:
 * - Opening files from the tree
 * - Editing content
 * - Saving files
 * - Tab management
 */

import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import App from '../../App';
import { invoke } from '@tauri-apps/api/tauri';

// Mock Tauri API
jest.mock('@tauri-apps/api/tauri', () => ({
  invoke: jest.fn(),
}));

jest.mock('@tauri-apps/api/path', () => ({
  appDir: jest.fn(() => Promise.resolve('/test/workspace')),
}));

describe('File Operations Integration', () => {
  const mockInvoke = invoke as jest.MockedFunction<typeof invoke>;

  beforeEach(() => {
    jest.clearAllMocks();

    // Default mock: return empty directory
    mockInvoke.mockImplementation((command: string) => {
      if (command === 'read_directory') {
        return Promise.resolve([]);
      }
      return Promise.resolve();
    });
  });

  describe('Opening Files', () => {
    it('should open file from tree and display in editor', async () => {
      // Mock directory with files
      const mockFileTree = [
        {
          name: 'main.rs',
          path: '/test/workspace/main.rs',
          isDirectory: false,
          children: null,
        },
      ];

      const mockFileContent = {
        bufferId: 'buffer-1',
        content: 'fn main() {\n    println!("Hello");\n}',
        path: '/test/workspace/main.rs',
        language: 'rust',
      };

      mockInvoke.mockImplementation((command: string, args?: any) => {
        if (command === 'read_directory') {
          return Promise.resolve(mockFileTree);
        }
        if (command === 'open_file') {
          return Promise.resolve(mockFileContent);
        }
        return Promise.resolve();
      });

      render(<App />);

      // Wait for file tree to load
      await waitFor(() => {
        expect(screen.getByText('main.rs')).toBeInTheDocument();
      });

      // Click on file
      fireEvent.click(screen.getByText('main.rs'));

      // Wait for file to open
      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith('open_file', {
          path: '/test/workspace/main.rs',
        });
      });

      // Verify tab is created
      await waitFor(() => {
        expect(screen.getByText('main.rs', { selector: '.tab' })).toBeInTheDocument();
      });

      // Verify content is displayed
      await waitFor(() => {
        const editor = screen.getByTestId('monaco-textarea');
        expect(editor).toHaveValue('fn main() {\n    println!("Hello");\n}');
      });
    });

    it('should open multiple files in separate tabs', async () => {
      const mockFileTree = [
        {
          name: 'main.rs',
          path: '/test/workspace/main.rs',
          isDirectory: false,
          children: null,
        },
        {
          name: 'lib.rs',
          path: '/test/workspace/lib.rs',
          isDirectory: false,
          children: null,
        },
      ];

      mockInvoke.mockImplementation((command: string, args?: any) => {
        if (command === 'read_directory') {
          return Promise.resolve(mockFileTree);
        }
        if (command === 'open_file') {
          if (args?.path === '/test/workspace/main.rs') {
            return Promise.resolve({
              bufferId: 'buffer-1',
              content: 'fn main() {}',
              path: '/test/workspace/main.rs',
              language: 'rust',
            });
          }
          if (args?.path === '/test/workspace/lib.rs') {
            return Promise.resolve({
              bufferId: 'buffer-2',
              content: 'pub fn test() {}',
              path: '/test/workspace/lib.rs',
              language: 'rust',
            });
          }
        }
        return Promise.resolve();
      });

      render(<App />);

      await waitFor(() => {
        expect(screen.getByText('main.rs')).toBeInTheDocument();
      });

      // Open first file
      fireEvent.click(screen.getByText('main.rs'));

      await waitFor(() => {
        expect(screen.getByText('main.rs', { selector: '.tab' })).toBeInTheDocument();
      });

      // Open second file
      fireEvent.click(screen.getByText('lib.rs'));

      await waitFor(() => {
        expect(screen.getByText('lib.rs', { selector: '.tab' })).toBeInTheDocument();
      });

      // Verify both tabs exist
      const tabs = screen.getAllByRole('tab');
      expect(tabs).toHaveLength(2);
    });

    it('should handle file open errors gracefully', async () => {
      const consoleError = jest.spyOn(console, 'error').mockImplementation();

      const mockFileTree = [
        {
          name: 'locked.rs',
          path: '/test/workspace/locked.rs',
          isDirectory: false,
          children: null,
        },
      ];

      mockInvoke.mockImplementation((command: string) => {
        if (command === 'read_directory') {
          return Promise.resolve(mockFileTree);
        }
        if (command === 'open_file') {
          return Promise.reject(new Error('Permission denied'));
        }
        return Promise.resolve();
      });

      render(<App />);

      await waitFor(() => {
        expect(screen.getByText('locked.rs')).toBeInTheDocument();
      });

      fireEvent.click(screen.getByText('locked.rs'));

      await waitFor(() => {
        expect(consoleError).toHaveBeenCalled();
      });

      // Should not create tab
      expect(screen.queryByText('locked.rs', { selector: '.tab' })).not.toBeInTheDocument();

      consoleError.mockRestore();
    });
  });

  describe('Editing Files', () => {
    it('should edit and mark tab as dirty', async () => {
      const mockFileTree = [
        {
          name: 'main.rs',
          path: '/test/workspace/main.rs',
          isDirectory: false,
          children: null,
        },
      ];

      mockInvoke.mockImplementation((command: string) => {
        if (command === 'read_directory') {
          return Promise.resolve(mockFileTree);
        }
        if (command === 'open_file') {
          return Promise.resolve({
            bufferId: 'buffer-1',
            content: 'fn main() {}',
            path: '/test/workspace/main.rs',
            language: 'rust',
          });
        }
        return Promise.resolve();
      });

      render(<App />);

      await waitFor(() => {
        expect(screen.getByText('main.rs')).toBeInTheDocument();
      });

      // Open file
      fireEvent.click(screen.getByText('main.rs'));

      await waitFor(() => {
        expect(screen.getByTestId('monaco-textarea')).toBeInTheDocument();
      });

      // Edit content
      const editor = screen.getByTestId('monaco-textarea');
      fireEvent.change(editor, { target: { value: 'fn main() {\n    // New code\n}' } });

      // Verify dirty indicator appears
      await waitFor(() => {
        const tab = screen.getByText('main.rs', { selector: '.tab' }).closest('.tab');
        expect(tab).toHaveClass('dirty');
      });
    });

    it('should preserve edits when switching tabs', async () => {
      const mockFileTree = [
        {
          name: 'file1.txt',
          path: '/test/workspace/file1.txt',
          isDirectory: false,
          children: null,
        },
        {
          name: 'file2.txt',
          path: '/test/workspace/file2.txt',
          isDirectory: false,
          children: null,
        },
      ];

      mockInvoke.mockImplementation((command: string, args?: any) => {
        if (command === 'read_directory') {
          return Promise.resolve(mockFileTree);
        }
        if (command === 'open_file') {
          if (args?.path === '/test/workspace/file1.txt') {
            return Promise.resolve({
              bufferId: 'buffer-1',
              content: 'Content 1',
              path: '/test/workspace/file1.txt',
              language: 'plaintext',
            });
          }
          if (args?.path === '/test/workspace/file2.txt') {
            return Promise.resolve({
              bufferId: 'buffer-2',
              content: 'Content 2',
              path: '/test/workspace/file2.txt',
              language: 'plaintext',
            });
          }
        }
        return Promise.resolve();
      });

      render(<App />);

      await waitFor(() => {
        expect(screen.getByText('file1.txt')).toBeInTheDocument();
      });

      // Open first file
      fireEvent.click(screen.getByText('file1.txt'));
      await waitFor(() => {
        expect(screen.getByTestId('monaco-textarea')).toHaveValue('Content 1');
      });

      // Edit first file
      const editor = screen.getByTestId('monaco-textarea');
      fireEvent.change(editor, { target: { value: 'Modified Content 1' } });

      // Open second file
      fireEvent.click(screen.getByText('file2.txt'));
      await waitFor(() => {
        expect(screen.getByTestId('monaco-textarea')).toHaveValue('Content 2');
      });

      // Switch back to first file
      fireEvent.click(screen.getByText('file1.txt', { selector: '.tab' }));

      // Verify edits are preserved
      await waitFor(() => {
        expect(screen.getByTestId('monaco-textarea')).toHaveValue('Modified Content 1');
      });
    });
  });

  describe('Saving Files', () => {
    it('should save file on Cmd+S', async () => {
      const mockFileTree = [
        {
          name: 'main.rs',
          path: '/test/workspace/main.rs',
          isDirectory: false,
          children: null,
        },
      ];

      mockInvoke.mockImplementation((command: string) => {
        if (command === 'read_directory') {
          return Promise.resolve(mockFileTree);
        }
        if (command === 'open_file') {
          return Promise.resolve({
            bufferId: 'buffer-1',
            content: 'fn main() {}',
            path: '/test/workspace/main.rs',
            language: 'rust',
          });
        }
        if (command === 'save_file') {
          return Promise.resolve();
        }
        return Promise.resolve();
      });

      render(<App />);

      await waitFor(() => {
        expect(screen.getByText('main.rs')).toBeInTheDocument();
      });

      // Open file
      fireEvent.click(screen.getByText('main.rs'));

      await waitFor(() => {
        expect(screen.getByTestId('monaco-textarea')).toBeInTheDocument();
      });

      // Edit content
      const editor = screen.getByTestId('monaco-textarea');
      const newContent = 'fn main() {\n    println!("Modified");\n}';
      fireEvent.change(editor, { target: { value: newContent } });

      // Simulate Cmd+S
      fireEvent.keyDown(editor, {
        key: 's',
        code: 'KeyS',
        metaKey: true,
      });

      // Verify save was called
      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith('save_file', {
          path: '/test/workspace/main.rs',
          content: newContent,
        });
      });

      // Verify dirty indicator is removed
      await waitFor(() => {
        const tab = screen.getByText('main.rs', { selector: '.tab' }).closest('.tab');
        expect(tab).not.toHaveClass('dirty');
      });
    });

    it('should handle save errors', async () => {
      const consoleError = jest.spyOn(console, 'error').mockImplementation();

      const mockFileTree = [
        {
          name: 'readonly.txt',
          path: '/test/workspace/readonly.txt',
          isDirectory: false,
          children: null,
        },
      ];

      mockInvoke.mockImplementation((command: string) => {
        if (command === 'read_directory') {
          return Promise.resolve(mockFileTree);
        }
        if (command === 'open_file') {
          return Promise.resolve({
            bufferId: 'buffer-1',
            content: 'Original content',
            path: '/test/workspace/readonly.txt',
            language: 'plaintext',
          });
        }
        if (command === 'save_file') {
          return Promise.reject(new Error('Read-only file system'));
        }
        return Promise.resolve();
      });

      render(<App />);

      await waitFor(() => {
        expect(screen.getByText('readonly.txt')).toBeInTheDocument();
      });

      // Open file
      fireEvent.click(screen.getByText('readonly.txt'));

      await waitFor(() => {
        expect(screen.getByTestId('monaco-textarea')).toBeInTheDocument();
      });

      // Edit and try to save
      const editor = screen.getByTestId('monaco-textarea');
      fireEvent.change(editor, { target: { value: 'Modified' } });

      fireEvent.keyDown(editor, {
        key: 's',
        code: 'KeyS',
        metaKey: true,
      });

      await waitFor(() => {
        expect(consoleError).toHaveBeenCalled();
      });

      // Tab should still be dirty
      const tab = screen.getByText('readonly.txt', { selector: '.tab' }).closest('.tab');
      expect(tab).toHaveClass('dirty');

      consoleError.mockRestore();
    });
  });

  describe('Complete Workflow', () => {
    it('should complete full file editing workflow', async () => {
      const mockFileTree = [
        {
          name: 'src',
          path: '/test/workspace/src',
          isDirectory: true,
          children: [
            {
              name: 'main.rs',
              path: '/test/workspace/src/main.rs',
              isDirectory: false,
              children: null,
            },
          ],
        },
      ];

      mockInvoke.mockImplementation((command: string, args?: any) => {
        if (command === 'read_directory') {
          if (args?.path === '/test/workspace') {
            return Promise.resolve(mockFileTree);
          }
          if (args?.path === '/test/workspace/src') {
            return Promise.resolve(mockFileTree[0].children);
          }
        }
        if (command === 'open_file') {
          return Promise.resolve({
            bufferId: 'buffer-1',
            content: 'fn main() {}',
            path: '/test/workspace/src/main.rs',
            language: 'rust',
          });
        }
        if (command === 'save_file') {
          return Promise.resolve();
        }
        return Promise.resolve();
      });

      render(<App />);

      // 1. Load file tree
      await waitFor(() => {
        expect(screen.getByText('src')).toBeInTheDocument();
      });

      // 2. Expand directory
      fireEvent.click(screen.getByText('src'));

      await waitFor(() => {
        expect(screen.getByText('main.rs')).toBeInTheDocument();
      });

      // 3. Open file
      fireEvent.click(screen.getByText('main.rs'));

      await waitFor(() => {
        expect(screen.getByTestId('monaco-textarea')).toHaveValue('fn main() {}');
      });

      // 4. Edit content
      const editor = screen.getByTestId('monaco-textarea');
      const newContent = 'fn main() {\n    println!("Hello, World!");\n}';
      fireEvent.change(editor, { target: { value: newContent } });

      // 5. Verify dirty state
      await waitFor(() => {
        const tab = screen.getByText('main.rs', { selector: '.tab' }).closest('.tab');
        expect(tab).toHaveClass('dirty');
      });

      // 6. Save file
      fireEvent.keyDown(editor, {
        key: 's',
        code: 'KeyS',
        metaKey: true,
      });

      // 7. Verify saved
      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith('save_file', {
          path: '/test/workspace/src/main.rs',
          content: newContent,
        });
      });

      // 8. Verify dirty state cleared
      await waitFor(() => {
        const tab = screen.getByText('main.rs', { selector: '.tab' }).closest('.tab');
        expect(tab).not.toHaveClass('dirty');
      });
    });
  });
});
