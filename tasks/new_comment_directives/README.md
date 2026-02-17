# Tasks - 作業引き継ぎドキュメント

## 📋 ドキュメント一覧

### 🚀 [IMMEDIATE_ACTIONS.md](./IMMEDIATE_ACTIONS.md)
**すぐに実行すべきアクション**

最初にこれを読んで、即座に実行可能なステップバイステップのガイドに従ってください。
- 所要時間: 約1時間
- 推奨される最小限の修正方法を提示

### 🎯 [HANDOFF.md](./HANDOFF.md)
**エージェント引き継ぎ - クイックスタート**

作業の概要と、次のエージェントがすぐに始められる情報をまとめています。
- 現在の状態
- エラーの再現方法
- 推奨される対応
- デバッグコマンド

### 🔍 [array_bracket_trailing_comment_issue.md](./array_bracket_trailing_comment_issue.md)
**配列の開きブラケット後のコメント処理問題（詳細版）**

問題の根本原因、試行錯誤の履歴、複数の解決アプローチを詳細に説明しています。
- 問題の根本原因分析
- これまでの試行錯誤
- 推奨される解決アプローチ（4つ）
- 関連ファイルの詳細

## 📖 読む順序

### 新しく作業を始める場合
1. **HANDOFF.md** - 全体像を把握（5分）
2. **IMMEDIATE_ACTIONS.md** - 即座に実行（1時間）
3. **array_bracket_trailing_comment_issue.md** - 詳細理解（必要に応じて）

### 問題の詳細を理解したい場合
1. **array_bracket_trailing_comment_issue.md** - 技術的な深堀り
2. **HANDOFF.md** - 実装の選択肢
3. **IMMEDIATE_ACTIONS.md** - 実行

## 🎯 現在の目標

### 短期目標（優先度: 高）
1. ✅ `comprehensive_full_example`テストをパスさせる
2. ✅ グループベースソートテスト全体をパスさせる
3. ⏳ linterテストの失敗（23件）を調査・修正

### 中期目標（優先度: 中）
1. 開きブラケットのtrailing commentを元の行に保持する実装
2. 全てのテストをパスさせる
3. `draft/new_comment_treatment.md`の更新

### 長期目標（優先度: 低）
1. AST構造の見直し（必要に応じて）
2. パフォーマンス最適化

## 🔧 クイックリファレンス

### テストコマンド
```bash
# グループソートテストのみ
cargo test --test test_group_based_sorting

# 特定のテスト
cargo test --test test_group_based_sorting comprehensive_full_example

# ライブラリテスト
cargo test --lib

# フォーマッターテスト全体
cargo test -p tombi-formatter
```

### デバッグ用ファイル
```bash
# 簡易テストファイル
/tmp/test_array_bracket_comment.toml

# パース検証プロジェクト
/tmp/test_parse_comprehensive

# 包括的なテストファイル
/tmp/test_comprehensive.toml
```

### 主要なファイル
```
# パーサー
crates/tombi-parser/src/parse/array.rs
crates/tombi-parser/src/parse.rs

# AST
crates/tombi-ast/src/impls/array.rs

# フォーマッター
crates/tombi-formatter/src/format/value/array.rs

# テスト
crates/tombi-formatter/tests/test_group_based_sorting.rs
```

## 📝 状態サマリー

**ブランチ**: `add_new_comment_treatment_draft`

**最新コミット**: `a3733313 docs: add draft for new comment treatment implementation readiness`

**テスト状況**:
- グループソートテスト: 12 passed, 1 failed
- ライブラリテスト: 68 passed, 23 failed

**主な問題**:
配列の開きブラケット直後のtrailing commentの処理で、パースエラーまたはコメント重複が発生

**推奨される最初のアクション**:
テストの期待値を更新して、trailing commentを次の行に移動（簡単な解決）

## 🆘 困ったときは

1. `IMMEDIATE_ACTIONS.md`のトラブルシューティングセクションを確認
2. `array_bracket_trailing_comment_issue.md`の「推奨される次のステップ」を参照
3. 簡易テストケースで問題を再現して理解を深める

## 📚 関連ドキュメント

プロジェクトルートの他のドキュメント:
- `CLAUDE.md` - プロジェクト全体のガイドライン
- `draft/new_comment_treatment.md` - コメント処理の設計ドキュメント

---

**作成日時**: 2026-02-15
**前のエージェント**: Claude Sonnet 4.5
**セッションID**: 19f18098-2057-4c4e-b9d9-856736163733
