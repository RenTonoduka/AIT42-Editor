/**
 * AIT42 Editor - Editor Area Component
 */

import React from 'react';
import { TabBar } from './TabBar';
import { MonacoEditor } from './MonacoEditor';
import { useEditor } from '../../hooks/useEditor';
import styles from './Editor.module.css';

/**
 * Editor area component containing tab bar and Monaco editor
 *
 * Features:
 * - Tab management
 * - Monaco editor integration
 * - Welcome screen when no tabs
 */
export const EditorArea: React.FC = () => {
  const { tabs, activeTab } = useEditor();

  return (
    <div className={styles.editorArea}>
      {/* Tab Bar */}
      <TabBar />

      {/* Editor Content */}
      <div className={styles.editorContent}>
        {activeTab ? (
          <MonacoEditor
            path={activeTab.path}
            language={activeTab.language}
            value={activeTab.content}
          />
        ) : (
          <WelcomeScreen />
        )}
      </div>
    </div>
  );
};

/**
 * Welcome screen shown when no files are open
 */
const WelcomeScreen: React.FC = () => {
  return (
    <div className={styles.welcomeScreen}>
      <div className={styles.welcomeContent}>
        <h1 className={styles.welcomeTitle}>AIT42 Editor</h1>
        <p className={styles.welcomeSubtitle}>AI-Powered Development Environment</p>

        <div className={styles.shortcuts}>
          <h2>Quick Start</h2>
          <ul>
            <li>
              <kbd>Cmd/Ctrl + P</kbd>
              <span>Open command palette</span>
            </li>
            <li>
              <kbd>Cmd/Ctrl + O</kbd>
              <span>Open file</span>
            </li>
            <li>
              <kbd>Cmd/Ctrl + B</kbd>
              <span>Toggle sidebar</span>
            </li>
            <li>
              <kbd>Cmd/Ctrl + J</kbd>
              <span>Toggle terminal</span>
            </li>
            <li>
              <kbd>Cmd/Ctrl + S</kbd>
              <span>Save file</span>
            </li>
          </ul>
        </div>

        <div className={styles.features}>
          <div className={styles.feature}>
            <span className={styles.featureIcon}>🤖</span>
            <h3>49 AI Agents</h3>
            <p>Specialized agents for every development task</p>
          </div>
          <div className={styles.feature}>
            <span className={styles.featureIcon}>⚡</span>
            <h3>Tmux Integration</h3>
            <p>Parallel execution with session management</p>
          </div>
          <div className={styles.feature}>
            <span className={styles.featureIcon}>✨</span>
            <h3>Smart Coordinator</h3>
            <p>Automatic agent selection and orchestration</p>
          </div>
        </div>
      </div>
    </div>
  );
};
