# ディベートモード Phase 1 - Rust Backend実装仕様書

## Document Information

- **Version**: 1.0.0
- **Date**: 2025-11-04
- **Status**: Design Complete
- **Target Phase**: Phase 1 (Sequential Execution)
- **Implementation Timeline**: 4 weeks

## Executive Summary

本仕様書は、AIT42 Editor v2のディベートモード Phase 1における**Rust Backend**の実装詳細を定義します。Phase 1では**シーケンシャル実行**（1エージェントずつ順次実行）に焦点を当て、Phase 2以降の並列実行・リアルタイムストリーミングのための基盤を構築します。

### Key Design Principles

1. **Separation of Concerns**: 各コンポーネントの責任を明確に分離
2. **Error Resilience**: Graceful Degradation（一部失敗でも継続可能）
3. **Test-Driven**: ユニットテスト可能な設計
4. **Type Safety**: Rust型システムを最大限活用
5. **Async-First**: Tokio非同期ランタイム使用

---

## 1. System Architecture Overview

### 1.1 Component Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                      Tauri Frontend (TypeScript)            │
│                  (src/services/DebateService.ts)            │
└────────────────────────────┬────────────────────────────────┘
                             │ IPC (Tauri Commands)
                             ▼
┌─────────────────────────────────────────────────────────────┐
│                   Tauri Command Layer                        │
│               (src-tauri/src/commands/debate/)              │
│   - execute_debate()                                        │
│   - get_debate_status()                                     │
│   - cancel_debate()                                         │
│   - get_round_outputs()                                     │
└────────────────────────────┬────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────┐
│                    DebateOrchestrator                       │
│           (src-tauri/src/debate/orchestrator.rs)            │
│   - Overall debate lifecycle management                     │
│   - Round coordination                                      │
│   - Error handling & recovery                               │
└─────┬──────────────┬────────────────┬──────────────┬────────┘
      │              │                │              │
      ▼              ▼                ▼              ▼
┌──────────┐  ┌──────────────┐  ┌─────────────┐  ┌──────────┐
│ Context  │  │    Round     │  │  Worktree   │  │  Tmux    │
│ Manager  │  │   Executor   │  │  Manager    │  │ Manager  │
└──────────┘  └──────────────┘  └─────────────┘  └──────────┘
   (JSON)     (Claude Code)        (Git)          (Session)
```

### 1.2 Module Structure

```
src-tauri/
├── src/
│   ├── debate/                     # 新規ディベートモジュール
│   │   ├── mod.rs                  # モジュール定義
│   │   ├── orchestrator.rs         # 主制御ロジック
│   │   ├── round_executor.rs       # ラウンド実行エンジン
│   │   ├── context_manager.rs      # コンテキストファイル管理
│   │   ├── worktree_manager.rs     # Git Worktree操作
│   │   ├── tmux_manager.rs         # Tmuxセッション管理
│   │   ├── types.rs                # 共通型定義
│   │   └── error.rs                # エラー型定義
│   ├── commands/
│   │   └── debate.rs               # Tauri Commands
│   ├── state.rs                    # AppState (既存)
│   └── main.rs                     # エントリポイント
└── Cargo.toml                      # 依存関係更新
```

---

## 2. Type Definitions (types.rs)

### 2.1 Core Types

```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// ディベート実行リクエスト (Frontend → Backend)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebateRequest {
    /// ディベートのタスク記述
    pub task: String,

    /// 参加エージェント役割定義
    pub roles: Vec<RoleDefinition>,

    /// 使用するClaudeモデル
    pub model: ClaudeModel,

    /// ラウンドごとのタイムアウト (秒)
    pub timeout_seconds: u64,

    /// Worktreeを保持するか (デバッグ用)
    pub preserve_worktrees: bool,

    /// ベースディレクトリ (省略時は現在のプロジェクトルート)
    pub base_dir: Option<PathBuf>,
}

/// エージェント役割定義
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleDefinition {
    /// 役割ID (例: "architect", "developer", "qa")
    pub id: String,

    /// 役割名 (UI表示用)
    pub name: String,

    /// システムプロンプト
    pub system_prompt: String,

    /// 初期ファイルコンテキスト (オプション)
    pub initial_context: Option<Vec<PathBuf>>,
}

/// Claudeモデル選択
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ClaudeModel {
    #[serde(rename = "sonnet")]
    Sonnet,

    #[serde(rename = "haiku")]
    Haiku,

    #[serde(rename = "opus")]
    Opus,
}

impl ClaudeModel {
    pub fn to_cli_arg(&self) -> &'static str {
        match self {
            ClaudeModel::Sonnet => "--model=sonnet",
            ClaudeModel::Haiku => "--model=haiku",
            ClaudeModel::Opus => "--model=opus",
        }
    }
}

/// ディベート実行結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebateResult {
    /// ディベートID (UUID)
    pub debate_id: String,

    /// 現在のステータス
    pub status: DebateStatus,

    /// メッセージ (エラー時の詳細など)
    pub message: String,

    /// 作成されたWorktreeパス
    pub worktree_paths: Vec<PathBuf>,

    /// コンテキストファイルパス
    pub context_path: PathBuf,
}

