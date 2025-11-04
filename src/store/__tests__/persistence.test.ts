/**
 * Tests for state persistence across hot reloads
 */
import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { useFileTreeStore } from '../fileTreeStore';
import { useEditorStore } from '../editorStore';

describe('State Persistence', () => {
  beforeEach(() => {
    // Clear localStorage before each test
    localStorage.clear();
  });

  afterEach(() => {
    localStorage.clear();
  });

  describe('FileTreeStore Persistence', () => {
    it('should persist expandedPaths to localStorage', () => {
      const { toggleExpand } = useFileTreeStore.getState();

      // Expand some paths
      toggleExpand('/test/dir1');
      toggleExpand('/test/dir2');

      // Check localStorage
      const stored = localStorage.getItem('ait42-file-tree-storage');
      expect(stored).toBeTruthy();

      const parsed = JSON.parse(stored!);
      expect(parsed.state.expandedPaths).toEqual(['/test/dir1', '/test/dir2']);
    });

    it('should persist selectedPath to localStorage', () => {
      const { selectPath } = useFileTreeStore.getState();

      selectPath('/test/file.ts');

      const stored = localStorage.getItem('ait42-file-tree-storage');
      const parsed = JSON.parse(stored!);
      expect(parsed.state.selectedPath).toBe('/test/file.ts');
    });

    it('should restore expandedPaths from localStorage', () => {
      // Simulate persisted state
      localStorage.setItem(
        'ait42-file-tree-storage',
        JSON.stringify({
          state: {
            expandedPaths: ['/restored/dir1', '/restored/dir2'],
            selectedPath: '/restored/file.ts',
          },
          version: 0,
        })
      );

      // Get state (simulates reload)
      const state = useFileTreeStore.getState();
      expect(state.expandedPaths).toEqual(['/restored/dir1', '/restored/dir2']);
      expect(state.selectedPath).toBe('/restored/file.ts');
    });
  });

  describe('EditorStore Persistence', () => {
    it('should persist tabs to localStorage', () => {
      const state = useEditorStore.getState();

      // Manually add tab to state (simulating addTab without async)
      useEditorStore.setState({
        tabs: [
          {
            id: 'tab-test-file-ts',
            path: '/test/file.ts',
            name: 'file.ts',
            content: 'console.log("test")',
            language: 'typescript',
            isDirty: false,
            isActive: true,
          },
        ],
        activeTabId: 'tab-test-file-ts',
      });

      const stored = localStorage.getItem('ait42-editor-storage');
      expect(stored).toBeTruthy();

      const parsed = JSON.parse(stored!);
      expect(parsed.state.tabs).toHaveLength(1);
      expect(parsed.state.tabs[0].path).toBe('/test/file.ts');
      expect(parsed.state.activeTabId).toBe('tab-test-file-ts');
    });

    it('should restore tabs from localStorage', () => {
      // Simulate persisted state
      localStorage.setItem(
        'ait42-editor-storage',
        JSON.stringify({
          state: {
            tabs: [
              {
                id: 'tab-restored-ts',
                path: '/restored.ts',
                name: 'restored.ts',
                content: 'const x = 1;',
                language: 'typescript',
                isDirty: false,
                isActive: true,
              },
            ],
            activeTabId: 'tab-restored-ts',
          },
          version: 0,
        })
      );

      // Get state (simulates reload)
      const state = useEditorStore.getState();
      expect(state.tabs).toHaveLength(1);
      expect(state.tabs[0].path).toBe('/restored.ts');
      expect(state.activeTabId).toBe('tab-restored-ts');
    });
  });
});
