# Session History System

AIT42 Editor v1.6.0の新機能: Vibe Kanban風のWorktreeセッション履歴管理システム

## 概要

Session History Systemは、Competition/Ensemble/Debateモードで作成されたWorktreeセッションを永続化し、Kanban Board UIで視覚的に管理できる機能です。

Vibe Kanbanのタスク中心アプローチとAtlassian Design Systemのインタラクションパターンをベースに設計されています。

## 主要機能

### 1. 永続的セッション管理
- **自動保存**: すべてのセッションを `.ait42/sessions.json` に保存
- **履歴追跡**: 過去のすべてのCompetition/Ensemble/Debateを記録
- **メタデータ**: インスタンス情報、実行時間、コード変更統計

### 2. Kanban Board UI
- **4カラム構成**: Running / Paused / Completed / Failed
- **ドラッグ&ドロップ**: セッションステータスの変更
- **リアルタイム更新**: Zustand state管理によるリアクティブUI

### 3. 詳細ビューモーダル
- **Overview Tab**: サマリーカード + インスタンス一覧
- **Worktrees Tab**: WorktreeExplorerとの統合
- **Metrics Tab**: 将来の拡張用プレースホルダー
- **Chat Tab**: 対話型チャットインターフェース

### 4. 対話型チャット
- **tmux統合**: 実行中のClaude Codeインスタンスと対話
- **コマンド送信**: リアルタイムでコマンド実行
- **出力キャプチャ**: コマンド結果の自動表示
- **永続化**: チャット履歴をセッションに保存

## アーキテクチャ

```
┌─────────────────────────────────────────────────────┐
│                  Frontend (React)                    │
├─────────────────────────────────────────────────────┤
│  SessionHistory (Main Component)                     │
│  ├── SessionFilters (Filter/Sort/Search)             │
│  ├── KanbanBoard (Drag & Drop)                       │
│  │   ├── KanbanColumn x4 (Drop Zones)                │
│  │   │   └── SessionCard (Draggable)                 │
│  └── SessionDetailView (Modal)                       │
│      ├── Overview Tab                                │
│      ├── Worktrees Tab                               │
│      ├── Metrics Tab                                 │
│      └── Chat Tab (ChatPanel)                        │
├─────────────────────────────────────────────────────┤
│           Zustand State Management                   │
│  sessionHistoryStore.ts                              │
│  ├── sessions: WorktreeSession[]                     │
│  ├── filters & sorting                               │
│  └── CRUD operations                                 │
├─────────────────────────────────────────────────────┤
│              Tauri IPC Layer                         │
│  7 Session History Commands:                         │
│  ├── create_session                                  │
│  ├── update_session                                  │
│  ├── get_session                                     │
│  ├── get_all_sessions                                │
│  ├── delete_session                                  │
│  ├── add_chat_message                                │
│  └── update_instance_status                          │
├─────────────────────────────────────────────────────┤
│            Backend (Rust/Tauri)                      │
│  src-tauri/src/commands/session_history.rs           │
│  ├── JSON persistence                                │
│  └── .ait42/sessions.json storage                    │
└─────────────────────────────────────────────────────┘
```

## データモデル

### WorktreeSession
```typescript
interface WorktreeSession {
  id: string;                      // 一意のセッションID
  type: 'competition' | 'ensemble' | 'debate';
  task: string;                    // タスク記述
  status: 'running' | 'completed' | 'failed' | 'paused';
  createdAt: string;               // ISO 8601
  updatedAt: string;
  completedAt?: string;

  // インスタンス
  instances: WorktreeInstance[];

  // チャット履歴
  chatHistory: ChatMessage[];

  // メタデータ
  model?: string;
  timeoutSeconds?: number;
  preserveWorktrees?: boolean;
  runtimeMix?: Array<'claude' | 'codex' | 'gemini'>;
  winnerId?: number;

  // 統計
  totalDuration?: number;
  totalFilesChanged?: number;
  totalLinesAdded?: number;
  totalLinesDeleted?: number;
}
```

### WorktreeInstance
```typescript
interface WorktreeInstance {
  instanceId: number;
  worktreePath: string;
  branch: string;
  agentName: string;
  status: InstanceStatus;
  tmuxSessionId: string;
  output?: string;
  startTime?: string;
  endTime?: string;
  filesChanged?: number;
  linesAdded?: number;
  linesDeleted?: number;
  runtime?: 'claude' | 'codex' | 'gemini';
  model?: string;
  runtimeLabel?: string;
}
```

### ChatMessage
```typescript
interface ChatMessage {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: string;
  instanceId?: number;
}
```

## 使い方

