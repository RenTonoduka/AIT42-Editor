/**
 * Session Filters Component
 *
 * Provides filtering and sorting controls for session history
 */
import React from 'react';
import { useSessionHistoryStore } from '@/store/sessionHistoryStore';
import type { SessionType, SessionStatus, SessionSortOptions } from '@/types/worktree';
import { Search, SlidersHorizontal, ArrowUpDown } from 'lucide-react';

export const SessionFilters: React.FC = () => {
  const { filters, sortOptions, setFilters, setSortOptions } = useSessionHistoryStore();

  const handleTypeToggle = (type: SessionType) => {
    const currentTypes = filters.type || [];
    const newTypes = currentTypes.includes(type)
      ? currentTypes.filter((t) => t !== type)
      : [...currentTypes, type];

    setFilters({ type: newTypes.length > 0 ? newTypes : undefined });
  };

  const handleStatusToggle = (status: SessionStatus) => {
    const currentStatuses = filters.status || [];
    const newStatuses = currentStatuses.includes(status)
      ? currentStatuses.filter((s) => s !== status)
      : [...currentStatuses, status];

    setFilters({ status: newStatuses.length > 0 ? newStatuses : undefined });
  };

  const handleSearchChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setFilters({ searchQuery: e.target.value || undefined });
  };

  const handleSortChange = (field: SessionSortOptions['field']) => {
    const newDirection =
      sortOptions.field === field && sortOptions.direction === 'desc' ? 'asc' : 'desc';

    setSortOptions({ field, direction: newDirection });
  };

  return (
    <div className="bg-white border-b border-gray-200 p-4 space-y-4">
      {/* Search Bar */}
      <div className="relative">
        <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" />
        <input
          type="text"
          placeholder="タスクまたはエージェント名で検索..."
          value={filters.searchQuery || ''}
          onChange={handleSearchChange}
          className="
            w-full pl-10 pr-4 py-2
            border border-gray-300 rounded-lg
            focus:ring-2 focus:ring-blue-500 focus:border-blue-500
            text-sm
          "
        />
      </div>

      <div className="flex items-center gap-4">
        {/* Type Filters */}
        <div className="flex items-center gap-2">
          <SlidersHorizontal className="w-4 h-4 text-gray-500" />
          <span className="text-sm font-medium text-gray-700">種類:</span>

          {(['competition', 'ensemble', 'debate'] as SessionType[]).map((type) => {
            const typeLabels = {
              competition: '競争',
              ensemble: 'アンサンブル',
              debate: 'ディベート',
            };
            return (
              <button
                key={type}
                onClick={() => handleTypeToggle(type)}
                className={`
                  px-3 py-1 rounded-full text-xs font-medium
                  border transition-colors
                  ${
                    filters.type?.includes(type)
                      ? 'bg-blue-500 text-white border-blue-500'
                      : 'bg-white text-gray-700 border-gray-300 hover:border-blue-300'
                  }
                `}
              >
                {typeLabels[type]}
              </button>
            );
          })}
        </div>

        {/* Status Filters */}
        <div className="flex items-center gap-2 border-l pl-4">
          <span className="text-sm font-medium text-gray-700">状態:</span>

          {(['running', 'paused', 'completed', 'failed'] as SessionStatus[]).map((status) => {
            const statusLabels = {
              running: '実行中',
              paused: '一時停止',
              completed: '完了',
              failed: '失敗',
            };
            return (
              <button
                key={status}
                onClick={() => handleStatusToggle(status)}
                className={`
                  px-3 py-1 rounded-full text-xs font-medium
                  border transition-colors
                  ${
                    filters.status?.includes(status)
                      ? 'bg-blue-500 text-white border-blue-500'
                      : 'bg-white text-gray-700 border-gray-300 hover:border-blue-300'
                  }
                `}
              >
                {statusLabels[status]}
              </button>
            );
          })}
        </div>

        {/* Sort Options */}
        <div className="flex items-center gap-2 border-l pl-4 ml-auto">
          <ArrowUpDown className="w-4 h-4 text-gray-500" />
          <span className="text-sm font-medium text-gray-700">並び替え:</span>

          <select
            value={sortOptions.field}
            onChange={(e) =>
              handleSortChange(e.target.value as SessionSortOptions['field'])
            }
            className="
              px-3 py-1 pr-8 rounded-lg text-xs font-medium
              border border-gray-300
              bg-white text-gray-700
              hover:border-blue-300
              focus:ring-2 focus:ring-blue-500 focus:border-blue-500
            "
          >
            <option value="updatedAt">更新日時</option>
            <option value="createdAt">作成日時</option>
            <option value="duration">実行時間</option>
            <option value="filesChanged">変更ファイル数</option>
          </select>

          <button
            onClick={() =>
              setSortOptions({
                ...sortOptions,
                direction: sortOptions.direction === 'asc' ? 'desc' : 'asc',
              })
            }
            className="
              px-2 py-1 rounded text-xs font-medium
              bg-gray-100 text-gray-700
              hover:bg-gray-200
              transition-colors
            "
          >
            {sortOptions.direction === 'asc' ? '↑' : '↓'}
          </button>
        </div>
      </div>
    </div>
  );
};
