# Ensemble統合フェーズ - 統合テストシナリオ

作成日: 2025-11-13
対象: AIT42-Editor v1.6.0 - Ensemble統合フェーズ実装

## 正常系テスト

### シナリオ1: 正常な統合フェーズ実行（3インスタンス）

**事前条件**:
- AIT42-Editorが起動している
- ワークスペースがGitリポジトリである
- Claude CLIがインストールされている

**実行手順**:
1. Ensembleモードで3インスタンスを起動
   ```typescript
   await tauriApi.executeMultiRuntimeCompetition({
     task: "Reactコンポーネントを3つの異なるアプローチで実装",
     allocations: [
       { runtime: 'claude', count: 3, model: 'sonnet' }
     ],
     timeoutSeconds: 300,
     preserveWorktrees: true,
     mode: 'ensemble'
   });
   ```

2. 全インスタンスが正常完了するまで待機

3. 統合フェーズが自動起動されることを確認
   - `MultiAgentPanel`のuseEffectが統合完了を検知
   - `startIntegrationPhase`が自動的に呼び出される

4. 統合AIが正常に実行されることを確認
   - 統合インスタンスが追加される
   - UIが紫色の背景で表示される
   - 「🔄 統合フェーズ - Integration Phase」バッジが表示される

5. リアルタイム出力が更新されることを確認
   - `competition-output`イベントが受信される
   - 統合AIの出力がリアルタイムで表示される

6. セッションステータスが"completed"になることを確認

**期待結果**:
- ✅ 統合インスタンスが`localInstances`配列に追加される
- ✅ 統合インスタンスが紫色のUIで表示される
- ✅ 統合バッジ「🔄 統合フェーズ - Integration Phase」が表示される
- ✅ 出力がリアルタイムで更新される
- ✅ セッションステータスが"completed"になる
- ✅ セッションの`integrationPhase`フィールドが"completed"になる

**検証ポイント**:
```typescript
// セッション状態の検証
const session = await tauriApi.getSession(workspacePath, sessionId);
assert(session.status === 'completed');
assert(session.integrationPhase === 'completed');
assert(session.instances.length === 4); // 3 + 1 integration

// 統合インスタンスの検証
const integrationInstance = session.instances[3];
assert(integrationInstance.agentName?.includes('Integration'));
assert(integrationInstance.status === 'completed');
assert(integrationInstance.output?.length > 0);
```

---

### シナリオ2: 統合フェーズ実行（5インスタンス）

**事前条件**: シナリオ1と同様

**実行手順**:
1. Ensembleモードで5インスタンスを起動
2. 全インスタンス完了後、統合フェーズが自動起動
3. 統合AIが5つの出力を統合

**期待結果**:
- ✅ 統合プロンプトに5つのインスタンス出力が含まれる
- ✅ 統合AIが正常に実行される
- ✅ セッションに計6インスタンス（5 + 1統合）が記録される

---

### シナリオ3: 多様なランタイムでの統合

**事前条件**: シナリオ1と同様

**実行手順**:
1. 複数のランタイムでEnsembleモードを起動
   ```typescript
   allocations: [
     { runtime: 'claude', count: 2, model: 'sonnet' },
     { runtime: 'codex', count: 1, model: 'gpt-4' }
   ]
   ```
2. 統合フェーズが各ランタイムの出力を統合

**期待結果**:
- ✅ 統合プロンプトにランタイム情報が含まれる
- ✅ 統合AIが異なるランタイムの成果物を適切に統合する

---

## 異常系テスト

### シナリオ4: バックエンドエラー（Worktree作成失敗）

**事前条件**:
- Gitリポジトリが破損している
- または、`.worktrees`ディレクトリの権限がない

**実行手順**:
1. Ensembleモードで3インスタンスを起動
2. 全インスタンス完了
3. 統合フェーズが起動を試みる
4. Worktree作成に失敗

**期待結果**:
- ✅ エラーがコンソールに表示される
  ```
  [MultiAgentPanel] Failed to start integration phase: Failed to create integration worktree: ...
  ```
- ✅ UIがエラー状態を示す（統合インスタンスが追加されない）
- ✅ セッションが"completed"のままハングしない
- ✅ 他のインスタンスの結果は保持される

