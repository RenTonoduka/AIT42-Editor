/**
 * Session Card Component
 *
 * Displays a session card in the Kanban board with drag support
 * Inspired by Vibe Kanban's task card design
 */
import React from 'react';
import { useDrag } from 'react-dnd';
import type { WorktreeSession } from '@/types/worktree';
import { Clock, Users, FileText, CheckCircle, AlertCircle, Loader } from 'lucide-react';

interface SessionCardProps {
  session: WorktreeSession;
  onSelect: (sessionId: string) => void;
}

const ITEM_TYPE = 'SESSION_CARD';

/**
 * Get status icon based on session status
 */
const getStatusIcon = (status: string) => {
  switch (status) {
    case 'completed':
      return <CheckCircle className="w-4 h-4 text-green-500" />;
    case 'failed':
      return <AlertCircle className="w-4 h-4 text-red-500" />;
    case 'running':
      return <Loader className="w-4 h-4 text-blue-500 animate-spin" />;
    case 'paused':
      return <Clock className="w-4 h-4 text-yellow-500" />;
    default:
      return <Clock className="w-4 h-4 text-gray-400" />;
  }
};

/**
 * Get session type badge color
 */
const getTypeBadgeColor = (type: string) => {
  switch (type) {
    case 'competition':
      return 'bg-purple-100 text-purple-800 border-purple-200';
    case 'ensemble':
      return 'bg-blue-100 text-blue-800 border-blue-200';
    case 'debate':
      return 'bg-green-100 text-green-800 border-green-200';
    default:
      return 'bg-gray-100 text-gray-800 border-gray-200';
  }
};

/**
 * Format duration in human-readable format
 */
const formatDuration = (seconds?: number): string => {
  if (!seconds) return 'N/A';

  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const secs = seconds % 60;

  if (hours > 0) {
    return `${hours}h ${minutes}m`;
  } else if (minutes > 0) {
    return `${minutes}m ${secs}s`;
  } else {
    return `${secs}s`;
  }
};

/**
 * Format date to relative time or absolute
 */
const formatDate = (dateStr: string): string => {
  const date = new Date(dateStr);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMins = Math.floor(diffMs / 60000);

  if (diffMins < 1) return 'Just now';
  if (diffMins < 60) return `${diffMins}m ago`;
  if (diffMins < 1440) return `${Math.floor(diffMins / 60)}h ago`;
  if (diffMins < 10080) return `${Math.floor(diffMins / 1440)}d ago`;

  return date.toLocaleDateString();
};

export const SessionCard: React.FC<SessionCardProps> = ({ session, onSelect }) => {
  const [{ isDragging }, drag] = useDrag(() => ({
    type: ITEM_TYPE,
    item: { id: session.id, currentStatus: session.status },
    collect: (monitor) => ({
      isDragging: monitor.isDragging(),
    }),
  }));

  const handleClick = () => {
    onSelect(session.id);
  };

  return (
    <div
      ref={drag}
      onClick={handleClick}
      className={`
        bg-white rounded-lg border shadow-sm p-4 mb-3
        cursor-pointer transition-all duration-200
        hover:shadow-md hover:border-blue-300
        ${isDragging ? 'opacity-50 cursor-grabbing' : 'cursor-grab'}
      `}
    >
      {/* Header: Type Badge + Status Icon */}
      <div className="flex items-start justify-between mb-3">
        <span
          className={`
            inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium
            border ${getTypeBadgeColor(session.type)}
          `}
        >
          {session.type.toUpperCase()}
        </span>
        {getStatusIcon(session.status)}
      </div>

      {/* Task Title */}
      <h3 className="text-sm font-semibold text-gray-900 mb-2 line-clamp-2">
        {session.task}
      </h3>

      {/* Metadata */}
      <div className="space-y-2 text-xs text-gray-600">
        {/* Instances Count */}
        <div className="flex items-center gap-1.5">
          <Users className="w-3.5 h-3.5" />
          <span>{session.instances.length} instances</span>
        </div>

        {/* Files Changed */}
        {session.totalFilesChanged !== undefined && (
          <div className="flex items-center gap-1.5">
            <FileText className="w-3.5 h-3.5" />
            <span>{session.totalFilesChanged} files changed</span>
          </div>
        )}

        {/* Duration */}
        {session.totalDuration !== undefined && (
          <div className="flex items-center gap-1.5">
            <Clock className="w-3.5 h-3.5" />
            <span>{formatDuration(session.totalDuration)}</span>
          </div>
        )}
      </div>

      {/* Footer: Updated Time */}
      <div className="mt-3 pt-3 border-t border-gray-100">
        <p className="text-xs text-gray-500">
          Updated {formatDate(session.updatedAt)}
        </p>
      </div>
    </div>
  );
};
