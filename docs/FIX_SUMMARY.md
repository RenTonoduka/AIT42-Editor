# CompetitionDialog Timeout Bug - Fix Summary

## 修正概要

**問題**: CompetitionDialogが「Claude Codeがタスクを分析中...」で止まり、ユーザーが操作不能になる
**原因**: 120秒タイムアウトエラーがUIでキャッチされず、エラー状態が表示されない
**修正**: エラーハンドリング改善、ユーザーフィードバック追加、フォールバック動作実装

## 修正内容

### 1. エラーハンドリング改善
**ファイル**: `src/components/AI/CompetitionDialog.tsx:85-101`

```diff
- const debounceTimer = setTimeout(() => {
-   analyze(task.trim());
- }, 1500);
+ const debounceTimer = setTimeout(async () => {
+   try {
+     await analyze(task.trim());
+   } catch (error) {
+     console.warn('[CompetitionDialog] Auto-analysis failed (non-critical):', error);
+   }
+ }, 1500);
```

**効果**:
- タイムアウトエラーを適切にキャッチ
- UIが「分析中」状態で固まらない
- 非同期処理の正しい実装

### 2. エラーメッセージ改善
**ファイル**: `src/components/AI/CompetitionDialog.tsx:232-248`

**変更点**:
- 色を赤→黄色（警告レベル）
- エラータイトル追加（⚠️ 自動分析失敗）
- フォールバック手順を明示（手動設定可能）

**UI表示例**:
```
⚠️ 自動分析失敗
Request timed out after 120 seconds
手動でインスタンス数を設定してCompetitionを開始できます（推奨: 3インスタンス）
```

### 3. 分析中のユーザーガイダンス
**ファイル**: `src/components/AI/CompetitionDialog.tsx:208-220`

**追加機能**:
- 分析中でも操作可能であることを明示
- 分析をスキップしてCompetition開始可能

**UI表示例**:
```
🔄 Claude Codeがタスクを分析中...
分析完了を待たずにCompetitionを開始することもできます
```

### 4. リグレッションテスト追加
**ファイル**: `src/components/AI/__tests__/CompetitionDialog.test.tsx`

**テストカバレッジ**:
1. タイムアウトエラー処理（エラーメッセージ表示確認）
2. エラー後のCompetition開始（フォールバック動作）
3. 分析中のダイアログクローズ（操作性確認）
4. 成功時の動作（正常系）
5. 空エラーメッセージ処理（エッジケース）

## 修正前後の比較

### Before (バグあり)
```
1. ユーザーがタスク入力: "AIT42最適化したいです"
2. 1.5秒後に自動分析開始
3. バックエンドで120秒タイムアウト
4. ❌ UIが「分析中...」で固まる
5. ❌ エラーメッセージなし
6. ❌ ユーザーは操作不能
```

### After (修正後)
```
1. ユーザーがタスク入力: "AIT42最適化したいです"
2. 1.5秒後に自動分析開始
3. バックエンドで120秒タイムアウト
4. ✅ エラーメッセージ表示: "⚠️ 自動分析失敗"
5. ✅ フォールバック案内: "手動設定で継続可能"
6. ✅ ユーザーはデフォルト値でCompetition開始可能
```

## 動作確認手順

### 手動テスト
1. CompetitionDialogを開く
2. タスクフィールドに「AIT42最適化したいです」と入力
3. 1.5秒後、「Claude Codeがタスクを分析中...」が表示される
4. 120秒待つとエラーメッセージが表示される
5. インスタンス数を手動調整できることを確認
6. 「コンペティション開始」ボタンが有効であることを確認
7. Competitionが正常に開始されることを確認

### 自動テスト
```bash
npm run test src/components/AI/__tests__/CompetitionDialog.test.tsx
```

## 影響範囲

### 変更ファイル
- ✅ `src/components/AI/CompetitionDialog.tsx` - エラーハンドリング、UI改善
- ✅ `src/components/AI/__tests__/CompetitionDialog.test.tsx` - テスト追加（新規）
- ✅ `docs/BUG_FIX_COMPETITION_DIALOG_TIMEOUT.md` - 詳細ドキュメント（新規）
- ✅ `docs/FIX_SUMMARY.md` - この修正サマリー（新規）

### 非変更ファイル
- `src/hooks/useTaskOptimizer.ts` - エラーハンドリングは既に実装済み
- `src/services/tauri.ts` - APIインターフェース変更なし
- `src-tauri/src/commands/claude_code.rs` - バックエンドロジック変更なし

## 設計判断

### 1. なぜtry-catchをuseEffect内に追加したか？
**理由**: Promiseの拒否（rejection）はuseEffect外でキャッチできない
- setTimeout内で`analyze()`が実行される
- `analyze()`は非同期関数（Promise返す）
- エラーをキャッチするには`await`+`try-catch`が必要

### 2. なぜエラーを「赤」ではなく「黄色」にしたか？
**理由**: Ω理論分析は「オプショナル機能」であり、失敗しても致命的ではない
- ユーザーは手動設定で継続可能
- 警告レベル（Warning）の扱い
- UXの観点でパニックを避ける

### 3. なぜuseTaskOptimizerを変更しなかったか？
**理由**: 既にエラーハンドリングが正しく実装されている
- `state.status = 'error'`が正しく設定される
- `state.error`にエラーメッセージが格納される
- UI側のバグのため、バックエンドロジック変更不要

## パフォーマンス影響

### メモリ
- 影響なし（新規状態追加なし）

### CPU
- 影響なし（try-catchのオーバーヘッドは無視できる）

### ネットワーク
- 影響なし（API呼び出し回数変わらず）

### UX
- ✅ 改善: エラー発生時でもUIがブロックされない
- ✅ 改善: ユーザーは常に操作可能
- ✅ 改善: エラーメッセージで次の行動が明確

## セキュリティ影響

- 影響なし（エラーメッセージに機密情報なし）
- XSS脆弱性なし（`{optimizerState.error}`はReactが自動エスケープ）

## 今後の改善案

### 短期（v1.6.1）
- [ ] タイムアウト時間をユーザー設定可能に（60s/120s/300s）
- [ ] 分析中にキャンセルボタン追加
- [ ] エラー時にリトライボタン追加

### 長期（v1.7.0）
- [ ] 分析をバックグラウンドワーカーで実行
- [ ] 分析結果をキャッシュ（同じタスクで再利用）
- [ ] 段階的タイムアウト（30s警告 → 120s失敗）

## 参考資料

- [React useEffect with async/await](https://react.dev/reference/react/useEffect#fetching-data-with-effects)
- [Error Handling Best Practices](https://kentcdodds.com/blog/use-state-lazy-initialization-and-function-updates)
- [UX Writing: Error Messages](https://uxdesign.cc/how-to-write-better-error-messages-a7da67a90483)

---

**修正日**: 2025-11-09
**作成者**: Claude Code (bug-fixer agent)
**レビュー**: User (tonodukaren)
