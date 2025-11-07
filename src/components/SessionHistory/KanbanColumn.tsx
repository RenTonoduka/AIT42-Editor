/**
 * Kanban Column Component
 *
 * Represents a single column in the Kanban board with drop support
 * Inspired by Atlassian Design System's drag and drop patterns
 */
import React from 'react';
import { useDrop } from 'react-dnd';
import type { WorktreeSession, SessionStatus } from '@/types/worktree';
import { SessionCard } from './SessionCard';

interface KanbanColumnProps {
  title: string;
  status: SessionStatus;
  sessions: WorktreeSession[];
  onSelectSession: (sessionId: string) => void;
  onDropSession: (sessionId: string, newStatus: SessionStatus) => void;
}

const ITEM_TYPE = 'SESSION_CARD';

/**
 * Get column header color based on status
 */
const getColumnColor = (status: SessionStatus) => {
  switch (status) {
    case 'running':
      return 'bg-blue-50 border-blue-200';
    case 'completed':
      return 'bg-green-50 border-green-200';
    case 'failed':
      return 'bg-red-50 border-red-200';
    case 'paused':
      return 'bg-yellow-50 border-yellow-200';
    default:
      return 'bg-gray-50 border-gray-200';
  }
};

/**
 * Get column header text color
 */
const getColumnTextColor = (status: SessionStatus) => {
  switch (status) {
    case 'running':
      return 'text-blue-700';
    case 'completed':
      return 'text-green-700';
    case 'failed':
      return 'text-red-700';
    case 'paused':
      return 'text-yellow-700';
    default:
      return 'text-gray-700';
  }
};

export const KanbanColumn: React.FC<KanbanColumnProps> = ({
  title,
  status,
  sessions,
  onSelectSession,
  onDropSession,
}) => {
  const [{ isOver, canDrop }, drop] = useDrop(() => ({
    accept: ITEM_TYPE,
    drop: (item: { id: string; currentStatus: SessionStatus }) => {
      if (item.currentStatus !== status) {
        onDropSession(item.id, status);
      }
    },
    collect: (monitor) => ({
      isOver: monitor.isOver(),
      canDrop: monitor.canDrop(),
    }),
  }));

  const isActive = isOver && canDrop;

  return (
    <div className="flex-shrink-0 w-80">
      {/* Column Header */}
      <div
        className={`
          px-4 py-3 rounded-t-lg border-t border-x
          ${getColumnColor(status)}
        `}
      >
        <div className="flex items-center justify-between">
          <h2 className={`text-sm font-semibold ${getColumnTextColor(status)}`}>
            {title}
          </h2>
          <span
            className={`
              inline-flex items-center justify-center
              w-6 h-6 rounded-full text-xs font-medium
              ${getColumnColor(status)} ${getColumnTextColor(status)}
            `}
          >
            {sessions.length}
          </span>
        </div>
      </div>

      {/* Drop Zone */}
      <div
        ref={drop}
        className={`
          min-h-[calc(100vh-300px)] p-3
          bg-gray-50 rounded-b-lg border-x border-b border-gray-200
          transition-colors duration-200
          ${isActive ? 'bg-blue-50 border-blue-300' : ''}
          ${canDrop && !isOver ? 'border-dashed' : ''}
        `}
      >
        {/* Empty State */}
        {sessions.length === 0 && (
          <div className="flex items-center justify-center h-32 text-gray-400 text-sm">
            {canDrop && isOver ? 'Drop here' : 'No sessions'}
          </div>
        )}

        {/* Session Cards */}
        <div className="space-y-3">
          {sessions.map((session) => (
            <SessionCard
              key={session.id}
              session={session}
              onSelect={onSelectSession}
            />
          ))}
        </div>
      </div>
    </div>
  );
};
