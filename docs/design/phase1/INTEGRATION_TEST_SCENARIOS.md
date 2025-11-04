# ディベートモード Phase 1 - 統合テストシナリオ

**Document Version**: 1.0
**Date**: 2025-11-05
**Project**: AIT42-Editor (ディベートモード)
**Target Release**: v1.5.0 Phase 1 (MVP)
**Status**: Ready for Testing

---

## 概要

Phase 1（MVP）の統合テストシナリオを定義します。正常系5シナリオ、異常系5シナリオの計10シナリオ。

**テストの目的**:
- ディベートワークフロー全体の動作確認
- エラーハンドリング・Graceful Degradationの検証
- 実装チームへの明確な合格基準の提示

**テスト環境**:
- OS: macOS 14.0以上 / Ubuntu 22.04以上
- Git: 2.40以上
- Tmux: 3.3以上
- Claude Code CLI: 最新版
- Node.js: 20.0以上
- Rust: 1.75以上

---

## 正常系シナリオ

### シナリオ1: 基本的な3ラウンドディベート（技術スタック選定）

#### 目的
最もシンプルな正常フローを確認

#### 前提条件
- [x] AIT42-Editorが起動済み
- [x] Git repositoryが初期化済み（`.git/`ディレクトリ存在）
- [x] Tmuxがインストール済み（`tmux -V`で`3.0`以上）
- [x] Claude Code CLIが利用可能（`claude --version`で確認）
- [x] 環境変数`ANTHROPIC_API_KEY`が設定済み

#### テストステップ

1. **ディベート起動**
   - ユーザーがヘッダーの「💬 討論」ボタンをクリック
   - DebateDialogが表示される
   - **確認**: ダイアログタイトルが「ディベートモード」

2. **タスク入力**
   - タスク入力欄に「Next.js vs Astro でブログを作るべきか？」と入力
   - **確認**: 入力が反映される（文字数カウント表示）

3. **ロール選択**
   - ロール選択: 「技術スタック選定」を選択
   - **確認**: 3つのロールが表示される
     - Architect (技術アーキテクト)
     - Pragmatist (現実主義者)
     - Innovator (革新者)

4. **モデル選択**
   - モデル選択: 「Sonnet 4.5」を選択
   - **確認**: コスト見積もりが表示される（例: $0.35-0.40）

5. **ディベート開始**
   - 「💬 ディベート開始」ボタンをクリック
   - **確認**: DebateDialogが閉じる
   - **確認**: MultiAgentパネルに切り替わる
   - **確認**: プログレスバーが表示される（Round 1/3）

6. **Round 1実行**
   - **期待時間**: 5-10分
   - **確認**: Git Worktreeが9つ作成される
     ```bash
     ls -la /tmp/debate-{id}/round1/
     # 期待出力:
     # role-architect/
     # role-pragmatist/
     # role-innovator/
     ```
   - **確認**: Tmuxセッションが3つ起動
     ```bash
     tmux list-sessions | grep debate-{id}-r1
     # 期待出力:
     # debate-{id}-r1-architect
     # debate-{id}-r1-pragmatist
     # debate-{id}-r1-innovator
     ```
   - **確認**: DebateStatusPanelに進捗表示
     - 「Round 1: 独立提案」（進捗バー: 0-10分）
     - 各ロールのステータス: Running → Completed

7. **Round 1完了**
   - **確認**: コンテキストファイルが生成される
     ```bash
     cat /tmp/debate-{id}/round1/role-architect-proposal.md
     # 期待: 500-1500単語のMarkdown文書
     ```
   - **確認**: プログレスバーが「Round 2/3」に進む

8. **Round 2実行**
   - **期待時間**: 8-12分
   - **確認**: 各ロールが他の提案を読んで批判的分析
   - **確認**: 2つのファイルが生成される
     ```bash
     ls /tmp/debate-{id}/round2/role-architect-*
     # 期待出力:
     # role-architect-critique.md
     # role-architect-revised.md
     ```
   - **確認**: 他の提案への言及がある
     ```bash
     grep "Pragmatist" /tmp/debate-{id}/round2/role-architect-critique.md
     # 期待: Pragmatistの提案への評価が含まれる
     ```

9. **Round 3実行**
   - **期待時間**: 5-8分
   - **確認**: 統合された最終提案が生成される
     ```bash
     cat /tmp/debate-{id}/round3/consensus.md
     # 期待: 3つの視点を統合した提案
     ```

