/**
 * File Preview Component
 *
 * Display file contents with syntax highlighting
 */
import React, { useMemo } from 'react';
import { FileText, Copy, AlertTriangle, Loader2, FileCode } from 'lucide-react';
import { useWorktreeStore } from '@/store/worktreeStore';

const MAX_FILE_SIZE_WARN = 1024 * 1024; // 1MB

/**
 * Get file extension from path
 */
const getFileExtension = (path: string): string => {
  const match = path.match(/\.([^.]+)$/);
  return match ? match[1].toLowerCase() : '';
};

/**
 * Get language label from extension
 */
const getLanguageLabel = (ext: string): string => {
  const languages: Record<string, string> = {
    ts: 'TypeScript',
    tsx: 'TypeScript React',
    js: 'JavaScript',
    jsx: 'JavaScript React',
    rs: 'Rust',
    toml: 'TOML',
    json: 'JSON',
    md: 'Markdown',
    txt: 'Text',
    css: 'CSS',
    scss: 'SCSS',
    html: 'HTML',
    py: 'Python',
    go: 'Go',
    c: 'C',
    cpp: 'C++',
    java: 'Java',
  };
  return languages[ext] || ext.toUpperCase();
};

/**
 * Get breadcrumb from file path
 */
const getBreadcrumb = (path: string): string[] => {
  return path.split('/').filter(Boolean);
};

export const FilePreview: React.FC = () => {
  const { selectedFile, fileContent, isLoadingFile, error } = useWorktreeStore();

  const fileSize = useMemo(() => {
    if (!fileContent) return 0;
    return new Blob([fileContent]).size;
  }, [fileContent]);

  const lineCount = useMemo(() => {
    if (!fileContent) return 0;
    return fileContent.split('\n').length;
  }, [fileContent]);

  const handleCopy = async () => {
    if (fileContent) {
      await navigator.clipboard.writeText(fileContent);
    }
  };

  // No file selected
  if (!selectedFile) {
    return (
      <div className="flex flex-col items-center justify-center h-full text-gray-400">
        <FileText className="w-16 h-16 mb-4 text-gray-600" />
        <p className="text-sm">Select a file to preview</p>
      </div>
    );
  }

  // Loading state
  if (isLoadingFile) {
    return (
      <div className="flex flex-col items-center justify-center h-full">
        <Loader2 className="w-8 h-8 animate-spin text-blue-400 mb-4" />
        <p className="text-sm text-gray-400">Loading file...</p>
      </div>
    );
  }

  // Error state
  if (error) {
    return (
      <div className="flex flex-col items-center justify-center h-full">
        <AlertTriangle className="w-12 h-12 text-red-400 mb-4" />
        <p className="text-sm text-red-400">{error}</p>
      </div>
    );
  }

  const breadcrumb = getBreadcrumb(selectedFile);
  const extension = getFileExtension(selectedFile);
  const language = getLanguageLabel(extension);

  return (
    <div className="flex flex-col h-full bg-gray-900">
      {/* Header */}
      <div className="flex-shrink-0 border-b border-gray-700 bg-gray-800">
        {/* Breadcrumb */}
        <div className="px-4 py-2 border-b border-gray-700">
          <div className="flex items-center space-x-2 text-xs text-gray-400 overflow-x-auto">
            {breadcrumb.map((part, idx) => (
              <React.Fragment key={idx}>
                {idx > 0 && <span>/</span>}
                <span
                  className={idx === breadcrumb.length - 1 ? 'text-gray-200 font-semibold' : ''}
                >
                  {part}
                </span>
              </React.Fragment>
            ))}
          </div>
        </div>

        {/* Toolbar */}
        <div className="flex items-center justify-between px-4 py-2">
          <div className="flex items-center space-x-4 text-xs text-gray-400">
            <div className="flex items-center space-x-1">
              <FileCode className="w-3 h-3" />
              <span>{language}</span>
            </div>
            <span>{lineCount} lines</span>
            <span>{(fileSize / 1024).toFixed(1)} KB</span>
          </div>

          <button
            onClick={handleCopy}
            className="
              flex items-center space-x-1 px-3 py-1
              bg-gray-700 hover:bg-gray-600
              text-gray-200 text-xs rounded
              transition-colors duration-200
            "
            title="Copy to clipboard"
          >
            <Copy className="w-3 h-3" />
            <span>Copy</span>
          </button>
        </div>

        {/* Large file warning */}
        {fileSize > MAX_FILE_SIZE_WARN && (
          <div className="px-4 py-2 bg-yellow-500/10 border-y border-yellow-500/30">
            <div className="flex items-center space-x-2 text-xs text-yellow-400">
              <AlertTriangle className="w-4 h-4 flex-shrink-0" />
              <span>
                Large file ({(fileSize / 1024 / 1024).toFixed(1)} MB) - rendering may be slow
              </span>
            </div>
          </div>
        )}
      </div>

      {/* Content */}
      <div className="flex-1 overflow-auto">
        <div className="relative">
          {/* Line numbers */}
          <div className="absolute left-0 top-0 bottom-0 w-12 bg-gray-800/50 border-r border-gray-700 select-none">
            {fileContent?.split('\n').map((_, idx) => (
              <div
                key={idx}
                className="px-2 text-right text-xs text-gray-500 leading-6"
                style={{ height: '24px' }}
              >
                {idx + 1}
              </div>
            ))}
          </div>

          {/* Code content */}
          <pre className="pl-14 pr-4 py-3 text-sm font-mono leading-6 text-gray-200">
            <code>{fileContent}</code>
          </pre>
        </div>
      </div>
    </div>
  );
};
