# New Comment Treatment 打ち取り計画

## 1. main 差分の確認結果

`main...HEAD` の差分はコメント処理基盤の再設計を含む広範囲変更。

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
- `draft/new_comment_treatment/*`: ドキュメント再配置あり

## 2. 目的

- 新しいコメントモデル (`DanglingCommentGroup`, `DanglingCommentGroupOr`) を全 crate で一貫運用する。
- 責務境界を固定する。
  - Parser: 構文受理（comma は optional を許可）
  - Linter: 構文上の不備を diagnostics 化
  - Ast-Editor: 自動修正（非末尾 missing comma の補完）
  - Formatter: AST をそのまま出力（意味解釈を追加しない）

## 3. 実施順序（固定）

1. Parser / AST 契約を確定
2. Ast-Editor の補正ロジックを契約準拠に統一
3. Formatter の出力責務を最小化
4. Linter / Diagnostics を契約に追従
5. Extension の code action を追従
6. 生成コード更新（xtask）と総合テスト

## 4. crate 別打ち取りタスク

## `crates/tombi-parser`

- `value_with_comma_group`, `key_value_with_comma_group` の comma optional 前提を再点検。
- `dangling_comment_group` の抽出境界（`[`, `{`, table header 前後）を再点検。
- negative テストは「元のエラーメッセージ維持」を確認。

## `crates/tombi-ast`

- 自動生成ノードと手書きノードの境界を維持（`values()/key_values()` の実体が group を通ること）。
- `dangling_comment_groups` / `*_with_comma_groups` の bracket/brace trailing comment 分離を再確認。
- `has_last_*_trailing_comma` 系の仕様を固定。

## `crates/tombi-ast-editor`

- 非末尾 missing comma は必ず補完。
- 末尾 missing comma は保持（array/inline_table 共通）。
- `array_values_order` / `inline_table_keys_order` での `ReplaceRange` 構築時、
  「末尾欠落保持」を末尾要素にのみ適用。
- group 単位ソートを維持（group 間移動禁止）。

## `crates/tombi-formatter`

- AST 解釈を増やさない（document directive の格上げなどは editor 側責務）。
- `DanglingCommentGroup` / `Vec<DanglingCommentGroup>` /
  `Vec<DanglingCommentGroupOr<T>>` のフォーマット契約を固定。
- trailing comment alignment は group 単位のまま維持。

## `crates/tombi-linter`

- missing comma を parser ではなく linter で報告する契約を固定。
- array/inline_table のルール対称性を確認。
- diagnostics 文言の後方互換を維持。

## `crates/tombi-document-tree` / `crates/tombi-validator`

- accessor 解決と schema 解決で comment/group 変更の影響を再確認。
- document comment directive 解決は validator/editor 側で完結することを確認。

## `extensions/*`

- code action の修正提案が新ルール（non-last comma 補完 / last comma 保持）に一致するか確認。

## `xtask/*`

- 生成対象から除外すべきノード accessor を再確認。
- `xtask` 再生成 -> 差分が期待通りか確認。

## 5. テストゲート

各フェーズで以下を最小ゲートにする。

```bash
cargo nextest run --package tombi-parser
cargo nextest run --package tombi-formatter
cargo nextest run --package tombi-linter
cargo check --workspace
```

必要に応じて追加:

```bash
cargo nextest run --package tombi-formatter test_x_tombi_order
cargo nextest run --package tombi-formatter test_format_options
```

## 6. 完了条件

- `parser/editor/formatter/linter` の責務境界が README/draft と実装で一致。
- 末尾 missing comma 保持、非末尾 missing comma 補完が array/inline_table で対称。
- group 単位ソートが維持され、group 間移動が発生しない。
- 上記テストゲートを全通。

## 7. 進め方

- 1 PR = 1責務（Parser, Ast-Editor, Formatter, Linter を分割）。
- 各 PR で「契約」「変更点」「非変更点」「回帰テスト」を明記。
- 仕様迷いが出た場合は `draft/new_comment_treatment/new_comment_treatment.md` を正とする。