10. **ディベート完了**
    - **確認**: ステータスが「Completed」
    - **確認**: 最終出力が表示される
    - **確認**: 実行時間が表示される（例: 25分32秒）
    - **確認**: コスト表示（例: $0.38）

11. **クリーンアップ（オプション）**
    - **確認**: Worktreeが自動削除（preserveWorktrees=falseの場合）
      ```bash
      ls /tmp/debate-{id}/round1/
      # 期待: ディレクトリが存在しない
      ```

#### 期待される出力

**Round 1**:
- 3つの独立した提案（各500-1500単語）
- Architectの提案: 長期的視点、技術的正統性
- Pragmatistの提案: 実装可能性、現実的制約
- Innovatorの提案: 革新的アイデア、差別化

**Round 2**:
- 3つの修正された提案
- 他の提案への具体的な言及がある
- トレードオフが明示されている

**Round 3**:
- 統合された最終提案
- 3つの視点を統合している
- 実装ロードマップが含まれる

#### 成功基準

- [x] 実行時間 < 40分
- [x] 全3ラウンドが完了
- [x] エラーなし（ログにERRORレベルなし）
- [x] コンテキストファイルが正しく生成（9個のMarkdownファイル）
- [x] Worktreeが自動削除（preserveWorktrees=falseの場合）
- [x] 最終提案が3つの視点を統合している（手動レビュー）

#### 失敗基準

- [x] 実行時間 > 40分
- [x] ラウンドが途中で停止（ステータスが「Failed」）
- [x] コンテキストファイルが破損（JSONパースエラー）
- [x] Worktreeが残っている（`git worktree list`で確認）
- [x] 最終提案が3つの視点を統合していない

---

### シナリオ2: セキュリティレビューディベート

#### 目的
異なるロールプリセットでも正常動作を確認

#### 前提条件
シナリオ1と同様

#### テストステップ

1. 「💬 討論」ボタンをクリック
2. タスク入力: 「決済APIのセキュリティレビューをして」
3. ロール選択: 「セキュリティレビュー」を選択
   - Security Architect (セキュリティ設計者)
   - PenTester (侵入テスター)
   - Compliance (コンプライアンス専門家)
4. モデル選択: 「Opus 4」（高品質モデル）
5. 詳細設定を開く
   - タイムアウト: 3600秒（60分）に設定
6. 「💬 ディベート開始」をクリック

#### 期待される動作

**Round 1**:
- Security Architect: OWASP Top 10の観点から分析
- PenTester: 実践的な攻撃シナリオを提示
- Compliance: GDPR、PCI DSS準拠を確認

**Round 2**:
- 互いの指摘を補完
- 例: Security ArchitectがOWASP #1（認証破綻）を指摘 → PenTesterが具体的な攻撃手法を追加

**Round 3**:
- 包括的なセキュリティ対策リストを生成
- 優先順位付きアクションアイテム

#### 成功基準

- [x] OWASP Top 10の項目が網羅されている（最低5項目以上）
  ```bash
  grep -i "injection\|authentication\|sensitive data" /tmp/debate-{id}/round3/consensus.md
  ```
- [x] 具体的な攻撃シナリオが含まれている
  ```bash
  grep -i "攻撃\|脆弱性\|exploit" /tmp/debate-{id}/round2/role-pentester-revised.md
  ```
- [x] 法的準拠チェックリストが含まれている
  ```bash
  grep -i "GDPR\|PCI DSS" /tmp/debate-{id}/round3/consensus.md
  ```
- [x] 優先順位付きアクションアイテムが生成されている
  ```bash
  grep -i "Priority\|優先順位" /tmp/debate-{id}/round3/consensus.md
  ```

---

### シナリオ3: モデル切り替え（Haiku使用）

#### 目的
異なるClaudeモデルでも正常動作を確認

#### テストステップ

1. タスク入力: 「簡単な認証機能を実装して」
2. ロール選択: 「技術スタック選定」
3. モデル選択: 「Haiku 3.5」（高速モデル）
4. 「💬 ディベート開始」をクリック

#### 期待される動作

- Haikuモデルでの実行（より高速、コスト削減）
- 実行時間: 15-25分（Sonnetより5-10分短縮）
- 出力品質: Sonnetより若干低いが許容範囲

#### 成功基準

