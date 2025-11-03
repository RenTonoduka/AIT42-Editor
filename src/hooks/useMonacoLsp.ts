/**
 * Monaco LSP Integration Hook
 *
 * Integrates Language Server Protocol features with Monaco Editor:
 * - Lifecycle notifications (did_open, did_change, did_save, did_close)
 * - Completion provider (autocomplete)
 * - Hover provider (tooltips)
 * - Definition provider (go-to-definition)
 * - Diagnostics (error/warning markers)
 */

import { useEffect, useCallback, useRef } from 'react';
import type { Monaco } from '@monaco-editor/react';
import type { editor as MonacoEditor } from 'monaco-editor';
import { tauriApi } from '@/services/tauri';
import { useLspStore } from '@/store/lspStore';
import { detectLanguageFromPath } from '@/utils/monaco';

export interface UseMonacoLspProps {
  /** Monaco instance */
  monaco: Monaco | null;
  /** Editor instance */
  editor: MonacoEditor.IStandaloneCodeEditor | null;
  /** File path */
  filePath: string;
  /** File content */
  content: string;
  /** Language ID */
  language: string;
}

/**
 * Convert LSP diagnostic severity to Monaco marker severity
 */
function lspSeverityToMonaco(severity: number): number {
  // LSP: 1=Error, 2=Warning, 3=Information, 4=Hint
  // Monaco: 8=Error, 4=Warning, 2=Info, 1=Hint
  switch (severity) {
    case 1:
      return 8; // MarkerSeverity.Error
    case 2:
      return 4; // MarkerSeverity.Warning
    case 3:
      return 2; // MarkerSeverity.Info
    case 4:
      return 1; // MarkerSeverity.Hint
    default:
      return 2; // Default to Info
  }
}

/**
 * Convert LSP completion item kind to Monaco completion item kind
 */
function lspKindToMonaco(kind: number | undefined): number {
  if (!kind) return 0; // Text

  // LSP and Monaco use similar numbering, but verify mapping
  // LSP: TEXT=1, METHOD=2, FUNCTION=3, CONSTRUCTOR=4, FIELD=5, VARIABLE=6, CLASS=7, etc.
  // Monaco: Text=0, Method=1, Function=2, Constructor=3, Field=4, Variable=5, Class=6, etc.
  return kind - 1; // LSP is 1-indexed, Monaco is 0-indexed
}

/**
 * Hook for integrating LSP with Monaco Editor
 */
