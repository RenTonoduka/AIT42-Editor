import { useState } from 'react';
import { Code, Terminal, Settings, Sparkles, Trophy } from 'lucide-react';
import { Sidebar } from './components/Sidebar/Sidebar';
import { EditorContainer } from '@/components/Editor';
import { StatusBar } from '@/components/StatusBar';
import { SettingsPanel } from '@/components/Settings/SettingsPanel';
import { CommandPalette, CompetitionDialog } from '@/components/AI';
import { useEditorStore } from '@/store/editorStore';
import { useSettingsStore } from '@/store/settingsStore';
import { useTerminalStore } from '@/store/terminalStore';
import { useKeyboardShortcuts } from '@/hooks/useKeyboardShortcuts';
import { tauriApi } from '@/services/tauri';

function App() {
  const [activePanel, setActivePanel] = useState<'editor' | 'terminal' | 'settings'>('editor');
  const [isCommandPaletteOpen, setIsCommandPaletteOpen] = useState(false);
  const [isCompetitionDialogOpen, setIsCompetitionDialogOpen] = useState(false);
  const [commandPaletteContext, setCommandPaletteContext] = useState<string | undefined>();
  const { addTab, activeTabId, tabs } = useEditorStore();
  const { toggleSettingsPanel } = useSettingsStore();
  const { toggleTerminal } = useTerminalStore();

  const handleFileOpen = async (path: string) => {
    try {
      await addTab(path);
    } catch (error) {
      console.error('Failed to open file:', error);
    }
  };

  // Get selected text from active editor (for AI context)
  const getSelectedText = (): string | undefined => {
    const activeTab = tabs.find(t => t.id === activeTabId);
    // TODO: Get actual selection from Monaco editor
    return activeTab?.content;
  };

  // Handle AI actions from editor context menu
  const handleAIAction = async (action: string, selectedText: string) => {
    if (action === 'ask-ai') {
      // Open CommandPalette with selected text as context
      setCommandPaletteContext(selectedText);
      setIsCommandPaletteOpen(true);
    } else {
      // Execute appropriate agent based on action
      const agentMap: Record<string, string> = {
        'explain': 'code-reviewer',
        'generate-tests': 'test-generator',
        'refactor': 'refactor-specialist',
        'find-bugs': 'bug-fixer',
      };

      const agentName = agentMap[action];
      if (agentName) {
        try {
          const response = await tauriApi.executeAgent({
            agentName,
            task: `${action} this code: ${selectedText}`,
            context: selectedText,
          });
          console.log('Agent execution started:', response);
          // TODO: Show notification/toast with execution status
        } catch (error) {
          console.error('Failed to execute agent:', error);
          // TODO: Show error notification
        }
      }
    }
  };

  // Register global keyboard shortcuts
  useKeyboardShortcuts({
    onOpenFile: () => console.log('TODO: Open file dialog'),
    onNewFile: () => console.log('TODO: New file dialog'),
    onSave: () => console.log('File saved'),
    onSaveAll: () => console.log('All files saved'),
    onCommandPalette: () => setIsCommandPaletteOpen(true),
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
          onClick={() => setIsCommandPaletteOpen(true)}
          className="group relative p-3 rounded-xl transition-all duration-300 text-text-tertiary hover:text-text-primary hover:bg-editor-hover hover:shadow-glow-sm"
          title="AI Command Palette (⌘K)"
        >
          <Sparkles size={22} className="transition-transform group-hover:scale-110" />
        </button>
        <button
          onClick={() => setIsCompetitionDialogOpen(true)}
          className="group relative p-3 rounded-xl transition-all duration-300 text-text-tertiary hover:text-text-primary hover:bg-editor-hover hover:shadow-glow-sm"
          title="🏆 AI Competition Mode"
        >
          <Trophy size={22} className="transition-transform group-hover:scale-110" />
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
        <EditorContainer onFileOpen={handleFileOpen} onAIAction={handleAIAction} />

        {/* Status Bar */}
        <StatusBar />
      </main>

      {/* Settings Panel (Modal) */}
      <SettingsPanel />

      {/* AI Command Palette (Cmd+K) */}
      <CommandPalette
        isOpen={isCommandPaletteOpen}
        onClose={() => {
          setIsCommandPaletteOpen(false);
          setCommandPaletteContext(undefined);
        }}
        initialContext={commandPaletteContext || getSelectedText()}
      />

      {/* AI Competition Dialog */}
      <CompetitionDialog
        isOpen={isCompetitionDialogOpen}
        onClose={() => setIsCompetitionDialogOpen(false)}
        onStart={(competitionId) => {
          console.log('Competition started:', competitionId);
          // TODO: Show competition progress panel
        }}
      />
    </div>
  );
}

export default App;