- [x] 実行時間 < 30分
- [x] 出力品質 >= 70/100（ReflectionAgentによる評価）
- [x] コスト < $0.25/debate
- [x] エラーなし

#### 品質評価方法

```bash
# ReflectionAgentによる自動評価
claude --agent=reflection-agent --input=/tmp/debate-{id}/round3/consensus.md
# 期待出力: {"overall_score": 70-85, "correctness": 75, "completeness": 70, ...}
```

---

### シナリオ4: Worktree保持オプション

#### 目的
完了後もWorktreeを保持する機能を確認

#### テストステップ

1. タスク入力: 「マイクロサービス化の計画を立てて」
2. ロール選択: 「技術スタック選定」
3. 詳細設定:
   - 「完了後もworktreeを保持」をON
4. 「💬 ディベート開始」をクリック

#### 期待される動作

- ディベート完了後もWorktreeが残る
- `/tmp/debate-{id}/round*/role-*/`配下に9つのディレクトリ
- 各Worktreeで`git log`を確認可能

#### 成功基準

- [x] Worktreeが削除されていない
  ```bash
  git worktree list | grep debate-{id}
  # 期待: 9行のWorktreeリスト
  ```
- [x] 各Worktreeで`git status`が正常
  ```bash
  cd /tmp/debate-{id}/round1/role-architect
  git status
  # 期待: "On branch debate-{id}-r1-architect"
  ```
- [x] ログファイルが参照可能
  ```bash
  cat /tmp/debate-logs/debate-{id}-r1-architect.log
  # 期待: Tmuxセッションのログ
  ```

---

### シナリオ5: 並行実行（2つのディベート同時起動）

#### 目的
複数ディベートの同時実行を確認

#### テストステップ

1. **ディベート1を開始**
   - タスク: 「Next.js vs Astro」
   - ロール: 技術スタック選定
   - ディベートID: `debate-001`
2. **すぐにディベート2を開始**（Round 1実行中）
   - タスク: 「認証方式の選定」
   - ロール: 技術スタック選定
   - ディベートID: `debate-002`
3. 両方のステータスをMultiAgentパネルで確認

#### 期待される動作

- 2つのディベートが独立して実行
- Worktreeの名前衝突なし（`debate-{id}`で区別）
- Tmuxセッションの名前衝突なし

#### 成功基準

- [x] 両方のディベートが完了
- [x] エラーなし
- [x] 実行時間に大きな遅延なし（並列実行により遅延 < 10%）
- [x] Worktreeが正しく分離されている
  ```bash
  ls /tmp/ | grep debate
  # 期待出力:
  # debate-001/
  # debate-002/
  ```
- [x] Tmuxセッションが正しく分離されている
  ```bash
  tmux list-sessions | grep debate
  # 期待出力:
  # debate-001-r1-architect
  # debate-002-r1-architect
  ```

---

## 異常系シナリオ

### シナリオ6: Round 2でエージェント1つがタイムアウト

#### 目的
Graceful Degradationを確認

#### 前提条件
- タイムアウトをシミュレートするため、Round 2のタイムアウトを5分に設定
- または、Pragmatistのプロンプトに「10分以上かかる複雑な分析を実行」を注入

#### シミュレーション方法（開発者向け）

**方法1: タイムアウト設定短縮**
```rust
// src-tauri/src/debate/orchestrator.rs
let config = DebateConfig {
    // ...
    round_timeout_secs: 300, // 5分に短縮（通常は600秒）
};
```

**方法2: プロンプト注入**
```rust
// src-tauri/src/debate/role_prompts.rs
let pragmatist_prompt = format!(
    "{}\n\n[TEST] この分析には15分かけてください",
    base_prompt
);
```

#### テストステップ

1. タスク入力: 「複雑なシステム設計（マイクロサービス + Kubernetes）」
2. ロール選択: 「技術スタック選定」
3. 詳細設定:
   - ラウンドタイムアウト: 300秒（5分）
4. 「💬 ディベート開始」をクリック

#### 期待される動作

**Round 1**: 正常完了（Architect, Pragmatist, Innovator）

**Round 2**:
- Architect: 正常完了（5分以内）
- **Pragmatist: タイムアウト**（5分超過）
- Innovator: 正常完了（5分以内）

**Graceful Degradation発動**:
- エラーログ: 「Pragmatist timed out in Round 2」
  ```bash
  tail -n 50 /tmp/debate-{id}/logs/error.log | grep timeout
  # 期待出力: "ERROR: Pragmatist timed out in Round 2"
  ```
