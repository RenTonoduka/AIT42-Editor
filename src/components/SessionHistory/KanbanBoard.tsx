/**
 * Kanban Board Component
 *
 * Main Kanban board layout with drag & drop support
 * Inspired by Vibe Kanban's visual design and Atlassian's interaction patterns
 */
import React, { useEffect } from 'react';
import { DndProvider } from 'react-dnd';
import { HTML5Backend } from 'react-dnd-html5-backend';
import { useSessionHistoryStore } from '@/store/sessionHistoryStore';
import type { SessionStatus, KanbanColumn as KanbanColumnType } from '@/types/worktree';
import { KanbanColumn } from './KanbanColumn';
import { Loader, RefreshCw } from 'lucide-react';

interface KanbanBoardProps {
  onSelectSession: (sessionId: string) => void;
}

/**
 * Define Kanban columns
 */
const KANBAN_COLUMNS: KanbanColumnType[] = [
  {
    id: 'running',
    title: '実行中',
    status: 'running',
    sessions: [],
  },
  {
    id: 'paused',
    title: '一時停止',
    status: 'paused',
    sessions: [],
  },
  {
    id: 'completed',
    title: '完了',
    status: 'completed',
    sessions: [],
  },
  {
    id: 'failed',
    title: '失敗',
    status: 'failed',
    sessions: [],
  },
];

export const KanbanBoard: React.FC<KanbanBoardProps> = ({ onSelectSession }) => {
  const { sessions, isLoading, error, loadSessions, updateSession, getFilteredSessions } =
    useSessionHistoryStore();

  // Load sessions on mount
  useEffect(() => {
    loadSessions();
  }, [loadSessions]);

  /**
   * Handle session drop on column
   */
  const handleDropSession = async (sessionId: string, newStatus: SessionStatus) => {
    const session = sessions.find((s) => s.id === sessionId);
    if (!session) return;

    // Update session status
    const updatedSession = {
      ...session,
      status: newStatus,
      updatedAt: new Date().toISOString(),
      completedAt: newStatus === 'completed' ? new Date().toISOString() : session.completedAt,
    };

    try {
      await updateSession(updatedSession);
    } catch (error) {
      console.error('Failed to update session status:', error);
    }
  };

  /**
   * Refresh sessions manually
   */
  const handleRefresh = () => {
    loadSessions();
  };

  /**
   * Group sessions by status
   */
  const groupSessionsByStatus = () => {
    const filtered = getFilteredSessions();
    const grouped = new Map<SessionStatus, typeof sessions>();

    // Initialize with empty arrays
    KANBAN_COLUMNS.forEach((col) => {
      grouped.set(col.status, []);
    });

    // Group sessions
    filtered.forEach((session) => {
      const statusSessions = grouped.get(session.status) || [];
      statusSessions.push(session);
      grouped.set(session.status, statusSessions);
    });

    return grouped;
  };

  const groupedSessions = groupSessionsByStatus();

  // Loading state
  if (isLoading && sessions.length === 0) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-center">
          <Loader className="w-8 h-8 text-blue-500 animate-spin mx-auto mb-2" />
          <p className="text-gray-600">セッション読み込み中...</p>
        </div>
      </div>
    );
  }

  // Error state
  if (error && sessions.length === 0) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-center">
          <p className="text-red-600 mb-4">{error}</p>
          <button
            onClick={handleRefresh}
            className="px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition-colors"
          >
            再試行
          </button>
        </div>
      </div>
    );
  }

  return (
    <DndProvider backend={HTML5Backend}>
      <div className="h-full flex flex-col bg-gray-100">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 bg-white border-b">
          <div>
            <h1 className="text-xl font-semibold text-gray-900">セッション履歴</h1>
            <p className="text-sm text-gray-600 mt-0.5">
              合計 {sessions.length} セッション
            </p>
          </div>

          <button
            onClick={handleRefresh}
            disabled={isLoading}
            className="
              flex items-center gap-2 px-4 py-2
              bg-blue-500 text-white rounded-lg
              hover:bg-blue-600 disabled:opacity-50
              transition-colors
            "
          >
            <RefreshCw className={`w-4 h-4 ${isLoading ? 'animate-spin' : ''}`} />
            更新
          </button>
        </div>

        {/* Kanban Board */}
        <div className="flex-1 overflow-x-auto overflow-y-hidden">
          <div className="flex gap-4 p-6 h-full min-w-max">
            {KANBAN_COLUMNS.map((column) => (
              <KanbanColumn
                key={column.id}
                title={column.title}
                status={column.status}
                sessions={groupedSessions.get(column.status) || []}
                onSelectSession={onSelectSession}
                onDropSession={handleDropSession}
              />
            ))}
          </div>
        </div>
      </div>
    </DndProvider>
  );
};
