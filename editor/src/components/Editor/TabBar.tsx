/**
 * AIT42 Editor - Tab Bar Component
 */

import React from 'react';
import { Tab } from './Tab';
import { useEditor } from '../../hooks/useEditor';
import styles from './Editor.module.css';

/**
 * Tab bar component for managing open files
 *
 * Features:
 * - Display all open tabs
 * - Tab switching
 * - Tab closing
 * - Modified indicator
 */
export const TabBar: React.FC = () => {
  const { tabs, activeTabId, setActiveTab } = useEditor();

  if (tabs.length === 0) {
    return null;
  }

  return (
    <div className={styles.tabBar}>
      <div className={styles.tabList}>
        {tabs.map(tab => (
          <Tab
            key={tab.id}
            tab={tab}
            isActive={tab.id === activeTabId}
            onActivate={() => setActiveTab(tab.id)}
          />
        ))}
      </div>
    </div>
  );
};