- ArchitectとInnovatorの2つの出力でRound 3に進む

**Round 3**:
- 2つの提案を統合して最終提案
- 最終提案に「Pragmatistのタイムアウトにより、2つの視点を統合」と明記

#### 成功基準

- [x] ディベートが中断しない（ステータスが「Partially Completed」）
- [x] エラーメッセージが表示される
  ```bash
  grep "Pragmatist timed out" /tmp/debate-{id}/logs/error.log
  ```
- [x] 残り2ロールでRound 3が完了
- [x] 最終提案が生成される（2つの視点を統合）
- [x] UIにワーニングが表示される（「一部エージェントがタイムアウト」）

#### 失敗基準

- [x] ディベート全体が失敗（ステータスが「Failed」）
- [x] エラーメッセージが表示されない
- [x] Round 3に進まない
- [x] 最終提案が生成されない

---

### シナリオ7: Git Worktree作成失敗

#### 目的
インフラエラーのハンドリングを確認

#### シミュレーション方法

**方法1: `/tmp`の書き込み権限を削除**
```bash
sudo chmod 000 /tmp
```

**方法2: ディスク容量枯渇**
```bash
# 100GBのダミーファイル作成
dd if=/dev/zero of=/tmp/dummy bs=1G count=100
```

**方法3: Rust コード内で強制失敗**
```rust
// src-tauri/src/debate/git_manager.rs
impl GitWorktreeManager {
    pub fn create_worktree(&self, path: &str, branch: &str) -> Result<()> {
        // テスト用の強制失敗
        return Err(anyhow::anyhow!("TEST: Simulated worktree creation failure"));
    }
}
```

#### テストステップ

1. シミュレーション実施（上記のいずれか）
2. タスク入力: 「技術スタック選定」
3. 「💬 ディベート開始」をクリック

#### 期待される動作

- DebateOrchestratorがWorktree作成を試みる
- 失敗を検出
- エラーメッセージを表示: 「Worktree creation failed: Permission denied」
- ディベートステータスを「Failed」に変更
- リトライ3回後、完全失敗
- Worktreeのクリーンアップを試みる

#### 成功基準

- [x] 明確なエラーメッセージが表示される
  ```
  エラー: Git Worktreeの作成に失敗しました
  詳細: Permission denied (/tmp/debate-xxx)
  対処法: /tmpディレクトリの書き込み権限を確認してください
  ```
- [x] ディベートステータスが「Failed」
- [x] 部分的に作成されたWorktreeがクリーンアップされる
  ```bash
  git worktree list | grep debate-{id}
  # 期待: 結果なし（全て削除済み）
  ```
- [x] アプリケーションがクラッシュしない
- [x] 次のディベートが正常に起動できる

---

### シナリオ8: Tmuxセッション起動失敗

#### 目的
Tmux依存のエラーハンドリングを確認

#### シミュレーション方法

**方法1: Tmuxをアンインストール**
```bash
brew uninstall tmux  # macOS
sudo apt remove tmux  # Ubuntu
```

**方法2: Tmuxプロセスを強制終了**
```bash
pkill -9 tmux
```

**方法3: Rust コード内で強制失敗**
```rust
// src-tauri/src/debate/tmux_manager.rs
impl TmuxManager {
    pub fn create_session(&self, session_id: &str) -> Result<()> {
        // テスト用の強制失敗
        return Err(anyhow::anyhow!("TEST: Tmux not available"));
    }
}
```

#### テストステップ

1. シミュレーション実施（Tmuxアンインストール）
2. タスク入力: 「技術スタック選定」
3. 「💬 ディベート開始」をクリック

#### 期待される動作

- DebateOrchestratorがTmuxセッション起動を試みる
- 失敗を検出
- エラーメッセージを表示: 「Tmux is not installed or not available」
- ディベートステータスを「Failed」に変更
- フォールバック: Tmuxなしでの実行（Phase 1ではサポートしない）

#### 成功基準

- [x] 明確なエラーメッセージが表示される
  ```
  エラー: Tmuxが利用できません
  詳細: Tmuxがインストールされていないか、パスが通っていません
  対処法: 以下のコマンドでTmuxをインストールしてください
    macOS: brew install tmux
    Ubuntu: sudo apt install tmux
  ```
