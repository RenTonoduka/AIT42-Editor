/**
 * Tests for FileTree Store
 */
import { describe, it, expect, beforeEach } from 'vitest';
import { useFileTreeStore } from './fileTreeStore';

describe('fileTreeStore', () => {
  beforeEach(() => {
    // Reset store before each test
    useFileTreeStore.setState({
      rootPath: null,
      tree: [],
      expandedPaths: [], // Changed from Set to array
      selectedPath: null,
      loading: false,
      error: null,
    });
  });

  it('should initialize with default state', () => {
    const state = useFileTreeStore.getState();
    expect(state.rootPath).toBeNull();
    expect(state.tree).toEqual([]);
    expect(state.expandedPaths.length).toBe(0);
    expect(state.selectedPath).toBeNull();
    expect(state.loading).toBe(false);
    expect(state.error).toBeNull();
  });

  it('should set root path', () => {
    useFileTreeStore.getState().setRootPath('/test/path');
    const state = useFileTreeStore.getState();
    expect(state.rootPath).toBe('/test/path');
  });

  it('should toggle expand state', () => {
    const { toggleExpand } = useFileTreeStore.getState();

    toggleExpand('/test/dir');
    expect(useFileTreeStore.getState().expandedPaths.includes('/test/dir')).toBe(true);

    toggleExpand('/test/dir');
    expect(useFileTreeStore.getState().expandedPaths.includes('/test/dir')).toBe(false);
  });

  it('should select path', () => {
    useFileTreeStore.getState().selectPath('/test/file.ts');
    expect(useFileTreeStore.getState().selectedPath).toBe('/test/file.ts');
  });

  it('should set tree data', () => {
    const mockTree = [
      { name: 'file1.ts', path: '/test/file1.ts', isDirectory: false, children: null },
      { name: 'dir1', path: '/test/dir1', isDirectory: true, children: [] },
    ];

    useFileTreeStore.getState().setTree(mockTree);
    expect(useFileTreeStore.getState().tree).toEqual(mockTree);
  });

  it('should set loading state', () => {
    useFileTreeStore.getState().setLoading(true);
    expect(useFileTreeStore.getState().loading).toBe(true);
  });

  it('should set error state', () => {
    useFileTreeStore.getState().setError('Test error');
    expect(useFileTreeStore.getState().error).toBe('Test error');
  });

  it('should expand all parent paths', () => {
    const { expandPath } = useFileTreeStore.getState();

    expandPath('/root/parent/child/file.ts');
    const expanded = useFileTreeStore.getState().expandedPaths;

    expect(expanded.includes('/root/parent/child')).toBe(true);
    expect(expanded.includes('/root/parent')).toBe(true);
    expect(expanded.includes('/root')).toBe(true);
  });
});
