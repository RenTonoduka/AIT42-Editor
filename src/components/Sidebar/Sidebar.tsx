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
    <aside className="w-72 bg-editor-surface/90 border-r border-editor-border/30 flex flex-col shadow-glass" style={{ willChange: 'transform' }}>
      {/* Header - Glass morphism with gradient */}
      <div className="flex items-center justify-between px-4 py-3 border-b border-editor-border/20 bg-gradient-to-r from-editor-elevated/30 to-transparent">
        <div className="flex items-center space-x-2.5">
          <div className="p-1.5 rounded-lg bg-gradient-to-br from-accent-primary/20 to-accent-secondary/20">
            <FolderTree size={16} className="text-accent-primary" />
          </div>
          <h2 className="text-xs font-bold text-text-primary uppercase tracking-wider">
            Explorer
          </h2>
        </div>

        {/* Toolbar - Modern icon buttons with glass effect */}
        {rootPath && (
          <div className="flex items-center space-x-1">
            <button
              onClick={handleNewFile}
              className="group p-1.5 hover:bg-editor-hover/50 rounded-lg transition-all duration-200 backdrop-blur-sm"
              title="New File"
            >
              <FileText size={14} className="text-text-tertiary group-hover:text-accent-primary transition-colors" />
            </button>
            <button
              onClick={handleNewFolder}
              className="group p-1.5 hover:bg-editor-hover/50 rounded-lg transition-all duration-200 backdrop-blur-sm"
              title="New Folder"
            >
              <FolderPlus size={14} className="text-text-tertiary group-hover:text-accent-primary transition-colors" />
            </button>
          </div>
        )}
      </div>

      {/* Content */}
      <div className="flex-1 overflow-hidden">
        {rootPath ? (
          <>
            {/* Root Path Display - Elegant card */}
            <div className="mx-3 my-2 px-3 py-2 rounded-lg bg-editor-elevated/50 border border-editor-border/20 backdrop-blur-sm">
              <p className="text-xs text-text-secondary truncate font-mono" title={rootPath}>
                📁 {rootPath.split('/').pop() || rootPath}
              </p>
            </div>

            {/* File Tree */}
            <FileTree onFileOpen={onFileOpen} />
          </>
        ) : (
          <>
            {/* Empty State - Elegant and inviting */}
            <div className="flex flex-col items-center justify-center h-full px-6 animate-fade-in">
            <div className="relative mb-6" style={{ transform: 'translateZ(0)' }}>
              <div className="absolute inset-0 bg-gradient-to-br from-accent-primary/15 to-accent-secondary/15 rounded-full blur-xl" />
              <FolderOpen size={56} className="relative text-text-tertiary" />
            </div>
            <h3 className="text-base font-semibold text-text-primary mb-2">
              No Folder Open
            </h3>
            <p className="text-sm text-text-tertiary text-center mb-6 leading-relaxed">
              Open a folder to start exploring<br />your project files
            </p>
            <button
              onClick={handleOpenFolder}
              className="group relative px-6 py-2.5 bg-gradient-to-r from-accent-primary to-accent-secondary rounded-xl text-white text-sm font-semibold transition-all duration-300 hover:shadow-glow-lg hover:scale-105 overflow-hidden"
            >
              <span className="relative z-10 flex items-center gap-2">
                <FolderOpen size={16} />
                Open Folder
              </span>
              <div className="absolute inset-0 bg-gradient-to-r from-accent-secondary to-accent-primary opacity-0 group-hover:opacity-100 transition-opacity duration-300" />
            </button>
          </div>
          </>
        )}
      </div>
    </aside>
  );
}
