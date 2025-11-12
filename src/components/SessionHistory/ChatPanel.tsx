/**
 * Chat Panel Component
 *
 * Interactive chat interface for communicating with Claude Code instances
 * Supports message history, tmux integration, and real-time output
 * Includes split view with terminal integration
 */
import React, { useState, useRef, useEffect } from 'react';
import { useSessionHistoryStore } from '@/store/sessionHistoryStore';
import { tauriApi } from '@/services/tauri';
import type { WorktreeSession, ChatMessage } from '@/types/worktree';
import { Send, Terminal, User, Bot, AlertCircle, Loader, MessageSquare, SplitSquareHorizontal, MonitorPlay, AlertTriangle } from 'lucide-react';
import { getRuntimeDefinition } from '@/config/runtimes';
import { TerminalView } from './TerminalView';

interface ChatPanelProps {
  session: WorktreeSession;
}

type ViewMode = 'chat' | 'terminal' | 'split';

export const ChatPanel: React.FC<ChatPanelProps> = ({ session }) => {
  const [message, setMessage] = useState('');
  const [selectedInstanceId, setSelectedInstanceId] = useState<number | null>(
    session.instances.length > 0 ? session.instances[0].instanceId : null
  );
  const [isSending, setIsSending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [isLoadingHistory, setIsLoadingHistory] = useState(false);
  const [viewMode, setViewMode] = useState<ViewMode>('chat');
  const [splitPosition, setSplitPosition] = useState(50); // Percentage for split view
  const [sessionAlive, setSessionAlive] = useState(true);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const splitContainerRef = useRef<HTMLDivElement>(null);
  const isDraggingRef = useRef(false);
  const { addChatMessage } = useSessionHistoryStore();

  /**
   * Get selected instance - MUST be defined before useEffect that uses it
   */
  const selectedInstance = session.instances.find(
    (i) => i.instanceId === selectedInstanceId
  );

  /**
   * Check if error indicates session termination
   */
  const isSessionTerminationError = (error: any): boolean => {
    const errorMessage = error?.message?.toLowerCase() || String(error).toLowerCase();
    return (
      errorMessage.includes('session not found') ||
      errorMessage.includes('no session') ||
      errorMessage.includes('can\'t find session') ||
      errorMessage.includes('session has been deleted') ||
      errorMessage.includes('no such session')
    );
  };

  // Auto-scroll to bottom when messages change
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [session.chatHistory]);

  /**
   * Load tmux session history when component mounts or instance changes
   */
  useEffect(() => {
    const loadTmuxHistory = async () => {
      console.log('[ChatPanel] Selected instance:', selectedInstance);
      console.log('[ChatPanel] Tmux session ID:', selectedInstance?.tmuxSessionId);

      if (!selectedInstance?.tmuxSessionId) {
        console.log('[ChatPanel] No tmux session ID found, skipping history load');
        return;
      }

      console.log('[ChatPanel] Loading tmux history for session:', selectedInstance.tmuxSessionId);
      setIsLoadingHistory(true);
      try {
        // Capture current tmux output
        const output = await tauriApi.captureTmuxOutput(selectedInstance.tmuxSessionId);
        console.log('[ChatPanel] Tmux output length:', output?.length || 0);

        if (output && output.trim()) {
          // Check if this history is already in chat
          const historyExists = session.chatHistory.some(
            (msg) =>
              msg.instanceId === selectedInstanceId &&
              msg.role === 'assistant' &&
              msg.content === output
          );

          if (!historyExists) {
            // Add history as system message
            const historyMessage: ChatMessage = {
              id: `history-${selectedInstanceId}-${Date.now()}`,
              role: 'system',
              content: `📜 Tmux Session History:\n\n${output}`,
              timestamp: new Date().toISOString(),
              instanceId: selectedInstanceId || undefined,
            };

            console.log('[ChatPanel] Adding history message to session:', session.id, historyMessage);
            await addChatMessage(session.id, historyMessage);
            console.log('[ChatPanel] History message added successfully');
          } else {
            console.log('[ChatPanel] History already exists, skipping');
          }
        }
      } catch (err) {
        console.error('Failed to load tmux history:', err);
      } finally {
        setIsLoadingHistory(false);
      }
    };

    loadTmuxHistory();
  }, [selectedInstanceId, selectedInstance?.tmuxSessionId]);

  /**
   * Get messages for selected instance or all messages
   */
  const filteredMessages =
    selectedInstanceId === null
      ? session.chatHistory
      : session.chatHistory.filter(
          (m) => m.instanceId === selectedInstanceId || m.role === 'system'
        );

  /**
   * Send message to tmux session
   */
  const handleSendMessage = async () => {
    if (!message.trim() || !selectedInstance) return;

    setIsSending(true);
    setError(null);

    try {
      // Create user message
      const userMessage: ChatMessage = {
        id: `msg-${Date.now()}`,
        role: 'user',
        content: message,
        timestamp: new Date().toISOString(),
        instanceId: selectedInstanceId || undefined,
      };

      // Add to session history
      await addChatMessage(session.id, userMessage);

      // Send to tmux session
      await tauriApi.sendTmuxKeys(selectedInstance.tmuxSessionId, message);

      // Wait a moment for output
      await new Promise((resolve) => setTimeout(resolve, 500));

      // Capture tmux output
      const output = await tauriApi.captureTmuxOutput(selectedInstance.tmuxSessionId);

      // Create assistant response
      const assistantMessage: ChatMessage = {
        id: `msg-${Date.now()}-response`,
        role: 'assistant',
        content: output || 'Command executed (no output)',
        timestamp: new Date().toISOString(),
        instanceId: selectedInstanceId || undefined,
      };

      // Add to session history
      await addChatMessage(session.id, assistantMessage);

      // Clear input
      setMessage('');
    } catch (err) {
      console.error('Failed to send message to tmux:', err);

      // Check if session has terminated
      if (isSessionTerminationError(err)) {
        setSessionAlive(false);
        setError('Tmux session has ended (exit command detected or session killed)');

        // Add system message about session termination
        const systemMessage: ChatMessage = {
          id: `msg-${Date.now()}-system`,
          role: 'system',
          content: '⚠️ Tmux session has ended. The terminal is no longer active.',
          timestamp: new Date().toISOString(),
          instanceId: selectedInstanceId || undefined,
        };
        await addChatMessage(session.id, systemMessage);
      } else {
        setError(err instanceof Error ? err.message : 'Failed to send message');
      }
    } finally {
      setIsSending(false);
    }
  };

  /**
   * Handle Enter key press
   */
  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSendMessage();
    }
  };

  /**
   * Handle split view resize
   */
  const handleMouseDown = (e: React.MouseEvent) => {
    e.preventDefault();
    isDraggingRef.current = true;
    document.body.style.cursor = 'col-resize';
    document.body.style.userSelect = 'none';
  };

  const handleMouseMove = (e: MouseEvent) => {
    if (!isDraggingRef.current || !splitContainerRef.current) return;

    const containerRect = splitContainerRef.current.getBoundingClientRect();
    const newPosition = ((e.clientX - containerRect.left) / containerRect.width) * 100;

    // Clamp between 20% and 80%
    setSplitPosition(Math.max(20, Math.min(80, newPosition)));
  };

  const handleMouseUp = () => {
    isDraggingRef.current = false;
    document.body.style.cursor = '';
    document.body.style.userSelect = '';
  };

  useEffect(() => {
    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);

    return () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };
  }, []);

  return (
    <div className="flex flex-col h-full">
      {/* Session ended warning banner */}
      {!sessionAlive && (
        <div className="flex-shrink-0 bg-yellow-50 border-b border-yellow-300 px-4 py-3">
          <div className="flex items-start gap-3">
            <AlertTriangle className="w-5 h-5 text-yellow-600 flex-shrink-0 mt-0.5" />
            <div className="flex-1">
              <div className="flex items-center gap-2 mb-1">
                <span className="font-semibold text-yellow-900 text-sm">
                  Tmux session has ended
                </span>
              </div>
              <p className="text-xs text-yellow-700 leading-relaxed">
                The terminal session is no longer active. This typically happens when an 'exit' command is executed
                or the session is killed externally. You can close this window or the agent may restart automatically.
              </p>
            </div>
          </div>
        </div>
      )}

      {/* Header: Instance Selector + View Mode Switcher */}
      <div className="border-b bg-gray-50 px-4 py-3">
        <div className="flex items-center gap-3 mb-3">
          <Terminal className="w-5 h-5 text-gray-600" />
          <span className="text-sm font-semibold text-gray-800">Send to:</span>
          <div className="flex-1 flex flex-wrap items-center gap-2">
            {/* All Instances Button */}
            <button
              onClick={() => setSelectedInstanceId(null)}
              className={`
                px-4 py-2 rounded-lg text-sm font-medium transition-all
                ${
                  selectedInstanceId === null
                    ? 'bg-blue-500 text-white shadow-md'
                    : 'bg-white text-gray-700 border border-gray-300 hover:bg-gray-50'
                }
              `}
            >
              🌐 All Instances
            </button>

            {/* Individual Instance Buttons */}
            {session.instances.map((instance) => {
              const runtimeDef = instance.runtime ? getRuntimeDefinition(instance.runtime) : null;
              const emoji = runtimeDef?.emoji || '🤖';
              const label = instance.runtimeLabel || instance.agentName;
              const isSelected = selectedInstanceId === instance.instanceId;

              return (
                <button
                  key={instance.instanceId}
                  onClick={() => setSelectedInstanceId(instance.instanceId)}
                  className={`
                    px-4 py-2 rounded-lg text-sm font-medium transition-all
                    ${
                      isSelected
                        ? 'bg-blue-500 text-white shadow-md'
                        : 'bg-white text-gray-700 border border-gray-300 hover:bg-gray-50'
                    }
                  `}
                  title={`Instance ${instance.instanceId} - ${label} (${instance.status})`}
                >
                  <span className="text-lg mr-2">{emoji}</span>
                  {label} #{instance.instanceId}
                </button>
              );
            })}
          </div>
        </div>

        {/* View Mode Switcher */}
        <div className="flex items-center gap-2">
          <span className="text-xs font-medium text-gray-600">View:</span>
          <div className="flex gap-1 bg-white border border-gray-300 rounded-lg p-1">
            <button
              onClick={() => setViewMode('chat')}
              className={`
                px-3 py-1 rounded text-xs font-medium transition-all flex items-center gap-1.5
                ${
                  viewMode === 'chat'
                    ? 'bg-blue-500 text-white shadow-sm'
                    : 'text-gray-600 hover:bg-gray-100'
                }
              `}
              title="Chat view only"
            >
              <MessageSquare className="w-3.5 h-3.5" />
              Chat
            </button>
            <button
              onClick={() => setViewMode('terminal')}
              className={`
                px-3 py-1 rounded text-xs font-medium transition-all flex items-center gap-1.5
                ${
                  viewMode === 'terminal'
                    ? 'bg-blue-500 text-white shadow-sm'
                    : 'text-gray-600 hover:bg-gray-100'
                }
              `}
              title="Terminal view only"
              disabled={!selectedInstance?.tmuxSessionId}
            >
              <MonitorPlay className="w-3.5 h-3.5" />
              Terminal
            </button>
            <button
              onClick={() => setViewMode('split')}
              className={`
                px-3 py-1 rounded text-xs font-medium transition-all flex items-center gap-1.5
                ${
                  viewMode === 'split'
                    ? 'bg-blue-500 text-white shadow-sm'
                    : 'text-gray-600 hover:bg-gray-100'
                }
              `}
              title="Split view (Chat + Terminal)"
              disabled={!selectedInstance?.tmuxSessionId}
            >
              <SplitSquareHorizontal className="w-3.5 h-3.5" />
              Split
            </button>
          </div>
          {!selectedInstance?.tmuxSessionId && (
            <span className="text-xs text-amber-600">
              Select an instance with tmux session for terminal view
            </span>
          )}
        </div>
      </div>

      {/* Main Content Area - Conditional Rendering based on viewMode */}
      {viewMode === 'chat' && (
        <ChatView
          isLoadingHistory={isLoadingHistory}
          filteredMessages={filteredMessages}
          session={session}
          error={error}
          messagesEndRef={messagesEndRef}
          message={message}
          setMessage={setMessage}
          handleKeyPress={handleKeyPress}
          selectedInstance={selectedInstance}
          isSending={isSending}
          handleSendMessage={handleSendMessage}
          sessionAlive={sessionAlive}
        />
      )}

      {viewMode === 'terminal' && selectedInstance?.tmuxSessionId && (
        <TerminalView
          tmuxSessionId={selectedInstance.tmuxSessionId}
          instanceName={selectedInstance.agentName}
        />
      )}

      {viewMode === 'split' && selectedInstance?.tmuxSessionId && (
        <div ref={splitContainerRef} className="flex-1 flex overflow-hidden">
          {/* Chat Panel */}
          <div style={{ width: `${splitPosition}%` }} className="flex flex-col border-r">
            <ChatView
              isLoadingHistory={isLoadingHistory}
              filteredMessages={filteredMessages}
              session={session}
              error={error}
              messagesEndRef={messagesEndRef}
              message={message}
              setMessage={setMessage}
              handleKeyPress={handleKeyPress}
              selectedInstance={selectedInstance}
              isSending={isSending}
              handleSendMessage={handleSendMessage}
              sessionAlive={sessionAlive}
            />
          </div>

          {/* Resizable Splitter */}
          <div
            onMouseDown={handleMouseDown}
            className="w-1 bg-gray-300 hover:bg-blue-500 cursor-col-resize transition-colors flex-shrink-0"
            style={{ cursor: 'col-resize' }}
          />

          {/* Terminal Panel */}
          <div style={{ width: `${100 - splitPosition}%` }} className="flex flex-col">
            <TerminalView
              tmuxSessionId={selectedInstance.tmuxSessionId}
              instanceName={selectedInstance.agentName}
            />
          </div>
        </div>
      )}
    </div>
  );
};