/// ディベートステータス
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DebateStatus {
    /// 開始前
    Pending,

    /// Round 1実行中
    Round1InProgress,

    /// Round 2実行中
    Round2InProgress,

    /// Round 3実行中
    Round3InProgress,

    /// 正常完了
    Completed,

    /// エラーで失敗
    Failed(String),

    /// ユーザーによるキャンセル
    Cancelled,
}

/// ラウンドごとの出力
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundOutput {
    /// 役割ID
    pub role: String,

    /// エージェントの提案内容 (Claude Codeの出力)
    pub proposal: String,

    /// 実行時刻 (RFC3339形式)
    pub timestamp: String,

    /// 実行時間 (秒)
    pub duration_secs: u64,

    /// 使用トークン数 (Phase 2で実装)
    pub tokens_used: u32,

    /// エラー情報 (失敗時のみ)
    pub error: Option<String>,
}

/// ディベート進行状況 (Frontend向けイベント)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebateProgress {
    pub debate_id: String,
    pub current_round: u8,
    pub current_role: String,
    pub status: DebateStatus,
    pub outputs: Vec<RoundOutput>,
}
```

---

## 3. Error Handling (error.rs)

### 3.1 Error Types

```rust
use thiserror::Error;
use std::io;

/// ディベートモジュール専用エラー型
#[derive(Debug, Error)]
pub enum DebateError {
    /// Worktree作成失敗
    #[error("Worktree creation failed for role '{role}': {source}")]
    WorktreeCreationFailed {
        role: String,
        source: String,
    },

    /// Worktree削除失敗
    #[error("Worktree removal failed: {0}")]
    WorktreeRemovalFailed(String),

    /// Tmuxセッション操作失敗
    #[error("Tmux session '{session}' failed: {source}")]
    TmuxSessionFailed {
        session: String,
        source: String,
    },

    /// ラウンド実行タイムアウト
    #[error("Round {round} execution timeout (role: {role})")]
    RoundExecutionTimeout {
        round: u8,
        role: String,
    },

    /// エージェント実行エラー
    #[error("Agent '{role}' crashed in round {round}: {details}")]
    AgentCrashed {
        round: u8,
        role: String,
        details: String,
    },

    /// コンテキストファイル破損
    #[error("Context file corrupted: {path}")]
    ContextFileCorrupted {
        path: String,
    },

    /// コンテキストファイル読み込み失敗
    #[error("Failed to load context for round {round}: {source}")]
    ContextLoadFailed {
        round: u8,
        source: String,
    },

    /// I/Oエラー
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    /// JSON Serializationエラー
    #[error("JSON serialization error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// 不明なエラー
    #[error("Unexpected error: {0}")]
    UnexpectedError(String),
}

/// Result型エイリアス
pub type Result<T> = std::result::Result<T, DebateError>;
```

---

## 4. DebateOrchestrator (orchestrator.rs)

### 4.1 Structure

```rust
use tokio::sync::{Mutex, RwLock};
use std::sync::Arc;
use std::path::PathBuf;
use super::*;

/// ディベート全体の制御ロジック
pub struct DebateOrchestrator {
    /// ディベートID (UUID)
    debate_id: String,

    /// リクエスト情報
    request: DebateRequest,

    /// 現在のラウンド番号 (0=未開始, 1-3=実行中)
    current_round: Arc<Mutex<u8>>,

    /// 現在のステータス
    status: Arc<RwLock<DebateStatus>>,

    /// Worktree管理
    worktree_manager: WorktreeManager,

    /// Tmux管理 (Phase 1では最小限)
    tmux_manager: TmuxManager,

    /// コンテキスト管理
    context_manager: ContextManager,

    /// ラウンド実行エンジン
    round_executor: RoundExecutor,

    /// ディベート作業ディレクトリ
    work_dir: PathBuf,
}

impl DebateOrchestrator {
    /// 新規ディベートオーケストレーター作成
    pub fn new(debate_id: String, request: DebateRequest) -> Self {
        let work_dir = PathBuf::from(format!("/tmp/debate-{}", debate_id));

        Self {
            debate_id: debate_id.clone(),
            worktree_manager: WorktreeManager::new(&debate_id, &work_dir),
            tmux_manager: TmuxManager::new(&debate_id),
            context_manager: ContextManager::new(&debate_id, &work_dir),
            round_executor: RoundExecutor::new(
                &debate_id,
                request.timeout_seconds,
            ),
            request,
            current_round: Arc::new(Mutex::new(0)),
            status: Arc::new(RwLock::new(DebateStatus::Pending)),
            work_dir,
        }
    }

