/**
 * Session Detail View Component
 *
 * Comprehensive detail view with tabs for different aspects of the session
 * Integrates WorktreeExplorer, metrics, and chat (Phase 4)
 */
import React, { useState, useEffect } from 'react';
import { useSessionHistoryStore } from '@/store/sessionHistoryStore';
import { useWorktreeStore } from '@/store/worktreeStore';
import type { WorktreeSession, WorktreeInstance } from '@/types/worktree';
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
import { WinnerSelectionPanel } from './WinnerSelectionPanel';

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
  const { getSession, sessions, isLoading } = useSessionHistoryStore();

  // Get session from store (reactive to changes)
  const session = sessions.find((s) => s.id === sessionId) || null;

  // Load session details on mount if not in store
  useEffect(() => {
    if (!session) {
      const loadSession = async () => {
        await getSession(sessionId);
      };
      loadSession();
    }
  }, [sessionId, session, getSession]);

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

              {/* Display instance output if available */}
              {instance.output && instance.output.trim() && (
                <details className="mt-3">
                  <summary className="text-sm font-medium text-blue-600 cursor-pointer hover:text-blue-800">
                    View Output ({instance.output.length} characters)
                  </summary>
                  <pre className="mt-2 p-3 bg-gray-50 border border-gray-200 rounded text-xs font-mono overflow-x-auto max-h-64 overflow-y-auto whitespace-pre-wrap">
                    {instance.output}
                  </pre>
                </details>
              )}
            </div>
          ))}
        </div>
      </div>

      {/* Winner Selection Panel (for competition mode) */}
      {session.type === 'competition' && session.status === 'completed' && (
        <WinnerSelectionPanel session={session} />
      )}
    </div>
  );
};

/**
 * Worktrees Tab - Displays session instances (worktrees)
 */
