/**
 * Settings Panel Component
 *
 * Displays editor and application settings
 */
import { X } from 'lucide-react';
import { useSettingsStore } from '@/store/settingsStore';

export function SettingsPanel() {
  const {
    editor,
    application,
    showSettingsPanel,
    toggleSettingsPanel,
    setFontSize,
    setTabSize,
    toggleLineNumbers,
    toggleWordWrap,
    toggleMinimap,
    toggleAutoSave,
    setAutoSaveDelay,
    toggleConfirmDelete,
    resetSettings,
  } = useSettingsStore();

  if (!showSettingsPanel) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-[#1E1E1E] border border-gray-700 rounded-lg w-[600px] max-h-[80vh] overflow-y-auto">
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-gray-700">
          <h2 className="text-lg font-semibold text-gray-100">設定</h2>
          <button
            onClick={toggleSettingsPanel}
            className="p-1 hover:bg-gray-700 rounded transition-colors"
          >
            <X size={20} className="text-gray-400" />
          </button>
        </div>

        {/* Content */}
        <div className="p-4 space-y-6">
          {/* Editor Settings */}
          <section>
            <h3 className="text-md font-semibold text-gray-200 mb-3">エディター</h3>
            <div className="space-y-3">
              {/* Font Size */}
              <div className="flex items-center justify-between">
                <label className="text-sm text-gray-300">Font Size</label>
                <input
                  type="number"
                  min={8}
                  max={32}
                  value={editor.fontSize}
                  onChange={(e) => setFontSize(parseInt(e.target.value))}
                  className="w-20 px-2 py-1 bg-[#2D2D30] border border-gray-600 rounded text-gray-200 text-sm"
                />
              </div>

              {/* Tab Size */}
              <div className="flex items-center justify-between">
                <label className="text-sm text-gray-300">Tab Size</label>
                <input
                  type="number"
                  min={2}
                  max={8}
                  value={editor.tabSize}
                  onChange={(e) => setTabSize(parseInt(e.target.value))}
                  className="w-20 px-2 py-1 bg-[#2D2D30] border border-gray-600 rounded text-gray-200 text-sm"
                />
              </div>

              {/* Line Numbers */}
              <div className="flex items-center justify-between">
                <label className="text-sm text-gray-300">Line Numbers</label>
                <button
                  onClick={toggleLineNumbers}
                  className={`w-12 h-6 rounded-full transition-colors ${
                    editor.lineNumbers ? 'bg-[#007ACC]' : 'bg-gray-600'
                  }`}
                >
                  <div
                    className={`w-5 h-5 bg-white rounded-full transition-transform ${
                      editor.lineNumbers ? 'translate-x-6' : 'translate-x-1'
                    }`}
                  />
                </button>
              </div>

              {/* Word Wrap */}
              <div className="flex items-center justify-between">
                <label className="text-sm text-gray-300">Word Wrap</label>
                <button
                  onClick={toggleWordWrap}
                  className={`w-12 h-6 rounded-full transition-colors ${
                    editor.wordWrap ? 'bg-[#007ACC]' : 'bg-gray-600'
                  }`}
                >
                  <div
                    className={`w-5 h-5 bg-white rounded-full transition-transform ${
                      editor.wordWrap ? 'translate-x-6' : 'translate-x-1'
                    }`}
                  />
                </button>
              </div>

              {/* Minimap */}
              <div className="flex items-center justify-between">
                <label className="text-sm text-gray-300">Minimap</label>
                <button
                  onClick={toggleMinimap}
                  className={`w-12 h-6 rounded-full transition-colors ${
                    editor.minimap ? 'bg-[#007ACC]' : 'bg-gray-600'
                  }`}
                >
                  <div
                    className={`w-5 h-5 bg-white rounded-full transition-transform ${
                      editor.minimap ? 'translate-x-6' : 'translate-x-1'
                    }`}
                  />
                </button>
              </div>
            </div>
          </section>

          {/* Application Settings */}
          <section>
            <h3 className="text-md font-semibold text-gray-200 mb-3">Application</h3>
            <div className="space-y-3">
              {/* Auto Save */}
              <div className="flex items-center justify-between">
                <label className="text-sm text-gray-300">Auto Save</label>
                <button
                  onClick={toggleAutoSave}
                  className={`w-12 h-6 rounded-full transition-colors ${
                    application.autoSave ? 'bg-[#007ACC]' : 'bg-gray-600'
                  }`}
                >
                  <div
                    className={`w-5 h-5 bg-white rounded-full transition-transform ${
                      application.autoSave ? 'translate-x-6' : 'translate-x-1'
                    }`}
                  />
                </button>
              </div>

              {/* Auto Save Delay */}
              {application.autoSave && (
                <div className="flex items-center justify-between">
                  <label className="text-sm text-gray-300">Auto Save Delay (ms)</label>
                  <input
                    type="number"
                    min={500}
                    max={5000}
                    step={100}
                    value={application.autoSaveDelay}
                    onChange={(e) => setAutoSaveDelay(parseInt(e.target.value))}
                    className="w-24 px-2 py-1 bg-[#2D2D30] border border-gray-600 rounded text-gray-200 text-sm"
                  />
                </div>
              )}

              {/* Confirm Delete */}
              <div className="flex items-center justify-between">
                <label className="text-sm text-gray-300">Confirm Delete</label>
                <button
                  onClick={toggleConfirmDelete}
                  className={`w-12 h-6 rounded-full transition-colors ${
                    application.confirmDelete ? 'bg-[#007ACC]' : 'bg-gray-600'
                  }`}
                >
                  <div
                    className={`w-5 h-5 bg-white rounded-full transition-transform ${
                      application.confirmDelete ? 'translate-x-6' : 'translate-x-1'
                    }`}
                  />
                </button>
              </div>
            </div>
          </section>

          {/* Actions */}
          <div className="flex justify-end pt-4 border-t border-gray-700">
            <button
              onClick={resetSettings}
              className="px-4 py-2 bg-[#3E3E42] hover:bg-[#505050] text-white rounded transition-colors"
            >
              Reset to Defaults
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
