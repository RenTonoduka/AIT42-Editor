/**
 * Git Panel Component
 *
 * Provides Git interface with:
 * - Branch management
 * - File status and staging
 * - Commit interface
 * - Push/pull operations
 * - Commit history
 */

import React, { useState, useEffect } from 'react';
import {
  GitBranch,
  GitCommit as GitCommitIcon,
  Upload,
  Download,
  Plus,
  Minus,
  X,
  RefreshCw,
  Check,
  FileText,
} from 'lucide-react';
import { useGitStore, type GitFileStatus } from '@/store/gitStore';

/**
 * Branch selector and management
 */
const BranchSelector: React.FC = () => {
  const { status, branches, fetchBranches, checkoutBranch, createBranch } =
    useGitStore();
  const [showBranchList, setShowBranchList] = useState(false);
  const [showNewBranch, setShowNewBranch] = useState(false);
  const [newBranchName, setNewBranchName] = useState('');

  useEffect(() => {
    fetchBranches();
  }, [fetchBranches]);

  const handleCheckout = async (branch: string) => {
    try {
      await checkoutBranch(branch);
      setShowBranchList(false);
    } catch (error) {
      console.error('Failed to checkout branch:', error);
    }
  };

  const handleCreateBranch = async () => {
    if (!newBranchName.trim()) return;
    try {
      await createBranch(newBranchName.trim());
      setNewBranchName('');
      setShowNewBranch(false);
    } catch (error) {
      console.error('Failed to create branch:', error);
    }
  };

  return (
    <div className="relative">
      <button
        className="flex items-center gap-2 w-full px-3 py-2 hover:bg-[#2A2D2E] transition-colors"
        onClick={() => setShowBranchList(!showBranchList)}
      >
        <GitBranch size={16} className="text-[#4EC9B0]" />
        <span className="text-[13px] text-[#CCCCCC] flex-1 text-left truncate">
          {status?.branch || 'No branch'}
        </span>
        {status && (
          <div className="flex items-center gap-2 text-[11px] text-[#858585]">
            {status.ahead > 0 && (
              <span className="flex items-center gap-1">
                <Upload size={12} />
                {status.ahead}
              </span>
            )}
            {status.behind > 0 && (
              <span className="flex items-center gap-1">
                <Download size={12} />
                {status.behind}
              </span>
            )}
          </div>
        )}
      </button>

      {showBranchList && (
        <div className="absolute top-full left-0 right-0 mt-1 bg-[#2D2D30] border border-[#454545] rounded shadow-lg z-50 max-h-64 overflow-y-auto">
          <div className="p-2 border-b border-[#454545]">
            <button
              className="w-full px-2 py-1.5 text-[12px] text-[#4EC9B0] hover:bg-[#3E3E42] rounded transition-colors text-left"
              onClick={() => {
                setShowNewBranch(true);
                setShowBranchList(false);
              }}
            >
              + Create new branch
            </button>
          </div>
          {branches.map((branch) => (
            <button
              key={branch}
              className={`w-full px-3 py-1.5 text-[12px] hover:bg-[#3E3E42] transition-colors text-left ${
                branch === status?.branch
                  ? 'text-[#4EC9B0] font-semibold'
                  : 'text-[#CCCCCC]'
              }`}
              onClick={() => handleCheckout(branch)}
            >
              {branch === status?.branch && (
                <Check size={12} className="inline mr-1" />
              )}
              {branch}
            </button>
          ))}
        </div>
      )}

      {showNewBranch && (
        <div className="absolute top-full left-0 right-0 mt-1 bg-[#2D2D30] border border-[#454545] rounded shadow-lg z-50 p-3">
          <div className="flex items-center gap-2 mb-2">
            <input
              type="text"
              className="flex-1 px-2 py-1 bg-[#3E3E42] text-[#CCCCCC] text-[12px] rounded border border-[#454545] focus:border-[#007ACC] focus:outline-none"
              placeholder="Branch name"
              value={newBranchName}
              onChange={(e) => setNewBranchName(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter') handleCreateBranch();
                if (e.key === 'Escape') {
                  setShowNewBranch(false);
                  setNewBranchName('');
                }
              }}
              autoFocus
            />
          </div>
          <div className="flex items-center gap-2">
            <button
              className="flex-1 px-2 py-1 bg-[#0E639C] hover:bg-[#1177BB] text-white text-[11px] rounded transition-colors"
              onClick={handleCreateBranch}
            >
              Create
            </button>
            <button
              className="px-2 py-1 hover:bg-[#3E3E42] text-[#CCCCCC] text-[11px] rounded transition-colors"
              onClick={() => {
                setShowNewBranch(false);
                setNewBranchName('');
              }}
            >
              Cancel
            </button>
          </div>
        </div>
      )}
    </div>
  );
};

/**
 * File item in changes list
 */