const WorktreesTab: React.FC<{ session: WorktreeSession }> = ({ session }) => {
  const { setWorktrees } = useWorktreeStore();

  // Sync session instances to worktreeStore for File Browser
  React.useEffect(() => {
    if (session.instances.length > 0) {
      const worktrees = session.instances.map((instance) => ({
        id: `${session.id}-${instance.instanceId}`,
        competition_id: session.id,
        path: instance.worktreePath,
        branch: instance.branch || `instance-${instance.instanceId}`,
        created_at: instance.startTime || session.createdAt,
        changed_files: instance.filesChanged || 0,
        lines_added: instance.linesAdded || 0,
        lines_deleted: instance.linesDeleted || 0,
        status: instance.status as 'running' | 'completed' | 'failed',
      }));

      console.log('[WorktreesTab] Setting worktrees from session instances:', {
        sessionId: session.id,
        instanceCount: session.instances.length,
        worktrees: worktrees,
      });

      setWorktrees(worktrees);
    } else {
      console.log('[WorktreesTab] No instances in session:', session.id);
    }
  }, [session.instances, session.id, session.createdAt, setWorktrees]);

  if (session.instances.length === 0) {
    return (
      <div className="p-6 text-center">
        <Users className="w-16 h-16 mx-auto mb-4 text-gray-400" />
        <p className="text-lg font-medium text-gray-600 mb-2">No Worktrees Found</p>
        <p className="text-sm text-gray-500">
          Instances will appear here when the session starts
        </p>
      </div>
    );
  }

  return (
    <div className="p-6 space-y-4">
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {session.instances.map((instance) => (
          <div
            key={instance.instanceId}
            className="bg-white border-2 border-gray-200 rounded-lg p-5 hover:shadow-lg transition-all hover:border-blue-300"
          >
            {/* Header */}
            <div className="flex items-start justify-between mb-4">
              <div className="flex items-center gap-3">
                <span className="inline-flex items-center justify-center w-10 h-10 rounded-full bg-gradient-to-br from-blue-500 to-blue-600 text-white text-lg font-bold shadow-md">
                  {instance.instanceId}
                </span>
                <div>
                  <div className="font-semibold text-gray-900">
                    {instance.runtimeLabel || instance.agentName}
                  </div>
                  <div className="text-xs text-gray-500">Instance #{instance.instanceId}</div>
                </div>
              </div>
              <span
                className={`
                  px-3 py-1 rounded-full text-xs font-semibold
                  ${
                    instance.status === 'completed'
                      ? 'bg-green-100 text-green-800'
                      : instance.status === 'failed'
                      ? 'bg-red-100 text-red-800'
                      : instance.status === 'running'
                      ? 'bg-blue-100 text-blue-800 animate-pulse'
                      : 'bg-gray-100 text-gray-800'
                  }
                `}
              >
                {instance.status}
              </span>
            </div>

            {/* Details */}
            <div className="space-y-2">
              {instance.branch && (
                <div className="flex items-center gap-2 text-sm">
                  <span className="text-gray-500 font-medium">Branch:</span>
                  <code className="px-2 py-0.5 bg-gray-100 rounded text-xs font-mono text-gray-700">
                    {instance.branch}
                  </code>
                </div>
              )}

              {instance.worktreePath && (
                <div className="flex items-start gap-2 text-sm">
                  <span className="text-gray-500 font-medium min-w-fit">Path:</span>
                  <code className="px-2 py-0.5 bg-gray-100 rounded text-xs font-mono text-gray-700 break-all">
                    {instance.worktreePath}
                  </code>
                </div>
              )}

              {instance.model && (
                <div className="flex items-center gap-2 text-sm">
                  <span className="text-gray-500 font-medium">Model:</span>
                  <span className="px-2 py-0.5 bg-purple-100 rounded text-xs font-medium text-purple-700">
                    {instance.model}
                  </span>
                </div>
              )}

              {instance.runtime && (
                <div className="flex items-center gap-2 text-sm">
                  <span className="text-gray-500 font-medium">Runtime:</span>
                  <span className="px-2 py-0.5 bg-blue-100 rounded text-xs font-medium text-blue-700">
                    {instance.runtime}
                  </span>
                </div>
              )}

              {instance.tmuxSessionId && (
                <div className="flex items-center gap-2 text-sm">
                  <span className="text-gray-500 font-medium">Tmux:</span>
                  <code className="px-2 py-0.5 bg-gray-100 rounded text-xs font-mono text-gray-700">
                    {instance.tmuxSessionId}
                  </code>
                </div>
              )}

              {instance.startTime && (
                <div className="flex items-center gap-2 text-sm">
                  <Calendar className="w-4 h-4 text-gray-400" />
                  <span className="text-xs text-gray-600">
                    {formatDate(instance.startTime)}
                  </span>
                </div>
              )}
            </div>

            {/* Stats */}
            {(instance.filesChanged !== undefined ||
              instance.linesAdded !== undefined ||
              instance.linesDeleted !== undefined) && (
              <div className="mt-4 pt-4 border-t border-gray-200">
                <div className="flex items-center justify-between text-sm">
                  {instance.filesChanged !== undefined && (
                    <span className="text-gray-600">
                      <strong className="text-gray-900">{instance.filesChanged}</strong> files
                    </span>
                  )}
                  <div className="flex items-center gap-3">
                    {instance.linesAdded !== undefined && (
                      <span className="text-green-600 font-mono">
                        +{instance.linesAdded}
                      </span>
                    )}
                    {instance.linesDeleted !== undefined && (
                      <span className="text-red-600 font-mono">
                        -{instance.linesDeleted}
                      </span>
                    )}
                  </div>
                </div>
              </div>
            )}
          </div>
        ))}
      </div>

      {/* Legacy WorktreeExplorer (for file browsing) */}
      <div className="mt-8 pt-8 border-t border-gray-300">
        <h3 className="text-lg font-semibold text-gray-900 mb-4">File Browser</h3>
        <WorktreeExplorer competitionId={session.id} />
      </div>
    </div>
  );
};

/**
 * Metrics Tab - Phase 1: Basic metrics visualization
 */
