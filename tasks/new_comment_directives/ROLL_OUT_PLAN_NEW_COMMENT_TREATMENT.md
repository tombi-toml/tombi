# New Comment Treatment 打ち取り計画（進捗チェックリスト）

最終更新: 2026-02-24

## 0. 現在の進捗サマリ

- [x] `cargo nextest run --package tombi-parser` が全通（74/74）
- [x] `cargo nextest run --package tombi-formatter` が全通（244/244）
- [ ] `cargo nextest run --package tombi-linter` が全通
  - 現状: `lint::value::array::tests::type_test::test_array_min_values_with_end_dangling_comment_directive` が失敗
- [ ] `cargo check --workspace` が通過
  - 現状: `tombi-lsp` で API 追従不足によるコンパイルエラーが多数発生

## 1. main 差分の確認

- [x] `main...HEAD` の差分棚卸しを実施
- [x] 影響 crate の洗い出しを実施
  - `crates/tombi-ast`: 23 files
  - `crates/tombi-parser`: 20 files
  - `crates/tombi-ast-editor`: 19 files
  - `crates/tombi-formatter`: 17 files
  - `crates/tombi-linter`: 7 files
  - `crates/tombi-document-tree`: 3 files
  - `crates/tombi-syntax`: 1 file
  - `crates/tombi-validator`: 1 file
  - `extensions/*`: 2 files
  - `xtask/*`: 2 files

## 2. 目的

- [ ] 新しいコメントモデル（`DanglingCommentGroup`, `DanglingCommentGroupOr`）を全 crate で一貫運用
- [ ] 責務境界を固定
  - Parser: 構文受理（comma optional）
  - Linter: 構文不備を diagnostics 化
  - Ast-Editor: 自動修正（非末尾 missing comma 補完）
  - Formatter: AST をそのまま出力（意味解釈追加なし）

## 3. 実施順序（固定）

- [x] 1. Parser / AST 契約を確定（parser package テストは通過）
- [ ] 2. Ast-Editor の補正ロジックを契約準拠に統一
- [ ] 3. Formatter の出力責務を最小化
- [ ] 4. Linter / Diagnostics を契約に追従
- [ ] 5. Extension の code action を追従
- [ ] 6. 生成コード更新（xtask）と総合テスト

## 4. crate 別チェックリスト

### `crates/tombi-parser`

- [x] package テストが全通
- [ ] `value_with_comma_group` / `key_value_with_comma_group` の comma optional 契約の再点検完了
- [ ] `dangling_comment_group` の抽出境界（`[`, `{`, table header 前後）の再点検完了
- [ ] negative テストの「元のエラーメッセージ維持」確認完了

### `crates/tombi-ast`

- [ ] 自動生成ノードと手書きノード境界の再確認完了
- [ ] `dangling_comment_groups` / `*_with_comma_groups` の bracket/brace trailing comment 分離確認完了
- [ ] `has_last_*_trailing_comma` 系仕様の固定完了

### `crates/tombi-ast-editor`

- [ ] 非末尾 missing comma の補完を array/inline_table で対称化
- [ ] 末尾 missing comma 保持を array/inline_table で対称化
- [ ] `array_values_order` / `inline_table_keys_order` の `ReplaceRange` 契約整理完了
- [ ] group 単位ソート維持（group 間移動禁止）の確認完了

### `crates/tombi-formatter`

- [x] package テストが全通
- [x] 回帰テストをマクロで追加済み（group 単位ソート + 末尾カンマ欠落保持）
  - `test_array_with_inner_comment_directive_with_separator_line_without_trailing_comma`
  - `test_inline_table_with_inner_comment_directive_with_separator_line_without_trailing_comma`
- [ ] AST 解釈の追加なし（責務境界の最終確認）
- [ ] `DanglingCommentGroup` 系 `Format` 契約の最終確認

### `crates/tombi-linter`

- [ ] package テストが全通
- [x] missing comma を parser ではなく linter で報告する契約は実装済み（`MissingCommaRule`）
- [ ] array/inline_table 対称性の最終確認
- [ ] diagnostics 文言の後方互換確認

### `crates/tombi-document-tree` / `crates/tombi-validator`

- [ ] accessor 解決と schema 解決への影響確認
- [ ] document comment directive 解決責務の再確認

### `extensions/*`

- [ ] code action が新ルール（non-last 補完 / last 保持）に一致することを確認

### `xtask/*`

- [ ] 生成対象から除外すべき node accessor の再確認
- [ ] `xtask` 再生成と差分確認

## 5. テストゲート

- [x] `cargo nextest run --package tombi-parser`
- [x] `cargo nextest run --package tombi-formatter`
- [ ] `cargo nextest run --package tombi-linter`
- [ ] `cargo check --workspace`
- [ ] 必要に応じて: `cargo nextest run --package tombi-formatter test_x_tombi_order`
- [ ] 必要に応じて: `cargo nextest run --package tombi-formatter test_format_options`

## 6. 完了条件

- [ ] `parser/editor/formatter/linter` の責務境界が `draft/new_comment_treatment/new_comment_treatment.md` と一致
- [ ] 末尾 missing comma 保持 / 非末尾 missing comma 補完が array/inline_table で対称
- [ ] group 単位ソート維持（group 間移動なし）
- [ ] テストゲート全通

## 7. 進め方

- [ ] 1 PR = 1責務（Parser / Ast-Editor / Formatter / Linter を分割）
- [ ] 各 PR で「契約」「変更点」「非変更点」「回帰テスト」を明記
- [ ] 仕様迷いが出た場合は `draft/new_comment_treatment/new_comment_treatment.md` を正とする