- [x] ユーザーに「Tmuxをインストールしてください」と指示
- [x] アプリケーションがクラッシュしない
- [x] ディベートステータスが「Failed」

---

### シナリオ9: コンテキストファイル破損

#### 目的
データ整合性エラーのハンドリングを確認

#### シミュレーション方法

**方法1: Round 1完了後、手動でファイルを破損**
```bash
# Round 1完了を待つ
sleep 600

# コンテキストファイルを破損
echo "Invalid JSON {{{" > /tmp/debate-{id}/round1/role-architect-proposal.md
```

**方法2: Rust コード内でJSONシリアライゼーションエラーを注入**
```rust
// src-tauri/src/debate/context_manager.rs
impl ContextManager {
    pub async fn save_proposal(&self, content: &str) -> Result<()> {
        // テスト用の破損データ注入
        let corrupted = "Invalid JSON {{{";
        tokio::fs::write(path, corrupted).await?;
        Ok(())
    }
}
```

#### テストステップ

1. タスク入力: 「技術スタック選定」
2. 「💬 ディベート開始」をクリック
3. Round 1完了を待つ
4. **手動でコンテキストファイルを破損**（上記の方法1）
5. Round 2が自動的に開始される

#### 期待される動作

- Round 2でコンテキストファイルを読み込もうとする
- JSONパースエラーを検出
- エラーメッセージを表示: 「Context file corrupted: Invalid JSON」
- Graceful Degradation:
  - **オプション1**: Round 2をコンテキストなしで実行（Round 1モードと同じ）
  - **オプション2**: ディベートを失敗として終了

#### 成功基準（オプション1の場合）

- [x] JSONパースエラーが検出される
  ```bash
  grep "Context file corrupted" /tmp/debate-{id}/logs/error.log
  ```
- [x] エラーログが記録される
- [x] アプリケーションがクラッシュしない
- [x] Round 2が独立提案モードで実行される（Round 1と同じ）
- [x] ユーザーに明確なエラーメッセージ
  ```
  警告: Round 1のコンテキストファイルが破損しています
  Round 2は独立提案モードで実行されます
  ```

#### 成功基準（オプション2の場合）

- [x] ディベートが失敗として終了
- [x] エラーメッセージが表示される
- [x] 部分的な結果（Round 1のみ）が保存される
- [x] ユーザーに再実行を促す

---

### シナリオ10: ユーザーによる手動キャンセル

#### 目的
キャンセル処理を確認

#### テストステップ

1. タスク入力: 「技術スタック選定」
2. 「💬 ディベート開始」をクリック
3. Round 1実行中（5分経過時点）
4. **キャンセルボタンをクリック**
   - MultiAgentパネルの「❌ キャンセル」ボタン

#### 期待される動作

- DebateOrchestrator.cancel()が呼ばれる
- 実行中のTmuxセッションが終了
- Worktreeが削除
- コンテキストファイルが削除
- ディベートステータスが「Cancelled」に変更
- MultiAgentパネルから該当ディベートが削除

#### 成功基準

- [x] キャンセル後5秒以内に全リソースがクリーンアップ
  ```bash
  # 5秒後に確認
  sleep 5
  tmux list-sessions | grep debate-{id}
  # 期待: 結果なし

  git worktree list | grep debate-{id}
  # 期待: 結果なし

  ls /tmp/debate-{id}/
  # 期待: No such file or directory
  ```
- [x] Tmuxセッションが残っていない
- [x] Worktreeが残っていない
- [x] `/tmp/debate-{id}/`ディレクトリが削除されている

#### 失敗基準

- [x] Tmuxセッションが残っている
- [x] Worktreeが残っている
- [x] キャンセル処理が10秒以上かかる
- [x] ディベートステータスが「Cancelled」に変更されない

---

## テスト実行環境

### 必要な環境

```yaml
OS:
  - macOS: 14.0以上
  - Ubuntu: 22.04以上
  - Windows: WSL2（Ubuntu 22.04以上）

Dependencies:
  - Git: 2.40以上
  - Tmux: 3.3以上
  - Node.js: 20.0以上
  - Rust: 1.75以上
  - Claude Code CLI: 最新版

Environment Variables:
  - ANTHROPIC_API_KEY: Anthropic APIキー（必須）
  - GITHUB_TOKEN: GitHub Personal Access Token（オプション）

System Resources:
  - CPU: 4コア以上
  - RAM: 8GB以上
  - Disk: 10GB以上の空き容量
```

