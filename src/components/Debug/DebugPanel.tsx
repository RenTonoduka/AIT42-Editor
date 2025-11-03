/**
 * Debug Panel Component
 *
 * Provides debugging interface with:
 * - Debug configuration selector
 * - Debug controls (start, stop, pause, continue, step)
 * - Breakpoints list
 * - Debug output console
 */

import React, { useState } from 'react';
import {
  Play,
  Square,
  Pause,
  SkipForward,
  ArrowDownToLine,
  ArrowUpFromLine,
  Circle,
  X,
} from 'lucide-react';
import { useDebugStore } from '@/store/debugStore';

/**
 * Debug toolbar with controls
 */
const DebugToolbar: React.FC = () => {
  const {
    state,
    activeConfig,
    startDebugging,
    stopDebugging,
    pauseDebugging,
    continueDebugging,
    stepOver,
    stepInto,
    stepOut,
    configurations,
  } = useDebugStore();

  const [selectedConfig, setSelectedConfig] = useState<string>(
    configurations[0]?.name || ''
  );

  const handleStart = () => {
    const config = configurations.find((c) => c.name === selectedConfig);
    if (config) {
      startDebugging(config);
    }
  };

  const isStopped = state === 'stopped' || state === 'terminated';
  const isRunning = state === 'running';
  const isPaused = state === 'paused';

  return (
    <div className="flex items-center gap-2 px-3 py-2 border-b border-[#2A2D2E]">
      {/* Configuration selector */}
      {isStopped && (
        <select
          className="px-2 py-1 bg-[#3E3E42] text-[#CCCCCC] text-[12px] rounded border border-[#454545] focus:border-[#007ACC] focus:outline-none"
          value={selectedConfig}
          onChange={(e) => setSelectedConfig(e.target.value)}
        >
          {configurations.length === 0 ? (
            <option>No configurations</option>
          ) : (
            configurations.map((config) => (
              <option key={config.name} value={config.name}>
                {config.name}
              </option>
            ))
          )}
        </select>
      )}

      {/* Active config display */}
      {!isStopped && activeConfig && (
        <span className="text-[12px] text-[#CCCCCC] px-2">
          {activeConfig.name}
        </span>
      )}

      {/* Debug controls */}
      <div className="flex items-center gap-1">
        {isStopped && (
          <button
            className="p-1.5 hover:bg-[#3E3E42] rounded text-[#4EC9B0] transition-colors"
            onClick={handleStart}
            title="Start Debugging (F5)"
            disabled={configurations.length === 0}
          >
            <Play size={16} />
          </button>
        )}

        {!isStopped && (
          <>
            <button
              className="p-1.5 hover:bg-[#3E3E42] rounded text-[#F48771] transition-colors"
              onClick={stopDebugging}
              title="Stop (Shift+F5)"
            >
              <Square size={16} />
            </button>

            {isRunning && (
              <button
                className="p-1.5 hover:bg-[#3E3E42] rounded text-[#CCCCCC] transition-colors"
                onClick={pauseDebugging}
                title="Pause (F6)"
              >
                <Pause size={16} />
              </button>
            )}

            {isPaused && (
              <>
                <button
                  className="p-1.5 hover:bg-[#3E3E42] rounded text-[#4EC9B0] transition-colors"
                  onClick={continueDebugging}
                  title="Continue (F5)"
                >
                  <Play size={16} />
                </button>

                <div className="w-px h-6 bg-[#454545] mx-1" />

                <button
                  className="p-1.5 hover:bg-[#3E3E42] rounded text-[#CCCCCC] transition-colors"
                  onClick={stepOver}
                  title="Step Over (F10)"
                >
                  <SkipForward size={16} />
                </button>

                <button
                  className="p-1.5 hover:bg-[#3E3E42] rounded text-[#CCCCCC] transition-colors"
                  onClick={stepInto}
                  title="Step Into (F11)"
                >
                  <ArrowDownToLine size={16} />
                </button>

                <button
                  className="p-1.5 hover:bg-[#3E3E42] rounded text-[#CCCCCC] transition-colors"
                  onClick={stepOut}
                  title="Step Out (Shift+F11)"
                >
                  <ArrowUpFromLine size={16} />
                </button>
              </>
            )}
          </>
        )}
      </div>

      {/* Status indicator */}
      <div className="ml-auto flex items-center gap-2 text-[11px]">
        <div
          className={`w-2 h-2 rounded-full ${
            isRunning
              ? 'bg-[#4EC9B0]'
              : isPaused
              ? 'bg-[#CE9178]'
              : 'bg-[#858585]'
          }`}
        />
        <span className="text-[#CCCCCC] capitalize">{state}</span>
      </div>
    </div>
  );
};

