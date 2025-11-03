/**
 * Cursor Dark Theme for Monaco Editor
 *
 * Replicates the professional dark aesthetic of Cursor AI's code editor.
 * Colors sourced from AIT42 TUI Cursor theme.
 */

import type { editor } from 'monaco-editor';

export const cursorDarkTheme: editor.IStandaloneThemeData = {
  base: 'vs-dark',
  inherit: true,
  rules: [
    // Keywords
    { token: 'keyword', foreground: 'C586C0', fontStyle: 'bold' },
    { token: 'keyword.control', foreground: 'D8A0DF', fontStyle: 'bold' },

    // Types & Classes
    { token: 'type', foreground: '4EC9B0' },
    { token: 'class', foreground: '4EC9B0' },
    { token: 'interface', foreground: '4EC9B0' },
    { token: 'enum', foreground: '4EC9B0' },
    { token: 'struct', foreground: '4EC9B0' },

    // Functions & Methods
    { token: 'function', foreground: 'DCDCAA' },
    { token: 'method', foreground: 'DCDCAA' },

    // Strings & Characters
    { token: 'string', foreground: 'CE9178' },
    { token: 'string.escape', foreground: 'D7BA7D' },
    { token: 'string.regexp', foreground: 'D16969' },
    { token: 'character', foreground: 'CE9178' },

    // Numbers
    { token: 'number', foreground: 'B5CEA8' },

    // Comments
    { token: 'comment', foreground: '6A9955', fontStyle: 'italic' },
    { token: 'comment.doc', foreground: '6A9955', fontStyle: 'italic' },

    // Variables & Parameters
    { token: 'variable', foreground: '9CDCFE' },
    { token: 'parameter', foreground: '9CDCFE' },
    { token: 'property', foreground: '9CDCFE' },

    // Constants
    { token: 'constant', foreground: '569CD6' },
    { token: 'constant.language', foreground: '569CD6' },

    // Operators
    { token: 'operator', foreground: 'D4D4D4' },
    { token: 'delimiter', foreground: 'D4D4D4' },

    // Macros & Attributes
    { token: 'macro', foreground: 'BD93F9' },
    { token: 'attribute', foreground: 'DCDCAA' },
    { token: 'annotation', foreground: 'DCDCAA' },

    // Tags (HTML/XML)
    { token: 'tag', foreground: '569CD6' },
    { token: 'tag.attribute', foreground: '9CDCFE' },

    // Invalid/Deprecated
    { token: 'invalid', foreground: 'F44747', fontStyle: 'underline' },
    { token: 'invalid.deprecated', foreground: 'F59E0B', fontStyle: 'strikethrough' },
  ],
  colors: {
    // Base colors
    'editor.background': '#1E1E1E',
    'editor.foreground': '#CCCCCC',
    'editorLineNumber.foreground': '#858585',
    'editorLineNumber.activeForeground': '#CCCCCC',

    // Cursor
    'editorCursor.foreground': '#007ACC',
    'editorCursor.background': '#1E1E1E',

    // Selection
    'editor.selectionBackground': '#264F78',
    'editor.inactiveSelectionBackground': '#3A3D41',
    'editor.selectionHighlightBackground': '#ADD6FF26',

    // Line highlight
    'editor.lineHighlightBackground': '#2D2D2D',
    'editor.lineHighlightBorder': '#00000000',

    // Word highlight
    'editor.wordHighlightBackground': '#575757B8',
    'editor.wordHighlightStrongBackground': '#004972B8',

    // Find/match
    'editor.findMatchBackground': '#515C6A',
    'editor.findMatchHighlightBackground': '#EA5C0055',
    'editor.findRangeHighlightBackground': '#3A3D4166',

    // Bracket matching
    'editorBracketMatch.background': '#0064001A',
    'editorBracketMatch.border': '#888888',

    // Whitespace
    'editorWhitespace.foreground': '#404040',

    // Indent guides
    'editorIndentGuide.background': '#404040',
    'editorIndentGuide.activeBackground': '#707070',

    // Rulers
    'editorRuler.foreground': '#5A5A5A',

    // Links
    'editorLink.activeForeground': '#4E94CE',

    // Gutter
    'editorGutter.background': '#1E1E1E',
    'editorGutter.modifiedBackground': '#148EE0',
    'editorGutter.addedBackground': '#10B981',
    'editorGutter.deletedBackground': '#EF4444',

    // Overview ruler
    'editorOverviewRuler.border': '#3E3E42',
    'editorOverviewRuler.modifiedForeground': '#148EE099',
    'editorOverviewRuler.addedForeground': '#10B98199',
    'editorOverviewRuler.deletedForeground': '#EF444499',
    'editorOverviewRuler.errorForeground': '#EF4444',
    'editorOverviewRuler.warningForeground': '#F59E0B',
    'editorOverviewRuler.infoForeground': '#3B82F6',

    // Errors and warnings
    'editorError.foreground': '#EF4444',
    'editorWarning.foreground': '#F59E0B',
    'editorInfo.foreground': '#3B82F6',
    'editorHint.foreground': '#10B981',

    // Widget (autocomplete, hover, etc)
    'editorWidget.background': '#252525',
    'editorWidget.border': '#3E3E42',
    'editorWidget.foreground': '#CCCCCC',
    'editorSuggestWidget.background': '#252525',
    'editorSuggestWidget.border': '#3E3E42',
    'editorSuggestWidget.foreground': '#CCCCCC',
    'editorSuggestWidget.highlightForeground': '#007ACC',
    'editorSuggestWidget.selectedBackground': '#2D2D2D',
    'editorHoverWidget.background': '#252525',
    'editorHoverWidget.border': '#3E3E42',

    // Peek view
    'peekView.border': '#007ACC',
    'peekViewEditor.background': '#1E1E1E',
    'peekViewEditor.matchHighlightBackground': '#515C6A',
    'peekViewResult.background': '#252525',
    'peekViewResult.fileForeground': '#CCCCCC',
    'peekViewResult.lineForeground': '#858585',
    'peekViewResult.matchHighlightBackground': '#515C6A',
    'peekViewResult.selectionBackground': '#2D2D2D',
    'peekViewResult.selectionForeground': '#FFFFFF',
    'peekViewTitle.background': '#1E1E1E',
    'peekViewTitleDescription.foreground': '#CCCCCC99',
    'peekViewTitleLabel.foreground': '#FFFFFF',

    // Scrollbar
    'scrollbar.shadow': '#000000',
    'scrollbarSlider.background': '#79797966',
    'scrollbarSlider.hoverBackground': '#646464B3',
    'scrollbarSlider.activeBackground': '#BFBFBF66',

    // Minimap
    'minimap.background': '#1E1E1E',
    'minimap.selectionHighlight': '#264F78',
    'minimap.errorHighlight': '#EF4444',
    'minimap.warningHighlight': '#F59E0B',
    'minimapGutter.addedBackground': '#10B981',
    'minimapGutter.modifiedBackground': '#148EE0',
    'minimapGutter.deletedBackground': '#EF4444',
  },
};

/**
 * Register the Cursor Dark theme with Monaco Editor
 */
export function registerCursorDarkTheme(monaco: typeof import('monaco-editor')) {
  monaco.editor.defineTheme('cursor-dark', cursorDarkTheme);
}