### テスト実行コマンド

#### ユニットテスト（Rust）
```bash
cd src-tauri
cargo test --all-features
cargo test debate::orchestrator --nocapture  # 詳細ログ付き
```

#### ユニットテスト（TypeScript）
```bash
npm run test
npm run test:watch  # Watch mode
npm run test:coverage  # カバレッジレポート
```

#### 統合テスト（Playwright）
```bash
npm run test:e2e

# 特定のシナリオのみ実行
npm run test:e2e -- --grep "シナリオ1"

# ヘッドレスモードなし（ブラウザ表示）
npm run test:e2e -- --headed
```

#### 手動テスト
```bash
# AIT42-Editorを起動
npm run tauri dev

# 各シナリオを手動で実行
# シナリオ1-10の手順に従う
```

---

## テスト記録フォーマット

### テスト実行ログ

```markdown
# テスト実行記録

**日時**: 2025-11-05 10:00:00
**実行者**: 山田太郎
**環境**:
- OS: macOS 14.1
- Git: 2.42
- Tmux: 3.3a
- Claude Code: 1.2.0
- Node.js: 20.10.0
- Rust: 1.75.0

## 正常系シナリオ

| シナリオ | 結果 | 実行時間 | 備考 |
|---------|------|---------|------|
| シナリオ1: 基本3ラウンドディベート | ✅ PASS | 32分 | 正常完了、最終提案品質 92/100 |
| シナリオ2: セキュリティレビュー | ✅ PASS | 45分 | Opusモデルで実行、OWASP Top 10網羅 |
| シナリオ3: Haiku使用 | ✅ PASS | 22分 | Haikuで高速実行、品質 78/100 |
| シナリオ4: Worktree保持 | ✅ PASS | 35分 | Worktree保持確認、git log正常 |
| シナリオ5: 並行実行 | ✅ PASS | 34分 | 2つのディベート同時実行、遅延 <5% |

## 異常系シナリオ

| シナリオ | 結果 | 実行時間 | 備考 |
|---------|------|---------|------|
| シナリオ6: エージェントタイムアウト | ✅ PASS | 28分 | Graceful degradation動作、2ロールで完了 |
| シナリオ7: Worktree作成失敗 | ✅ PASS | 1秒 | エラーメッセージ表示、アプリクラッシュなし |
| シナリオ8: Tmux起動失敗 | ✅ PASS | 1秒 | エラーメッセージ表示、インストール手順提示 |
| シナリオ9: コンテキストファイル破損 | ⚠️ PARTIAL | 25分 | Round 2でエラー検出、独立提案モードで実行 |
| シナリオ10: 手動キャンセル | ✅ PASS | 5秒 | クリーンアップ完了、リソース残留なし |

**総合結果**: 9/10 PASS（90%）
**備考**: シナリオ9は期待動作と若干異なるが、エラーハンドリングは適切

## 発見した問題

### 問題1: Round 2のコンテキスト読み込みエラー
- **シナリオ**: シナリオ9
- **症状**: コンテキストファイル破損時、Round 2が失敗
- **期待**: 独立提案モードで実行
- **実際**: エラーメッセージ表示後、ディベート終了
- **優先度**: P2（中）
- **対応**: Round 2のエラーハンドリングを改善

### 問題2: Tmuxセッションのクリーンアップ遅延
- **シナリオ**: シナリオ10
- **症状**: キャンセル後、Tmuxセッション削除に8秒かかる
- **期待**: 5秒以内
- **実際**: 8秒
- **優先度**: P3（低）
- **対応**: Tmuxセッション削除を非同期化

## 改善提案

1. **シナリオ9の改善**: コンテキストファイル検証機能を追加
2. **シナリオ10の最適化**: Tmuxセッション削除を並列化
3. **エラーメッセージの改善**: ユーザーフレンドリーな表現に変更

## 次回テスト計画

- **日時**: 2025-11-12 10:00
- **目的**: 問題1-2の修正確認
- **シナリオ**: シナリオ9、10を再実行
```

---

## テスト自動化スクリプト

### Playwright E2Eテスト（TypeScript）