export function useMonacoLsp({
  monaco,
  editor,
  filePath,
  content,
  language,
}: UseMonacoLspProps) {
  const lspStore = useLspStore();
  const changeTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const lastContentRef = useRef<string>(content);
  const isOpenRef = useRef(false);

  /**
   * Notify LSP server that document was opened
   */
  const notifyDidOpen = useCallback(async () => {
    if (!lspStore.enabled || !filePath) return;

    try {
      // Detect language ID from file path
      const languageId = detectLanguageFromPath(filePath);

      await tauriApi.lspDidOpen(filePath, content, languageId);
      isOpenRef.current = true;

      // Reset document version
      lspStore.resetDocumentVersion(filePath);

      console.log(`[LSP] Document opened: ${filePath} (${languageId})`);
    } catch (error) {
      console.error('[LSP] Failed to notify did_open:', error);
    }
  }, [lspStore, filePath, content]);

  /**
   * Notify LSP server of document changes (debounced)
   */
  const notifyDidChange = useCallback(
    async (newContent: string) => {
      if (!lspStore.enabled || !filePath || !isOpenRef.current) return;

      // Clear previous timeout
      if (changeTimeoutRef.current) {
        clearTimeout(changeTimeoutRef.current);
      }

      // Debounce changes (300ms)
      changeTimeoutRef.current = setTimeout(async () => {
        try {
          const version = lspStore.incrementDocumentVersion(filePath);
          await tauriApi.lspDidChange(filePath, newContent, version);
          lastContentRef.current = newContent;

          console.log(`[LSP] Document changed: ${filePath} (v${version})`);

          // Fetch updated diagnostics
          const diagnostics = await tauriApi.lspDiagnostics(filePath);
          lspStore.setFileDiagnostics(filePath, diagnostics);

          // Update Monaco markers
          if (monaco && editor) {
            const model = editor.getModel();
            if (model) {
              const markers = diagnostics.map((diag) => ({
                severity: lspSeverityToMonaco(diag.severity),
                startLineNumber: diag.startLine + 1,
                startColumn: diag.startCharacter + 1,
                endLineNumber: diag.endLine + 1,
                endColumn: diag.endCharacter + 1,
                message: diag.message,
                code: diag.code,
                source: diag.source,
              }));

              monaco.editor.setModelMarkers(model, 'lsp', markers);
            }
          }
        } catch (error) {
          console.error('[LSP] Failed to notify did_change:', error);
        }
      }, 300);
    },
    [lspStore, filePath, monaco, editor]
  );

  /**
   * Notify LSP server that document was saved
   */
  const notifyDidSave = useCallback(async () => {
    if (!lspStore.enabled || !filePath || !isOpenRef.current) return;

    try {
      await tauriApi.lspDidSave(filePath, lastContentRef.current);
      console.log(`[LSP] Document saved: ${filePath}`);

      // Fetch updated diagnostics after save
      const diagnostics = await tauriApi.lspDiagnostics(filePath);
      lspStore.setFileDiagnostics(filePath, diagnostics);
    } catch (error) {
      console.error('[LSP] Failed to notify did_save:', error);
    }
  }, [lspStore, filePath]);

  /**
   * Notify LSP server that document was closed
   */
  const notifyDidClose = useCallback(async () => {
    if (!lspStore.enabled || !filePath || !isOpenRef.current) return;

    try {
      await tauriApi.lspDidClose(filePath);
      isOpenRef.current = false;

      // Clear diagnostics for this file
      lspStore.clearFileDiagnostics(filePath);

      console.log(`[LSP] Document closed: ${filePath}`);
    } catch (error) {
      console.error('[LSP] Failed to notify did_close:', error);
    }
  }, [lspStore, filePath]);

  /**
   * Register completion provider
   */
  useEffect(() => {
    if (!monaco || !lspStore.enabled) return;

    const disposable = monaco.languages.registerCompletionItemProvider(
      language,
      {
        triggerCharacters: ['.', ':', '<', '"', "'", '/', '@', '#'],
        provideCompletionItems: async (model, position) => {
          try {
            const filePath = model.uri.path;
            const line = position.lineNumber - 1; // Monaco is 1-indexed, LSP is 0-indexed
            const character = position.column - 1;

            const completions = await tauriApi.lspCompletion(
              filePath,
              line,
              character
            );

            return {
              suggestions: completions.map((item) => {
                const suggestion: any = {
                  label: item.label,
                  kind: lspKindToMonaco(item.kind),
                  insertText: item.insertText || item.label,
                };

                if (item.detail) suggestion.detail = item.detail;
                if (item.documentation) {
                  suggestion.documentation = { value: item.documentation };
                }
                if (item.sortText) suggestion.sortText = item.sortText;

                return suggestion;
              }),
            };
          } catch (error) {
            console.error('[LSP] Completion failed:', error);
            return { suggestions: [] };
          }
        },
      }
    );

    return () => disposable.dispose();
  }, [monaco, language, lspStore.enabled]);

  /**
   * Register hover provider
   */
  useEffect(() => {
    if (!monaco || !lspStore.enabled) return;

    const disposable = monaco.languages.registerHoverProvider(language, {
      provideHover: async (model, position) => {
        try {
          const filePath = model.uri.path;
          const line = position.lineNumber - 1;
          const character = position.column - 1;

          const hover = await tauriApi.lspHover(filePath, line, character);

          if (!hover) return null;

          return {
            contents: [{ value: hover.contents }],
          };
        } catch (error) {
          console.error('[LSP] Hover failed:', error);
          return null;
        }
      },
    });

    return () => disposable.dispose();
  }, [monaco, language, lspStore.enabled]);

  /**
   * Register definition provider
   */
  useEffect(() => {
    if (!monaco || !lspStore.enabled) return;

    const disposable = monaco.languages.registerDefinitionProvider(language, {
      provideDefinition: async (model, position) => {
        try {
          const filePath = model.uri.path;
          const line = position.lineNumber - 1;
          const character = position.column - 1;

          const locations = await tauriApi.lspGotoDefinition(
            filePath,
            line,
            character
          );

          return locations.map((loc) => ({
            uri: monaco.Uri.parse(loc.uri),
            range: {
              startLineNumber: loc.startLine + 1,
              startColumn: loc.startCharacter + 1,
              endLineNumber: loc.endLine + 1,
              endColumn: loc.endCharacter + 1,
            },
          }));
        } catch (error) {
          console.error('[LSP] Go to definition failed:', error);
          return [];
        }
      },
    });

    return () => disposable.dispose();
  }, [monaco, language, lspStore.enabled]);

  /**
   * Lifecycle: Notify did_open when component mounts
   */
  useEffect(() => {
    if (monaco && editor && filePath) {
      notifyDidOpen();
    }

    return () => {
      // Notify did_close when component unmounts
      if (isOpenRef.current) {
        notifyDidClose();
      }
    };
  }, [monaco, editor, filePath, notifyDidOpen, notifyDidClose]);

  /**
   * Watch for content changes
   */
  useEffect(() => {
    if (content !== lastContentRef.current) {
      notifyDidChange(content);
    }
  }, [content, notifyDidChange]);

  /**
   * Cleanup on unmount
   */
  useEffect(() => {
    return () => {
      if (changeTimeoutRef.current) {
        clearTimeout(changeTimeoutRef.current);
      }
    };
  }, []);

  return {
    notifyDidSave,
    notifyDidChange,
  };
}