**検証ポイント**:
```typescript
// エラーハンドリングの検証
try {
  await tauriApi.startIntegrationPhase(request);
} catch (error) {
  assert(error.message.includes('Failed to create integration worktree'));
}

// セッション状態は正常なまま
const session = await tauriApi.getSession(workspacePath, sessionId);
assert(session.instances.length === 3); // 統合インスタンスなし
```

---

### シナリオ5: 一部のインスタンスが失敗

**事前条件**: シナリオ1と同様

**実行手順**:
1. Ensembleモードで3インスタンスを起動
2. 1つが`failed`、2つが`completed`
3. 統合フェーズが起動されるか確認

**期待結果**:
- ✅ 統合フェーズが起動される（全インスタンス完了条件を満たす）
- ✅ 統合プロンプトにfailedインスタンスの情報も含まれる
- ✅ 統合AIが利用可能な出力のみで統合を実行

**検証ポイント**:
```typescript
const session = await tauriApi.getSession(workspacePath, sessionId);
const failedCount = session.instances.filter(i => i.status === 'failed').length;
const completedCount = session.instances.filter(i => i.status === 'completed').length;

assert(failedCount === 1);
assert(completedCount === 2);
assert(session.integrationPhase === 'completed');
```

---

### シナリオ6: 統合フェーズのタイムアウト

**事前条件**: シナリオ1と同様

**実行手順**:
1. Ensembleモードで3インスタンスを起動
2. 統合フェーズが起動
3. 統合AIが30分以上実行される場合

**期待結果**:
- ✅ 統合インスタンスのステータスが`running`のまま
- ✅ メモリ使用量が安定している
- ✅ UIが応答し続ける
- ✅ 手動でキャンセル可能

**検証ポイント**:
- メモリリーク検出: 長時間実行後もメモリ使用量が一定範囲内
- CPU使用率: 異常な高負荷がない

---

### シナリオ7: 統合フェーズの重複起動防止

**事前条件**: シナリオ1と同様

**実行手順**:
1. Ensembleモードで3インスタンスを起動
2. 統合フェーズが自動起動
3. 同じセッションで再度統合フェーズを起動しようとする

**期待結果**:
- ✅ 統合フェーズが1回のみ起動される
- ✅ `hasIntegrationStarted`チェックが機能する
- ✅ 重複する統合インスタンスが作成されない

**検証ポイント**:
```typescript
// MultiAgentPanel.tsx の useEffect (224-266行目)
const hasIntegrationStarted =
  session.integrationPhase === 'in_progress' ||
  session.integrationPhase === 'completed';

assert(hasIntegrationStarted === true); // 2回目の起動はスキップ
```

---

## エッジケースのテスト

### シナリオ8: インスタンス数0での統合フェーズ

**実行手順**:
1. `instanceCount: 0`で`startIntegrationPhase`を呼び出す

**期待結果**:
- ✅ エラーが返される: "Instance count must be greater than 0"

---

### シナリオ9: 空のタスクでの統合フェーズ

**実行手順**:
1. `originalTask: ""`で`startIntegrationPhase`を呼び出す

**期待結果**:
- ✅ エラーが返される: "Original task cannot be empty"

---

### シナリオ10: 出力ファイルが存在しない場合

**実行手順**:
1. Ensembleモード実行後、`.claude-output-*.log`ファイルを削除
2. 統合フェーズを起動

**期待結果**:
- ✅ 統合フェーズが起動される
- ✅ 警告ログが出力される: "⚠️ No log file found for instance X"
- ✅ 統合プロンプトにプレースホルダーが含まれる: "⚠️ No output captured for instance X"

---

## パフォーマンステスト

### シナリオ11: 大量出力の処理

**実行手順**:
1. 各インスタンスが10,000行の出力を生成
2. 統合AIが全出力を処理

**期待結果**:
- ✅ 統合プロンプト生成時間 < 5秒
- ✅ UI描画が滑らか（フレーム落ちなし）
- ✅ メモリ使用量が適切（< 500MB増加）

