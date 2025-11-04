import React, { useState, useEffect, useRef } from 'react';
import { X, RefreshCw, Trophy, Clock, CheckCircle, AlertCircle } from 'lucide-react';
import { listen } from '@tauri-apps/api/event';

interface CompetitionMonitorPanelProps {
  isVisible: boolean;
  onClose: () => void;
  competitionId: string;
  instanceCount: number;
  task: string;
}

interface InstanceState {
  id: number;
  status: 'running' | 'completed' | 'error' | 'idle';
  output: string;
  error?: string;
  startTime?: number;
  endTime?: number;
}

export const CompetitionMonitorPanel: React.FC<CompetitionMonitorPanelProps> = ({
  isVisible,
  onClose,
  competitionId,
  instanceCount,
  task,
}) => {
  const [instances, setInstances] = useState<InstanceState[]>([]);
  const [selectedInstance, setSelectedInstance] = useState<number>(0);
  const outputRef = useRef<HTMLDivElement>(null);
  const [autoScroll, setAutoScroll] = useState(true);

  // インスタンス初期化
  useEffect(() => {
    if (!isVisible) return;

    const initialInstances: InstanceState[] = Array.from({ length: instanceCount }, (_, i) => ({
      id: i + 1,
      status: 'running',
      output: '',
      startTime: Date.now(),
    }));
    setInstances(initialInstances);
  }, [isVisible, instanceCount]);

  // 出力リスナー
  useEffect(() => {
    if (!isVisible) return;

    const unlistenPromise = listen<{
      instance: number;
      output: string;
      status?: 'completed' | 'error';
      error?: string;
    }>('competition-output', (event) => {
      const { instance, output, status, error } = event.payload;

      setInstances((prev) =>
        prev.map((inst) =>
          inst.id === instance
            ? {
                ...inst,
                output: inst.output + output,
                status: status || inst.status,
                error: error || inst.error,
                endTime: status ? Date.now() : inst.endTime,
              }
            : inst
        )
      );
    });

    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, [isVisible]);

  // 自動スクロール
  useEffect(() => {
    if (autoScroll && outputRef.current) {
      outputRef.current.scrollTop = outputRef.current.scrollHeight;
    }
  }, [instances, selectedInstance, autoScroll]);

  if (!isVisible) return null;

  const currentInstance = instances[selectedInstance];
  const completedCount = instances.filter((i) => i.status === 'completed').length;
  const errorCount = instances.filter((i) => i.status === 'error').length;
  const runningCount = instances.filter((i) => i.status === 'running').length;

  const getStatusIcon = (status: InstanceState['status']) => {
    switch (status) {
      case 'running':
        return <RefreshCw className="w-4 h-4 animate-spin text-blue-500" />;
      case 'completed':
        return <CheckCircle className="w-4 h-4 text-green-500" />;
      case 'error':
        return <AlertCircle className="w-4 h-4 text-red-500" />;
      default:
        return <Clock className="w-4 h-4 text-gray-400" />;
    }
  };

  const getStatusText = (status: InstanceState['status']) => {
    switch (status) {
      case 'running':
        return '実行中';
      case 'completed':
        return '完了';
      case 'error':
        return 'エラー';
      default:
        return '待機中';
    }
  };

  const formatDuration = (startTime?: number, endTime?: number) => {
    if (!startTime) return '-';
    const duration = (endTime || Date.now()) - startTime;
    const seconds = Math.floor(duration / 1000);
    const minutes = Math.floor(seconds / 60);
    return `${minutes}:${(seconds % 60).toString().padStart(2, '0')}`;
  };

  return (
    <div className="fixed inset-y-0 right-0 w-[600px] bg-editor-elevated border-l border-editor-border shadow-2xl flex flex-col z-40">
      {/* ヘッダー */}
      <div className="flex items-center justify-between px-4 py-3 border-b border-editor-border bg-editor-surface">
        <div className="flex items-center gap-3">
          <Trophy className="w-5 h-5 text-accent-primary" />
          <div>
            <h2 className="text-sm font-semibold text-text-primary">コンペティション実行モニター</h2>
            <p className="text-xs text-text-tertiary">
              {runningCount}実行中 | {completedCount}完了 | {errorCount}エラー
            </p>
          </div>
        </div>
        <button
          onClick={onClose}
          className="p-1 hover:bg-editor-border/30 rounded transition-colors"
          title="閉じる"
        >
          <X size={18} className="text-text-tertiary" />
        </button>
      </div>

      {/* タスク表示 */}
      <div className="px-4 py-2 bg-editor-bg border-b border-editor-border">
        <p className="text-xs font-medium text-text-secondary">実行タスク:</p>
        <p className="text-xs text-text-primary mt-1 line-clamp-2">{task}</p>
      </div>

      <div className="flex flex-1 overflow-hidden">
        {/* インスタンス一覧 */}
        <div className="w-40 border-r border-editor-border overflow-y-auto">
          <div className="p-2 space-y-1">
            {instances.map((instance, index) => (
              <button
                key={instance.id}
                onClick={() => setSelectedInstance(index)}
                className={`w-full p-2 rounded-lg text-left transition-colors ${
                  selectedInstance === index
                    ? 'bg-accent-primary/10 border border-accent-primary'
                    : 'bg-editor-bg border border-transparent hover:bg-editor-hover'
                }`}
              >
                <div className="flex items-center justify-between mb-1">
                  <span className="text-xs font-medium text-text-primary">#{instance.id}</span>
                  {getStatusIcon(instance.status)}
                </div>
                <div className="text-xs text-text-tertiary">
                  {formatDuration(instance.startTime, instance.endTime)}
                </div>
              </button>
            ))}
          </div>
        </div>

        {/* 出力表示エリア */}
        <div className="flex-1 flex flex-col">
          {/* ツールバー */}
          <div className="flex items-center justify-between px-3 py-2 border-b border-editor-border bg-editor-surface">
            <div className="flex items-center gap-2">
              {getStatusIcon(currentInstance?.status || 'idle')}
              <span className="text-xs font-medium text-text-primary">
                インスタンス {currentInstance?.id} - {getStatusText(currentInstance?.status || 'idle')}
              </span>
            </div>
            <label className="flex items-center gap-2 text-xs text-text-secondary cursor-pointer">
              <input
                type="checkbox"
                checked={autoScroll}
                onChange={(e) => setAutoScroll(e.target.checked)}
                className="rounded"
              />
              自動スクロール
            </label>
          </div>

          {/* 出力エリア */}
          <div
            ref={outputRef}
            className="flex-1 p-3 bg-[#1e1e1e] text-[#d4d4d4] font-mono text-xs overflow-y-auto"
          >
            {currentInstance?.error ? (
              <div className="text-red-400 mb-3 p-2 bg-red-900/20 rounded">
                <p className="font-bold mb-1">エラー:</p>
                <pre className="whitespace-pre-wrap text-xs">{currentInstance.error}</pre>
              </div>
            ) : null}
            <pre className="whitespace-pre-wrap text-xs">
              {currentInstance?.output || '出力を待機中...\n\n各Claude Codeインスタンスの実行出力がここに表示されます。'}
            </pre>
          </div>

          {/* フッター統計 */}
          <div className="px-3 py-2 border-t border-editor-border bg-editor-surface flex items-center justify-between text-xs">
            <div className="flex items-center gap-4 text-text-tertiary">
              <span>実行時間: {formatDuration(currentInstance?.startTime, currentInstance?.endTime)}</span>
              <span>出力行数: {currentInstance?.output.split('\n').length || 0}</span>
            </div>
            {currentInstance?.status === 'completed' && (
              <span className="flex items-center gap-1 text-green-500 font-medium">
                <CheckCircle className="w-3 h-3" />
                完了
              </span>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};