const FileItem: React.FC<{
  file: GitFileStatus;
  onStage?: () => void;
  onUnstage?: () => void;
}> = ({ file, onStage, onUnstage }) => {
  const getStatusColor = (status: string) => {
    switch (status) {
      case 'modified':
        return 'text-[#CE9178]';
      case 'added':
        return 'text-[#4EC9B0]';
      case 'deleted':
        return 'text-[#F48771]';
      case 'untracked':
        return 'text-[#858585]';
      default:
        return 'text-[#CCCCCC]';
    }
  };

  const getStatusLabel = (status: string) => {
    switch (status) {
      case 'modified':
        return 'M';
      case 'added':
        return 'A';
      case 'deleted':
        return 'D';
      case 'untracked':
        return 'U';
      case 'renamed':
        return 'R';
      default:
        return '?';
    }
  };

  const fileName = file.path.split('/').pop() || file.path;
  const folderPath = file.path.substring(0, file.path.length - fileName.length);

  return (
    <div className="flex items-center gap-2 px-3 py-1.5 hover:bg-[#2A2D2E] group transition-colors">
      <span
        className={`text-[11px] font-semibold w-4 ${getStatusColor(
          file.status
        )}`}
      >
        {getStatusLabel(file.status)}
      </span>
      <FileText size={14} className="text-[#858585]" />
      <div className="flex-1 min-w-0">
        <div className="text-[12px] text-[#CCCCCC] truncate">{fileName}</div>
        {folderPath && (
          <div className="text-[11px] text-[#858585] truncate">{folderPath}</div>
        )}
      </div>
      {onStage && (
        <button
          className="opacity-0 group-hover:opacity-100 p-1 hover:bg-[#3E3E42] rounded transition-all"
          onClick={onStage}
          title="Stage changes"
        >
          <Plus size={14} className="text-[#4EC9B0]" />
        </button>
      )}
      {onUnstage && (
        <button
          className="opacity-0 group-hover:opacity-100 p-1 hover:bg-[#3E3E42] rounded transition-all"
          onClick={onUnstage}
          title="Unstage changes"
        >
          <Minus size={14} className="text-[#F48771]" />
        </button>
      )}
    </div>
  );
};

/**
 * Changes view with staging
 */
const ChangesView: React.FC = () => {
  const { status, addFiles } = useGitStore();
  const [commitMessage, setCommitMessage] = useState('');
  const { commit, push, pull, fetchStatus } = useGitStore();

  const handleStage = async (filePath: string) => {
    try {
      await addFiles([filePath]);
    } catch (error) {
      console.error('Failed to stage file:', error);
    }
  };

  const handleStageAll = async () => {
    if (!status?.files.length) return;
    try {
      await addFiles(status.files.map((f) => f.path));
    } catch (error) {
      console.error('Failed to stage all files:', error);
    }
  };

  const handleCommit = async () => {
    if (!commitMessage.trim()) return;
    try {
      await commit(commitMessage);
      setCommitMessage('');
    } catch (error) {
      console.error('Failed to commit:', error);
    }
  };

  const handlePush = async () => {
    try {
      await push();
    } catch (error) {
      console.error('Failed to push:', error);
    }
  };

  const handlePull = async () => {
    try {
      await pull();
    } catch (error) {
      console.error('Failed to pull:', error);
    }
  };

  const handleRefresh = async () => {
    try {
      await fetchStatus();
    } catch (error) {
      console.error('Failed to refresh status:', error);
    }
  };

  if (!status) {
    return (
      <div className="flex items-center justify-center h-full text-[#858585] text-[12px]">
        No Git repository
      </div>
    );
  }

  const changedFiles = status.files || [];
  const hasChanges = changedFiles.length > 0;

  return (
    <div className="flex flex-col h-full">
      {/* Actions bar */}
      <div className="flex items-center gap-2 px-3 py-2 border-b border-[#2A2D2E]">
        <button
          className="p-1.5 hover:bg-[#3E3E42] rounded transition-colors"
          onClick={handleRefresh}
          title="Refresh"
        >
          <RefreshCw size={14} className="text-[#CCCCCC]" />
        </button>
        <div className="flex-1" />
        {hasChanges && (
          <button
            className="p-1.5 hover:bg-[#3E3E42] rounded transition-colors"
            onClick={handleStageAll}
            title="Stage All Changes"
          >
            <Plus size={14} className="text-[#4EC9B0]" />
          </button>
        )}
        <button
          className="p-1.5 hover:bg-[#3E3E42] rounded transition-colors"
          onClick={handlePull}
          title="Pull"
        >
          <Download size={14} className="text-[#4EC9B0]" />
        </button>
        <button
          className="p-1.5 hover:bg-[#3E3E42] rounded transition-colors"
          onClick={handlePush}
          title="Push"
        >
          <Upload size={14} className="text-[#4EC9B0]" />
        </button>
      </div>

      {/* Commit message */}
      <div className="px-3 py-2 border-b border-[#2A2D2E]">
        <textarea
          className="w-full px-2 py-1.5 bg-[#3E3E42] text-[#CCCCCC] text-[12px] rounded border border-[#454545] focus:border-[#007ACC] focus:outline-none resize-none"
          placeholder="Commit message (Ctrl+Enter to commit)"
          rows={3}
          value={commitMessage}
          onChange={(e) => setCommitMessage(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) {
              handleCommit();
            }
          }}
        />
        <button
          className="mt-2 w-full px-3 py-1.5 bg-[#0E639C] hover:bg-[#1177BB] text-white text-[12px] rounded transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          onClick={handleCommit}
          disabled={!commitMessage.trim() || !hasChanges}
        >
          <GitCommitIcon size={14} className="inline mr-1" />
          Commit
        </button>
      </div>

      {/* Changed files */}
      <div className="flex-1 overflow-y-auto">
        {!hasChanges ? (
          <div className="flex items-center justify-center h-full text-[#858585] text-[12px]">
            No changes
          </div>
        ) : (
          <div>
            <div className="px-3 py-2 text-[11px] text-[#858585] font-semibold uppercase">
              Changes ({changedFiles.length})
            </div>
            {changedFiles.map((file) => (
              <FileItem
                key={file.path}
                file={file}
                onStage={() => handleStage(file.path)}
              />
            ))}
          </div>
        )}
      </div>
    </div>
  );
};

