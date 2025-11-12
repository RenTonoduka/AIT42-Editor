# Ensembleモード統合テスト手順書

## 概要

このドキュメントは、AIT42-EditorのEnsembleモード（Multi-runtime collaborative execution）の統合テスト手順と環境設定方法を説明します。

---

## 前提条件

### 必須ソフトウェア

1. **Node.js** (v18以上)
   ```bash
   node --version  # v18.0.0+
   ```

2. **Rust** (latest stable)
   ```bash
   rustc --version  # 1.70.0+
   ```

3. **Tauri CLI**
   ```bash
   cargo install tauri-cli
   ```

4. **tmux** (v3.0以上)
   ```bash
   tmux -V  # tmux 3.0+
   ```

5. **Git** (v2.30以上)
   ```bash
   git --version  # git 2.30.0+
   ```

### ランタイムCLIのインストール

#### 1. Claude CLI（必須）

```bash
# macOS/Linux
curl -sSL https://get.anthropic.com | sh

# または npm経由
npm install -g @anthropic-ai/claude-cli

# インストール確認
claude --version
```

#### 2. Codex CLI（オプション）

```bash
# OpenAI CLI
pip install openai

# または chatgpt CLI
npm install -g @michaelornelas/chatgpt-cli

# インストール確認
chatgpt --version
```

#### 3. Gemini CLI（オプション）

```bash
# Gemini CLI
npm install -g gemini-cli

# インストール確認
gemini --version
```

---

## 環境変数の設定

### 1. ANTHROPIC_API_KEY（必須）

Claude Codeを使用するために必要です。

```bash
# 一時的な設定（現在のシェルセッションのみ）
export ANTHROPIC_API_KEY="sk-ant-api03-..."

# 永続的な設定（推奨）
echo 'export ANTHROPIC_API_KEY="sk-ant-api03-..."' >> ~/.bashrc
source ~/.bashrc

# または .zshrc の場合
echo 'export ANTHROPIC_API_KEY="sk-ant-api03-..."' >> ~/.zshrc
source ~/.zshrc
```