/**
 * Chat View Component (Reusable)
 */
interface ChatViewProps {
  isLoadingHistory: boolean;
  filteredMessages: ChatMessage[];
  session: WorktreeSession;
  error: string | null;
  messagesEndRef: React.RefObject<HTMLDivElement>;
  message: string;
  setMessage: (msg: string) => void;
  handleKeyPress: (e: React.KeyboardEvent) => void;
  selectedInstance: any;
  isSending: boolean;
  handleSendMessage: () => void;
  sessionAlive: boolean;
}

const ChatView: React.FC<ChatViewProps> = ({
  isLoadingHistory,
  filteredMessages,
  session,
  error,
  messagesEndRef,
  message,
  setMessage,
  handleKeyPress,
  selectedInstance,
  isSending,
  handleSendMessage,
  sessionAlive,
}) => {
  return (
    <>
      {/* Message List */}
      <div className="flex-1 overflow-y-auto p-4 space-y-4">
        {/* Loading History */}
        {isLoadingHistory && (
          <div className="flex items-center gap-2 p-3 bg-blue-50 border border-blue-200 rounded-lg">
            <Loader className="w-4 h-4 text-blue-500 animate-spin" />
            <span className="text-sm text-blue-700">Loading tmux session history...</span>
          </div>
        )}

        {filteredMessages.length === 0 ? (
          <div className="flex items-center justify-center h-full text-gray-400">
            <div className="text-center">
              <Bot className="w-12 h-12 mx-auto mb-2" />
              <p className="text-sm">No messages yet</p>
              <p className="text-xs mt-1">Send a command to get started</p>
            </div>
          </div>
        ) : (
          filteredMessages.map((msg) => (
            <MessageBubble key={msg.id} message={msg} session={session} />
          ))
        )}

        {/* Error display */}
        {error && (
          <div className="flex items-start gap-2 p-3 bg-red-50 border border-red-200 rounded-lg">
            <AlertCircle className="w-5 h-5 text-red-500 flex-shrink-0 mt-0.5" />
            <div className="text-sm text-red-700">{error}</div>
          </div>
        )}

        <div ref={messagesEndRef} />
      </div>

      {/* Input Area */}
      <div className="border-t bg-white p-4">
        {!sessionAlive && (
          <div className="mb-3 flex items-center gap-2 px-3 py-2 bg-yellow-50 border border-yellow-200 rounded-lg">
            <AlertTriangle className="w-4 h-4 text-yellow-600" />
            <span className="text-xs text-yellow-700">
              Session has ended. Commands cannot be sent.
            </span>
          </div>
        )}
        <div className="flex gap-2">
          <textarea
            value={message}
            onChange={(e) => setMessage(e.target.value)}
            onKeyPress={handleKeyPress}
            placeholder={
              !sessionAlive
                ? 'Session has ended - input disabled'
                : selectedInstance
                ? `Send command to ${selectedInstance.agentName}...`
                : 'Select an instance to send commands'
            }
            disabled={!selectedInstance || isSending || !sessionAlive}
            rows={2}
            className="
              flex-1 px-4 py-2 rounded-lg
              border border-gray-300
              focus:ring-2 focus:ring-blue-500 focus:border-blue-500
              resize-none text-sm
              disabled:bg-gray-50 disabled:text-gray-400
            "
          />
          <button
            onClick={handleSendMessage}
            disabled={!message.trim() || !selectedInstance || isSending || !sessionAlive}
            className="
              px-6 py-2 rounded-lg
              bg-blue-500 text-white
              hover:bg-blue-600
              disabled:bg-gray-300 disabled:cursor-not-allowed
              transition-colors
              flex items-center gap-2
            "
          >
            {isSending ? (
              <Loader className="w-4 h-4 animate-spin" />
            ) : (
              <Send className="w-4 h-4" />
            )}
            Send
          </button>
        </div>
        <p className="text-xs text-gray-500 mt-2">
          {sessionAlive
            ? 'Press Enter to send, Shift+Enter for new line'
            : 'Session has ended - commands disabled'}
        </p>
      </div>
    </>
  );
};