/**
 * Breakpoints list view
 */
const BreakpointsList: React.FC = () => {
  const { getAllBreakpoints, removeBreakpoint } = useDebugStore();
  const breakpoints = getAllBreakpoints();

  if (breakpoints.length === 0) {
    return (
      <div className="flex items-center justify-center h-full text-[#858585] text-[12px]">
        No breakpoints set
      </div>
    );
  }

  return (
    <div className="overflow-y-auto">
      {breakpoints.map((bp) => {
        const fileName = bp.filePath.split('/').pop();
        const folderPath = bp.filePath.substring(
          0,
          bp.filePath.length - (fileName?.length || 0)
        );

        return (
          <div
            key={bp.id}
            className="flex items-center gap-2 px-3 py-2 hover:bg-[#2A2D2E] cursor-pointer transition-colors border-b border-[#2A2D2E] last:border-b-0"
          >
            <Circle size={12} className="text-[#F48771] fill-current" />
            <div className="flex-1 min-w-0">
              <div className="text-[12px] text-[#CCCCCC] truncate">
                {fileName}:{bp.line + 1}
              </div>
              <div className="text-[11px] text-[#858585] truncate">
                {folderPath}
              </div>
            </div>
            <button
              className="p-1 hover:bg-[#3E3E42] rounded transition-colors"
              onClick={() => removeBreakpoint(bp.id)}
              title="Remove breakpoint"
            >
              <X size={14} className="text-[#858585]" />
            </button>
          </div>
        );
      })}
    </div>
  );
};

/**
 * Debug output console
 */
const DebugOutput: React.FC = () => {
  const { outputLines, clearOutput } = useDebugStore();

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-between px-3 py-1.5 border-b border-[#2A2D2E]">
        <span className="text-[11px] text-[#CCCCCC] font-semibold">
          DEBUG OUTPUT
        </span>
        {outputLines.length > 0 && (
          <button
            className="text-[11px] text-[#858585] hover:text-[#CCCCCC] transition-colors"
            onClick={clearOutput}
          >
            Clear
          </button>
        )}
      </div>
      <div className="flex-1 overflow-y-auto p-2 font-mono text-[11px] text-[#CCCCCC]">
        {outputLines.length === 0 ? (
          <div className="text-[#858585] italic">No output</div>
        ) : (
          outputLines.map((line, index) => (
            <div key={index} className="py-0.5">
              {line}
            </div>
          ))
        )}
      </div>
    </div>
  );
};

/**
 * Debug Panel Component
 */
export const DebugPanel: React.FC = () => {
  const { showDebugPanel, hideDebug } = useDebugStore();
  const [activeTab, setActiveTab] = useState<'breakpoints' | 'output'>(
    'breakpoints'
  );

  if (!showDebugPanel) {
    return null;
  }

  return (
    <div className="h-full flex flex-col bg-[#1E1E1E] text-[#CCCCCC] border-t border-[#2A2D2E]">
      {/* Header with toolbar */}
      <div className="flex items-center justify-between border-b border-[#2A2D2E]">
        <div className="flex items-center">
          <h3 className="text-[13px] font-semibold px-3 py-2">DEBUG</h3>
        </div>
        <button
          className="p-2 hover:bg-[#3E3E42] rounded transition-colors mx-1"
          onClick={hideDebug}
          title="Close debug panel"
        >
          <X size={16} />
        </button>
      </div>

      {/* Debug toolbar */}
      <DebugToolbar />

      {/* Tabs */}
      <div className="flex items-center border-b border-[#2A2D2E]">
        <button
          className={`px-3 py-2 text-[12px] transition-colors ${
            activeTab === 'breakpoints'
              ? 'text-[#CCCCCC] border-b-2 border-[#007ACC]'
              : 'text-[#858585] hover:text-[#CCCCCC]'
          }`}
          onClick={() => setActiveTab('breakpoints')}
        >
          Breakpoints
        </button>
        <button
          className={`px-3 py-2 text-[12px] transition-colors ${
            activeTab === 'output'
              ? 'text-[#CCCCCC] border-b-2 border-[#007ACC]'
              : 'text-[#858585] hover:text-[#CCCCCC]'
          }`}
          onClick={() => setActiveTab('output')}
        >
          Output
        </button>
      </div>

      {/* Content area */}
      <div className="flex-1 overflow-hidden">
        {activeTab === 'breakpoints' && <BreakpointsList />}
        {activeTab === 'output' && <DebugOutput />}
      </div>
    </div>
  );
};
