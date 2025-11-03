/**
 * Sidebar Component - File Explorer Panel
 *
 * Features:
 * - Open folder dialog
 * - File tree display
 * - File operations toolbar
 */
import { FolderTree, FolderOpen, FileText, FolderPlus } from 'lucide-react';
import { FileTree } from './FileTree';
import { useFileTreeStore } from '@/store/fileTreeStore';
import { tauriApi } from '@/services/tauri';
import { open } from '@tauri-apps/api/dialog';

interface SidebarProps {
  onFileOpen?: (path: string) => void;
}

export function Sidebar({ onFileOpen }: SidebarProps) {
  const { rootPath, setRootPath, setTree, setLoading, setError } = useFileTreeStore();

  /**
   * Handle open folder dialog
   */
  const handleOpenFolder = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Select Project Folder',
      });

      if (selected && typeof selected === 'string') {
        await loadDirectory(selected);
      }
    } catch (error) {
      console.error('Failed to open folder:', error);
      setError(`Failed to open folder: ${error}`);
    }
  };

  /**
   * Load directory contents
   */
  const loadDirectory = async (path: string) => {
    setLoading(true);
    setError(null);

    try {
      const entries = await tauriApi.readDirectory(path);
      setRootPath(path);
      setTree(entries);
    } catch (error) {
      console.error('Failed to load directory:', error);
      setError(`Failed to load directory: ${error}`);
    } finally {
      setLoading(false);
    }
  };

  /**
   * Create new file in current directory
   */
  const handleNewFile = async () => {
    if (!rootPath) return;

    const fileName = prompt('Enter file name:');
    if (!fileName) return;

    try {
      const filePath = `${rootPath}/${fileName}`;
      await tauriApi.createFile(filePath);
      // Reload directory
      await loadDirectory(rootPath);
    } catch (error) {
      console.error('Failed to create file:', error);
      alert(`Failed to create file: ${error}`);
    }
  };

  /**
   * Create new folder in current directory
   */
  const handleNewFolder = async () => {
    if (!rootPath) return;

    const folderName = prompt('Enter folder name:');
    if (!folderName) return;

    try {
      const folderPath = `${rootPath}/${folderName}`;
      await tauriApi.createDirectory(folderPath);
      // Reload directory
      await loadDirectory(rootPath);
    } catch (error) {
      console.error('Failed to create folder:', error);
      alert(`Failed to create folder: ${error}`);
    }
  };

  return (
    <aside className="w-64 bg-gray-800 border-r border-gray-700 flex flex-col">
      {/* Header */}
      <div className="flex items-center justify-between p-3 border-b border-gray-700">
        <div className="flex items-center space-x-2">
          <FolderTree size={18} className="text-gray-400" />
          <h2 className="text-xs font-semibold text-gray-300 uppercase tracking-wide">
            Explorer
          </h2>
        </div>

        {/* Toolbar */}
        {rootPath && (
          <div className="flex items-center space-x-1">
            <button
              onClick={handleNewFile}
              className="p-1 hover:bg-gray-700 rounded transition-colors"
              title="New File"
            >
              <FileText size={14} className="text-gray-400" />
            </button>
            <button
              onClick={handleNewFolder}
              className="p-1 hover:bg-gray-700 rounded transition-colors"
              title="New Folder"
            >
              <FolderPlus size={14} className="text-gray-400" />
            </button>
          </div>
        )}
      </div>

      {/* Content */}
      <div className="flex-1 overflow-hidden">
        {rootPath ? (
          <>
            {/* Root Path Display */}
            <div className="px-3 py-2 border-b border-gray-700">
              <p className="text-xs text-gray-400 truncate" title={rootPath}>
                {rootPath.split('/').pop() || rootPath}
              </p>
            </div>

            {/* File Tree */}
            <FileTree onFileOpen={onFileOpen} />
          </>
        ) : (
          /* Empty State */
          <div className="flex flex-col items-center justify-center h-full px-4">
            <FolderOpen size={48} className="text-gray-600 mb-4" />
            <p className="text-sm text-gray-400 text-center mb-4">
              No folder opened
            </p>
            <button
              onClick={handleOpenFolder}
              className="px-4 py-2 bg-primary-600 hover:bg-primary-700 rounded text-white text-sm font-medium transition-colors"
            >
              Open Folder
            </button>
          </div>
        )}
      </div>
    </aside>
  );
}
