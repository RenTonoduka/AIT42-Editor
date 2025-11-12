# AIT42 Unified System

このディレクトリは、AIT42の統合システムです。
Claude Code、Gemini CLI、CodeX CLIを1つのシステムで管理します。

## ディレクトリ構造

```
.ait42/
├── agents/              # 共通エージェント定義（49 agents）
├── memory/              # 統合メモリシステム
│   ├── tasks/          # タスク履歴（全プロバイダー共有）
│   ├── agents/         # エージェント統計
│   └── providers/      # プロバイダー統計
├── coordinators/        # 各CLI用Coordinator
├── commands/            # スラッシュコマンド
├── logs/               # ログファイル
├── config.yaml         # 統一設定ファイル
└── README.md           # このファイル
```

## 使い方

```bash
# タスク実行（自動プロバイダー選択）
python scripts/ait42_unified.py "タスクの説明"

# プロバイダー指定
python scripts/ait42_unified.py "タスク" --provider=gemini

# 複数プロバイダーで実行
python scripts/ait42_unified.py "タスク" --multi

# システムテスト
python scripts/ait42_unified.py --test
```

## 参考資料

- [統合移行ガイド](../docs/MIGRATION_TO_UNIFIED_AIT42.md)
- [Multi-CLI Orchestration Proposal](../docs/MULTI_CLI_ORCHESTRATION_PROPOSAL.md)