    /// ディベート実行 (非同期)
    pub async fn execute(&mut self) -> Result<DebateResult> {
        tracing::info!("Starting debate {}", self.debate_id);

        // 1. ディレクトリ構造作成
        self.setup_directories().await?;

        // 2. Round 1実行 (独立提案)
        self.execute_round(1).await?;

        // 3. Round 2実行 (批判的分析)
        self.execute_round(2).await?;

        // 4. Round 3実行 (合意形成)
        self.execute_round(3).await?;

        // 5. 最終ステータス更新
        *self.status.write().await = DebateStatus::Completed;

        // 6. クリーンアップ (オプション)
        if !self.request.preserve_worktrees {
            self.cleanup().await?;
        }

        tracing::info!("Debate {} completed successfully", self.debate_id);

        Ok(DebateResult {
            debate_id: self.debate_id.clone(),
            status: DebateStatus::Completed,
            message: "Debate completed successfully".to_string(),
            worktree_paths: self.worktree_manager.get_all_paths(),
            context_path: self.work_dir.join("context"),
        })
    }

    /// ディレクトリ構造セットアップ
    async fn setup_directories(&self) -> Result<()> {
        tracing::debug!("Setting up directories for debate {}", self.debate_id);

        tokio::fs::create_dir_all(&self.work_dir).await?;
        tokio::fs::create_dir_all(self.work_dir.join("worktrees")).await?;
        tokio::fs::create_dir_all(self.work_dir.join("context")).await?;
        tokio::fs::create_dir_all(self.work_dir.join("logs")).await?;

        tracing::info!("Debate workspace created at: {:?}", self.work_dir);
        Ok(())
    }

    /// ラウンド実行 (シーケンシャル)
    async fn execute_round(&mut self, round: u8) -> Result<()> {
        tracing::info!("Executing round {}", round);

        // ステータス更新
        *self.current_round.lock().await = round;
        *self.status.write().await = match round {
            1 => DebateStatus::Round1InProgress,
            2 => DebateStatus::Round2InProgress,
            3 => DebateStatus::Round3InProgress,
            _ => return Err(DebateError::UnexpectedError(
                format!("Invalid round number: {}", round)
            )),
        };

        let mut round_outputs = Vec::new();
        let mut failed_roles = Vec::new();

        // Phase 1: 各役割をシーケンシャルに実行
        for role in &self.request.roles {
            tracing::info!("Round {}: Executing role '{}'", round, role.id);

            match self.execute_single_role(round, role).await {
                Ok(output) => {
                    round_outputs.push(output);
                    tracing::info!("Round {}: Role '{}' succeeded", round, role.id);
                },
                Err(e) => {
                    tracing::error!("Round {}: Role '{}' failed: {}", round, role.id, e);
                    failed_roles.push(role.id.clone());

                    // Graceful Degradation: エラーを記録するが継続
                    round_outputs.push(RoundOutput {
                        role: role.id.clone(),
                        proposal: String::new(),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                        duration_secs: 0,
                        tokens_used: 0,
                        error: Some(e.to_string()),
                    });
                }
            }
        }

        // 少なくとも1つ成功すれば継続
        if round_outputs.iter().all(|o| o.error.is_some()) {
            return Err(DebateError::RoundExecutionTimeout {
                round,
                role: "all".to_string(),
            });
        }

        // コンテキストファイル保存
        self.context_manager
            .save_round_outputs(round, &round_outputs)
            .await?;

        tracing::info!(
            "Round {} completed: {} succeeded, {} failed",
            round,
            round_outputs.len() - failed_roles.len(),
            failed_roles.len()
        );

        Ok(())
    }

    /// 単一役割の実行
    async fn execute_single_role(
        &mut self,
        round: u8,
        role: &RoleDefinition,
    ) -> Result<RoundOutput> {
        // 1. Worktree作成
        let worktree_path = self.worktree_manager
            .create_worktree(round, &role.id)
            .await?;

        // 2. 前ラウンドのコンテキスト取得
        let context = if round > 1 {
            self.context_manager.load_round_context(round - 1).await?
        } else {
            None
        };

        // 3. ラウンド実行
        let output = self.round_executor
            .execute(
                round,
                role,
                &worktree_path,
                context,
                self.request.model,
            )
            .await?;

        Ok(output)
    }

    /// クリーンアップ
    async fn cleanup(&mut self) -> Result<()> {
        tracing::info!("Cleaning up debate {}", self.debate_id);

        // Worktree削除
        self.worktree_manager.cleanup_all().await?;

        // Tmuxセッション終了
        self.tmux_manager.kill_all_sessions().await?;

        // 一時ファイル削除
        tokio::fs::remove_dir_all(&self.work_dir).await?;

        Ok(())
    }

    /// 現在のステータス取得
    pub async fn get_status(&self) -> DebateStatus {
        self.status.read().await.clone()
    }

    /// 現在の進行状況取得
    pub async fn get_progress(&self) -> Result<DebateProgress> {
        let current_round = *self.current_round.lock().await;
        let status = self.status.read().await.clone();

        // すべてのラウンドの出力を集約
        let mut all_outputs = Vec::new();
        for r in 1..=current_round {
            if let Some(outputs) = self.context_manager.load_round_outputs(r).await? {
                all_outputs.extend(outputs);
            }
        }

        Ok(DebateProgress {
            debate_id: self.debate_id.clone(),
            current_round,
            current_role: all_outputs.last().map(|o| o.role.clone()).unwrap_or_default(),
            status,
            outputs: all_outputs,
        })
    }