const MetricsTab: React.FC<{ session: WorktreeSession }> = ({ session }) => {
  // Calculate derived metrics
  const completedInstances = session.instances.filter(
    (i) => i.status === 'completed'
  ).length;
  const failedInstances = session.instances.filter((i) => i.status === 'failed').length;
  const successRate =
    session.instances.length > 0
      ? Math.round((completedInstances / session.instances.length) * 100)
      : 0;

  // Calculate average duration
  const instancesWithDuration = session.instances.filter(
    (i) => i.startTime && i.endTime
  );
  const avgDuration =
    instancesWithDuration.length > 0
      ? instancesWithDuration.reduce((sum, instance) => {
          const start = new Date(instance.startTime!).getTime();
          const end = new Date(instance.endTime!).getTime();
          return sum + (end - start) / 1000;
        }, 0) / instancesWithDuration.length
      : 0;

  // Add Winner Selection Panel for Competition mode
  if (session.type === 'competition') {
    return (
      <div className="space-y-6">
        <WinnerSelectionPanel session={session} />

        {/* Existing Metrics Content */}
        <div className="border-t pt-6 px-6">
          <MetricsContent
            session={session}
            completedInstances={completedInstances}
            failedInstances={failedInstances}
            successRate={successRate}
            avgDuration={avgDuration}
          />
        </div>
      </div>
    );
  }

  // For non-competition modes, show regular metrics
  return (
    <div className="p-6 space-y-6">
      <MetricsContent
        session={session}
        completedInstances={completedInstances}
        failedInstances={failedInstances}
        successRate={successRate}
        avgDuration={avgDuration}
      />
    </div>
  );
};

/**
 * Metrics Content Component (extracted for reuse)
 */
interface MetricsContentProps {
  session: WorktreeSession;
  completedInstances: number;
  failedInstances: number;
  successRate: number;
  avgDuration: number;
}

