/**
 * AIT42 Editor - Tab Component
 */

import React from 'react';
import { EditorTab } from '../../types';
import { useEditor } from '../../hooks/useEditor';
import styles from './Editor.module.css';

interface TabProps {
  tab: EditorTab;
  isActive: boolean;
  onActivate: () => void;
}

/**
 * Individual tab component
 *
 * Features:
 * - Show file name and icon
 * - Modified indicator (dot)
 * - Close button
 * - Active state styling
 */
export const Tab: React.FC<TabProps> = ({ tab, isActive, onActivate }) => {
  const { closeFile } = useEditor();

  const handleClose = (e: React.MouseEvent) => {
    e.stopPropagation();
    closeFile(tab.id);
  };

  const getFileIcon = (language: string): string => {
    const iconMap: Record<string, string> = {
      'typescript': '📘',
      'javascript': '📜',
      'rust': '🦀',
      'python': '🐍',
      'go': '🐹',
      'json': '📋',
      'html': '🌐',
      'css': '🎨',
      'markdown': '📝',
    };

    return iconMap[language] || '📄';
  };

  return (
    <div
      className={`${styles.tab} ${isActive ? styles.tabActive : ''}`}
      onClick={onActivate}
      role="tab"
      aria-selected={isActive}
      tabIndex={0}
      onKeyDown={(e) => {
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault();
          onActivate();
        }
      }}
    >
      {/* File icon */}
      <span className={styles.tabIcon}>
        {getFileIcon(tab.language)}
      </span>

      {/* File name */}
      <span className={styles.tabLabel}>
        {tab.name}
      </span>

      {/* Modified indicator */}
      {tab.isDirty && (
        <span className={styles.tabModified} title="Modified">
          ●
        </span>
      )}

      {/* Close button */}
      <button
        className={styles.tabClose}
        onClick={handleClose}
        aria-label={`Close ${tab.name}`}
        title="Close"
      >
        <svg
          width="16"
          height="16"
          viewBox="0 0 16 16"
          fill="currentColor"
        >
          <path d="M8 8.707l3.646 3.647.708-.707L8.707 8l3.647-3.646-.707-.708L8 7.293 4.354 3.646l-.707.708L7.293 8l-3.646 3.646.707.708L8 8.707z" />
        </svg>
      </button>
    </div>
  );
};
