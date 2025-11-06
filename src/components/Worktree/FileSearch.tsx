/**
 * File Search Component
 *
 * Search and filter files in the file tree
 */
import React from 'react';
import { Search, X } from 'lucide-react';
import { useWorktreeStore } from '@/store/worktreeStore';

export const FileSearch: React.FC = () => {
  const { searchQuery, setSearchQuery } = useWorktreeStore();

  const handleClear = () => {
    setSearchQuery('');
  };

  return (
    <div className="p-3 border-b border-gray-700 bg-gray-800/50">
      <div className="relative">
        <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" />
        <input
          type="text"
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          placeholder="Search files..."
          className="
            w-full pl-10 pr-8 py-2
            bg-gray-900 text-gray-200 placeholder-gray-500
            border border-gray-700 rounded-lg
            focus:outline-none focus:ring-2 focus:ring-blue-400/50 focus:border-blue-400
            transition-all duration-200
            text-sm
          "
        />
        {searchQuery && (
          <button
            onClick={handleClear}
            className="
              absolute right-2 top-1/2 -translate-y-1/2
              p-1 hover:bg-gray-700 rounded
              transition-colors duration-200
            "
            title="Clear search"
          >
            <X className="w-4 h-4 text-gray-400" />
          </button>
        )}
      </div>
    </div>
  );
};