const MetricsContent: React.FC<MetricsContentProps> = ({
  session,
  completedInstances,
  failedInstances,
  successRate,
  avgDuration,
}) => {
  return (
    <div className="space-y-6">
      {/* Summary Cards */}
      <div>
        <h3 className="text-lg font-semibold text-gray-900 mb-4">Session Summary</h3>
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          <MetricCard
            title="Total Duration"
            value={formatDuration(session.totalDuration)}
            icon={<Clock className="w-5 h-5" />}
            color="blue"
          />
          <MetricCard
            title="Files Changed"
            value={session.totalFilesChanged || 0}
            icon={<FileText className="w-5 h-5" />}
            color="purple"
          />
          <MetricCard
            title="Lines Added"
            value={`+${session.totalLinesAdded || 0}`}
            icon={<CheckCircle className="w-5 h-5" />}
            color="green"
          />
          <MetricCard
            title="Lines Deleted"
            value={`-${session.totalLinesDeleted || 0}`}
            icon={<AlertCircle className="w-5 h-5" />}
            color="red"
          />
          <MetricCard
            title="Total Instances"
            value={session.instances.length}
            icon={<Users className="w-5 h-5" />}
            color="indigo"
          />
          <MetricCard
            title="Success Rate"
            value={`${successRate}%`}
            icon={<BarChart3 className="w-5 h-5" />}
            color="emerald"
          />
          <MetricCard
            title="Completed"
            value={completedInstances}
            icon={<CheckCircle className="w-5 h-5" />}
            color="green"
          />
          <MetricCard
            title="Failed"
            value={failedInstances}
            icon={<AlertCircle className="w-5 h-5" />}
            color="red"
          />
        </div>
      </div>

      {/* Instance Performance Comparison */}
      <div>
        <h3 className="text-lg font-semibold text-gray-900 mb-4">
          Instance Performance Comparison
        </h3>
        <InstanceMetricsTable instances={session.instances} avgDuration={avgDuration} />
      </div>

      {/* Code Changes Visualization */}
      <div>
        <h3 className="text-lg font-semibold text-gray-900 mb-4">
          Code Changes Distribution
        </h3>
        <CodeChangesChart instances={session.instances} />
      </div>

      {/* Future: Advanced Charts */}
      <div className="mt-8 p-4 bg-gray-50 border border-gray-200 rounded-lg">
        <p className="text-sm text-gray-600">
          <strong>Phase 2 (Coming Soon):</strong> Timeline charts, heatmaps, and advanced
          visualizations will be added here.
        </p>
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

  return <ChatPanel key={session.id} session={session} />;
};

/**
 * Metric Card Component
 */
interface MetricCardProps {
  title: string;
  value: string | number;
  icon: React.ReactNode;
  color: 'blue' | 'purple' | 'green' | 'red' | 'indigo' | 'emerald' | 'yellow';
}

const MetricCard: React.FC<MetricCardProps> = ({ title, value, icon, color }) => {
  const colorClasses = {
    blue: 'bg-blue-50 border-blue-200 text-blue-600',
    purple: 'bg-purple-50 border-purple-200 text-purple-600',
    green: 'bg-green-50 border-green-200 text-green-600',
    red: 'bg-red-50 border-red-200 text-red-600',
    indigo: 'bg-indigo-50 border-indigo-200 text-indigo-600',
    emerald: 'bg-emerald-50 border-emerald-200 text-emerald-600',
    yellow: 'bg-yellow-50 border-yellow-200 text-yellow-600',
  };

  const textColorClasses = {
    blue: 'text-blue-900',
    purple: 'text-purple-900',
    green: 'text-green-900',
    red: 'text-red-900',
    indigo: 'text-indigo-900',
    emerald: 'text-emerald-900',
    yellow: 'text-yellow-900',
  };

  return (
    <div className={`${colorClasses[color]} border rounded-lg p-4`}>
      <div className="flex items-center gap-2 mb-2">
        {icon}
        <div className="text-sm font-medium">{title}</div>
      </div>
      <div className={`text-2xl font-bold ${textColorClasses[color]}`}>{value}</div>
    </div>
  );
};

/**
 * Instance Metrics Table Component
 */
interface InstanceMetricsTableProps {
  instances: WorktreeInstance[];
  avgDuration: number;
}

const InstanceMetricsTable: React.FC<InstanceMetricsTableProps> = ({
  instances,
  avgDuration,
}) => {
  // Calculate instance duration
  const getInstanceDuration = (instance: WorktreeInstance): number | null => {
    if (!instance.startTime || !instance.endTime) return null;
    const start = new Date(instance.startTime).getTime();
    const end = new Date(instance.endTime).getTime();
    return (end - start) / 1000;
  };

  // Sort instances by performance (files changed desc)
  const sortedInstances = [...instances].sort((a, b) => {
    const aFiles = a.filesChanged || 0;
    const bFiles = b.filesChanged || 0;
    return bFiles - aFiles;
  });

  return (
    <div className="overflow-x-auto">
      <table className="min-w-full divide-y divide-gray-200 border border-gray-200 rounded-lg">
        <thead className="bg-gray-50">
          <tr>
            <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Instance
            </th>
            <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Status
            </th>
            <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Duration
            </th>
            <th className="px-4 py-3 text-center text-xs font-medium text-gray-500 uppercase tracking-wider">
              Files
            </th>
            <th className="px-4 py-3 text-center text-xs font-medium text-gray-500 uppercase tracking-wider">
              Lines Added
            </th>
            <th className="px-4 py-3 text-center text-xs font-medium text-gray-500 uppercase tracking-wider">
              Lines Deleted
            </th>
            <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Runtime
            </th>
          </tr>
        </thead>
        <tbody className="bg-white divide-y divide-gray-200">
          {sortedInstances.map((instance) => {
            const duration = getInstanceDuration(instance);
            const isFaster = duration !== null && avgDuration > 0 && duration < avgDuration;
            const isSlower = duration !== null && avgDuration > 0 && duration > avgDuration;

            return (
              <tr key={instance.instanceId} className="hover:bg-gray-50">
                <td className="px-4 py-3 whitespace-nowrap">
                  <div className="flex items-center gap-2">
                    <span className="inline-flex items-center justify-center w-6 h-6 rounded-full bg-blue-100 text-blue-700 text-xs font-semibold">
                      {instance.instanceId}
                    </span>
                    <span className="text-sm font-medium text-gray-900">
                      {instance.agentName}
                    </span>
                  </div>
                </td>
                <td className="px-4 py-3 whitespace-nowrap">
                  <span
                    className={`
                      px-2 py-1 rounded-full text-xs font-medium
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
                </td>
                <td className="px-4 py-3 whitespace-nowrap">
                  <div className="flex items-center gap-1">
                    <span className="text-sm text-gray-900">
                      {duration !== null ? formatDuration(duration) : 'N/A'}
                    </span>
                    {isFaster && (
                      <span className="text-xs text-green-600 font-medium">▼</span>
                    )}
                    {isSlower && <span className="text-xs text-red-600 font-medium">▲</span>}
                  </div>
                </td>
                <td className="px-4 py-3 text-center">
                  <span className="text-sm font-semibold text-gray-900">
                    {instance.filesChanged !== undefined ? instance.filesChanged : '-'}
                  </span>
                </td>
                <td className="px-4 py-3 text-center">
                  <span className="text-sm font-mono text-green-600">
                    {instance.linesAdded !== undefined ? `+${instance.linesAdded}` : '-'}
                  </span>
                </td>
                <td className="px-4 py-3 text-center">
                  <span className="text-sm font-mono text-red-600">
                    {instance.linesDeleted !== undefined ? `-${instance.linesDeleted}` : '-'}
                  </span>
                </td>
                <td className="px-4 py-3 whitespace-nowrap">
                  <div className="flex items-center gap-2">
                    {instance.runtime && (
                      <span className="px-2 py-0.5 bg-blue-100 rounded text-xs font-medium text-blue-700">
                        {instance.runtime}
                      </span>
                    )}
                    {instance.model && (
                      <span className="text-xs text-gray-500">{instance.model}</span>
                    )}
                  </div>
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>

      {avgDuration > 0 && (
        <div className="mt-2 text-xs text-gray-500">
          Average Duration: <strong>{formatDuration(avgDuration)}</strong>
          <span className="ml-3">
            ▼ = Faster than average | ▲ = Slower than average
          </span>
        </div>
      )}
    </div>
  );
};

/**
 * Code Changes Chart Component - Visualizes code changes per instance
 * Uses pure CSS for bar chart visualization (no external chart libraries)
 */
interface CodeChangesChartProps {
  instances: WorktreeInstance[];
}

const CodeChangesChart: React.FC<CodeChangesChartProps> = ({ instances }) => {
  // Find max values for scaling
  const maxFiles = Math.max(
    ...instances.map((i) => i.filesChanged || 0),
    1 // Avoid divide by zero
  );
  const maxLinesAdded = Math.max(...instances.map((i) => i.linesAdded || 0), 1);
  const maxLinesDeleted = Math.max(...instances.map((i) => i.linesDeleted || 0), 1);

  // Sort instances by instanceId for consistent display
  const sortedInstances = [...instances].sort((a, b) => a.instanceId - b.instanceId);

  return (
    <div className="space-y-6">
      {/* Files Changed Chart */}
      <div>
        <h4 className="text-sm font-semibold text-gray-700 mb-3">Files Changed</h4>
        <div className="space-y-2">
          {sortedInstances.map((instance) => {
            const files = instance.filesChanged || 0;
            const widthPercent = (files / maxFiles) * 100;

            return (
              <div key={`files-${instance.instanceId}`} className="flex items-center gap-3">
                <span className="inline-flex items-center justify-center w-6 h-6 rounded-full bg-blue-100 text-blue-700 text-xs font-semibold flex-shrink-0">
                  {instance.instanceId}
                </span>
                <div className="flex-1">
                  <div className="relative h-8 bg-gray-100 rounded overflow-hidden">
                    <div
                      className="h-full bg-gradient-to-r from-purple-500 to-purple-600 transition-all duration-500 flex items-center justify-end px-2"
                      style={{ width: `${widthPercent}%` }}
                    >
                      {files > 0 && (
                        <span className="text-white text-xs font-semibold">{files}</span>
                      )}
                    </div>
                  </div>
                </div>
                <span className="text-sm text-gray-600 w-20 text-right">
                  {instance.agentName.slice(0, 10)}
                </span>
              </div>
            );
          })}
        </div>
      </div>

      {/* Lines Added/Deleted Chart */}
      <div>
        <h4 className="text-sm font-semibold text-gray-700 mb-3">
          Lines Added vs Deleted
        </h4>
        <div className="space-y-3">
          {sortedInstances.map((instance) => {
            const added = instance.linesAdded || 0;
            const deleted = instance.linesDeleted || 0;
            const addedPercent = (added / maxLinesAdded) * 100;
            const deletedPercent = (deleted / maxLinesDeleted) * 100;

            return (
              <div key={`lines-${instance.instanceId}`} className="space-y-1">
                <div className="flex items-center gap-2">
                  <span className="inline-flex items-center justify-center w-6 h-6 rounded-full bg-blue-100 text-blue-700 text-xs font-semibold flex-shrink-0">
                    {instance.instanceId}
                  </span>
                  <span className="text-sm text-gray-700 font-medium">
                    {instance.agentName}
                  </span>
                </div>

                {/* Added Lines Bar */}
                <div className="flex items-center gap-2 ml-8">
                  <span className="text-xs text-green-600 w-12">+Added</span>
                  <div className="flex-1">
                    <div className="relative h-6 bg-gray-100 rounded overflow-hidden">
                      <div
                        className="h-full bg-gradient-to-r from-green-500 to-green-600 transition-all duration-500 flex items-center justify-end px-2"
                        style={{ width: `${addedPercent}%` }}
                      >
                        {added > 0 && (
                          <span className="text-white text-xs font-semibold">
                            +{added}
                          </span>
                        )}
                      </div>
                    </div>
                  </div>
                </div>

                {/* Deleted Lines Bar */}
                <div className="flex items-center gap-2 ml-8">
                  <span className="text-xs text-red-600 w-12">-Deleted</span>
                  <div className="flex-1">
                    <div className="relative h-6 bg-gray-100 rounded overflow-hidden">
                      <div
                        className="h-full bg-gradient-to-r from-red-500 to-red-600 transition-all duration-500 flex items-center justify-end px-2"
                        style={{ width: `${deletedPercent}%` }}
                      >
                        {deleted > 0 && (
                          <span className="text-white text-xs font-semibold">
                            -{deleted}
                          </span>
                        )}
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {/* Net Change Summary */}
      <div className="bg-gradient-to-r from-blue-50 to-purple-50 border border-blue-200 rounded-lg p-4">
        <h4 className="text-sm font-semibold text-gray-700 mb-2">Net Change Summary</h4>
        <div className="grid grid-cols-3 gap-4 text-center">
          <div>
            <div className="text-2xl font-bold text-purple-600">
              {instances.reduce((sum, i) => sum + (i.filesChanged || 0), 0)}
            </div>
            <div className="text-xs text-gray-600">Total Files</div>
          </div>
          <div>
            <div className="text-2xl font-bold text-green-600">
              +{instances.reduce((sum, i) => sum + (i.linesAdded || 0), 0)}
            </div>
            <div className="text-xs text-gray-600">Total Added</div>
          </div>
          <div>
            <div className="text-2xl font-bold text-red-600">
              -{instances.reduce((sum, i) => sum + (i.linesDeleted || 0), 0)}
            </div>
            <div className="text-xs text-gray-600">Total Deleted</div>
          </div>
        </div>
      </div>
    </div>
  );
};
