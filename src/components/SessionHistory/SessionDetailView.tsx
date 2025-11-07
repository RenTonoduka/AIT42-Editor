/**
 * Session Detail View Component
 *
 * Comprehensive detail view with tabs for different aspects of the session
 * Integrates WorktreeExplorer, metrics, and chat (Phase 4)
 */
import React, { useState, useEffect } from 'react';
import { useSessionHistoryStore } from '@/store/sessionHistoryStore';
import type { WorktreeSession } from '@/types/worktree';
import {
  X,
  FileText,
  BarChart3,
  MessageSquare,
  Clock,
  Users,
  CheckCircle,
  AlertCircle,
  Loader,
  Calendar,
} from 'lucide-react';
import { WorktreeExplorer } from '@/components/Worktree/WorktreeExplorer';

interface SessionDetailViewProps {
  sessionId: string;
  onClose: () => void;
}

type TabType = 'overview' | 'worktrees' | 'metrics' | 'chat';

const TABS: { id: TabType; label: string; icon: React.ReactNode }[] = [
  { id: 'overview', label: 'Overview', icon: <FileText className="w-4 h-4" /> },
  { id: 'worktrees', label: 'Worktrees', icon: <Users className="w-4 h-4" /> },
  { id: 'metrics', label: 'Metrics', icon: <BarChart3 className="w-4 h-4" /> },
  { id: 'chat', label: 'Chat', icon: <MessageSquare className="w-4 h-4" /> },
];

/**
 * Get status icon based on session status
 */
const getStatusIcon = (status: string) => {
  switch (status) {
    case 'completed':
      return <CheckCircle className="w-5 h-5 text-green-500" />;
    case 'failed':
      return <AlertCircle className="w-5 h-5 text-red-500" />;
    case 'running':
      return <Loader className="w-5 h-5 text-blue-500 animate-spin" />;
    case 'paused':
      return <Clock className="w-5 h-5 text-yellow-500" />;
    default:
      return <Clock className="w-5 h-5 text-gray-400" />;
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

  const parts = [];
  if (hours > 0) parts.push(`${hours}h`);
  if (minutes > 0) parts.push(`${minutes}m`);
  if (secs > 0 || parts.length === 0) parts.push(`${secs}s`);

  return parts.join(' ');
};

/**
 * Format date to readable string
 */
const formatDate = (dateStr: string): string => {
  const date = new Date(dateStr);
  return date.toLocaleString();
};

export const SessionDetailView: React.FC<SessionDetailViewProps> = ({
  sessionId,
  onClose,
}) => {
  const [activeTab, setActiveTab] = useState<TabType>('overview');
  const [session, setSession] = useState<WorktreeSession | null>(null);
  const { getSession, isLoading } = useSessionHistoryStore();

  // Load session details
  useEffect(() => {
    const loadSession = async () => {
      const sessionData = await getSession(sessionId);
      if (sessionData) {
        setSession(sessionData);
      }
    };

    loadSession();
  }, [sessionId, getSession]);

  if (isLoading || !session) {
    return (
      <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
        <div className="bg-white rounded-lg shadow-xl w-4/5 h-4/5 flex items-center justify-center">
          <div className="text-center">
            <Loader className="w-8 h-8 text-blue-500 animate-spin mx-auto mb-2" />
            <p className="text-gray-600">Loading session details...</p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg shadow-xl w-[90%] h-[90%] flex flex-col">
        {/* Header */}
        <div className="flex items-start justify-between px-6 py-4 border-b bg-gray-50">
          <div className="flex-1">
            <div className="flex items-center gap-3 mb-2">
              {getStatusIcon(session.status)}
              <h2 className="text-xl font-semibold text-gray-900">{session.task}</h2>
            </div>
            <div className="flex items-center gap-4 text-sm text-gray-600">
              <span className="inline-flex items-center gap-1">
                <Calendar className="w-4 h-4" />
                {formatDate(session.createdAt)}
              </span>
              <span className="inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full bg-purple-100 text-purple-800 text-xs font-medium">
                {session.type.toUpperCase()}
              </span>
              <span className="inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full bg-gray-100 text-gray-800 text-xs font-medium">
                {session.status.toUpperCase()}
              </span>
            </div>
          </div>
          <button
            onClick={onClose}
            className="p-2 hover:bg-gray-200 rounded-lg transition-colors"
          >
            <X className="w-5 h-5 text-gray-500" />
          </button>
        </div>

        {/* Tabs */}
        <div className="border-b">
          <div className="flex gap-1 px-6">
            {TABS.map((tab) => (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id)}
                className={`
                  flex items-center gap-2 px-4 py-3 text-sm font-medium
                  border-b-2 transition-colors
                  ${
                    activeTab === tab.id
                      ? 'border-blue-500 text-blue-600'
                      : 'border-transparent text-gray-600 hover:text-gray-900 hover:border-gray-300'
                  }
                `}
              >
                {tab.icon}
                {tab.label}
              </button>
            ))}
          </div>
        </div>

        {/* Tab Content */}
        <div className="flex-1 overflow-auto">
          {activeTab === 'overview' && <OverviewTab session={session} />}
          {activeTab === 'worktrees' && <WorktreesTab session={session} />}
          {activeTab === 'metrics' && <MetricsTab session={session} />}
          {activeTab === 'chat' && <ChatTab session={session} />}
        </div>
      </div>
    </div>
  );
};

