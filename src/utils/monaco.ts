/**
 * Monaco Editor utilities and configuration
 */

import type { editor as MonacoEditor } from 'monaco-editor';

/**
 * Default editor options for AIT42 Editor
 */
export const defaultEditorOptions: MonacoEditor.IStandaloneEditorConstructionOptions =
  {
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
  };

/**
 * Language file extensions mapping
 */
export const languageExtensions: Record<string, string[]> = {
  javascript: ['js', 'jsx', 'mjs', 'cjs'],
  typescript: ['ts', 'tsx', 'mts', 'cts'],
  rust: ['rs'],
  python: ['py', 'pyw', 'pyi'],
  html: ['html', 'htm'],
  css: ['css'],
  scss: ['scss'],
  sass: ['sass'],
  less: ['less'],
  json: ['json', 'jsonc'],
  yaml: ['yaml', 'yml'],
  markdown: ['md', 'mdx'],
  shell: ['sh', 'bash', 'zsh'],
  c: ['c', 'h'],
  cpp: ['cpp', 'cc', 'cxx', 'hpp', 'h++'],
  go: ['go'],
  java: ['java'],
  php: ['php', 'phtml'],
  ruby: ['rb'],
  sql: ['sql'],
  xml: ['xml'],
  dockerfile: ['dockerfile'],
  toml: ['toml'],
  plaintext: ['txt'],
};

/**
 * Detect language from file extension
 */
export function detectLanguageFromPath(path: string): string {
  const ext = path.split('.').pop()?.toLowerCase();
  if (!ext) return 'plaintext';

  for (const [language, extensions] of Object.entries(languageExtensions)) {
    if (extensions.includes(ext)) {
      return language;
    }
  }

  return 'plaintext';
}

/**
 * Get file icon based on language/extension
 */
export function getFileIcon(language: string): string {
  const iconMap: Record<string, string> = {
    javascript: '📜',
    typescript: '📘',
    rust: '🦀',
    python: '🐍',
    html: '🌐',
    css: '🎨',
    json: '📋',
    markdown: '📝',
    shell: '💻',
    go: '🐹',
    java: '☕',
    php: '🐘',
    ruby: '💎',
  };

  return iconMap[language] || '📄';
}

/**
 * Format file size for display
 */
export function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 B';

  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));

  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
}

/**
 * Get line count for display
 */
export function getLineCount(content: string): number {
  return content.split('\n').length;
}

/**
 * Format line/column position
 */
export function formatPosition(line: number, column: number): string {
  return `Ln ${line}, Col ${column}`;
}
