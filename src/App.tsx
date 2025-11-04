import React, { useState, useEffect, useCallback } from 'react';
import { Terminal, FileText, Settings, Activity, GitBranch, Users, Layout } from 'lucide-react';
import Editor from './components/Editor/Editor';
import TerminalPanel from './components/Terminal/TerminalPanel';
import FileTree from './components/FileTree/FileTree';
import StatusBar from './components/StatusBar/StatusBar';
import SettingsPanel from './components/Settings/SettingsPanel';
import AIAssistantPanel from './components/AI/AIAssistantPanel';
import MultiAgentPanel, { ClaudeCodeInstance } from './components/AI/MultiAgentPanel';
import { setupEventHandlers } from './events/eventHandlers';
import type { FileTreeItem, Settings as SettingsType } from './types';

// View Mode Type
type ViewMode = 'editor' | 'multi-agent';

function App() {
  const [files, setFiles] = useState<FileTreeItem[]>([]);
  const [selectedFile, setSelectedFile] = useState<FileTreeItem | null>(null);
  const [settings, setSettings] = useState<SettingsType>({
    theme: 'dark',
    fontSize: 14,
    tabSize: 2,
  });
  const [showSettings, setShowSettings] = useState(false);
  const [showAI, setShowAI] = useState(false);
  const [claudeInstances, setClaudeInstances] = useState<ClaudeCodeInstance[]>([]);
  const [viewMode, setViewMode] = useState<ViewMode>('editor');

  useEffect(() => {
    setupEventHandlers({
      setFiles,
      setSelectedFile,
      setClaudeInstances,
    });
  }, []);

  const handleFileSelect = useCallback((file: FileTreeItem) => {
    setSelectedFile(file);
  }, []);

  const handleSave = useCallback((content: string) => {
    if (selectedFile) {
      console.log('Saving file:', selectedFile.path);
    }
  }, [selectedFile]);

  const handleSettingsSave = useCallback((newSettings: SettingsType) => {
    setSettings(newSettings);
    setShowSettings(false);
  }, []);

  // Auto-switch to multi-agent view when instances exist
  useEffect(() => {
    if (claudeInstances.length > 0) {
      setViewMode('multi-agent');
    }
  }, [claudeInstances.length]);

  return (
    <div className="flex flex-col h-screen bg-gray-900 text-gray-100">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-2 bg-gray-800 border-b border-gray-700">
        <div className="flex items-center space-x-4">
          <FileText className="w-5 h-5 text-blue-400" />
          <span className="text-sm font-semibold">AIT42-Editor</span>
          {selectedFile && (
            <span className="text-xs text-gray-400">{selectedFile.name}</span>
          )}
        </div>

        {/* View Mode Toggle */}
        <div className="flex items-center space-x-2">
          <button
            onClick={() => setViewMode('editor')}
            className={`px-3 py-1 rounded-md text-sm transition-colors ${
              viewMode === 'editor'
                ? 'bg-blue-600 text-white'
                : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
            }`}
          >
            <Layout className="w-4 h-4 inline-block mr-1" />
            Editor
          </button>
          <button
            onClick={() => setViewMode('multi-agent')}
            className={`px-3 py-1 rounded-md text-sm transition-colors ${
              viewMode === 'multi-agent'
                ? 'bg-purple-600 text-white'
                : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
            } ${claudeInstances.length === 0 ? 'opacity-50 cursor-not-allowed' : ''}`}
            disabled={claudeInstances.length === 0}
          >
            <Users className="w-4 h-4 inline-block mr-1" />
            Multi-Agent ({claudeInstances.length})
          </button>
        </div>

        <div className="flex items-center space-x-2">
          <button
            onClick={() => setShowAI(!showAI)}
            className={`p-2 rounded hover:bg-gray-700 ${
              showAI ? 'bg-gray-700' : ''
            }`}
          >
            <Activity className="w-4 h-4" />
          </button>
          <button
            onClick={() => setShowSettings(!showSettings)}
            className={`p-2 rounded hover:bg-gray-700 ${
              showSettings ? 'bg-gray-700' : ''
            }`}
          >
            <Settings className="w-4 h-4" />
          </button>
        </div>
      </div>

      {/* Main Content */}
      <div className="flex flex-1 overflow-hidden">
        {/* Editor View Mode */}
        {viewMode === 'editor' && (
          <>
            {/* Left Sidebar - File Tree */}
            <div className="w-64 bg-gray-800 border-r border-gray-700 overflow-y-auto">
              <FileTree files={files} onFileSelect={handleFileSelect} />
            </div>

            {/* Main Editor Area */}
            <div className="flex-1 flex flex-col">
              <Editor
                file={selectedFile}
                settings={settings}
                onSave={handleSave}
              />
              <TerminalPanel />
            </div>

            {/* Right Sidebar - AI Assistant */}
            {showAI && (
              <div className="w-96 bg-gray-800 border-l border-gray-700 overflow-y-auto">
                <AIAssistantPanel />
              </div>
            )}
          </>
        )}

        {/* Multi-Agent View Mode */}
        {viewMode === 'multi-agent' && (
          <div className="flex-1 bg-gray-900">
            <MultiAgentPanel instances={claudeInstances} />
          </div>
        )}
      </div>

      {/* Settings Modal */}
      {showSettings && (
        <SettingsPanel
          settings={settings}
          onSave={handleSettingsSave}
          onClose={() => setShowSettings(false)}
        />
      )}

      {/* Status Bar */}
      <StatusBar selectedFile={selectedFile} settings={settings} />
    </div>
  );
}

export default App;
