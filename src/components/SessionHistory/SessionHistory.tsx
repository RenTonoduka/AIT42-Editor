/**
 * Session History Component
 *
 * Main component for session history management with Kanban board
 * Combines filters, board, and detail view
 */
import React, { useState } from 'react';
import { KanbanBoard } from './KanbanBoard';
import { SessionFilters } from './SessionFilters';
import { X } from 'lucide-react';

export const SessionHistory: React.FC = () => {
  const [selectedSessionId, setSelectedSessionId] = useState<string | null>(null);

  const handleSelectSession = (sessionId: string) => {
    setSelectedSessionId(sessionId);
  };

  const handleCloseDetail = () => {
    setSelectedSessionId(null);
  };

  return (
    <div className="h-full flex flex-col">
      {/* Filters */}
      <SessionFilters />

      {/* Kanban Board */}
      <div className="flex-1 overflow-hidden">
        <KanbanBoard onSelectSession={handleSelectSession} />
      </div>

      {/* Detail View Modal (Phase 3 will implement this) */}
      {selectedSessionId && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg shadow-xl w-4/5 h-4/5 flex flex-col">
            {/* Modal Header */}
            <div className="flex items-center justify-between px-6 py-4 border-b">
              <h2 className="text-lg font-semibold text-gray-900">Session Details</h2>
              <button
                onClick={handleCloseDetail}
                className="p-2 hover:bg-gray-100 rounded-lg transition-colors"
              >
                <X className="w-5 h-5 text-gray-500" />
              </button>
            </div>

            {/* Modal Content (Placeholder for Phase 3) */}
            <div className="flex-1 p-6 overflow-auto">
              <div className="text-center text-gray-500">
                <p className="text-lg font-medium mb-2">Detail View Coming Soon</p>
                <p className="text-sm">
                  Session ID: <code className="bg-gray-100 px-2 py-1 rounded">{selectedSessionId}</code>
                </p>
                <p className="text-sm mt-4">
                  Phase 3 will implement:
                </p>
                <ul className="text-sm mt-2 space-y-1">
                  <li>• Worktree Explorer integration</li>
                  <li>• File diff viewer</li>
                  <li>• Metrics and statistics</li>
                  <li>• Interactive chat panel</li>
                </ul>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