**検証ポイント**:
```typescript
const startTime = performance.now();
const prompt = generate_integration_prompt(task, outputs);
const endTime = performance.now();
assert((endTime - startTime) < 5000); // 5秒以内
```

---

### シナリオ12: 多数インスタンス（10インスタンス）

**実行手順**:
1. Ensembleモードで10インスタンスを起動
2. 統合フェーズの起動時間を計測

**期待結果**:
- ✅ 統合フェーズ起動時間 < 10秒
- ✅ 全10インスタンスの出力が収集される
- ✅ UIが応答し続ける

**検証ポイント**:
```rust
let start = std::time::Instant::now();
let result = start_integration_phase(...).await;
let duration = start.elapsed();
assert!(duration.as_secs() < 10);
```

---

### シナリオ13: 長時間実行（30分以上）

**実行手順**:
1. 統合AIが30分以上実行される場合
2. メモリ使用量、CPU使用率、UIの応答性を監視

**期待結果**:
- ✅ メモリリークがない
- ✅ CPU使用率が安定している
- ✅ UIが応答し続ける
- ✅ イベントループがブロックされない

**監視項目**:
- メモリ使用量（5分ごと）
- CPU使用率（1分ごと）
- UI FPS（常時）
- イベントループ遅延（常時）

---

## セキュリティテスト

### シナリオ14: 悪意のある出力の処理

**実行手順**:
1. インスタンス出力に特殊文字やエスケープシーケンスを含める
   - ANSI escape codes
   - Shell injection attempts
   - XSS payloads

**期待結果**:
- ✅ 特殊文字が適切にエスケープされる
- ✅ Shell injectionが防止される
- ✅ UIに悪意のあるコードが実行されない

**検証ポイント**:
```rust
// strip_ansi関数がANSI codesを削除
let cleaned = strip_ansi("\x1b[31mRed text\x1b[0m");
assert_eq!(cleaned, "Red text");

// Shell escaping
let escaped = escape_for_shell("'; rm -rf /");
assert!(!escaped.contains("rm -rf"));
```

---

## UI/UXテスト

### シナリオ15: 統合インスタンスの視覚的区別

**実行手順**:
1. Ensembleモードを実行
2. 統合フェーズが起動
3. UIで統合インスタンスを確認

**期待結果**:
- ✅ 統合インスタンスが紫色の背景で表示される（`bg-purple-900/20 border-purple-500`）
- ✅ 統合バッジが目立つ色で表示される（グラデーション: purple-600 → pink-600）
- ✅ 統合インスタンスが他のインスタンスと明確に区別される
- ✅ アニメーション効果がある（`animate-pulse`）

**検証ポイント**:
```typescript
// isIntegrationInstance関数のテスト
assert(isIntegrationInstance({
  agentName: '🔄 Integration Agent',
  ...
}) === true);

assert(isIntegrationInstance({
  agentName: 'Claude Instance 1',
  ...
}) === false);
```

---

### シナリオ16: リアルタイム出力の表示

**実行手順**:
1. 統合フェーズを起動
2. 出力がリアルタイムで更新されることを確認

**期待結果**:
- ✅ 出力が1秒以内に表示される
- ✅ 出力が増分的に更新される（全体を再レンダリングしない）
- ✅ スクロール位置が維持される
- ✅ UIがスムーズに更新される

---

## 自動テストスクリプト

### テストランナー（TypeScript/Jest）