**APIキーの取得方法**:
1. [Anthropic Console](https://console.anthropic.com/) にアクセス
2. "API Keys" セクションで新しいキーを生成
3. 生成されたキーをコピーして上記のコマンドで設定

### 2. OPENAI_API_KEY（Codex使用時のみ）

```bash
export OPENAI_API_KEY="sk-..."
echo 'export OPENAI_API_KEY="sk-..."' >> ~/.bashrc
```

### 3. GOOGLE_API_KEY（Gemini使用時のみ）

```bash
export GOOGLE_API_KEY="AIza..."
echo 'export GOOGLE_API_KEY="AIza..."' >> ~/.bashrc
```

### 4. AIT42_SOURCE_PATH（オプション）

AIT42エージェントのソースディレクトリを指定します（デフォルト: `~/.ait42`）。

```bash
export AIT42_SOURCE_PATH="/path/to/AIT42"
echo 'export AIT42_SOURCE_PATH="/path/to/AIT42"' >> ~/.bashrc
```

---

## アプリケーションのビルドと起動

### 開発モード

```bash
cd /home/user/AIT42-Editor

# 依存関係のインストール
npm install

# Tauri開発サーバー起動
npm run tauri:dev
```

### プロダクションビルド

```bash
# ビルド
npm run tauri:build

# 生成されたバイナリを実行
./src-tauri/target/release/ait42-editor
```

---

## 統合テストシナリオ

### シナリオ1: 基本的なEnsemble実行（Claude x 2）

#### ステップ1: Ensembleダイアログを開く

1. アプリケーションを起動
2. 右上の **✨ アンサンブル** ボタンをクリック
3. Ensembleダイアログが表示されることを確認

#### ステップ2: タスクの入力

```
タスク例:
"Create a simple REST API endpoint in TypeScript using Express.js that returns a list of users."
```

#### ステップ3: ランタイム設定

- **Claude**: 2インスタンス
- **Model**: claude-3-7-sonnet-20250219（推奨）

#### ステップ4: 実行

1. 「🤝 アンサンブル開始」ボタンをクリック
2. Multi-Agentビューに自動的に切り替わる

#### 期待される動作

✅ **正常動作の確認ポイント**:

1. **ビュー切り替え**: Multi-Agentビューに切り替わる
2. **インスタンスカード表示**: 2つのインスタンスカードが表示される
   - カードには以下が表示される：
     - エージェント名（タスクの要約 + ランタイム絵文字 + インスタンス番号）
     - ステータス（running）
     - 開始時刻
3. **tmuxセッション作成**: `tmux ls` で確認
   ```bash
   tmux ls
   # 出力例:
   # ait42-claude-ensemble-abc123de-1: 1 windows ...
   # ait42-claude-ensemble-abc123de-2: 1 windows ...
   ```
4. **出力のリアルタイム表示**: 各インスタンスの出力がリアルタイムで更新される
5. **セッション履歴への保存**: ダッシュボードで確認
   - 「📊 ダッシュボード」ボタンをクリック
   - Kanban Boardにセッションが表示される
   - セッションタイプが "ensemble" として表示される

#### トラブルシューティング

**問題: Claude CLIが起動しない**
```bash
# 原因確認
echo $ANTHROPIC_API_KEY  # 空白の場合は未設定

# 解決策
export ANTHROPIC_API_KEY="sk-ant-..."
```

**問題: tmuxセッションが見つからない**
```bash
# tmux インストール確認
which tmux

# セッション一覧確認
tmux ls

# セッションの手動削除（テスト後のクリーンアップ）
tmux kill-session -t ait42-claude-ensemble-abc123de-1
```

**問題: Git worktreeエラー**
```bash
# 現在のworktreeを確認
git worktree list

# 不要なworktreeを削除
git worktree remove src-tauri/.worktrees/competition-abc123de/instance-1

# pruneで不正なworktreeをクリーンアップ
git worktree prune
```

---

### シナリオ2: 複数ランタイムのMix（Claude + Codex）

#### ステップ1: Ensembleダイアログを開く

✨ アンサンブル ボタンをクリック

#### ステップ2: タスクの入力

```
"Implement a user authentication system with JWT tokens in Node.js"
```

#### ステップ3: ランタイム設定

- **Claude**: 2インスタンス
  - Model: claude-3-7-sonnet-20250219
- **Codex**: 1インスタンス
  - Model: gpt-4

#### ステップ4: 実行

「🤝 アンサンブル開始」をクリック

#### 期待される動作

✅ **確認ポイント**:

1. 3つのインスタンスカードが表示される
2. 各カードにランタイム絵文字が表示される：
   - 🤖 Claude x 2
   - 🔮 Codex x 1
3. それぞれ異なるCLIが起動する：
   - Claude: `claude` コマンド
   - Codex: `chatgpt` または `openai` コマンド
4. 各インスタンスが独立したworktreeで実行される
5. 出力がリアルタイムで更新される

---

### シナリオ3: セッション履歴とChat機能

#### ステップ1: Ensembleを実行

シナリオ1または2を実行してセッションを作成

#### ステップ2: ダッシュボードを開く

「📊 ダッシュボード」ボタンをクリック

#### ステップ3: セッション詳細を確認

1. Kanban Boardから実行中のセッションをクリック
2. セッション詳細ビューが表示される
3. 以下の情報が表示されることを確認：
   - セッションID
   - タスク説明
   - ステータス（running/completed）
   - インスタンス一覧
   - 各インスタンスの詳細（worktree path, branch, runtime）

#### ステップ4: Chatタブでコマンド送信

1. Chatタブをクリック
2. メッセージ入力欄に以下を入力：
   ```
   Can you explain what you've implemented so far?
   ```
3. 送信ボタンをクリック
4. メッセージがChat履歴に表示されることを確認

---

## テストログの確認

### フロントエンドログ

```bash
# ブラウザの開発者コンソールを開く（Tauri開発モード）
# Chrome: Cmd+Option+I (macOS) / Ctrl+Shift+I (Linux/Windows)

# 確認すべきログ:
# - [Ensemble] Starting ensemble: { competitionId, allocations, task, instances }
# - [MultiAgentPanel] Registering listener for competition {competitionId}
# - [MultiAgentPanel] Received competition-output: instance=1, output_len=...
```

### バックエンドログ

```bash
# Tauriコンソール（Rustログ）
# 確認すべきログ:
# - Starting ensemble mode multi-runtime competition: 2 instances
# - 📁 Project root for competition: /path/to/AIT42-Editor
# - 📁 Worktrees directory: /path/to/.worktrees
# - Creating worktree 1 (runtime: claude) at ...
# - 🚀 Installing AIT42 in worktree instance 1
# - ✅ AIT42 installed in instance 1
# - ✅ [HANDSHAKE] Frontend ready signal received for competition {id}
```

### tmuxセッションログ

```bash
# セッション一覧
tmux ls

# セッションにアタッチ（ライブ表示）
tmux attach -t ait42-claude-ensemble-abc123de-1

# デタッチ: Ctrl+B → D

# セッション出力をキャプチャ（ログファイル確認）
tmux capture-pane -t ait42-claude-ensemble-abc123de-1 -p

# ログファイルの確認（自動保存されている）
cat /path/to/worktree/.claude-output-1.log
```

---

## 修正履歴

### v1.0.0 (2025-11-12)

#### 修正 #1: tmuxSessionID統一化

**問題**: フロントエンドとバックエンドでtmuxセッションID形式が不一致

**修正内容**:
- `src/App.tsx`: tmuxSessionIdを `ait42-{runtime}-{mode}-{shortId}-{instance}` 形式に変更

**変更前**:
```typescript
tmuxSessionId: `${allocation.runtime}-${prefix}-${competitionId}-${index}`
// 例: "claude-ens-abc123def456-1"
```

**変更後**:
```typescript
tmuxSessionId: `ait42-${allocation.runtime}-${mode}-${shortId}-${index}`
// 例: "ait42-claude-ensemble-abc123de-1"
```

#### 修正 #2: ハードコードされたパスの環境変数化

**問題**: AIT42ソースパスがハードコードされていた

**修正内容**:
- `src-tauri/src/commands/ait42.rs`: 環境変数 `AIT42_SOURCE_PATH` から読み込むように変更
- フォールバック: `~/.ait42` → レガシーパス → デフォルト

---

## パフォーマンス目標

| メトリック | 目標値 | 実測値 | ステータス |
|-----------|-------|-------|-----------|
| Ensembleダイアログ起動時間 | < 100ms | - | 未測定 |
| tmuxセッション作成時間 | < 2秒/インスタンス | - | 未測定 |
| Git worktree作成時間 | < 3秒/インスタンス | - | 未測定 |
| Claude Code起動時間 | < 5秒 | - | 未測定 |
| 出力更新レイテンシ | < 500ms | - | 未測定 |

---

## トラブルシューティングチェックリスト

### 起動失敗

- [ ] `ANTHROPIC_API_KEY` が設定されている
- [ ] Claude CLIがインストールされている (`claude --version`)
- [ ] tmuxがインストールされている (`tmux -V`)
- [ ] Git リポジトリが初期化されている (`git status`)
- [ ] Node.jsバージョンがv18以上 (`node --version`)
- [ ] Rustがインストールされている (`rustc --version`)

### tmuxセッション問題

- [ ] `tmux ls` でセッションが表示される
- [ ] セッション名が `ait42-{runtime}-{mode}-{shortId}-{instance}` 形式
- [ ] セッションが実行中（status: running）
- [ ] ログファイルが作成されている (`.{runtime}-output-{instance}.log`)

### 出力が表示されない

- [ ] ブラウザコンソールでエラーがないか確認
- [ ] Tauriコンソールでエラーがないか確認
- [ ] ハンドシェイクプロトコルが成功しているか確認
  - `✅ [HANDSHAKE] Frontend ready signal received` ログ
- [ ] tmuxセッションが実行中か確認 (`tmux ls`)

### worktree問題

- [ ] `git worktree list` で worktree が表示される
- [ ] worktree パスが存在する
- [ ] worktree ブランチが作成されている
- [ ] 不要な worktree を削除 (`git worktree prune`)

---

## 次のステップ

1. **自動テストスクリプトの作成**
   - Playwrightでの E2E テスト自動化
   - CI/CD パイプラインへの統合

2. **パフォーマンス測定**
   - 各フェーズの実行時間計測
   - メモリ使用量の監視

3. **エラー処理の強化**
   - ネットワークタイムアウト対応
   - CLI起動失敗時のリトライ機構
   - worktree クリーンアップの自動化

4. **ユーザビリティ改善**
   - プログレスバーの追加
   - エラーメッセージの改善
   - ヘルプドキュメントの充実