/**
 * Message Bubble Component
 */
const MessageBubble: React.FC<{
  message: ChatMessage;
  session: WorktreeSession;
}> = ({ message, session }) => {
  const isUser = message.role === 'user';
  const isSystem = message.role === 'system';

  const instance = message.instanceId
    ? session.instances.find((i) => i.instanceId === message.instanceId)
    : null;

  const formatTime = (timestamp: string) => {
    const date = new Date(timestamp);
    return date.toLocaleTimeString();
  };

  if (isSystem) {
    return (
      <div className="flex justify-center">
        <div className="inline-flex items-center gap-2 px-3 py-1 bg-gray-100 rounded-full text-xs text-gray-600">
          <AlertCircle className="w-3 h-3" />
          {message.content}
        </div>
      </div>
    );
  }

  return (
    <div className={`flex ${isUser ? 'justify-end' : 'justify-start'}`}>
      <div
        className={`
          max-w-[70%] rounded-lg px-4 py-3
          ${
            isUser
              ? 'bg-blue-500 text-white'
              : 'bg-gray-100 text-gray-900 border border-gray-200'
          }
        `}
      >
        {/* Header */}
        <div className="flex items-center gap-2 mb-1">
          {isUser ? (
            <User className="w-3 h-3" />
          ) : (
            <Bot className="w-3 h-3" />
          )}
          <span className="text-xs font-medium opacity-75">
            {isUser ? 'You' : instance?.agentName || 'Assistant'}
          </span>
          <span className="text-xs opacity-50">{formatTime(message.timestamp)}</span>
        </div>

        {/* Content */}
        <div className="text-sm whitespace-pre-wrap break-words font-mono">
          {message.content}
        </div>

        {/* Instance badge */}
        {message.instanceId && (
          <div className="mt-2 pt-2 border-t border-current border-opacity-20">
            <span className="text-xs opacity-75">
              Instance {message.instanceId}
            </span>
          </div>
        )}
      </div>
    </div>
  );
};
