/**
 * Session History Component
 *
 * Main component for session history management with Kanban board
 * Combines filters, board, and detail view
 */
import React, { useState, useEffect } from 'react';
import { FolderOpen } from 'lucide-react';
import { KanbanBoard } from './KanbanBoard';
import { SessionFilters } from './SessionFilters';
import { SessionDetailView } from './SessionDetailView';
import { useSessionHistoryStore } from '@/store/sessionHistoryStore';
import { tauriApi } from '@/services/tauri';

export const SessionHistory: React.FC = () => {
  const [selectedSessionId, setSelectedSessionId] = useState<string | null>(null);
  const [isValidWorkspace, setIsValidWorkspace] = useState<boolean>(false);
  const workspacePath = useSessionHistoryStore((state) => state.workspacePath);

  // Check if workspace is valid (has .git directory)
  useEffect(() => {
    const checkWorkspace = async () => {
      try {
        const workspace = await tauriApi.getWorkspace();
        setIsValidWorkspace(workspace.is_git_repo);
      } catch (error) {
        setIsValidWorkspace(false);
      }
    };

    if (workspacePath) {
      checkWorkspace();
    } else {
      setIsValidWorkspace(false);
    }
  }, [workspacePath]);

  const handleSelectSession = (sessionId: string) => {
    setSelectedSessionId(sessionId);
  };

  const handleCloseDetail = () => {
    setSelectedSessionId(null);
  };

  // Show placeholder if no valid workspace is selected
  if (!workspacePath || !isValidWorkspace) {
    return (
      <div className="h-full flex items-center justify-center">
        <div className="text-center space-y-4 max-w-md px-8">
          <div className="inline-flex items-center justify-center w-20 h-20 rounded-full bg-editor-surface border-2 border-editor-border">
            <FolderOpen className="w-10 h-10 text-text-tertiary" />
          </div>
          <div className="space-y-2">
            <h3 className="text-xl font-semibold text-text-primary">
              プロジェクトフォルダを開いてください
            </h3>
            <p className="text-text-secondary">
              セッション履歴を表示するには、Gitリポジトリを含むプロジェクトフォルダを開いてください。
            </p>
          </div>
          <p className="text-sm text-text-tertiary">
            右上の「📁 フォルダを開く」ボタンからプロジェクトを選択できます
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col">
      {/* Filters */}
      <SessionFilters />

      {/* Kanban Board */}
      <div className="flex-1 overflow-hidden">
        <KanbanBoard onSelectSession={handleSelectSession} />
      </div>

      {/* Detail View Modal */}
      {selectedSessionId && (
        <SessionDetailView sessionId={selectedSessionId} onClose={handleCloseDetail} />
      )}
    </div>
  );
};
