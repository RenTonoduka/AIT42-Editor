import { useState } from 'react';
import { FileText, Settings, Activity, Users, Layout, Sparkles } from 'lucide-react';
import { Sidebar } from './components/Sidebar/Sidebar';
import { EditorContainer } from '@/components/Editor';
import { StatusBar } from '@/components/StatusBar';
import { SettingsPanel } from '@/components/Settings/SettingsPanel';
import MultiAgentPanel, { ClaudeCodeInstance } from '@/components/AI/MultiAgentPanel';
import { CompetitionDialog } from '@/components/AI/CompetitionDialog';
import { useEditorStore } from '@/store/editorStore';
import { useSettingsStore } from '@/store/settingsStore';

// View Mode Type
type ViewMode = 'editor' | 'multi-agent';

function App() {
  const [viewMode, setViewMode] = useState<ViewMode>('editor');
  const [showSettings, setShowSettings] = useState(false);
  const [showCompetitionDialog, setShowCompetitionDialog] = useState(false);
  const [claudeInstances, setClaudeInstances] = useState<ClaudeCodeInstance[]>([]);

  const { activeFile } = useEditorStore();
  const { showDiagnostics } = useSettingsStore();

  // Handle competition start
  const handleCompetitionStart = (competitionId: string, instanceCount: number, task: string) => {
    // Create mock instances for now (will be updated with real data from Tauri events)
    const newInstances: ClaudeCodeInstance[] = Array.from({ length: instanceCount }, (_, i) => ({
      id: `${competitionId}-${i}`,
      agentName: `Agent ${i + 1}`,
      task: task,
      status: 'running',
      output: '',
      startTime: new Date().toISOString(),
      tmuxSessionId: `claude-${competitionId}-${i}`,
      worktreePath: `/tmp/worktree-${competitionId}-${i}`,
      worktreeBranch: `claude-competition-${competitionId}-${i}`,
    }));

    setClaudeInstances(newInstances);
    setShowCompetitionDialog(false);
    setViewMode('multi-agent'); // Switch to multi-agent view
  };

  return (
    <div className="flex flex-col h-screen bg-gray-900 text-gray-100">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-2 bg-gray-800 border-b border-gray-700">
        <div className="flex items-center space-x-4">
          <FileText className="w-5 h-5 text-blue-400" />
          <span className="text-sm font-semibold">AIT42-Editor</span>
          {activeFile && (
            <span className="text-xs text-gray-400">{activeFile.name}</span>
          )}
        </div>

        {/* AI Competition and View Mode Toggle */}
        <div className="flex items-center space-x-3">
          {/* AI Competition Launch Button */}
          <button
            onClick={() => setShowCompetitionDialog(true)}
            className="px-3 py-1.5 rounded-md text-sm font-medium transition-all bg-gradient-to-r from-purple-600 to-pink-600 hover:from-purple-500 hover:to-pink-500 text-white shadow-md hover:shadow-lg"
          >
            <Sparkles className="w-4 h-4 inline-block mr-1.5" />
            AIコンペ起動
          </button>

          {/* View Mode Toggle */}
          <div className="flex items-center space-x-2 ml-2 pl-2 border-l border-gray-700">
            <button
              onClick={() => setViewMode('editor')}
              className={`px-3 py-1 rounded-md text-sm transition-colors ${
                viewMode === 'editor'
                  ? 'bg-blue-600 text-white'
                  : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
              }`}
            >
              <Layout className="w-4 h-4 inline-block mr-1" />
              Editor
            </button>
            <button
              onClick={() => setViewMode('multi-agent')}
              className={`px-3 py-1 rounded-md text-sm transition-colors ${
                viewMode === 'multi-agent'
                  ? 'bg-purple-600 text-white'
                  : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
              } ${claudeInstances.length === 0 ? 'opacity-50 cursor-not-allowed' : ''}`}
              disabled={claudeInstances.length === 0}
            >
              <Users className="w-4 h-4 inline-block mr-1" />
              Multi-Agent ({claudeInstances.length})
            </button>
          </div>
        </div>

        <div className="flex items-center space-x-2">
          <button
            onClick={() => setShowSettings(!showSettings)}
            className={`p-2 rounded hover:bg-gray-700 ${
              showSettings ? 'bg-gray-700' : ''
            }`}
          >
            <Settings className="w-4 h-4" />
          </button>
        </div>
      </div>

      {/* Main Content */}
      <div className="flex flex-1 overflow-hidden">
        {/* Editor View Mode */}
        {viewMode === 'editor' && (
          <>
            <Sidebar />
            <div className="flex-1">
              <EditorContainer />
            </div>
          </>
        )}

        {/* Multi-Agent View Mode */}
        {viewMode === 'multi-agent' && (
          <div className="flex-1 bg-gray-900">
            <MultiAgentPanel instances={claudeInstances} />
          </div>
        )}
      </div>

      {/* Settings Modal */}
      {showSettings && (
        <SettingsPanel onClose={() => setShowSettings(false)} />
      )}

      {/* Competition Dialog */}
      <CompetitionDialog
        isOpen={showCompetitionDialog}
        onClose={() => setShowCompetitionDialog(false)}
        onStart={handleCompetitionStart}
      />

      {/* Status Bar */}
      <StatusBar />
    </div>
  );
}

export default App;
