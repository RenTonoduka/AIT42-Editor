/**
 * Chat Panel Component
 *
 * Interactive chat interface for communicating with Claude Code instances
 * Supports message history, tmux integration, and real-time output
 */
import React, { useState, useRef, useEffect } from 'react';
import { useSessionHistoryStore } from '@/store/sessionHistoryStore';
import { tauriApi } from '@/services/tauri';
import type { WorktreeSession, ChatMessage } from '@/types/worktree';
import { Send, Terminal, User, Bot, AlertCircle, Loader } from 'lucide-react';

interface ChatPanelProps {
  session: WorktreeSession;
}

export const ChatPanel: React.FC<ChatPanelProps> = ({ session }) => {
  const [message, setMessage] = useState('');
  const [selectedInstanceId, setSelectedInstanceId] = useState<number | null>(
    session.instances.length > 0 ? session.instances[0].instanceId : null
  );
  const [isSending, setIsSending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const { addChatMessage } = useSessionHistoryStore();

  // Auto-scroll to bottom when messages change
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [session.chatHistory]);

  /**
   * Get selected instance
   */
  const selectedInstance = session.instances.find(
    (i) => i.instanceId === selectedInstanceId
  );

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
      setError(err instanceof Error ? err.message : 'Failed to send message');
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

  return (
    <div className="flex flex-col h-full">
      {/* Instance Selector */}
      <div className="border-b bg-gray-50 px-4 py-3">
        <div className="flex items-center gap-3">
          <Terminal className="w-4 h-4 text-gray-500" />
          <span className="text-sm font-medium text-gray-700">Send to:</span>
          <select
            value={selectedInstanceId || 'all'}
            onChange={(e) =>
              setSelectedInstanceId(e.target.value === 'all' ? null : Number(e.target.value))
            }
            className="
              px-3 py-1.5 rounded-lg text-sm
              border border-gray-300 bg-white
              focus:ring-2 focus:ring-blue-500 focus:border-blue-500
            "
          >
            <option value="all">All Instances</option>
            {session.instances.map((instance) => (
              <option key={instance.instanceId} value={instance.instanceId}>
                Instance {instance.instanceId} - {instance.agentName} ({instance.status})
              </option>
            ))}
          </select>
        </div>
      </div>

      {/* Message List */}
      <div className="flex-1 overflow-y-auto p-4 space-y-4">
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
        <div className="flex gap-2">
          <textarea
            value={message}
            onChange={(e) => setMessage(e.target.value)}
            onKeyPress={handleKeyPress}
            placeholder={
              selectedInstance
                ? `Send command to ${selectedInstance.agentName}...`
                : 'Select an instance to send commands'
            }
            disabled={!selectedInstance || isSending}
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
            disabled={!message.trim() || !selectedInstance || isSending}
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
          Press Enter to send, Shift+Enter for new line
        </p>
      </div>
    </div>
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
