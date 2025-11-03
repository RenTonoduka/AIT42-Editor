/**
 * EditorPane Component - Monaco Editor integration
 *
 * Features:
 * - Syntax highlighting
 * - IntelliSense (autocomplete)
 * - Line numbers and minimap
 * - Keyboard shortcuts (Cmd+S to save)
 * - Cursor Dark theme
 * - Auto-save on content change
 */

import React, { useRef, useCallback, useEffect } from 'react';
import Editor, { Monaco, OnMount } from '@monaco-editor/react';
import type { editor as MonacoEditor } from 'monaco-editor';
import { registerCursorDarkTheme } from '@/themes/monaco-cursor-dark';

export interface EditorPaneProps {
  /** Buffer/tab ID */
  bufferId: string;
  /** Editor content */
  content: string;
  /** Language identifier */
  language: string;
  /** Callback when content changes */
  onChange: (content: string) => void;
  /** Callback when save is triggered (Cmd+S) */
  onSave: () => void;
}

/**
 * EditorPane - Monaco Editor wrapper component
 */
export const EditorPane: React.FC<EditorPaneProps> = ({
  bufferId,
  content,
  language,
  onChange,
  onSave,
}) => {
  const editorRef = useRef<MonacoEditor.IStandaloneCodeEditor | null>(null);
  const monacoRef = useRef<Monaco | null>(null);

  /**
   * Handle editor mount
   */
  const handleEditorDidMount: OnMount = useCallback((editor, monaco) => {
    editorRef.current = editor;
    monacoRef.current = monaco;

    // Register Cursor Dark theme
    registerCursorDarkTheme(monaco);

    // Add save keyboard shortcut (Cmd+S / Ctrl+S)
    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, () => {
      onSave();
    });

    // Focus editor
    editor.focus();

    // Configure editor options after mount
    editor.updateOptions({
      fontSize: 14,
      fontFamily: "'JetBrains Mono', 'Fira Code', 'Consolas', monospace",
      lineHeight: 21,
      letterSpacing: 0.5,
      fontLigatures: true,
      cursorBlinking: 'smooth',
      cursorSmoothCaretAnimation: 'on',
      smoothScrolling: true,
      minimap: {
        enabled: true,
        scale: 1,
        showSlider: 'mouseover',
      },
      scrollbar: {
        verticalScrollbarSize: 10,
        horizontalScrollbarSize: 10,
        useShadows: false,
      },
      bracketPairColorization: {
        enabled: true,
      },
      guides: {
        bracketPairs: true,
        indentation: true,
      },
    });
  }, [onSave]);

  /**
   * Handle content change
   */
  const handleChange = useCallback(
    (value: string | undefined) => {
      if (value !== undefined) {
        onChange(value);
      }
    },
    [onChange]
  );

  /**
   * Focus editor when buffer changes
   */
  useEffect(() => {
    if (editorRef.current) {
      editorRef.current.focus();
    }
  }, [bufferId]);

  return (
    <div className="w-full h-full" data-buffer-id={bufferId}>
      <Editor
        height="100%"
        language={language}
        value={content}
        theme="cursor-dark"
        onChange={handleChange}
        onMount={handleEditorDidMount}
        options={{
          automaticLayout: true,
          wordWrap: 'off',
          lineNumbers: 'on',
          renderLineHighlight: 'all',
          renderWhitespace: 'selection',
          tabSize: 2,
          insertSpaces: true,
          detectIndentation: true,
          trimAutoWhitespace: true,
          formatOnPaste: true,
          formatOnType: false,
          autoClosingBrackets: 'languageDefined',
          autoClosingQuotes: 'languageDefined',
          autoSurround: 'languageDefined',
          quickSuggestions: true,
          suggestOnTriggerCharacters: true,
          acceptSuggestionOnCommitCharacter: true,
          acceptSuggestionOnEnter: 'on',
          snippetSuggestions: 'inline',
          folding: true,
          foldingStrategy: 'indentation',
          showFoldingControls: 'mouseover',
          matchBrackets: 'always',
          renderControlCharacters: false,
          links: true,
          mouseWheelZoom: false,
          multiCursorModifier: 'ctrlCmd',
          occurrencesHighlight: 'singleFile',
          overviewRulerBorder: false,
          padding: {
            top: 16,
            bottom: 16,
          },
          parameterHints: {
            enabled: true,
          },
          quickSuggestionsDelay: 100,
          readOnly: false,
          scrollBeyondLastLine: true,
          selectionHighlight: true,
          selectOnLineNumbers: true,
          showDeprecated: true,
          showUnused: true,
          wordBasedSuggestions: 'currentDocument',
        }}
        loading={
          <div className="flex items-center justify-center h-full bg-[#1E1E1E] text-[#CCCCCC]">
            <div className="text-center">
              <div className="animate-spin rounded-full h-8 w-8 border-t-2 border-b-2 border-[#007ACC] mx-auto mb-4"></div>
              <p>Loading editor...</p>
            </div>
          </div>
        }
      />
    </div>
  );
};