/**
 * Commit history view
 */
const HistoryView: React.FC = () => {
  const { commits, fetchCommits } = useGitStore();

  useEffect(() => {
    fetchCommits(50);
  }, [fetchCommits]);

  if (commits.length === 0) {
    return (
      <div className="flex items-center justify-center h-full text-[#858585] text-[12px]">
        No commits
      </div>
    );
  }

  return (
    <div className="overflow-y-auto">
      {commits.map((commit) => {
        const date = new Date(commit.timestamp * 1000);
        const dateStr = date.toLocaleDateString();
        const timeStr = date.toLocaleTimeString();

        return (
          <div
            key={commit.sha}
            className="px-3 py-2 border-b border-[#2A2D2E] hover:bg-[#2A2D2E] transition-colors"
          >
            <div className="flex items-center gap-2 mb-1">
              <GitCommitIcon size={12} className="text-[#4EC9B0]" />
              <span className="text-[11px] text-[#858585] font-mono">
                {commit.sha.substring(0, 7)}
              </span>
              <span className="text-[11px] text-[#858585]">
                {dateStr} {timeStr}
              </span>
            </div>
            <div className="text-[12px] text-[#CCCCCC] mb-1">
              {commit.message}
            </div>
            <div className="text-[11px] text-[#858585]">
              {commit.author} &lt;{commit.email}&gt;
            </div>
          </div>
        );
      })}
    </div>
  );
};

/**
 * Git Panel Component
 */
export const GitPanel: React.FC = () => {
  const { showGitPanel, hideGit, fetchStatus } = useGitStore();
  const [activeTab, setActiveTab] = useState<'changes' | 'history'>('changes');

  useEffect(() => {
    if (showGitPanel) {
      fetchStatus();
    }
  }, [showGitPanel, fetchStatus]);

  if (!showGitPanel) {
    return null;
  }

  return (
    <div className="h-full flex flex-col bg-[#1E1E1E] text-[#CCCCCC] border-t border-[#2A2D2E]">
      {/* Header */}
      <div className="flex items-center justify-between border-b border-[#2A2D2E]">
        <h3 className="text-[13px] font-semibold px-3 py-2">SOURCE CONTROL</h3>
        <button
          className="p-2 hover:bg-[#3E3E42] rounded transition-colors mx-1"
          onClick={hideGit}
          title="Close source control panel"
        >
          <X size={16} />
        </button>
      </div>

      {/* Branch selector */}
      <BranchSelector />

      {/* Tabs */}
      <div className="flex items-center border-b border-[#2A2D2E]">
        <button
          className={`px-3 py-2 text-[12px] transition-colors ${
            activeTab === 'changes'
              ? 'text-[#CCCCCC] border-b-2 border-[#007ACC]'
              : 'text-[#858585] hover:text-[#CCCCCC]'
          }`}
          onClick={() => setActiveTab('changes')}
        >
          Changes
        </button>
        <button
          className={`px-3 py-2 text-[12px] transition-colors ${
            activeTab === 'history'
              ? 'text-[#CCCCCC] border-b-2 border-[#007ACC]'
              : 'text-[#858585] hover:text-[#CCCCCC]'
          }`}
          onClick={() => setActiveTab('history')}
        >
          History
        </button>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-hidden">
        {activeTab === 'changes' && <ChangesView />}
        {activeTab === 'history' && <HistoryView />}
      </div>
    </div>
  );
};