```typescript
// __tests__/integration/ensemble-integration-phase.test.ts

import { tauriApi } from '@/services/tauri';
import { useSessionHistoryStore } from '@/store/sessionHistoryStore';

describe('Ensemble Integration Phase', () => {
  let sessionId: string;
  let workspacePath: string;

  beforeEach(async () => {
    workspacePath = await tauriApi.getCurrentDirectory();
  });

  test('should start integration phase after all instances complete', async () => {
    // 1. Start ensemble competition
    const result = await tauriApi.executeMultiRuntimeCompetition({
      task: 'Test task',
      allocations: [{ runtime: 'claude', count: 3, model: 'sonnet' }],
      timeoutSeconds: 300,
      preserveWorktrees: true,
      mode: 'ensemble'
    });

    sessionId = result.competitionId;

    // 2. Wait for all instances to complete
    await waitForInstancesComplete(sessionId, 3);

    // 3. Wait for integration phase to start
    await new Promise(resolve => setTimeout(resolve, 5000));

    // 4. Verify integration instance exists
    const session = await tauriApi.getSession(workspacePath, sessionId);
    expect(session.instances.length).toBe(4); // 3 + 1 integration
    expect(session.integrationPhase).toBe('in_progress');

    const integrationInstance = session.instances[3];
    expect(integrationInstance.agentName).toContain('Integration');
  });

  test('should handle integration phase errors gracefully', async () => {
    // Test error scenarios
    await expect(
      tauriApi.startIntegrationPhase({
        sessionId: 'invalid',
        workspacePath: '/nonexistent',
        instanceCount: 0,
        originalTask: ''
      })
    ).rejects.toThrow();
  });

  // Helper functions
  async function waitForInstancesComplete(sessionId: string, count: number) {
    while (true) {
      const session = await tauriApi.getSession(workspacePath, sessionId);
      const completed = session.instances.filter(
        i => i.status === 'completed' || i.status === 'failed'
      ).length;

      if (completed >= count) break;
      await new Promise(resolve => setTimeout(resolve, 1000));
    }
  }
});
```

---

## テスト実行チェックリスト

- [ ] シナリオ1: 正常な統合フェーズ実行（3インスタンス）
- [ ] シナリオ2: 正常な統合フェーズ実行（5インスタンス）
- [ ] シナリオ3: 多様なランタイムでの統合
- [ ] シナリオ4: バックエンドエラー（Worktree作成失敗）
- [ ] シナリオ5: 一部のインスタンスが失敗
- [ ] シナリオ6: 統合フェーズのタイムアウト
- [ ] シナリオ7: 統合フェーズの重複起動防止
- [ ] シナリオ8: インスタンス数0での統合フェーズ
- [ ] シナリオ9: 空のタスクでの統合フェーズ
- [ ] シナリオ10: 出力ファイルが存在しない場合
- [ ] シナリオ11: 大量出力の処理
- [ ] シナリオ12: 多数インスタンス（10インスタンス）
- [ ] シナリオ13: 長時間実行（30分以上）
- [ ] シナリオ14: 悪意のある出力の処理
- [ ] シナリオ15: 統合インスタンスの視覚的区別
- [ ] シナリオ16: リアルタイム出力の表示

---

## テスト環境要件

### 最小要件
- OS: Linux / macOS
- Node.js: 20.x以上
- Rust: 1.91.1以上
- Claude CLI: 最新版
- Git: 2.30以上
- tmux: 3.0以上

### 推奨環境
- メモリ: 8GB以上
- ディスク: 10GB以上の空き容量
- CPU: 4コア以上

---

## テスト実行方法

### 手動テスト
```bash
# 1. AIT42-Editorを起動
npm run tauri dev

# 2. Ensembleモードを実行
# UIからEnsembleモードを選択し、3インスタンスで実行

# 3. 統合フェーズが自動起動することを確認

# 4. ブラウザのコンソールでログを確認
# [MultiAgentPanel] 🔥 All instances completed, starting integration phase...
# [MultiAgentPanel] Integration phase started: { integrationInstanceId: 4, ... }
```

### 自動テスト
```bash
# Jestテストを実行
npm run test -- integration/ensemble-integration-phase.test.ts
```

---

## トラブルシューティング

### 問題1: 統合フェーズが起動しない
- セッションタイプが"ensemble"であることを確認
- 全インスタンスが`completed`または`failed`であることを確認
- ブラウザコンソールでエラーログを確認

### 問題2: 統合インスタンスの出力が表示されない
- `competition-output`イベントがリッスンされていることを確認
- tmuxセッションが正常に作成されているか確認: `tmux ls`
- ログファイルが作成されているか確認: `ls src-tauri/.worktrees/competition-*/integration/.integration-output.log`

### 問題3: 統合フェーズがエラーで終了
- Rustバックエンドのログを確認: `tail -f src-tauri/target/debug/ait42-editor.log`
- Gitリポジトリが正常であることを確認: `git status`
- ディスク容量が十分であることを確認: `df -h`
