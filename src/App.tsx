import { useState, useEffect, useCallback } from 'react';
import { FileText, Settings, Users, Layout, Sparkles, Trophy, MessageSquare, LayoutDashboard, FolderOpen } from 'lucide-react';
import { Sidebar } from './components/Sidebar/Sidebar';
import { EditorContainer } from '@/components/Editor';
import { StatusBar } from '@/components/StatusBar';
import { SettingsPanel } from '@/components/Settings/SettingsPanel';
import MultiAgentPanel, { ClaudeCodeInstance } from '@/components/AI/MultiAgentPanel';
import { CompetitionDialog } from '@/components/AI/CompetitionDialog';
import { EnsembleDialog } from '@/components/AI/EnsembleDialog';
import { DebateDialog } from '@/components/AI/DebateDialog';
import DebateStatusPanel from '@/components/AI/DebateStatusPanel';
import { SessionHistory } from '@/components/SessionHistory';
import { useEditorStore } from '@/store/editorStore';
import { useSessionHistoryStore } from '@/store/sessionHistoryStore';
import { useFileTreeStore } from '@/store/fileTreeStore';
import { tauriApi } from '@/services/tauri';

// View Mode Type
type ViewMode = 'editor' | 'multi-agent' | 'debate' | 'session-history';

/**
 * タスク説明を短縮（最大50文字）
 */
function shortenTaskDescription(task: string, maxLength: number = 50): string {
  const cleaned = task.trim().replace(/\s+/g, ' ');
  if (cleaned.length <= maxLength) return cleaned;
  return cleaned.substring(0, maxLength - 3) + '...';
}

/**
 * 現在時刻を HH:MM 形式で取得
 */
