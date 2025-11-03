import { useState } from 'react';
import { FolderTree, Code, Terminal, Settings } from 'lucide-react';

function App() {
  const [activePanel, setActivePanel] = useState<'editor' | 'terminal' | 'settings'>('editor');

  return (
    <div className="flex h-screen bg-gray-900 text-gray-100">
      {/* Sidebar */}
      <aside className="w-16 bg-gray-800 border-r border-gray-700 flex flex-col items-center py-4 space-y-4">
        <button
          onClick={() => setActivePanel('editor')}
          className={`p-3 rounded-lg transition-colors ${
            activePanel === 'editor'
              ? 'bg-primary-600 text-white'
              : 'text-gray-400 hover:bg-gray-700 hover:text-white'
          }`}
          title="Editor"
        >
          <Code size={24} />
        </button>
        <button
          onClick={() => setActivePanel('terminal')}
          className={`p-3 rounded-lg transition-colors ${
            activePanel === 'terminal'
              ? 'bg-primary-600 text-white'
              : 'text-gray-400 hover:bg-gray-700 hover:text-white'
          }`}
          title="Terminal"
        >
          <Terminal size={24} />
        </button>
        <button
          onClick={() => setActivePanel('settings')}
          className={`p-3 rounded-lg transition-colors ${
            activePanel === 'settings'
              ? 'bg-primary-600 text-white'
              : 'text-gray-400 hover:bg-gray-700 hover:text-white'
          }`}
          title="Settings"
        >
          <Settings size={24} />
        </button>
      </aside>

      {/* File Explorer */}
      <aside className="w-64 bg-gray-800 border-r border-gray-700 p-4">
        <div className="flex items-center space-x-2 mb-4">
          <FolderTree size={20} />
          <h2 className="text-sm font-semibold text-gray-300">EXPLORER</h2>
        </div>
        <div className="text-sm text-gray-400">
          <p>No folder opened</p>
          <button className="mt-2 px-3 py-1.5 bg-primary-600 hover:bg-primary-700 rounded text-white text-xs transition-colors">
            Open Folder
          </button>
        </div>
      </aside>

      {/* Main Content */}
      <main className="flex-1 flex flex-col">
        {/* Tab Bar */}
        <div className="h-10 bg-gray-800 border-b border-gray-700 flex items-center px-4">
          <div className="text-sm text-gray-400">
            Welcome to AIT42 Editor
          </div>
        </div>

        {/* Editor Area */}
        <div className="flex-1 bg-gray-900 flex items-center justify-center">
          <div className="text-center space-y-4">
            <h1 className="text-4xl font-bold text-primary-400">AIT42 Editor</h1>
            <p className="text-xl text-gray-400">Modern Code Editor with AI Integration</p>
            <div className="flex space-x-4 justify-center mt-8">
              <button className="px-6 py-3 bg-primary-600 hover:bg-primary-700 rounded-lg text-white font-medium transition-colors">
                Open Folder
              </button>
              <button className="px-6 py-3 bg-gray-700 hover:bg-gray-600 rounded-lg text-white font-medium transition-colors">
                New File
              </button>
            </div>
          </div>
        </div>

        {/* Status Bar */}
        <div className="h-8 bg-primary-600 flex items-center justify-between px-4 text-sm">
          <div className="flex items-center space-x-4">
            <span>Ready</span>
          </div>
          <div className="flex items-center space-x-4">
            <span>UTF-8</span>
            <span>LF</span>
            <span>Plain Text</span>
          </div>
        </div>
      </main>
    </div>
  );
}

export default App;
