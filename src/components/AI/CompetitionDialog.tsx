/**
 * CompetitionDialog - AI Agent Competition Mode
 *
 * Allows users to run competitions between multiple AI agents
 * and compare their performance on the same task.
 */

import React, { useState, useEffect } from 'react';
import { Trophy, X, Plus, Trash2, Settings as SettingsIcon } from 'lucide-react';
import { tauriApi, AgentInfo } from '@/services/tauri';

export interface CompetitionDialogProps {
  /** Whether the dialog is visible */
  isOpen: boolean;
  /** Callback when dialog should close */
  onClose: () => void;
  /** Callback when competition starts */
  onStart?: (competitionId: string) => void;
}

/**
 * CompetitionDialog component
 */
export const CompetitionDialog: React.FC<CompetitionDialogProps> = ({
  isOpen,
  onClose,
  onStart,
}) => {
  const [task, setTask] = useState('');
  const [agents, setAgents] = useState<AgentInfo[]>([]);
  const [selectedAgents, setSelectedAgents] = useState<string[]>([]);
  const [concurrency, setConcurrency] = useState(3);
  const [timeout, setTimeout] = useState(300);
  const [preserve, setPreserve] = useState(false);
  const [showAdvanced, setShowAdvanced] = useState(false);
  const [isStarting, setIsStarting] = useState(false);

  // Load agents when dialog opens
  useEffect(() => {
    if (isOpen) {
      loadAgents();
      setTask('');
      setSelectedAgents([]);
      setShowAdvanced(false);
    }
  }, [isOpen]);

  const loadAgents = async () => {
    try {
      const agentList = await tauriApi.listAgents();
      setAgents(agentList);
    } catch (error) {
      console.error('Failed to load agents:', error);
    }
  };

  const toggleAgent = (agentName: string) => {
    setSelectedAgents((prev) =>
      prev.includes(agentName)
        ? prev.filter((name) => name !== agentName)
        : [...prev, agentName]
    );
  };

  const handleStart = async () => {
    if (!task.trim() || selectedAgents.length < 2) {
      alert('Please enter a task and select at least 2 agents');
      return;
    }

    setIsStarting(true);
    try {
      const responses = await tauriApi.executeParallel({
        agents: selectedAgents,
        task: task.trim(),
        context: undefined,
      });

      console.log('Competition started:', responses);

      // Generate competition ID from first execution
      const competitionId = responses[0]?.execution_id || Date.now().toString();

      if (onStart) {
        onStart(competitionId);
      }

      onClose();
    } catch (error) {
      console.error('Failed to start competition:', error);
      alert(`Failed to start competition: ${error}`);
    } finally {
      setIsStarting(false);
    }
  };

  if (!isOpen) return null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm"
      onClick={onClose}
    >
      <div
        className="w-full max-w-4xl max-h-[85vh] bg-editor-elevated border border-editor-border rounded-xl shadow-2xl overflow-hidden flex flex-col"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center gap-3 px-6 py-4 border-b border-editor-border bg-editor-surface">
          <Trophy size={24} className="text-accent-primary" />
          <h2 className="text-lg font-semibold text-text-primary">AI Agent Competition</h2>
          <button
            onClick={onClose}
            className="ml-auto p-1 hover:bg-editor-border/30 rounded transition-colors"
            title="Close (Esc)"
          >
            <X size={20} className="text-text-tertiary" />
          </button>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto p-6 space-y-6">
          {/* Task Input */}
          <div>
            <label className="block text-sm font-medium text-text-primary mb-2">
              Task Description
            </label>
            <textarea
              value={task}
              onChange={(e) => setTask(e.target.value)}
              placeholder="Describe the task for agents to compete on... (e.g., 'Implement user authentication with JWT')"
              className="w-full px-4 py-3 bg-editor-bg text-text-primary placeholder-text-tertiary border border-editor-border rounded-lg focus:outline-none focus:ring-2 focus:ring-accent-primary/50 resize-none"
              rows={3}
            />
          </div>

          {/* Agent Selection */}
          <div>
            <div className="flex items-center justify-between mb-3">
              <label className="text-sm font-medium text-text-primary">
                Select Agents ({selectedAgents.length} selected)
              </label>
              <button
                onClick={() => setSelectedAgents(agents.map((a) => a.name))}
                className="text-xs text-accent-primary hover:text-accent-secondary transition-colors"
              >
                Select All
              </button>
            </div>
            <div className="grid grid-cols-2 gap-3 max-h-64 overflow-y-auto p-2 bg-editor-bg rounded-lg border border-editor-border">
              {agents.map((agent) => (
                <div
                  key={agent.name}
                  onClick={() => toggleAgent(agent.name)}
                  className={`p-3 rounded-lg border-2 cursor-pointer transition-all ${
                    selectedAgents.includes(agent.name)
                      ? 'border-accent-primary bg-accent-primary/10'
                      : 'border-editor-border hover:border-editor-border/60 bg-editor-surface'
                  }`}
                >
                  <div className="flex items-start justify-between">
                    <div className="flex-1">
                      <div className="font-medium text-sm text-text-primary">{agent.name}</div>
                      <div className="text-xs text-text-secondary mt-1 line-clamp-2">
                        {agent.description}
                      </div>
                      <div className="text-xs text-text-tertiary mt-2">
                        <span className="px-2 py-0.5 bg-editor-border/30 rounded">
                          {agent.category}
                        </span>
                      </div>
                    </div>
                    {selectedAgents.includes(agent.name) && (
                      <div className="flex-shrink-0 w-5 h-5 bg-accent-primary rounded-full flex items-center justify-center ml-2">
                        <svg className="w-3 h-3 text-white" fill="currentColor" viewBox="0 0 20 20">
                          <path
                            fillRule="evenodd"
                            d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                            clipRule="evenodd"
                          />
                        </svg>
                      </div>
                    )}
                  </div>
                </div>
              ))}
            </div>
          </div>

          {/* Advanced Options */}
          <div>
            <button
              onClick={() => setShowAdvanced(!showAdvanced)}
              className="flex items-center gap-2 text-sm text-text-secondary hover:text-text-primary transition-colors"
            >
              <SettingsIcon size={16} />
              {showAdvanced ? 'Hide' : 'Show'} Advanced Options
            </button>

            {showAdvanced && (
              <div className="mt-4 p-4 bg-editor-bg rounded-lg border border-editor-border space-y-4">
                {/* Concurrency */}
                <div>
                  <label className="block text-xs font-medium text-text-secondary mb-2">
                    Concurrency (max parallel agents)
                  </label>
                  <input
                    type="number"
                    min="1"
                    max="10"
                    value={concurrency}
                    onChange={(e) => setConcurrency(parseInt(e.target.value, 10))}
                    className="w-full px-3 py-2 bg-editor-surface text-text-primary border border-editor-border rounded focus:outline-none focus:ring-2 focus:ring-accent-primary/50"
                  />
                  <div className="text-xs text-text-tertiary mt-1">
                    Higher values = faster but more resource-intensive
                  </div>
                </div>

                {/* Timeout */}
                <div>
                  <label className="block text-xs font-medium text-text-secondary mb-2">
                    Timeout (seconds per agent)
                  </label>
                  <input
                    type="number"
                    min="60"
                    max="3600"
                    step="60"
                    value={timeout}
                    onChange={(e) => setTimeout(parseInt(e.target.value, 10))}
                    className="w-full px-3 py-2 bg-editor-surface text-text-primary border border-editor-border rounded focus:outline-none focus:ring-2 focus:ring-accent-primary/50"
                  />
                  <div className="text-xs text-text-tertiary mt-1">
                    {timeout} seconds = {Math.floor(timeout / 60)} minutes
                  </div>
                </div>

                {/* Preserve */}
                <div className="flex items-start gap-3">
                  <input
                    type="checkbox"
                    checked={preserve}
                    onChange={(e) => setPreserve(e.target.checked)}
                    className="mt-1"
                  />
                  <div className="flex-1">
                    <label className="text-xs font-medium text-text-secondary">
                      Preserve Artifacts
                    </label>
                    <div className="text-xs text-text-tertiary mt-1">
                      Keep Git worktrees and output files for later inspection
                    </div>
                  </div>
                </div>
              </div>
            )}
          </div>
        </div>

        {/* Footer */}
        <div className="flex items-center justify-between px-6 py-4 border-t border-editor-border bg-editor-surface">
          <div className="text-sm text-text-tertiary">
            {selectedAgents.length < 2 && 'Select at least 2 agents to start'}
            {selectedAgents.length >= 2 &&
              `Ready to compete ${selectedAgents.length} agents`}
          </div>
          <div className="flex gap-3">
            <button
              onClick={onClose}
              className="px-4 py-2 text-text-secondary hover:text-text-primary transition-colors"
            >
              Cancel
            </button>
            <button
              onClick={handleStart}
              disabled={!task.trim() || selectedAgents.length < 2 || isStarting}
              className="px-6 py-2 bg-gradient-to-r from-accent-primary to-accent-secondary hover:from-accent-secondary hover:to-accent-primary disabled:from-editor-border disabled:to-editor-border disabled:text-text-tertiary text-white font-semibold rounded-lg transition-all shadow-glow-sm hover:shadow-glow-md"
            >
              {isStarting ? 'Starting...' : '🏆 Start Competition'}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};
