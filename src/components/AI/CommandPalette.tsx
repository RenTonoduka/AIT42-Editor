/**
 * CommandPalette - AI Agent Command Palette
 *
 * Cmd+K/Ctrl+K overlay for quick AI agent execution
 */

import React, { useState, useEffect, useRef } from 'react';
import { Search, Sparkles, X } from 'lucide-react';
import { tauriApi, AgentInfo, AgentExecutionResponse } from '@/services/tauri';

export interface CommandPaletteProps {
  /** Whether the palette is visible */
  isOpen: boolean;
  /** Callback when palette should close */
  onClose: () => void;
  /** Optional initial context (e.g., selected code) */
  initialContext?: string;
}

/**
 * CommandPalette component
 */
export const CommandPalette: React.FC<CommandPaletteProps> = ({
  isOpen,
  onClose,
  initialContext,
}) => {
  const [query, setQuery] = useState('');
  const [task, setTask] = useState('');
  const [agents, setAgents] = useState<AgentInfo[]>([]);
  const [filteredAgents, setFilteredAgents] = useState<AgentInfo[]>([]);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [isExecuting, setIsExecuting] = useState(false);
  const [executionResult, setExecutionResult] = useState<AgentExecutionResponse | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const taskInputRef = useRef<HTMLTextAreaElement>(null);

  // Load agents when palette opens
  useEffect(() => {
    if (isOpen) {
      loadAgents();
      setQuery('');
      setTask('');
      setExecutionResult(null);
      setSelectedIndex(0);
      // Focus input after a short delay
      setTimeout(() => {
        inputRef.current?.focus();
      }, 100);
    }
  }, [isOpen]);

  // Filter agents based on query
  useEffect(() => {
    if (query.trim() === '') {
      setFilteredAgents(agents);
    } else {
      const lowerQuery = query.toLowerCase();
      const filtered = agents.filter(
        (agent) =>
          agent.name.toLowerCase().includes(lowerQuery) ||
          agent.description.toLowerCase().includes(lowerQuery) ||
          agent.category.toLowerCase().includes(lowerQuery)
      );
      setFilteredAgents(filtered);
      setSelectedIndex(0);
    }
  }, [query, agents]);

  const loadAgents = async () => {
    try {
      const agentList = await tauriApi.listAgents();
      setAgents(agentList);
      setFilteredAgents(agentList);
    } catch (error) {
      console.error('Failed to load agents:', error);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Escape') {
      e.preventDefault();
      onClose();
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      setSelectedIndex((prev) => Math.min(prev + 1, filteredAgents.length - 1));
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      setSelectedIndex((prev) => Math.max(prev - 1, 0));
    } else if (e.key === 'Enter' && e.metaKey) {
      // Cmd+Enter to execute
      e.preventDefault();
      handleExecute();
    }
  };

  const handleExecute = async () => {
    if (filteredAgents.length === 0 || !task.trim()) return;

    const selectedAgent = filteredAgents[selectedIndex];
    setIsExecuting(true);

    try {
      const response = await tauriApi.executeAgent({
        agentName: selectedAgent.name,
        task: task.trim(),
        context: initialContext,
      });
      setExecutionResult(response);
    } catch (error) {
      console.error('Failed to execute agent:', error);
      setExecutionResult({
        executionId: '',
        agentName: selectedAgent.name,
        status: 'failed',
        error: String(error),
      });
    } finally {
      setIsExecuting(false);
    }
  };

  if (!isOpen) return null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-start justify-center pt-[20vh] bg-black/60 backdrop-blur-sm"
      onClick={onClose}
    >
      <div
        className="w-full max-w-2xl bg-editor-elevated border border-editor-border rounded-xl shadow-2xl overflow-hidden"
        onClick={(e) => e.stopPropagation()}
        onKeyDown={handleKeyDown}
      >
        {/* Header */}
        <div className="flex items-center gap-3 px-4 py-3 border-b border-editor-border bg-editor-surface">
          <Sparkles size={20} className="text-accent-primary" />
          <h2 className="text-sm font-semibold text-text-primary">AI Agent Command Palette</h2>
          <button
            onClick={onClose}
            className="ml-auto p-1 hover:bg-editor-border/30 rounded transition-colors"
            title="Close (Esc)"
          >
            <X size={16} className="text-text-tertiary" />
          </button>
        </div>

        {/* Search Input */}
        <div className="relative border-b border-editor-border">
          <Search
            size={18}
            className="absolute left-4 top-1/2 -translate-y-1/2 text-text-tertiary"
          />
          <input
            ref={inputRef}
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Search agents... (e.g., code-reviewer, test-generator)"
            className="w-full pl-12 pr-4 py-3 bg-editor-bg text-text-primary placeholder-text-tertiary focus:outline-none focus:ring-2 focus:ring-accent-primary/50"
          />
        </div>

        {/* Agent List */}
        <div className="max-h-64 overflow-y-auto">
          {filteredAgents.length === 0 ? (
            <div className="py-8 text-center text-text-tertiary text-sm">
              No agents found matching "{query}"
            </div>
          ) : (
            filteredAgents.map((agent, index) => (
              <div
                key={agent.name}
                className={`px-4 py-3 cursor-pointer transition-colors border-b border-editor-border/30 last:border-b-0 ${
                  index === selectedIndex
                    ? 'bg-accent-primary/10 border-l-2 border-l-accent-primary'
                    : 'hover:bg-editor-surface'
                }`}
                onClick={() => setSelectedIndex(index)}
              >
                <div className="flex items-start justify-between">
                  <div className="flex-1">
                    <div className="font-medium text-text-primary text-sm">{agent.name}</div>
                    <div className="text-xs text-text-secondary mt-1">{agent.description}</div>
                    <div className="flex items-center gap-2 mt-2">
                      <span className="text-xs px-2 py-0.5 bg-accent-secondary/20 text-accent-secondary rounded">
                        {agent.category}
                      </span>
                      <span className="text-xs text-text-tertiary">
                        {agent.tools.length} tools
                      </span>
                    </div>
                  </div>
                </div>
              </div>
            ))
          )}
        </div>

        {/* Task Input */}
        {filteredAgents.length > 0 && (
          <div className="border-t border-editor-border p-4">
            <label className="block text-xs font-medium text-text-secondary mb-2">
              Task Description
            </label>
            <textarea
              ref={taskInputRef}
              value={task}
              onChange={(e) => setTask(e.target.value)}
              placeholder="Describe what you want the agent to do..."
              className="w-full px-3 py-2 bg-editor-bg text-text-primary placeholder-text-tertiary border border-editor-border rounded focus:outline-none focus:ring-2 focus:ring-accent-primary/50 resize-none"
              rows={3}
            />
            <div className="flex items-center justify-between mt-3">
              <span className="text-xs text-text-tertiary">
                Press <kbd className="px-1.5 py-0.5 bg-editor-elevated rounded border border-editor-border">⌘</kbd> +{' '}
                <kbd className="px-1.5 py-0.5 bg-editor-elevated rounded border border-editor-border">Enter</kbd> to execute
              </span>
              <button
                onClick={handleExecute}
                disabled={!task.trim() || isExecuting}
                className="px-4 py-2 bg-accent-primary hover:bg-accent-primary/90 disabled:bg-editor-border disabled:text-text-tertiary text-white text-sm font-medium rounded transition-colors"
              >
                {isExecuting ? 'Executing...' : 'Execute Agent'}
              </button>
            </div>
          </div>
        )}

        {/* Execution Result */}
        {executionResult && (
          <div className="border-t border-editor-border p-4 bg-editor-surface">
            <div className="text-xs font-medium text-text-secondary mb-2">Result:</div>
            <div
              className={`p-3 rounded text-sm ${
                executionResult.status === 'completed'
                  ? 'bg-green-500/10 text-green-400'
                  : executionResult.status === 'failed'
                  ? 'bg-red-500/10 text-red-400'
                  : 'bg-blue-500/10 text-blue-400'
              }`}
            >
              {executionResult.output || executionResult.error || 'Processing...'}
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
