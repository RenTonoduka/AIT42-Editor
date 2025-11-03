import { useState } from 'react';
import { Code, Terminal, Settings } from 'lucide-react';
import { Sidebar } from './components/Sidebar/Sidebar';
import { EditorContainer } from '@/components/Editor';
import { StatusBar } from '@/components/StatusBar';
import { useEditorStore } from '@/store/editorStore';
import { useSettingsStore } from '@/store/settingsStore';
import { useKeyboardShortcuts } from '@/hooks/useKeyboardShortcuts';

function App() {
  const [activePanel, setActivePanel] = useState<'editor' | 'terminal' | 'settings'>('editor');
  const { addTab } = useEditorStore();
  const { toggleSettingsPanel } = useSettingsStore();

  const handleFileOpen = async (path: string) => {
    try {
      await addTab(path);
    } catch (error) {
      console.error('Failed to open file:', error);
    }
  };

  // Register global keyboard shortcuts
  useKeyboardShortcuts({
    onOpenFile: () => console.log('TODO: Open file dialog'),
    onNewFile: () => console.log('TODO: New file dialog'),
    onSave: () => console.log('File saved'),
    onSaveAll: () => console.log('All files saved'),
  });

  return (
    <div className="flex h-screen bg-gray-900 text-gray-100">
      {/* Activity Bar */}
      <aside className="w-16 bg-gray-800 border-r border-gray-700 flex flex-col items-center py-4 space-y-4">
        <button
          onClick={() => setActivePanel('editor')}
          className={`p-3 rounded-lg transition-colors ${
            activePanel === 'editor'
              ? 'bg-primary-600 text-white'
              : 'text-gray-400 hover:bg-gray-700 hover:text-white'
          }`}
          title="Editor"
        >
          <Code size={24} />
        </button>
        <button
          onClick={() => setActivePanel('terminal')}
          className={`p-3 rounded-lg transition-colors ${
            activePanel === 'terminal'
              ? 'bg-primary-600 text-white'
              : 'text-gray-400 hover:bg-gray-700 hover:text-white'
          }`}
          title="Terminal"
        >
          <Terminal size={24} />
        </button>
        <button
          onClick={() => {
            setActivePanel('settings');
            toggleSettingsPanel();
          }}
          className={`p-3 rounded-lg transition-colors ${
            activePanel === 'settings'
              ? 'bg-primary-600 text-white'
              : 'text-gray-400 hover:bg-gray-700 hover:text-white'
          }`}
          title="Settings"
        >
          <Settings size={24} />
        </button>
      </aside>

      {/* File Explorer Sidebar */}
      <Sidebar onFileOpen={handleFileOpen} />

      {/* Main Content */}
      <main className="flex-1 flex flex-col">
        {/* Editor Container with Tab Bar and Editor */}
        <EditorContainer onFileOpen={handleFileOpen} />

        {/* Status Bar */}
        <StatusBar />
      </main>
    </div>
  );
}

export default App;
