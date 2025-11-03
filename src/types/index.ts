/**
 * Type definitions for AIT42 Editor
 */

export interface FileContent {
  path: string;
  content: string;
  language?: string;
}

export interface FileTreeNode {
  name: string;
  path: string;
  is_dir: boolean;
  children?: FileTreeNode[];
}

export interface Position {
  line: number;
  character: number;
}

export interface Range {
  start: Position;
  end: Position;
}

export interface Diagnostic {
  range: Range;
  message: string;
  severity: 'error' | 'warning' | 'info' | 'hint';
}

export interface CompletionItem {
  label: string;
  kind: string;
  detail?: string;
  documentation?: string;
}

export interface EditorConfig {
  fontSize: number;
  tabSize: number;
  insertSpaces: boolean;
  wordWrap: boolean;
  theme: 'vs-dark' | 'vs-light';
}