/**
 * Overview Tab
 */
const OverviewTab: React.FC<{ session: WorktreeSession }> = ({ session }) => {
  return (
    <div className="p-6 space-y-6">
      {/* Summary Cards */}
      <div className="grid grid-cols-4 gap-4">
        <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
          <div className="text-blue-600 text-sm font-medium mb-1">Instances</div>
          <div className="text-2xl font-bold text-blue-900">{session.instances.length}</div>
        </div>

        <div className="bg-green-50 border border-green-200 rounded-lg p-4">
          <div className="text-green-600 text-sm font-medium mb-1">Duration</div>
          <div className="text-2xl font-bold text-green-900">
            {formatDuration(session.totalDuration)}
          </div>
        </div>

        <div className="bg-purple-50 border border-purple-200 rounded-lg p-4">
          <div className="text-purple-600 text-sm font-medium mb-1">Files Changed</div>
          <div className="text-2xl font-bold text-purple-900">
            {session.totalFilesChanged || 0}
          </div>
        </div>

        <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4">
          <div className="text-yellow-600 text-sm font-medium mb-1">Lines Modified</div>
          <div className="text-2xl font-bold text-yellow-900">
            +{session.totalLinesAdded || 0} -{session.totalLinesDeleted || 0}
          </div>
        </div>
      </div>

      {/* Instance List */}
      <div>
        <h3 className="text-lg font-semibold text-gray-900 mb-4">Instances</h3>
        <div className="space-y-3">
          {session.instances.map((instance) => (
            <div
              key={instance.instanceId}
              className="bg-white border rounded-lg p-4 hover:shadow-md transition-shadow"
            >
              <div className="flex items-center justify-between mb-2">
                <div className="flex items-center gap-3">
                  <span className="inline-flex items-center justify-center w-8 h-8 rounded-full bg-blue-100 text-blue-700 text-sm font-semibold">
                    {instance.instanceId}
                  </span>
                  <div>
                    <div className="font-medium text-gray-900">{instance.agentName}</div>
                    <div className="text-sm text-gray-600">{instance.branch}</div>
                  </div>
                </div>
                <span
                  className={`
                    px-2.5 py-0.5 rounded-full text-xs font-medium
                    ${
                      instance.status === 'completed'
                        ? 'bg-green-100 text-green-800'
                        : instance.status === 'failed'
                        ? 'bg-red-100 text-red-800'
                        : instance.status === 'running'
                        ? 'bg-blue-100 text-blue-800'
                        : 'bg-gray-100 text-gray-800'
                    }
                  `}
                >
                  {instance.status}
                </span>
              </div>

              {instance.filesChanged !== undefined && (
                <div className="text-sm text-gray-600 mt-2">
                  <span className="mr-4">
                    Files: <strong>{instance.filesChanged}</strong>
                  </span>
                  {instance.linesAdded !== undefined && (
                    <span className="text-green-600">
                      +{instance.linesAdded}
                    </span>
                  )}
                  {instance.linesDeleted !== undefined && (
                    <span className="text-red-600 ml-2">
                      -{instance.linesDeleted}
                    </span>
                  )}
                </div>
              )}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};

/**
 * Worktrees Tab - Integrates existing WorktreeExplorer
 */
const WorktreesTab: React.FC<{ session: WorktreeSession }> = ({ session }) => {
  return (
    <div className="h-full">
      <WorktreeExplorer />
    </div>
  );
};

/**
 * Metrics Tab
 */
const MetricsTab: React.FC<{ session: WorktreeSession }> = ({ session }) => {
  return (
    <div className="p-6">
      <div className="text-center text-gray-500">
        <BarChart3 className="w-16 h-16 mx-auto mb-4 text-gray-400" />
        <p className="text-lg font-medium mb-2">Metrics Visualization</p>
        <p className="text-sm">
          Detailed charts and statistics will be implemented here:
        </p>
        <ul className="text-sm mt-4 space-y-2 text-left max-w-md mx-auto">
          <li>• Performance comparison across instances</li>
          <li>• Code change timeline</li>
          <li>• Execution time breakdown</li>
          <li>• File modification heatmap</li>
        </ul>
      </div>
    </div>
  );
};

/**
 * Chat Tab - Interactive chat with Claude Code instances
 */
const ChatTab: React.FC<{ session: WorktreeSession }> = ({ session }) => {
  // Lazy import to avoid circular dependencies
  const [ChatPanel, setChatPanel] = React.useState<React.ComponentType<any> | null>(null);

  React.useEffect(() => {
    import('./ChatPanel').then((mod) => {
      setChatPanel(() => mod.ChatPanel);
    });
  }, []);

  if (!ChatPanel) {
    return (
      <div className="flex items-center justify-center h-full">
        <Loader className="w-8 h-8 text-blue-500 animate-spin" />
      </div>
    );
  }

  return <ChatPanel session={session} />;
};
