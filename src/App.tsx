import { useState } from 'react';
import { Code, Terminal, Settings } from 'lucide-react';
import { Sidebar } from './components/Sidebar/Sidebar';
import { EditorContainer } from '@/components/Editor';
import { StatusBar } from '@/components/StatusBar';
import { SettingsPanel } from '@/components/Settings/SettingsPanel';
import { useEditorStore } from '@/store/editorStore';
import { useSettingsStore } from '@/store/settingsStore';
import { useTerminalStore } from '@/store/terminalStore';
import { useKeyboardShortcuts } from '@/hooks/useKeyboardShortcuts';

function App() {
  const [activePanel, setActivePanel] = useState<'editor' | 'terminal' | 'settings'>('editor');
  const { addTab } = useEditorStore();
  const { toggleSettingsPanel } = useSettingsStore();
  const { toggleTerminal } = useTerminalStore();

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
    <div className="flex h-screen bg-editor-bg text-text-primary font-sans">
      {/* Activity Bar - Modern gradient with glass effect */}
      <aside className="w-16 bg-gradient-to-b from-editor-surface to-editor-elevated border-r border-editor-border/50 flex flex-col items-center py-6 space-y-3 backdrop-blur-sm">
        <button
          onClick={() => setActivePanel('editor')}
          className={`group relative p-3 rounded-xl transition-all duration-300 ${
            activePanel === 'editor'
              ? 'bg-gradient-to-br from-accent-primary to-accent-secondary text-white shadow-glow-md'
              : 'text-text-tertiary hover:text-text-primary hover:bg-editor-hover hover:shadow-glow-sm'
          }`}
          title="Editor"
        >
          <Code size={22} className="transition-transform group-hover:scale-110" />
          {activePanel === 'editor' && (
            <div className="absolute left-0 w-1 h-8 bg-accent-primary rounded-r-full animate-fade-in" />
          )}
        </button>
        <button
          onClick={() => {
            setActivePanel('terminal');
            toggleTerminal();
          }}
          className={`group relative p-3 rounded-xl transition-all duration-300 ${
            activePanel === 'terminal'
              ? 'bg-gradient-to-br from-accent-primary to-accent-secondary text-white shadow-glow-md'
              : 'text-text-tertiary hover:text-text-primary hover:bg-editor-hover hover:shadow-glow-sm'
          }`}
          title="Terminal"
        >
          <Terminal size={22} className="transition-transform group-hover:scale-110" />
          {activePanel === 'terminal' && (
            <div className="absolute left-0 w-1 h-8 bg-accent-primary rounded-r-full animate-fade-in" />
          )}
        </button>
        <button
          onClick={() => {
            setActivePanel('settings');
            toggleSettingsPanel();
          }}
          className={`group relative p-3 rounded-xl transition-all duration-300 ${
            activePanel === 'settings'
              ? 'bg-gradient-to-br from-accent-primary to-accent-secondary text-white shadow-glow-md'
              : 'text-text-tertiary hover:text-text-primary hover:bg-editor-hover hover:shadow-glow-sm'
          }`}
          title="Settings"
        >
          <Settings size={22} className="transition-transform group-hover:scale-110" />
          {activePanel === 'settings' && (
            <div className="absolute left-0 w-1 h-8 bg-accent-primary rounded-r-full animate-fade-in" />
          )}
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

      {/* Settings Panel (Modal) */}
      <SettingsPanel />
    </div>
  );
}

export default App;