    /// ディベートキャンセル
    pub async fn cancel(&mut self) -> Result<()> {
        tracing::warn!("Cancelling debate {}", self.debate_id);

        *self.status.write().await = DebateStatus::Cancelled;
        self.cleanup().await?;

        Ok(())
    }
}
```

---

## 5. RoundExecutor (round_executor.rs)

### 5.1 Implementation

```rust
use tokio::process::Command;
use tokio::time::{timeout, Duration};
use std::path::Path;
use super::*;

/// ラウンド実行エンジン
pub struct RoundExecutor {
    debate_id: String,
    timeout_duration: Duration,
}

impl RoundExecutor {
    pub fn new(debate_id: &str, timeout_seconds: u64) -> Self {
        Self {
            debate_id: debate_id.to_string(),
            timeout_duration: Duration::from_secs(timeout_seconds),
        }
    }

    /// ラウンド実行 (単一役割)
    pub async fn execute(
        &self,
        round: u8,
        role: &RoleDefinition,
        worktree_path: &Path,
        context: Option<String>,
        model: ClaudeModel,
    ) -> Result<RoundOutput> {
        let start_time = std::time::Instant::now();

        // プロンプト構築
        let prompt = self.build_prompt(round, role, context);

        // Claude Code実行 (タイムアウト付き)
        let proposal = timeout(
            self.timeout_duration,
            self.execute_claude_code(worktree_path, &prompt, model),
        )
        .await
        .map_err(|_| DebateError::RoundExecutionTimeout {
            round,
            role: role.id.clone(),
        })??;

        let duration = start_time.elapsed().as_secs();

        Ok(RoundOutput {
            role: role.id.clone(),
            proposal,
            timestamp: chrono::Utc::now().to_rfc3339(),
            duration_secs: duration,
            tokens_used: 0, // TODO: Phase 2でClaude APIから取得
            error: None,
        })
    }

    /// プロンプト構築
    fn build_prompt(
        &self,
        round: u8,
        role: &RoleDefinition,
        context: Option<String>,
    ) -> String {
        let round_instruction = match round {
            1 => {
                // Round 1: 独立提案
                format!(
                    "## Round 1: 独立提案\n\n\
                    与えられたタスクに対して、あなたの専門領域 ({}) の視点から提案してください。\n\
                    他のエージェントの意見を見る前に、独自の分析と提案を行ってください。",
                    role.name
                )
            },
            2 => {
                // Round 2: 批判的分析
                format!(
                    "## Round 2: 批判的分析\n\n\
                    前ラウンドで他のエージェントが提案した内容:\n\
                    ```json\n{}\n```\n\n\
                    上記の提案を批判的に分析し、改善点や代替案を提示してください。\n\
                    あなたの専門領域 ({}) の視点から、技術的・実装的な問題点を指摘してください。",
                    context.as_deref().unwrap_or("(コンテキストなし)"),
                    role.name
                )
            },
            3 => {
                // Round 3: 合意形成
                format!(
                    "## Round 3: 合意形成\n\n\
                    全ラウンドで議論された内容:\n\
                    ```json\n{}\n```\n\n\
                    上記の提案と批判を統合し、最終的な実装計画を作成してください。\n\
                    あなたの専門領域 ({}) の視点から、実装可能で合意可能な最終提案を行ってください。",
                    context.as_deref().unwrap_or("(コンテキストなし)"),
                    role.name
                )
            },
            _ => panic!("Invalid round number: {}", round),
        };

        format!(
            "{}\n\n{}\n\n---\n\n\
            重要な指示:\n\
            - Markdown形式で回答してください\n\
            - 具体的なコード例やアーキテクチャ図を含めてください\n\
            - 実装可能性を重視してください\n\
            - タイムアウトは{}秒です（簡潔に）",
            role.system_prompt,
            round_instruction,
            self.timeout_duration.as_secs()
        )
    }

