import { useState } from 'react';
import { FileText, Settings, Users, Layout, Sparkles, Trophy, MessageSquare, Target, History } from 'lucide-react';
import { Sidebar } from './components/Sidebar/Sidebar';
import { EditorContainer } from '@/components/Editor';
import { StatusBar } from '@/components/StatusBar';
import { SettingsPanel } from '@/components/Settings/SettingsPanel';
import MultiAgentPanel, { ClaudeCodeInstance } from '@/components/AI/MultiAgentPanel';
import { CompetitionDialog } from '@/components/AI/CompetitionDialog';
import { EnsembleDialog } from '@/components/AI/EnsembleDialog';
import { DebateDialog } from '@/components/AI/DebateDialog';
import DebateStatusPanel from '@/components/AI/DebateStatusPanel';
import { OptimizerDemo } from '@/components/Optimizer';
import { SessionHistory } from '@/components/SessionHistory';
import { useEditorStore } from '@/store/editorStore';

// View Mode Type
type ViewMode = 'editor' | 'multi-agent' | 'debate' | 'optimizer' | 'session-history';

function App() {
  const [viewMode, setViewMode] = useState<ViewMode>('editor');
  const [showSettings, setShowSettings] = useState(false);
  const [showCompetitionDialog, setShowCompetitionDialog] = useState(false);
  const [showEnsembleDialog, setShowEnsembleDialog] = useState(false);
  const [showDebateDialog, setShowDebateDialog] = useState(false);
  const [claudeInstances, setClaudeInstances] = useState<ClaudeCodeInstance[]>([]);
  const [debateId, setDebateId] = useState<string | null>(null);
  const [debateTask, setDebateTask] = useState<string>('');
  const [activeCompetitionId, setActiveCompetitionId] = useState<string | null>(null); // 🔥 NEW: Store competition ID

  // Get active file from editor store
  const getActiveTab = useEditorStore((state) => state.getActiveTab);
  const activeFile = getActiveTab();

  // Handle competition start (競争モード)
  const handleCompetitionStart = (competitionId: string, instanceCount: number, task: string) => {
    const newInstances: ClaudeCodeInstance[] = Array.from({ length: instanceCount }, (_, i) => ({
      id: `${competitionId}-${i}`,
      agentName: `競争 Agent ${i + 1}`,
      task: task,
      status: 'running',
      output: '',
      startTime: new Date().toISOString(),
      tmuxSessionId: `claude-comp-${competitionId}-${i}`,
      worktreePath: `/tmp/worktree-comp-${competitionId}-${i}`,
      worktreeBranch: `claude-competition-${competitionId}-${i}`,
    }));

    setClaudeInstances(newInstances);
    setActiveCompetitionId(competitionId); // 🔥 NEW: Store competition ID
    setShowCompetitionDialog(false);
    setViewMode('multi-agent');
  };

  // Handle ensemble start (アンサンブルモード)
  const handleEnsembleStart = (competitionId: string, instanceCount: number, task: string) => {
    const newInstances: ClaudeCodeInstance[] = Array.from({ length: instanceCount }, (_, i) => ({
      id: `${competitionId}-${i}`,
      agentName: `アンサンブル Agent ${i + 1}`,
      task: task,
      status: 'running',
      output: '',
      startTime: new Date().toISOString(),
      tmuxSessionId: `claude-ens-${competitionId}-${i}`,
      worktreePath: `/tmp/worktree-ens-${competitionId}-${i}`,
      worktreeBranch: `claude-ensemble-${competitionId}-${i}`,
    }));

    setClaudeInstances(newInstances);
    setActiveCompetitionId(competitionId); // 🔥 NEW: Store competition ID
    setShowEnsembleDialog(false);
    setViewMode('multi-agent');
  };

  // Handle debate start (ディベートモード)
  const handleDebateStart = (newDebateId: string, task: string) => {
    setDebateId(newDebateId);
    setDebateTask(task);
    setShowDebateDialog(false);
    setViewMode('debate');
  };

  // Handle debate close
  const handleDebateClose = () => {
    setViewMode('editor');
    // Keep debateId and debateTask for history viewing
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

        {/* AI Launch Buttons and View Mode Toggle */}
        <div className="flex items-center space-x-3">
          {/* AI Launch Buttons */}
          <div className="flex items-center space-x-2">
            {/* Competition Mode Button */}
            <button
              onClick={() => setShowCompetitionDialog(true)}
              className="px-3 py-1.5 rounded-md text-sm font-medium transition-all bg-gradient-to-r from-purple-600 to-pink-600 hover:from-purple-500 hover:to-pink-500 text-white shadow-md hover:shadow-lg"
              title="競争モード: 最良の結果を選択"
            >
              <Trophy className="w-4 h-4 inline-block mr-1.5" />
              競争
            </button>

            {/* Ensemble Mode Button */}
            <button
              onClick={() => setShowEnsembleDialog(true)}
              className="px-3 py-1.5 rounded-md text-sm font-medium transition-all bg-gradient-to-r from-purple-600 to-blue-600 hover:from-purple-500 hover:to-blue-500 text-white shadow-md hover:shadow-lg"
              title="アンサンブルモード: 統合AIが結果を統合"
            >
              <Sparkles className="w-4 h-4 inline-block mr-1.5" />
              アンサンブル
            </button>

            {/* Debate Mode Button */}
            <button
              onClick={() => setShowDebateDialog(true)}
              className="px-3 py-1.5 rounded-md text-sm font-medium transition-all bg-gradient-to-r from-indigo-600 to-purple-600 hover:from-indigo-500 hover:to-purple-500 text-white shadow-md hover:shadow-lg"
              title="ディベートモード: 3ラウンドの役割ベース議論"
            >
              <MessageSquare className="w-4 h-4 inline-block mr-1.5" />
              ディベート
            </button>
          </div>

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
            <button
              onClick={() => setViewMode('debate')}
              className={`px-3 py-1 rounded-md text-sm transition-colors ${
                viewMode === 'debate'
                  ? 'bg-indigo-600 text-white'
                  : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
              } ${!debateId ? 'opacity-50 cursor-not-allowed' : ''}`}
              disabled={!debateId}
            >
              <MessageSquare className="w-4 h-4 inline-block mr-1" />
              Debate
            </button>
            <button
              onClick={() => setViewMode('optimizer')}
              className={`px-3 py-1 rounded-md text-sm transition-colors ${
                viewMode === 'optimizer'
                  ? 'bg-green-600 text-white'
                  : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
              }`}
            >
              <Target className="w-4 h-4 inline-block mr-1" />
              Optimizer
            </button>
            <button
              onClick={() => setViewMode('session-history')}
              className={`px-3 py-1 rounded-md text-sm transition-colors ${
                viewMode === 'session-history'
                  ? 'bg-orange-600 text-white'
                  : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
              }`}
            >
              <History className="w-4 h-4 inline-block mr-1" />
              履歴
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
            <MultiAgentPanel instances={claudeInstances} competitionId={activeCompetitionId ||undefined} />
          </div>
        )}

        {/* Debate View Mode */}
        {viewMode === 'debate' && debateId && (
          <div className="flex-1 bg-gray-900">
            <DebateStatusPanel
              debateId={debateId}
              task={debateTask}
              onClose={handleDebateClose}
            />
          </div>
        )}

        {/* Optimizer View Mode */}
        {viewMode === 'optimizer' && (
          <div className="flex-1 bg-gray-900 overflow-auto">
            <OptimizerDemo />
          </div>
        )}
      </div>

      {/* Settings Modal */}
      {showSettings && (
        <SettingsPanel />
      )}

      {/* Competition Dialog (競争モード) */}
      <CompetitionDialog
        isOpen={showCompetitionDialog}
        onClose={() => setShowCompetitionDialog(false)}
        onStart={handleCompetitionStart}
      />

      {/* Ensemble Dialog (アンサンブルモード) */}
      <EnsembleDialog
        isOpen={showEnsembleDialog}
        onClose={() => setShowEnsembleDialog(false)}
        onStart={handleEnsembleStart}
      />

      {/* Debate Dialog (ディベートモード) */}
      <DebateDialog
        isOpen={showDebateDialog}
        onClose={() => setShowDebateDialog(false)}
        onStart={handleDebateStart}
      />

      {/* Status Bar */}
      <StatusBar />
    </div>
  );
}

export default App;
