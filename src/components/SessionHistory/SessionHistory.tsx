/**
 * Session History Component
 *
 * Main component for session history management with Kanban board
 * Combines filters, board, and detail view
 */
import React, { useState } from 'react';
import { KanbanBoard } from './KanbanBoard';
import { SessionFilters } from './SessionFilters';
import { SessionDetailView } from './SessionDetailView';

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

      {/* Detail View Modal */}
      {selectedSessionId && (
        <SessionDetailView sessionId={selectedSessionId} onClose={handleCloseDetail} />
      )}
    </div>
  );
};