    /// Claude Code実行
    async fn execute_claude_code(
        &self,
        worktree_path: &Path,
        prompt: &str,
        model: ClaudeModel,
    ) -> Result<String> {
        tracing::debug!("Executing claude with prompt length: {}", prompt.len());

        // Claude Code CLIを実行
        let output = Command::new("claude")
            .arg("--prompt")
            .arg(prompt)
            .arg(model.to_cli_arg())
            .current_dir(worktree_path)
            .output()
            .await
            .map_err(|e| DebateError::UnexpectedError(
                format!("Failed to execute claude CLI: {}", e)
            ))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(DebateError::AgentCrashed {
                round: 0, // TODO: ラウンド番号を渡す
                role: "claude".to_string(),
                details: stderr.to_string(),
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();

        tracing::debug!("Claude output length: {}", stdout.len());
        Ok(stdout)
    }
}
```

---

## 6. ContextManager (context_manager.rs)

### 6.1 Implementation

```rust
use std::path::{Path, PathBuf};
use super::*;

/// コンテキストファイル管理
pub struct ContextManager {
    debate_id: String,
    context_dir: PathBuf,
}

impl ContextManager {
    pub fn new(debate_id: &str, work_dir: &Path) -> Self {
        Self {
            debate_id: debate_id.to_string(),
            context_dir: work_dir.join("context"),
        }
    }

    /// ラウンド出力を保存
    pub async fn save_round_outputs(
        &self,
        round: u8,
        outputs: &[RoundOutput],
    ) -> Result<()> {
        let path = self.get_round_file_path(round);

        let json = serde_json::to_string_pretty(outputs)?;
        tokio::fs::write(&path, json).await?;

        tracing::info!("Saved round {} outputs to {:?}", round, path);
        Ok(())
    }

    /// ラウンドコンテキスト読み込み (JSON文字列)
    pub async fn load_round_context(&self, round: u8) -> Result<Option<String>> {
        let path = self.get_round_file_path(round);

        if !tokio::fs::try_exists(&path).await? {
            tracing::warn!("Round {} context file not found", round);
            return Ok(None);
        }

        let json = tokio::fs::read_to_string(&path).await?;

        // バリデーション: JSONパース可能か確認
        serde_json::from_str::<Vec<RoundOutput>>(&json)
            .map_err(|e| DebateError::ContextFileCorrupted {
                path: path.display().to_string(),
            })?;

        Ok(Some(json))
    }

    /// ラウンド出力を構造化データで取得
    pub async fn load_round_outputs(&self, round: u8) -> Result<Option<Vec<RoundOutput>>> {
        let context_json = self.load_round_context(round).await?;

        match context_json {
            Some(json) => {
                let outputs = serde_json::from_str(&json)?;
                Ok(Some(outputs))
            },
            None => Ok(None),
        }
    }

    /// すべてのラウンドのコンテキストを統合
    pub async fn get_all_context(&self) -> Result<Vec<RoundOutput>> {
        let mut all_outputs = Vec::new();

        for round in 1..=3 {
            if let Some(outputs) = self.load_round_outputs(round).await? {
                all_outputs.extend(outputs);
            }
        }

        Ok(all_outputs)
    }

    /// ラウンドファイルパス取得
    fn get_round_file_path(&self, round: u8) -> PathBuf {
        self.context_dir.join(format!("round{}-outputs.json", round))
    }
}
```

---

## 7. WorktreeManager (worktree_manager.rs)

### 7.1 Implementation

```rust
use tokio::process::Command;
use std::path::{Path, PathBuf};
use super::*;

/// Git Worktree管理
pub struct WorktreeManager {
    debate_id: String,
    worktree_base: PathBuf,
    worktrees: Vec<WorktreeInfo>,
}

#[derive(Debug, Clone)]
struct WorktreeInfo {
    path: PathBuf,
    branch: String,
    round: u8,
    role: String,
}

impl WorktreeManager {
    pub fn new(debate_id: &str, work_dir: &Path) -> Self {
        Self {
            debate_id: debate_id.to_string(),
            worktree_base: work_dir.join("worktrees"),
            worktrees: Vec::new(),
        }
    }

    /// Worktree作成
    pub async fn create_worktree(&mut self, round: u8, role: &str) -> Result<PathBuf> {
        let worktree_path = self.worktree_base.join(format!("round{}-{}", round, role));
        let branch_name = format!("debate-{}-r{}-{}", self.debate_id, round, role);

        tracing::info!(
            "Creating worktree: path={:?}, branch={}",
            worktree_path,
            branch_name
        );

        // git worktree add -b <branch> <path>
        let output = Command::new("git")
            .args(&[
                "worktree",
                "add",
                "-b",
                &branch_name,
                worktree_path.to_str().ok_or_else(|| {
                    DebateError::WorktreeCreationFailed {
                        role: role.to_string(),
                        source: "Invalid UTF-8 in path".to_string(),
                    }
                })?,
            ])
            .output()
            .await
            .map_err(|e| DebateError::WorktreeCreationFailed {
                role: role.to_string(),
                source: e.to_string(),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(DebateError::WorktreeCreationFailed {
                role: role.to_string(),
                source: stderr.to_string(),
            });
        }

        // 記録
        self.worktrees.push(WorktreeInfo {
            path: worktree_path.clone(),
            branch: branch_name,
            round,
            role: role.to_string(),
        });

        tracing::info!("Worktree created successfully: {:?}", worktree_path);
        Ok(worktree_path)
    }

    /// すべてのWorktree削除
    pub async fn cleanup_all(&self) -> Result<()> {
        tracing::info!("Cleaning up {} worktrees", self.worktrees.len());

        for worktree in &self.worktrees {
            match self.remove_worktree(&worktree.path).await {
                Ok(_) => tracing::info!("Removed worktree: {:?}", worktree.path),
                Err(e) => tracing::error!("Failed to remove worktree {:?}: {}", worktree.path, e),
            }
        }

        Ok(())
    }

    /// 単一Worktree削除
    async fn remove_worktree(&self, path: &Path) -> Result<()> {
        let output = Command::new("git")
            .args(&["worktree", "remove", "--force", path.to_str().unwrap()])
            .output()
            .await
            .map_err(|e| DebateError::WorktreeRemovalFailed(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(DebateError::WorktreeRemovalFailed(stderr.to_string()));
        }

        Ok(())
    }

    /// すべてのWorktreeパス取得
    pub fn get_all_paths(&self) -> Vec<PathBuf> {
        self.worktrees.iter().map(|w| w.path.clone()).collect()
    }
}
```

---

## 8. TmuxManager (tmux_manager.rs)

### 8.1 Implementation (Phase 1: Minimal)

```rust
use tokio::process::Command;
use super::*;

/// Tmuxセッション管理 (Phase 1では最小限)
pub struct TmuxManager {
    debate_id: String,
    sessions: Vec<String>,
}

impl TmuxManager {
    pub fn new(debate_id: &str) -> Self {
        Self {
            debate_id: debate_id.to_string(),
            sessions: Vec::new(),
        }
    }

    /// Tmuxセッション開始 (Phase 2で本格実装)
    pub async fn start_session(&mut self, round: u8, role: &str) -> Result<String> {
        let session_name = format!("debate-{}-r{}-{}", self.debate_id, round, role);

        tracing::debug!("Starting tmux session: {}", session_name);

        let output = Command::new("tmux")
            .args(&["new-session", "-d", "-s", &session_name])
            .output()
            .await
            .map_err(|e| DebateError::TmuxSessionFailed {
                session: session_name.clone(),
                source: e.to_string(),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);

            // 既存セッションエラーは無視 (冪等性)
            if !stderr.contains("duplicate session") {
                return Err(DebateError::TmuxSessionFailed {
                    session: session_name,
                    source: stderr.to_string(),
                });
            }
        }

        self.sessions.push(session_name.clone());
        Ok(session_name)
    }

    /// すべてのセッション終了
    pub async fn kill_all_sessions(&self) -> Result<()> {
        tracing::info!("Killing {} tmux sessions", self.sessions.len());

        for session in &self.sessions {
            let _ = Command::new("tmux")
                .args(&["kill-session", "-t", session])
                .output()
                .await; // エラーは無視 (既に終了している可能性)
        }

        Ok(())
    }
}
```

---

## 9. Tauri Commands (commands/debate.rs)

### 9.1 Command Definitions

```rust
use tauri::State;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use crate::debate::*;

/// アプリケーションステート (グローバル)
pub struct DebateState {
    /// 実行中のディベート (debate_id → orchestrator)
    pub orchestrators: Arc<Mutex<HashMap<String, DebateOrchestrator>>>,
}

impl DebateState {
    pub fn new() -> Self {
        Self {
            orchestrators: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

/// ディベート開始コマンド
#[tauri::command]
pub async fn execute_debate(
    state: State<'_, DebateState>,
    request: DebateRequest,
) -> Result<DebateResult, String> {
    tracing::info!("Received debate request: {} roles", request.roles.len());

    let debate_id = uuid::Uuid::new_v4().to_string();
    let mut orchestrator = DebateOrchestrator::new(debate_id.clone(), request);

    // 非同期実行 (バックグラウンド)
    let orchestrators = state.orchestrators.clone();
    let debate_id_clone = debate_id.clone();

    tokio::spawn(async move {
        match orchestrator.execute().await {
            Ok(result) => {
                tracing::info!("Debate {} completed: {:?}", debate_id_clone, result);
            },
            Err(e) => {
                tracing::error!("Debate {} failed: {}", debate_id_clone, e);
            }
        }

        // 完了後に削除
        orchestrators.lock().await.remove(&debate_id_clone);
    });

    // オーケストレーターを保存 (ステータス取得用)
    state.orchestrators.lock().await.insert(debate_id.clone(), orchestrator);

    Ok(DebateResult {
        debate_id,
        status: DebateStatus::Pending,
        message: "Debate started successfully".to_string(),
        worktree_paths: Vec::new(),
        context_path: PathBuf::from("/tmp"),
    })
}

/// ディベートステータス取得
#[tauri::command]
pub async fn get_debate_status(
    state: State<'_, DebateState>,
    debate_id: String,
) -> Result<DebateStatus, String> {
    let orchestrators = state.orchestrators.lock().await;

    match orchestrators.get(&debate_id) {
        Some(orchestrator) => Ok(orchestrator.get_status().await),
        None => Err(format!("Debate {} not found", debate_id)),
    }
}

/// ディベート進行状況取得
#[tauri::command]
pub async fn get_debate_progress(
    state: State<'_, DebateState>,
    debate_id: String,
) -> Result<DebateProgress, String> {
    let orchestrators = state.orchestrators.lock().await;

    match orchestrators.get(&debate_id) {
        Some(orchestrator) => orchestrator
            .get_progress()
            .await
            .map_err(|e| e.to_string()),
        None => Err(format!("Debate {} not found", debate_id)),
    }
}

/// ディベートキャンセル
#[tauri::command]
pub async fn cancel_debate(
    state: State<'_, DebateState>,
    debate_id: String,
) -> Result<(), String> {
    let mut orchestrators = state.orchestrators.lock().await;

    match orchestrators.get_mut(&debate_id) {
        Some(orchestrator) => {
            orchestrator.cancel().await.map_err(|e| e.to_string())?;
            orchestrators.remove(&debate_id);
            Ok(())
        },
        None => Err(format!("Debate {} not found", debate_id)),
    }
}

/// ラウンド出力取得
#[tauri::command]
pub async fn get_round_outputs(
    debate_id: String,
    round: u8,
) -> Result<Vec<RoundOutput>, String> {
    let work_dir = PathBuf::from(format!("/tmp/debate-{}", debate_id));
    let context_manager = ContextManager::new(&debate_id, &work_dir);

    context_manager
        .load_round_outputs(round)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Round {} outputs not found", round))
}
```

---

## 10. Main Integration (main.rs)

### 10.1 Setup

```rust
// main.rs に追加

mod debate;
mod commands;

use commands::debate::{
    DebateState,
    execute_debate,
    get_debate_status,
    get_debate_progress,
    cancel_debate,
    get_round_outputs,
};

fn main() {
    tauri::Builder::default()
        .manage(DebateState::new())
        .invoke_handler(tauri::generate_handler![
            // 既存のコマンド...

            // ディベートモードコマンド
            execute_debate,
            get_debate_status,
            get_debate_progress,
            cancel_debate,
            get_round_outputs,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

---

## 11. Testing Strategy

### 11.1 Unit Tests

```rust
// src-tauri/src/debate/orchestrator.rs

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_orchestrator_creation() {
        let request = DebateRequest {
            task: "Test task".to_string(),
            roles: vec![
                RoleDefinition {
                    id: "architect".to_string(),
                    name: "System Architect".to_string(),
                    system_prompt: "You are a system architect".to_string(),
                    initial_context: None,
                },
            ],
            model: ClaudeModel::Haiku,
            timeout_seconds: 60,
            preserve_worktrees: true,
            base_dir: None,
        };

        let orchestrator = DebateOrchestrator::new("test-id".to_string(), request);

        assert_eq!(orchestrator.get_status().await, DebateStatus::Pending);
    }
}

// src-tauri/src/debate/context_manager.rs

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_context_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let manager = ContextManager::new("test-id", temp_dir.path());

        let outputs = vec![
            RoundOutput {
                role: "architect".to_string(),
                proposal: "Test proposal".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
                duration_secs: 10,
                tokens_used: 1000,
                error: None,
            },
        ];

        manager.save_round_outputs(1, &outputs).await.unwrap();

        let loaded = manager.load_round_outputs(1).await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().len(), 1);
    }
}
```

### 11.2 Integration Tests

```rust
// tests/debate_integration_test.rs

use ait42_editor::debate::*;

#[tokio::test]
async fn test_full_debate_execution() {
    let request = DebateRequest {
        task: "Implement user authentication system".to_string(),
        roles: vec![
            RoleDefinition {
                id: "architect".to_string(),
                name: "System Architect".to_string(),
                system_prompt: "Design system architecture".to_string(),
                initial_context: None,
            },
            RoleDefinition {
                id: "security".to_string(),
                name: "Security Expert".to_string(),
                system_prompt: "Review security aspects".to_string(),
                initial_context: None,
            },
        ],
        model: ClaudeModel::Haiku,
        timeout_seconds: 300,
        preserve_worktrees: false,
        base_dir: None,
    };

    let mut orchestrator = DebateOrchestrator::new(
        uuid::Uuid::new_v4().to_string(),
        request,
    );

    let result = orchestrator.execute().await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().status, DebateStatus::Completed);
}
```

---

## 12. Cargo.toml Updates

### 12.1 Dependencies

```toml
# src-tauri/Cargo.toml に追加

[dependencies]
# ... 既存の依存関係 ...

# ディベートモード用
uuid = { version = "1.6", features = ["serde", "v4"] }
chrono = { version = "0.4", features = ["serde"] }
thiserror = "1.0"
```

---

## 13. Error Handling Strategy

### 13.1 Graceful Degradation Example

```rust
impl DebateOrchestrator {
    /// エラーハンドリング付きラウンド実行
    async fn execute_round_with_fallback(&mut self, round: u8) -> Result<()> {
        let mut successful_outputs = Vec::new();
        let mut failed_roles = Vec::new();

        for role in &self.request.roles {
            match self.execute_single_role(round, role).await {
                Ok(output) => {
                    successful_outputs.push(output);
                    tracing::info!("Role {} succeeded", role.id);
                },
                Err(e) => {
                    tracing::error!("Role {} failed: {}", role.id, e);
                    failed_roles.push((role.id.clone(), e.to_string()));

                    // 失敗情報を記録して継続
                    successful_outputs.push(RoundOutput {
                        role: role.id.clone(),
                        proposal: String::new(),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                        duration_secs: 0,
                        tokens_used: 0,
                        error: Some(e.to_string()),
                    });
                }
            }
        }

        // 少なくとも1つ成功すれば継続
        if successful_outputs.is_empty() {
            return Err(DebateError::RoundExecutionTimeout {
                round,
                role: "all".to_string(),
            });
        }

        // すべての結果を保存 (失敗も含む)
        self.context_manager
            .save_round_outputs(round, &successful_outputs)
            .await?;

        // 失敗情報をログに記録
        if !failed_roles.is_empty() {
            tracing::warn!(
                "Round {} completed with {} failures: {:?}",
                round,
                failed_roles.len(),
                failed_roles
            );
        }

        Ok(())
    }
}
```

---

## 14. Logging Configuration

### 14.1 Structured Logging

```rust
// main.rs に追加

use tracing_subscriber::{fmt, prelude::*, EnvFilter};

fn setup_logging() {
    tracing_subscriber::registry()
        .with(fmt::layer().with_target(true).with_thread_ids(true))
        .with(EnvFilter::from_default_env().add_directive(
            "ait42_editor::debate=debug".parse().unwrap()
        ))
        .init();
}

fn main() {
    setup_logging();

    // ... Tauri setup
}
```

---

## 15. Next Steps (Implementation Roadmap)

### Phase 1 Implementation (4 weeks)

#### Week 1: Core Infrastructure
- [ ] `debate/types.rs` - 型定義実装
- [ ] `debate/error.rs` - エラー型実装
- [ ] `debate/context_manager.rs` - コンテキスト管理実装
- [ ] ユニットテスト作成

#### Week 2: Git & Process Management
- [ ] `debate/worktree_manager.rs` - Worktree管理実装
- [ ] `debate/tmux_manager.rs` - Tmux管理 (最小限)
- [ ] `debate/round_executor.rs` - Claude Code実行
- [ ] 統合テスト作成

#### Week 3: Orchestration
- [ ] `debate/orchestrator.rs` - 主制御ロジック
- [ ] `commands/debate.rs` - Tauri Commands
- [ ] `main.rs` - 統合
- [ ] エンドツーエンドテスト

#### Week 4: Testing & Documentation
- [ ] 全コンポーネントのテストカバレッジ 80%+
- [ ] エラーハンドリングテスト
- [ ] ドキュメント作成
- [ ] Phase 2準備 (並列実行設計)

---

## 16. Success Criteria

### 16.1 Functional Requirements

- ✅ 3ラウンドのディベート実行が成功する
- ✅ Worktreeが正しく作成・削除される
- ✅ コンテキストファイルが各ラウンドで共有される
- ✅ エラー発生時もGraceful Degradation動作
- ✅ Tauri Frontendからステータス取得可能

### 16.2 Non-Functional Requirements

- ✅ テストカバレッジ >= 80%
- ✅ Rustコンパイル警告 0件
- ✅ `cargo clippy` 警告 0件
- ✅ すべての`unwrap()`を適切なエラーハンドリングに置き換え
- ✅ ドキュメント完備

---

## 17. References

- [Phase 1 System Architecture](./SYSTEM_ARCHITECTURE_SPEC.md) - 全体アーキテクチャ
- [Tauri Documentation](https://tauri.app/v1/guides/) - Tauri API
- [Tokio Documentation](https://tokio.rs/) - 非同期ランタイム
- [Git Worktree](https://git-scm.com/docs/git-worktree) - Git Worktree仕様

---

## Appendix A: Example Usage (TypeScript)

```typescript
// Frontend からのディベート実行例
import { invoke } from '@tauri-apps/api/tauri';

async function runDebate() {
  const request = {
    task: "Implement user authentication system",
    roles: [
      {
        id: "architect",
        name: "System Architect",
        system_prompt: "Design system architecture focusing on scalability",
        initial_context: null,
      },
      {
        id: "security",
        name: "Security Expert",
        system_prompt: "Review security vulnerabilities",
        initial_context: null,
      },
    ],
    model: "sonnet",
    timeout_seconds: 300,
    preserve_worktrees: false,
    base_dir: null,
  };

  try {
    const result = await invoke('execute_debate', { request });
    console.log('Debate started:', result);

    // ポーリングでステータス取得
    const interval = setInterval(async () => {
      const status = await invoke('get_debate_status', {
        debateId: result.debate_id,
      });

      console.log('Status:', status);

      if (status === 'Completed' || status.Failed) {
        clearInterval(interval);
      }
    }, 5000);
  } catch (error) {
    console.error('Debate failed:', error);
  }
}
```

---

**Document Revision History**:
- v1.0.0 (2025-11-04): Initial design complete