function getCurrentTime(): string {
  const now = new Date();
  return `${String(now.getHours()).padStart(2, '0')}:${String(now.getMinutes()).padStart(2, '0')}`;
}

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
  const [workspacePath, setWorkspacePath] = useState<string>('');
  const [isGitRepo, setIsGitRepo] = useState<boolean>(false);

  // Get active file from editor store
  const getActiveTab = useEditorStore((state) => state.getActiveTab);
  const activeFile = getActiveTab();

  // Get session history store methods
  const sessionHistorySetWorkspacePath = useSessionHistoryStore((state) => state.setWorkspacePath);

  // Get file tree store methods
  const { setRootPath, setTree, setLoading, setError } = useFileTreeStore();

  // Helper function to load directory contents into file tree
  const loadDirectoryIntoFileTree = useCallback(async (path: string) => {
    setLoading(true);
    setError(null);

    try {
      const entries = await tauriApi.readDirectory(path);
      setRootPath(path);
      setTree(entries);
      console.log(`✅ Loaded directory into Explorer: ${path}`);
    } catch (error) {
      console.error('Failed to load directory:', error);
      setError(`Failed to load directory: ${error}`);
    } finally {
      setLoading(false);
    }
  }, [setRootPath, setTree, setLoading, setError]);

  // Check workspace on mount
  useEffect(() => {
    const checkWorkspace = async () => {
      try {
        const workspace = await tauriApi.getWorkspace();
        setWorkspacePath(workspace.path);
        setIsGitRepo(workspace.is_git_repo);

        // If not a git repo, prompt user to select workspace
        if (!workspace.is_git_repo) {
          const confirmSelect = window.confirm(
            '現在のフォルダはGitリポジトリではありません。\nプロジェクトフォルダを選択しますか？'
          );
          if (confirmSelect) {
            handleSelectWorkspace();
          }
        }
      } catch (error) {
        console.error('Failed to check workspace:', error);
      }
    };

    checkWorkspace();
  }, []);

  // Sync workspace path to session history store (only if it's a valid git repo)
  useEffect(() => {
    if (workspacePath && isGitRepo) {
      sessionHistorySetWorkspacePath(workspacePath);
    } else if (!isGitRepo) {
      // Clear session history if workspace is not a git repo
      sessionHistorySetWorkspacePath('');
    }
  }, [workspacePath, isGitRepo, sessionHistorySetWorkspacePath]);

  // 🔥 Auto-load directory into Explorer when workspace path changes
  useEffect(() => {
    if (workspacePath && isGitRepo) {
      loadDirectoryIntoFileTree(workspacePath);
    }
  }, [workspacePath, isGitRepo, loadDirectoryIntoFileTree]);

  // Handle workspace selection
  const handleSelectWorkspace = async () => {
    try {
      const workspace = await tauriApi.selectWorkspace();
      setWorkspacePath(workspace.path);

      // 自動的にgit initを実行（Gitリポジトリでない場合）
      if (!workspace.is_git_repo) {
        try {
          await tauriApi.gitInit(workspace.path);
          setIsGitRepo(true);

          // 🔥 Load directory contents into Explorer
          await loadDirectoryIntoFileTree(workspace.path);

          alert(
            `ワークスペースを設定しました:\n${workspace.path}\n\n` +
            `Gitリポジトリが作成されました。\n` +
            `Competition/Ensembleモードが使用可能です。`
          );
        } catch (error) {
          setIsGitRepo(false);

          // 🔥 Load directory contents into Explorer even if git init failed
          await loadDirectoryIntoFileTree(workspace.path);

          alert(
            `ワークスペースを設定しました:\n${workspace.path}\n\n` +
            `警告: Gitリポジトリの初期化に失敗しました。\n${error}\n\n` +
            `Competition/Ensembleモードは使用できません。`
          );
        }
      } else {
        setIsGitRepo(true);

        // 🔥 Load directory contents into Explorer
        await loadDirectoryIntoFileTree(workspace.path);

        alert(`ワークスペースを設定しました:\n${workspace.path}`);
      }
    } catch (error) {
      alert(`ワークスペースの選択に失敗しました:\n${error}`);
    }
  };

  // Handle competition start (競争モード)
  const handleCompetitionStart = async (competitionId: string, instanceCount: number, task: string) => {
    const shortTask = shortenTaskDescription(task);
    const time = getCurrentTime();

    const newInstances: ClaudeCodeInstance[] = Array.from({ length: instanceCount }, (_, i) => ({
      id: `${competitionId}-${i}`,
      agentName: `🏆 ${shortTask} #${i + 1} (${time})`,
      task: task,
      status: 'running',
      output: '',
      startTime: new Date().toISOString(),
      tmuxSessionId: `claude-comp-${competitionId}-${i}`,
      worktreePath: `/tmp/worktree-comp-${competitionId}-${i}`,
      worktreeBranch: `claude-competition-${competitionId}-${i}`,
    }));

    // セッション履歴に保存 (only if valid workspace is open)
    if (workspacePath && isGitRepo) {
      try {
        await tauriApi.createSession(workspacePath, {
          id: competitionId,
          type: 'competition',
          task,
          status: 'running',
          createdAt: new Date().toISOString(),
          updatedAt: new Date().toISOString(),
          instances: newInstances.map((inst, i) => ({
            instanceId: i,
            worktreePath: inst.worktreePath || '',
            branch: inst.worktreeBranch || '',
            agentName: inst.agentName || '',
            status: 'running' as const,
            tmuxSessionId: inst.tmuxSessionId || '',
            startTime: typeof inst.startTime === 'string' ? inst.startTime : (typeof inst.startTime === 'number' ? new Date(inst.startTime).toISOString() : new Date().toISOString()),
          })),
          chatHistory: [],
        });
      } catch (error) {
        console.error('Failed to create session:', error);
      }
    } else {
      console.warn('Skipping session creation: no valid workspace open');
    }

    setClaudeInstances(newInstances);
    setActiveCompetitionId(competitionId);
    setShowCompetitionDialog(false);
    setViewMode('multi-agent');
  };

  // Handle ensemble start (アンサンブルモード)
  const handleEnsembleStart = async (competitionId: string, instanceCount: number, task: string) => {
    const shortTask = shortenTaskDescription(task);
    const time = getCurrentTime();

    const newInstances: ClaudeCodeInstance[] = Array.from({ length: instanceCount }, (_, i) => ({
      id: `${competitionId}-${i}`,
      agentName: `🎯 ${shortTask} #${i + 1} (${time})`,
      task: task,
      status: 'running',
      output: '',
      startTime: new Date().toISOString(),
      tmuxSessionId: `claude-ens-${competitionId}-${i}`,
      worktreePath: `/tmp/worktree-ens-${competitionId}-${i}`,
      worktreeBranch: `claude-ensemble-${competitionId}-${i}`,
    }));

    // セッション履歴に保存 (only if valid workspace is open)
    if (workspacePath && isGitRepo) {
      try {
        await tauriApi.createSession(workspacePath, {
          id: competitionId,
          type: 'ensemble',
          task,
          status: 'running',
          createdAt: new Date().toISOString(),
          updatedAt: new Date().toISOString(),
          instances: newInstances.map((inst, i) => ({
            instanceId: i,
            worktreePath: inst.worktreePath || '',
            branch: inst.worktreeBranch || '',
            agentName: inst.agentName || '',
            status: 'running' as const,
            tmuxSessionId: inst.tmuxSessionId || '',
            startTime: typeof inst.startTime === 'string' ? inst.startTime : (typeof inst.startTime === 'number' ? new Date(inst.startTime).toISOString() : new Date().toISOString()),
          })),
          chatHistory: [],
        });
      } catch (error) {
        console.error('Failed to create session:', error);
      }
    } else {
      console.warn('Skipping session creation: no valid workspace open');
    }

    setClaudeInstances(newInstances);
    setActiveCompetitionId(competitionId);
    setShowEnsembleDialog(false);
    setViewMode('multi-agent');
  };

  // Handle debate start (ディベートモード)
  const handleDebateStart = async (result: { debateId: string; worktreePath: string; branch: string }, task: string) => {
    const shortTask = shortenTaskDescription(task);
    const time = getCurrentTime();

    // Debateモードの3つの役割を定義
    const debateRoles = [
      { name: '🏛️ 設計者 (Architect)', description: 'システム設計に焦点' },
      { name: '⚙️ 実務者 (Pragmatist)', description: '実装可能性に焦点' },
      { name: '💡 革新者 (Innovator)', description: '創造的解決策に焦点' }
    ];

    // セッション履歴に保存 (only if valid workspace is open)
    if (workspacePath && isGitRepo) {
      try {
        await tauriApi.createSession(workspacePath, {
          id: result.debateId,
          type: 'debate',
          task,
          status: 'running',
          createdAt: new Date().toISOString(),
          updatedAt: new Date().toISOString(),
          instances: debateRoles.map((role, i) => ({
            instanceId: i,
            worktreePath: result.worktreePath,
            branch: result.branch,
            agentName: `${role.name} - ${shortTask} (${time})`,
            status: 'running',
            tmuxSessionId: `debate-${result.debateId}`, // Debate全体で単一のセッションID
            startTime: new Date().toISOString(),
            output: '', // 初期化
            filesChanged: 0, // 統計情報を初期化
            linesAdded: 0,
            linesDeleted: 0,
          })),
          chatHistory: [],
        });
      } catch (error) {
        console.error('Failed to create debate session:', error);
        // Note: Do NOT alert user - sessions are optional for execution
        console.warn('Debate will run without session history (no valid workspace)');
      }
    } else {
      console.warn('Skipping session creation: no valid workspace open');
    }

    setDebateId(result.debateId);
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
          {/* Workspace indicator */}
          {isGitRepo && workspacePath && (
            <div className="flex items-center space-x-2 text-xs text-gray-400">
              <FolderOpen className="w-3 h-3" />
              <span>{workspacePath.split('/').pop()}</span>
            </div>
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
              onClick={() => setViewMode('session-history')}
              className={`px-3 py-1 rounded-md text-sm transition-colors ${
                viewMode === 'session-history'
                  ? 'bg-orange-600 text-white'
                  : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
              }`}
            >
              <LayoutDashboard className="w-4 h-4 inline-block mr-1" />
              ダッシュボード
            </button>
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
          </div>
        </div>

        <div className="flex items-center space-x-2">
          <button
            onClick={handleSelectWorkspace}
            className="p-2 rounded hover:bg-gray-700"
            title="プロジェクトフォルダを開く"
          >
            <FolderOpen className="w-4 h-4" />
          </button>
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

        {/* Session History View Mode */}
        {viewMode === 'session-history' && (
          <div className="flex-1 bg-gray-900 overflow-hidden">
            <SessionHistory />
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