```typescript
// tests/e2e/debate-mode.spec.ts

import { test, expect } from '@playwright/test';

test.describe('ディベートモード統合テスト', () => {
  test('シナリオ1: 基本的な3ラウンドディベート', async ({ page }) => {
    // 1. アプリ起動
    await page.goto('http://localhost:1420');

    // 2. ディベートダイアログを開く
    await page.click('button:has-text("💬 討論")');
    await expect(page.locator('dialog')).toBeVisible();

    // 3. タスク入力
    await page.fill(
      'textarea[placeholder*="複雑な設計判断"]',
      'Next.js vs Astro でブログを作るべきか？'
    );

    // 4. ロール選択
    await page.selectOption('select[name="rolePreset"]', '技術スタック選定');

    // 5. モデル選択
    await page.selectOption('select[name="model"]', 'sonnet-4.5');

    // 6. ディベート開始
    await page.click('button:has-text("💬 ディベート開始")');

    // 7. MultiAgentパネルに切り替わることを確認
    await expect(page.locator('.multi-agent-panel')).toBeVisible();

    // 8. プログレスバーが表示されることを確認
    await expect(page.locator('.progress-bar:has-text("Round 1/3")')).toBeVisible();

    // 9. Round 1完了を待つ（タイムアウト: 15分）
    await page.waitForSelector('.progress-bar:has-text("Round 2/3")', {
      timeout: 15 * 60 * 1000,
    });

    // 10. Round 2完了を待つ
    await page.waitForSelector('.progress-bar:has-text("Round 3/3")', {
      timeout: 15 * 60 * 1000,
    });

    // 11. ディベート完了を待つ
    await page.waitForSelector('.debate-status:has-text("Completed")', {
      timeout: 10 * 60 * 1000,
    });

    // 12. 最終出力が表示されることを確認
    const finalOutput = await page.locator('.final-output').textContent();
    expect(finalOutput).toContain('最終提案');
    expect(finalOutput!.length).toBeGreaterThan(500);
  });

  test('シナリオ6: Round 2でエージェントタイムアウト', async ({ page }) => {
    // 1-6: シナリオ1と同じ
    // ...

    // 7. タイムアウトを5分に設定
    await page.click('button:has-text("詳細設定")');
    await page.fill('input[name="roundTimeout"]', '300');

    // 8. ディベート開始
    await page.click('button:has-text("💬 ディベート開始")');

    // 9. エラーメッセージが表示されることを確認
    await expect(
      page.locator('.error-message:has-text("timed out")')
    ).toBeVisible({ timeout: 20 * 60 * 1000 });

    // 10. ステータスが「Partially Completed」
    await expect(
      page.locator('.debate-status:has-text("Partially Completed")')
    ).toBeVisible();
  });

  test('シナリオ10: 手動キャンセル', async ({ page }) => {
    // 1-6: シナリオ1と同じ
    // ...

    // 7. 5分待つ
    await page.waitForTimeout(5 * 60 * 1000);

    // 8. キャンセルボタンをクリック
    await page.click('button:has-text("❌ キャンセル")');

    // 9. ステータスが「Cancelled」
    await expect(
      page.locator('.debate-status:has-text("Cancelled")')
    ).toBeVisible({ timeout: 10 * 1000 });

    // 10. MultiAgentパネルから削除されることを確認
    await expect(page.locator('.debate-card')).toHaveCount(0);
  });
});
```

---

## テスト成功の最終条件

Phase 1のリリース判定基準:

### 必須条件（Must Have）
- [x] 正常系5シナリオが全てPASS
- [x] 異常系シナリオのうち4つ以上がPASS（90%以上）
- [x] クリティカルなエラーがない（アプリクラッシュなし）

### 推奨条件（Should Have）
- [x] 実行時間が期待範囲内（正常系: <40分、異常系: <30分）
- [x] 出力品質が基準以上（ReflectionAgentスコア >=80）
- [x] リソースリークがない（Worktree、Tmuxセッション残留なし）

### オプション条件（Nice to Have）
- [x] 自動化テストカバレッジ >= 70%
- [x] ユーザーフィードバックが良好（手動テスト5名、満足度 >=80%）

---

## 次のステップ

実装チームへの指示:

1. **Week 1**: シナリオ1-5の実装
2. **Week 2**: シナリオ6-10のエラーハンドリング実装
3. **Week 3**: Playwrightテスト自動化
4. **Week 4**: 全10シナリオの実行と合格確認

**リリース判定**: Week 4終了時に全10シナリオがPASS（または9/10 PASS）することが条件

---

**END OF DOCUMENT**
