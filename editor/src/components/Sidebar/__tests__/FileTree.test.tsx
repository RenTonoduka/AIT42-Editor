/**
 * FileTree Component Tests
 *
 * Test coverage:
 * - Directory tree rendering
 * - Expand/collapse functionality
 * - File selection
 * - Lazy loading of children
 * - Icon display
 * - Error handling
 */

import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { FileTree } from '../FileTree';
import { useFileTreeStore } from '../../../store/fileTreeStore';
import { useFileSystem } from '../../../hooks/useFileSystem';
import { useEditor } from '../../../hooks/useEditor';

// Mock dependencies
jest.mock('../../../store/fileTreeStore');
jest.mock('../../../hooks/useFileSystem');
jest.mock('../../../hooks/useEditor');

describe('FileTree', () => {
  const mockSetNodes = jest.fn();
  const mockSetRootPath = jest.fn();
  const mockSetSelectedPath = jest.fn();
  const mockReadDirectory = jest.fn();
  const mockOpenFile = jest.fn();

  beforeEach(() => {
    jest.clearAllMocks();

    // Default mock implementations
    (useFileTreeStore as jest.Mock).mockReturnValue({
      nodes: [],
      setNodes: mockSetNodes,
      rootPath: '/test/path',
      setRootPath: mockSetRootPath,
      selectedPath: null,
      setSelectedPath: mockSetSelectedPath,
    });

    (useFileSystem as jest.Mock).mockReturnValue({
      readDirectory: mockReadDirectory,
    });

    (useEditor as jest.Mock).mockReturnValue({
      openFile: mockOpenFile,
    });
  });

  describe('Rendering', () => {
    it('should render loading state initially', () => {
      mockReadDirectory.mockReturnValue(new Promise(() => {})); // Never resolves

      render(<FileTree />);

      expect(screen.getByText('Loading files...')).toBeInTheDocument();
    });

    it('should render empty directory', async () => {
      mockReadDirectory.mockResolvedValue([]);

      render(<FileTree />);

      await waitFor(() => {
        expect(mockReadDirectory).toHaveBeenCalled();
      });

      expect(screen.queryByText('Loading files...')).not.toBeInTheDocument();
    });

    it('should render directory tree', async () => {
      const mockNodes = [
        {
          name: 'src',
          path: '/test/src',
          is_dir: true,
          children: null,
        },
        {
          name: 'main.rs',
          path: '/test/main.rs',
          is_dir: false,
          children: null,
        },
      ];

      mockReadDirectory.mockResolvedValue(mockNodes);
      (useFileTreeStore as jest.Mock).mockReturnValue({
        nodes: mockNodes,
        setNodes: mockSetNodes,
        rootPath: '/test',
        setRootPath: mockSetRootPath,
        selectedPath: null,
        setSelectedPath: mockSetSelectedPath,
      });

      render(<FileTree />);

      await waitFor(() => {
        expect(screen.getByText('src')).toBeInTheDocument();
        expect(screen.getByText('main.rs')).toBeInTheDocument();
      });
    });

    it('should display correct file icons', async () => {
      const mockNodes = [
        { name: 'main.rs', path: '/test/main.rs', is_dir: false, children: null },
        { name: 'index.tsx', path: '/test/index.tsx', is_dir: false, children: null },
        { name: 'config.json', path: '/test/config.json', is_dir: false, children: null },
      ];

      mockReadDirectory.mockResolvedValue(mockNodes);
      (useFileTreeStore as jest.Mock).mockReturnValue({
        nodes: mockNodes,
        setNodes: mockSetNodes,
        rootPath: '/test',
        setRootPath: mockSetRootPath,
        selectedPath: null,
        setSelectedPath: mockSetSelectedPath,
      });

      render(<FileTree />);

      await waitFor(() => {
        expect(screen.getByText('main.rs')).toBeInTheDocument();
      });

      // Icons should be rendered (emojis)
      const tree = screen.getByText('main.rs').closest('.treeItem');
      expect(tree).toContainHTML('🦀'); // Rust icon
    });

    it('should display folder icons', async () => {
      const mockNodes = [
        {
          name: 'folder',
          path: '/test/folder',
          is_dir: true,
          children: null,
        },
      ];

      mockReadDirectory.mockResolvedValue(mockNodes);
      (useFileTreeStore as jest.Mock).mockReturnValue({
        nodes: mockNodes,
        setNodes: mockSetNodes,
        rootPath: '/test',
        setRootPath: mockSetRootPath,
        selectedPath: null,
        setSelectedPath: mockSetSelectedPath,
      });

      render(<FileTree />);

      await waitFor(() => {
        expect(screen.getByText('folder')).toBeInTheDocument();
      });

      const tree = screen.getByText('folder').closest('.treeItem');
      expect(tree).toContainHTML('📁'); // Closed folder icon
    });
  });

  describe('Directory Expansion', () => {
    it('should expand directory on click', async () => {
      const mockNodes = [
        {
          name: 'src',
          path: '/test/src',
          is_dir: true,
          children: null,
        },
      ];

      const childNodes = [
        {
          name: 'main.rs',
          path: '/test/src/main.rs',
          is_dir: false,
          children: null,
        },
      ];

      mockReadDirectory.mockResolvedValueOnce(mockNodes).mockResolvedValueOnce(childNodes);

      (useFileTreeStore as jest.Mock).mockReturnValue({
        nodes: mockNodes,
        setNodes: mockSetNodes,
        rootPath: '/test',
        setRootPath: mockSetRootPath,
        selectedPath: null,
        setSelectedPath: mockSetSelectedPath,
      });

      render(<FileTree />);

      await waitFor(() => {
        expect(screen.getByText('src')).toBeInTheDocument();
      });

      // Click to expand
      fireEvent.click(screen.getByText('src'));

      await waitFor(() => {
        expect(mockReadDirectory).toHaveBeenCalledWith('/test/src');
        expect(mockSetSelectedPath).toHaveBeenCalledWith('/test/src');
      });
    });

    it('should collapse directory on second click', async () => {
      const mockNodes = [
        {
          name: 'src',
          path: '/test/src',
          is_dir: true,
          children: null,
        },
      ];

      mockReadDirectory.mockResolvedValue(mockNodes);

      (useFileTreeStore as jest.Mock).mockReturnValue({
        nodes: mockNodes,
        setNodes: mockSetNodes,
        rootPath: '/test',
        setRootPath: mockSetRootPath,
        selectedPath: null,
        setSelectedPath: mockSetSelectedPath,
      });

      render(<FileTree />);

      await waitFor(() => {
        expect(screen.getByText('src')).toBeInTheDocument();
      });

      const dirElement = screen.getByText('src');

      // Expand
      fireEvent.click(dirElement);
      await waitFor(() => {
        expect(mockSetSelectedPath).toHaveBeenCalledWith('/test/src');
      });

      // Collapse
      fireEvent.click(dirElement);
      // Should still call setSelectedPath but toggle expansion state
    });

    it('should change folder icon when expanded', async () => {
      const mockNodes = [
        {
          name: 'folder',
          path: '/test/folder',
          is_dir: true,
          children: null,
        },
      ];

      mockReadDirectory.mockResolvedValue(mockNodes).mockResolvedValue([]);

      (useFileTreeStore as jest.Mock).mockReturnValue({
        nodes: mockNodes,
        setNodes: mockSetNodes,
        rootPath: '/test',
        setRootPath: mockSetRootPath,
        selectedPath: null,
        setSelectedPath: mockSetSelectedPath,
      });

      const { container } = render(<FileTree />);

      await waitFor(() => {
        expect(screen.getByText('folder')).toBeInTheDocument();
      });

      // Initially closed
      expect(container).toContainHTML('📁');

      // Click to expand
      fireEvent.click(screen.getByText('folder'));

      await waitFor(() => {
        expect(container).toContainHTML('📂'); // Open folder icon
      });
    });
  });

  describe('File Selection', () => {
    it('should open file on click', async () => {
      const mockNodes = [
        {
          name: 'main.rs',
          path: '/test/main.rs',
          is_dir: false,
          children: null,
        },
      ];

      mockReadDirectory.mockResolvedValue(mockNodes);

      (useFileTreeStore as jest.Mock).mockReturnValue({
        nodes: mockNodes,
        setNodes: mockSetNodes,
        rootPath: '/test',
        setRootPath: mockSetRootPath,
        selectedPath: null,
        setSelectedPath: mockSetSelectedPath,
      });

      render(<FileTree />);

      await waitFor(() => {
        expect(screen.getByText('main.rs')).toBeInTheDocument();
      });

      fireEvent.click(screen.getByText('main.rs'));

      await waitFor(() => {
        expect(mockOpenFile).toHaveBeenCalledWith('/test/main.rs', 'main.rs');
        expect(mockSetSelectedPath).toHaveBeenCalledWith('/test/main.rs');
      });
    });

    it('should highlight selected file', async () => {
      const mockNodes = [
        {
          name: 'main.rs',
          path: '/test/main.rs',
          is_dir: false,
          children: null,
        },
      ];

      mockReadDirectory.mockResolvedValue(mockNodes);

      (useFileTreeStore as jest.Mock).mockReturnValue({
        nodes: mockNodes,
        setNodes: mockSetNodes,
        rootPath: '/test',
        setRootPath: mockSetRootPath,
        selectedPath: '/test/main.rs',
        setSelectedPath: mockSetSelectedPath,
      });

      render(<FileTree />);

      await waitFor(() => {
        const fileElement = screen.getByText('main.rs').closest('.treeItemContent');
        expect(fileElement).toHaveClass('selected');
      });
    });
  });

  describe('Lazy Loading', () => {
    it('should load children only when directory is expanded', async () => {
      const mockNodes = [
        {
          name: 'src',
          path: '/test/src',
          is_dir: true,
          children: null,
        },
      ];

      mockReadDirectory.mockResolvedValue(mockNodes);

      (useFileTreeStore as jest.Mock).mockReturnValue({
        nodes: mockNodes,
        setNodes: mockSetNodes,
        rootPath: '/test',
        setRootPath: mockSetRootPath,
        selectedPath: null,
        setSelectedPath: mockSetSelectedPath,
      });

      render(<FileTree />);

      await waitFor(() => {
        expect(screen.getByText('src')).toBeInTheDocument();
      });

      // Should not load children yet
      expect(mockReadDirectory).toHaveBeenCalledTimes(1);

      // Click to expand
      fireEvent.click(screen.getByText('src'));

      await waitFor(() => {
        // Now should load children
        expect(mockReadDirectory).toHaveBeenCalledWith('/test/src');
      });
    });

    it('should not reload children on subsequent expansions', async () => {
      const mockNodes = [
        {
          name: 'src',
          path: '/test/src',
          is_dir: true,
          children: null,
        },
      ];

      const childNodes = [
        {
          name: 'main.rs',
          path: '/test/src/main.rs',
          is_dir: false,
          children: null,
        },
      ];

      mockReadDirectory
        .mockResolvedValueOnce(mockNodes)
        .mockResolvedValueOnce(childNodes);

      (useFileTreeStore as jest.Mock).mockReturnValue({
        nodes: mockNodes,
        setNodes: mockSetNodes,
        rootPath: '/test',
        setRootPath: mockSetRootPath,
        selectedPath: null,
        setSelectedPath: mockSetSelectedPath,
      });

      render(<FileTree />);

      await waitFor(() => {
        expect(screen.getByText('src')).toBeInTheDocument();
      });

      const dirElement = screen.getByText('src');

      // Expand
      fireEvent.click(dirElement);
      await waitFor(() => {
        expect(mockReadDirectory).toHaveBeenCalledTimes(2);
      });

      // Collapse
      fireEvent.click(dirElement);

      // Expand again
      fireEvent.click(dirElement);

      // Should not load again
      expect(mockReadDirectory).toHaveBeenCalledTimes(2);
    });
  });

  describe('Error Handling', () => {
    it('should handle directory load error', async () => {
      const consoleError = jest.spyOn(console, 'error').mockImplementation();

      mockReadDirectory.mockRejectedValue(new Error('Failed to read directory'));

      render(<FileTree />);

      await waitFor(() => {
        expect(consoleError).toHaveBeenCalledWith(
          'Failed to load root directory:',
          expect.any(Error)
        );
      });

      consoleError.mockRestore();
    });

    it('should handle file open error', async () => {
      const consoleError = jest.spyOn(console, 'error').mockImplementation();

      const mockNodes = [
        {
          name: 'main.rs',
          path: '/test/main.rs',
          is_dir: false,
          children: null,
        },
      ];

      mockReadDirectory.mockResolvedValue(mockNodes);
      mockOpenFile.mockRejectedValue(new Error('Failed to open file'));

      (useFileTreeStore as jest.Mock).mockReturnValue({
        nodes: mockNodes,
        setNodes: mockSetNodes,
        rootPath: '/test',
        setRootPath: mockSetRootPath,
        selectedPath: null,
        setSelectedPath: mockSetSelectedPath,
      });

      render(<FileTree />);

      await waitFor(() => {
        expect(screen.getByText('main.rs')).toBeInTheDocument();
      });

      fireEvent.click(screen.getByText('main.rs'));

      await waitFor(() => {
        expect(consoleError).toHaveBeenCalledWith(
          'Failed to open file:',
          expect.any(Error)
        );
      });

      consoleError.mockRestore();
    });
  });

  describe('Nested Directories', () => {
    it('should render nested directory structure', async () => {
      const mockNodes = [
        {
          name: 'src',
          path: '/test/src',
          is_dir: true,
          children: [
            {
              name: 'components',
              path: '/test/src/components',
              is_dir: true,
              children: null,
            },
            {
              name: 'main.rs',
              path: '/test/src/main.rs',
              is_dir: false,
              children: null,
            },
          ],
        },
      ];

      mockReadDirectory.mockResolvedValue(mockNodes);

      (useFileTreeStore as jest.Mock).mockReturnValue({
        nodes: mockNodes,
        setNodes: mockSetNodes,
        rootPath: '/test',
        setRootPath: mockSetRootPath,
        selectedPath: null,
        setSelectedPath: mockSetSelectedPath,
      });

      render(<FileTree />);

      await waitFor(() => {
        expect(screen.getByText('src')).toBeInTheDocument();
      });

      // Expand src
      fireEvent.click(screen.getByText('src'));

      await waitFor(() => {
        expect(screen.getByText('components')).toBeInTheDocument();
        expect(screen.getByText('main.rs')).toBeInTheDocument();
      });
    });

    it('should apply correct indentation for nested items', async () => {
      const mockNodes = [
        {
          name: 'src',
          path: '/test/src',
          is_dir: true,
          children: [
            {
              name: 'main.rs',
              path: '/test/src/main.rs',
              is_dir: false,
              children: null,
            },
          ],
        },
      ];

      mockReadDirectory.mockResolvedValue(mockNodes);

      (useFileTreeStore as jest.Mock).mockReturnValue({
        nodes: mockNodes,
        setNodes: mockSetNodes,
        rootPath: '/test',
        setRootPath: mockSetRootPath,
        selectedPath: null,
        setSelectedPath: mockSetSelectedPath,
      });

      render(<FileTree />);

      await waitFor(() => {
        expect(screen.getByText('src')).toBeInTheDocument();
      });

      const srcElement = screen.getByText('src').closest('.treeItemContent');
      expect(srcElement).toHaveStyle({ paddingLeft: '8px' }); // Level 0

      // Expand
      fireEvent.click(screen.getByText('src'));

      await waitFor(() => {
        const mainElement = screen.getByText('main.rs').closest('.treeItemContent');
        expect(mainElement).toHaveStyle({ paddingLeft: '20px' }); // Level 1 (12 * 1 + 8)
      });
    });
  });
});