### 1. セッション一覧の表示
```typescript
import { SessionHistory } from '@/components/SessionHistory';

function App() {
  return <SessionHistory />;
}
```

### 2. フィルタリング
- **タイプフィルター**: Competition, Ensemble, Debate
- **ステータスフィルター**: Running, Paused, Completed, Failed
- **検索**: タスク名またはエージェント名で検索
- **ソート**: 更新日時、作成日時、期間、ファイル変更数

### 3. セッション操作
- **カードクリック**: 詳細ビューを開く
- **ドラッグ&ドロップ**: ステータスを変更
- **リフレッシュ**: 最新の状態に更新

### 4. チャット機能
1. セッションカードをクリック
2. "Chat" タブを選択
3. インスタンスを選択
4. コマンドを入力して送信
5. 出力がリアルタイムで表示される

## Tauri APIの使用例

### セッション作成
```typescript
import { tauriApi } from '@/services/tauri';

const session: WorktreeSession = {
  id: 'comp-123',
  type: 'competition',
  task: 'Implement feature X',
  status: 'running',
  createdAt: new Date().toISOString(),
  updatedAt: new Date().toISOString(),
  instances: [/* ... */],
  chatHistory: [],
};

await tauriApi.createSession(session);
```

### チャットメッセージ送信
```typescript
const message: ChatMessage = {
  id: `msg-${Date.now()}`,
  role: 'user',
  content: 'npm test',
  timestamp: new Date().toISOString(),
  instanceId: 1,
};

await tauriApi.addChatMessage('comp-123', message);
```

### Tmux統合
```typescript
// コマンド送信
await tauriApi.sendTmuxKeys('tmux-session-id', 'npm test');

// 出力キャプチャ
const output = await tauriApi.captureTmuxOutput('tmux-session-id');
```

## UI/UXの特徴

### カラーコーディング
- **Running**: Blue (青)
- **Paused**: Yellow (黄)
- **Completed**: Green (緑)
- **Failed**: Red (赤)

### アニメーション
- カードホバー: Shadow elevation
- ドラッグ中: Opacity 50%
- ドロップゾーン: Border highlight
- モーダル: Fade in/out

### レスポンシブデザイン
- Kanban Board: 水平スクロール対応
- モーダル: 画面の90%サイズ
- カード: 固定幅320px

## ストレージ

### ファイル構造
```
project-root/
└── .ait42/
    └── sessions.json    # すべてのセッション履歴
```

### データサイズの考慮
- 各セッション: 約5-10KB
- 100セッション: 約500KB-1MB
- チャット履歴を含む: セッションあたり+2-5KB

## パフォーマンス最適化

### 実装済み
- **遅延ロード**: ChatPanelの動的import
- **フィルタリング**: クライアントサイドフィルタリング
- **ソート**: Memoized sort function

### 将来の最適化
- 仮想スクロール (大量セッション時)
- ページネーション
- バックグラウンド同期
- IndexedDB統合 (オフライン対応)

## トラブルシューティング

### セッションが表示されない
1. `.ait42/sessions.json` が存在するか確認
2. Refreshボタンをクリック
3. ブラウザコンソールでエラーを確認

### チャットが動作しない
1. Tmuxセッションが実行中か確認
2. `tmux ls` でセッション一覧を確認
3. インスタンスのステータスを確認

### ドラッグ&ドロップが動作しない
1. react-dndライブラリがインストールされているか確認
2. ブラウザの開発者ツールでエラーを確認

## ベストプラクティス

### セッション管理
- 定期的に古いセッションを削除
- 重要なセッションはエクスポート
- チャット履歴は適度に保持

### パフォーマンス
- 100セッション以上の場合はフィルター使用
- 大量のチャット履歴は定期的にクリーンアップ
- ステータス更新は必要最小限に

## 今後の拡張予定

### Phase 5+
- [ ] メトリクスビジュアライゼーション (Chart.js統合)
- [ ] エクスポート機能 (JSON/CSV)
- [ ] セッション比較機能
- [ ] 自動タグ付け
- [ ] 検索強化 (全文検索)
- [ ] リアルタイム同期 (複数タブ間)
- [ ] バックアップ/リストア機能

## 関連ドキュメント

- [Vibe Kanban](https://github.com/MarkAI42/vibe-kanban) - インスピレーション元
- [Atlassian Design System](https://atlassian.design) - UIパターン参考
- [Tauri Documentation](https://tauri.app) - バックエンドAPI
- [Zustand Documentation](https://zustand-demo.pmnd.rs/) - 状態管理

## ライセンス

AIT42 Editor v1.6.0の一部として提供
