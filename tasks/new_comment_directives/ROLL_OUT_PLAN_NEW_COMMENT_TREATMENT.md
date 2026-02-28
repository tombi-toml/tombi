# New Comment Treatment 打ち取り計画（進捗チェックリスト）

最終更新: 2026-02-24

## 0. 現在の進捗サマリ

- [x] `cargo nextest run --package tombi-parser` が全通（74/74）
- [x] `cargo nextest run --package tombi-formatter` が全通（244/244）
- [x] `cargo nextest run --package tombi-linter` が全通（101/101）
- [x] `cargo check --workspace` が通過

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

- [x] 新しいコメントモデル（`DanglingCommentGroup`, `DanglingCommentGroupOr`）を全 crate で一貫運用
- [x] 責務境界を固定
  - Parser: 構文受理（comma optional）
  - Linter: 構文不備を diagnostics 化
  - Ast-Editor: 自動修正（非末尾 missing comma 補完）
  - Formatter: AST をそのまま出力（意味解釈追加なし）

## 3. 実施順序（固定）

- [x] 1. Parser / AST 契約を確定（parser package テストは通過）
- [x] 2. Ast-Editor の補正ロジックを契約準拠に統一
- [x] 3. Formatter の出力責務を最小化
- [x] 4. Linter / Diagnostics を契約に追従
- [x] 5. Extension の code action を追従
- [x] 6. 生成コード更新（xtask）と総合テスト

## 4. crate 別チェックリスト

### `crates/tombi-parser`

- [x] package テストが全通
- [x] `value_with_comma_group` / `key_value_with_comma_group` の comma optional 契約の再点検完了
- [x] `dangling_comment_group` の抽出境界（`[`, `{`, table header 前後）の再点検完了
- [x] negative テストの「元のエラーメッセージ維持」確認完了

### `crates/tombi-ast`

- [x] 自動生成ノードと手書きノード境界の再確認完了
  - 除外リスト: Array::values, Root/Table/ArrayOfTable/InlineTable::key_values
  - 手書きノード: DanglingCommentGroup, DanglingCommentGroupOr, KeyValueGroup, KeyValueWithCommaGroup, ValueWithCommaGroup
- [x] `dangling_comment_groups` / `*_with_comma_groups` の bracket/brace trailing comment 分離確認完了
- [x] `has_last_*_trailing_comma` 系仕様の固定完了

### `crates/tombi-ast-editor`

- [x] 非末尾 missing comma の補完を array/inline_table で対称化
  - array.rs:56-60 と inline_table.rs:36-40 で同一ロジック `!has_last_comma || !is_last_item`
- [x] 末尾 missing comma 保持を array/inline_table で対称化
  - array_values_order.rs:109-116 と inline_table_keys_order.rs:71-78 で同一ロジック
- [x] `array_values_order` / `inline_table_keys_order` の `ReplaceRange` 契約整理完了
  - old range: first..=last、new: reconstructed syntax elements
- [x] group 単位ソート維持（group 間移動禁止）の確認完了
  - array.rs:102-125, inline_table.rs:64-79 で group 単位イテレーション

### `crates/tombi-formatter`

- [x] package テストが全通
- [x] 回帰テストをマクロで追加済み（group 単位ソート + 末尾カンマ欠落保持）
  - `test_array_with_inner_comment_directive_with_separator_line_without_trailing_comma`
  - `test_inline_table_with_inner_comment_directive_with_separator_line_without_trailing_comma`
- [x] AST 解釈の追加なし（責務境界の最終確認）
  - Formatter はカンマ挿入・構造推論を一切行わない。AST をそのまま出力。
- [x] `DanglingCommentGroup` 系 `Format` 契約の最終確認
  - DanglingCommentGroup, Vec<DanglingCommentGroup>, Vec<DanglingCommentGroupOr<T>> の3層実装確認済み

### `crates/tombi-linter`

- [x] package テストが全通
- [x] missing comma を parser ではなく linter で報告する契約は実装済み（`MissingCommaRule`）
- [x] array/inline_table 対称性の最終確認
  - missing_comma.rs:9-28 (Array) と missing_comma.rs:31-51 (InlineTable) で対称
  - InlineTable の追加ルール (KeyEmpty, DottedKeysOutOfOrder, TomlVersion) は TOML 仕様由来で意図的
- [x] diagnostics 文言の後方互換確認
  - 全 diagnostic code/message が後方互換を維持

### `crates/tombi-document-tree` / `crates/tombi-validator`

- [x] accessor 解決と schema 解決への影響確認
  - inner comment directives は accessor logic をバイパス（正しい動作）
  - schema 解決に変更なし
- [x] document comment directive 解決責務の再確認
  - outer/inner の分離: AST (impls), document-tree, validator の3層で維持
  - value_with_comma_groups() からの dangling comment group も inner_comment_directives に収集

### `extensions/*`

- [x] code action が新ルール（non-last 補完 / last 保持）に一致することを確認
  - code_action.rs: dual-layer dangling comment 検出 (dangling_comment_groups + key_value_with_comma_groups)
  - completion.rs: dangling_comment_groups().next() で document header comment group を取得
  - hover, folding_range, semantic_tokens: DanglingCommentGroupOr パターンで統一

### `xtask/*`

- [x] 生成対象から除外すべき node accessor の再確認
  - is_excluded_auto_generated_method() で Array::values, Root/Table/ArrayOfTable/InlineTable::key_values を除外
- [x] `xtask` 再生成と差分確認
  - `cargo xtask codegen grammar` 実行後、差分なし（既に最新）

## 5. テストゲート

- [x] `cargo nextest run --package tombi-parser` (74/74)
- [x] `cargo nextest run --package tombi-formatter` (244/244)
- [x] `cargo nextest run --package tombi-linter` (101/101)
- [x] `cargo check --workspace`
- [x] 必要に応じて: `cargo nextest run --package tombi-formatter test_x_tombi_order`
- [x] 必要に応じて: `cargo nextest run --package tombi-formatter test_format_options`
- [x] `cargo xtask codegen grammar` 再生成後の差分なし

## 6. 完了条件

- [x] `parser/editor/formatter/linter` の責務境界が `draft/new_comment_treatment/new_comment_treatment.md` と一致
- [x] 末尾 missing comma 保持 / 非末尾 missing comma 補完が array/inline_table で対称
- [x] group 単位ソート維持（group 間移動なし）
- [x] テストゲート全通

## 7. 進め方

- [ ] 1 PR = 1責務（Parser / Ast-Editor / Formatter / Linter を分割）
- [ ] 各 PR で「契約」「変更点」「非変更点」「回帰テスト」を明記
- [ ] 仕様迷いが出た場合は `draft/new_comment_treatment/new_comment_treatment.md` を正とする

## 8. 後続作業（自動テスト）

以下の crate はマクロベースのテストが存在しないため、今回のロールアウトでは自動テストを実施していない。
後続の工程で統合テスト / E2E テストとして検証すること：

- **tombi-ast-editor**: comma 補完・ソートの統合テスト（現在マクロテストなし）
- **tombi-document-tree / tombi-validator**: comment directive 解決の統合テスト
- **tombi-lsp**: code action, completion, hover, folding range, semantic tokens の E2E テスト
