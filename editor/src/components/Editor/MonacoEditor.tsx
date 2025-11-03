/**
 * AIT42 Editor - Monaco Editor Wrapper Component
 */

import React, { useRef, useEffect } from 'react';
import Editor, { OnMount } from '@monaco-editor/react';
import * as monaco from 'monaco-editor';
import { useEditor } from '../../hooks/useEditor';
import styles from './Editor.module.css';

interface MonacoEditorProps {
  path: string;
  language: string;
  value: string;
}

/**
 * Monaco Editor wrapper with Cursor theme
 *
 * Features:
 * - Syntax highlighting
 * - Cursor dark theme
 * - Auto-save on Cmd/Ctrl+S
 * - Content change tracking
 */
export const MonacoEditor: React.FC<MonacoEditorProps> = ({
  path,
  language,
  value,
}) => {
  const editorRef = useRef<monaco.editor.IStandaloneCodeEditor | null>(null);
  const { updateContent, saveFile, activeTabId } = useEditor();

  const handleEditorDidMount: OnMount = (editor, monaco) => {
    editorRef.current = editor;

    // Configure Cursor dark theme
    monaco.editor.defineTheme('cursor-dark', {
      base: 'vs-dark',
      inherit: true,
      rules: [
        { token: 'comment', foreground: '6A9955' },
        { token: 'keyword', foreground: '569CD6' },
        { token: 'string', foreground: 'CE9178' },
        { token: 'number', foreground: 'B5CEA8' },
        { token: 'function', foreground: 'DCDCAA' },
        { token: 'variable', foreground: '9CDCFE' },
        { token: 'type', foreground: '4EC9B0' },
      ],
      colors: {
        'editor.background': '#1E1E1E',
        'editor.foreground': '#CCCCCC',
        'editor.lineHighlightBackground': '#2A2D2E',
        'editorLineNumber.foreground': '#858585',
        'editorLineNumber.activeForeground': '#C6C6C6',
        'editor.selectionBackground': '#264F78',
        'editor.inactiveSelectionBackground': '#3A3D41',
        'editorCursor.foreground': '#AEAFAD',
        'editor.findMatchBackground': '#515C6A',
        'editor.findMatchHighlightBackground': '#EA5C0055',
        'editorBracketMatch.background': '#0064001a',
        'editorBracketMatch.border': '#888888',
        'scrollbarSlider.background': '#42424280',
        'scrollbarSlider.hoverBackground': '#4F4F4F80',
        'scrollbarSlider.activeBackground': '#4F4F4F80',
      },
    });

    // Apply theme
    monaco.editor.setTheme('cursor-dark');

    // Configure editor options
    editor.updateOptions({
      fontSize: 14,
      fontFamily: "'Fira Code', 'Monaco', 'Menlo', 'Consolas', monospace",
      fontLigatures: true,
      lineNumbers: 'on',
      minimap: {
        enabled: true,
      },
      scrollBeyondLastLine: false,
      automaticLayout: true,
      tabSize: 2,
      insertSpaces: true,
      wordWrap: 'off',
      renderWhitespace: 'selection',
      bracketPairColorization: {
        enabled: true,
      },
      guides: {
        bracketPairs: true,
        indentation: true,
      },
    });

    // Add save keybinding
    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, () => {
      if (activeTabId) {
        saveFile(activeTabId);
      }
    });

    // Focus editor
    editor.focus();
  };

  const handleChange = (newValue: string | undefined) => {
    if (newValue !== undefined) {
      updateContent(newValue);
    }
  };

  return (
    <div className={styles.monacoWrapper}>
      <Editor
        path={path}
        language={language}
        value={value}
        onMount={handleEditorDidMount}
        onChange={handleChange}
        theme="cursor-dark"
        options={{
          readOnly: false,
          domReadOnly: false,
        }}
      />
    </div>
  );
};
