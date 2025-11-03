/**
 * AIT42 Editor - Sidebar Component
 */

import React from 'react';
import { FileTree } from './FileTree';
import styles from './Sidebar.module.css';

/**
 * Sidebar component containing file explorer
 *
 * Features:
 * - File tree navigation
 * - Collapsible sections
 * - Search (future)
 */
export const Sidebar: React.FC = () => {
  return (
    <div className={styles.sidebar}>
      {/* Header */}
      <div className={styles.header}>
        <h2 className={styles.title}>Explorer</h2>
        <div className={styles.actions}>
          <button
            className={styles.iconButton}
            title="New File"
            aria-label="New File"
          >
            <svg
              width="16"
              height="16"
              viewBox="0 0 16 16"
              fill="currentColor"
            >
              <path d="M9.5 1.1l3.4 3.5.1.4v2h-1V6H8V2H3v11h4v1H2.5l-.5-.5v-12l.5-.5h6.7l.3.1zM9 2v3h2.9L9 2zm4 14h-1v-3H9v-1h3V9h1v3h3v1h-3v3z" />
            </svg>
          </button>
          <button
            className={styles.iconButton}
            title="New Folder"
            aria-label="New Folder"
          >
            <svg
              width="16"
              height="16"
              viewBox="0 0 16 16"
              fill="currentColor"
            >
              <path d="M14.5 2H7.71l-.85-.85L6.51 1h-5l-.5.5v11l.5.5H7v-1H1.99V6h4.49l.35-.15.86-.86H14v1.5l-.001.51h1.011V2.5L14.5 2zm-.51 2h-6.5l-.35.15-.86.86H2v-3h4.29l.85.85.36.15H14l-.01.99zM13 16h-1v-3H9v-1h3V9h1v3h3v1h-3v3z" />
            </svg>
          </button>
          <button
            className={styles.iconButton}
            title="Refresh"
            aria-label="Refresh"
          >
            <svg
              width="16"
              height="16"
              viewBox="0 0 16 16"
              fill="currentColor"
            >
              <path d="M7.5 1a6.5 6.5 0 0 1 5.5 9.83l.89.89A7.5 7.5 0 1 0 1 7.5h1A6.5 6.5 0 0 1 7.5 1zM15 7.5V1h-1v5.5H8v1h7z" />
            </svg>
          </button>
        </div>
      </div>

      {/* File Tree */}
      <div className={styles.content}>
        <FileTree />
      </div>
    </div>
  );
};
