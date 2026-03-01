# New Comment Treatment 仕様（実装準拠）

## 目的
コメントの紐付けを壊さずに、空行で区切られたコメントグループを AST 上で保持し、
フォーマット・ソート・LSP/バリデーションの挙動を一貫させる。

## 対象スコープ
- Root
- Table (`[table]`)
- ArrayOfTable (`[[table]]`)
- Array (`[...]`)
- InlineTable (`{...}`)

## 用語
- 空行: `LINE_BREAK` が 2 つ以上連続した状態（間の `WHITESPACE` は無視して判定）
- グループ区切り: `is_group_separator(p)`
  - `p.at_ts(TS_LINE_END) && p.nth_at_ts(1, TS_LINE_END)`
  - `TS_LINE_END = { LINE_BREAK, EOF }`
- dangling comment group:
  - `DANGLING_COMMENT_GROUP` ノード
  - 連続したコメント行を 1 グループとして保持

## 構文モデル

### 1. `DANGLING_COMMENT_GROUP`
- SyntaxKind: `DANGLING_COMMENT_GROUP`
- AST ノード: `DanglingCommentGroup`
  - `comments()`
  - `into_comments()`
  - `range()`

`dangling_comment_group_len` の判定:
1. 先頭が `COMMENT`
2. 各コメント行の直後が `TS_LINE_END`
3. 次トークンが `TS_DANGLING_COMMENT_GROUP_END = { '}', ']', LINE_BREAK, EOF }` ならグループ終端
4. 次が `COMMENT` なら同一グループ継続

### 2. `DanglingCommentGroupOr<T>`
`DANGLING_COMMENT_GROUP` と要素グループを同じ列として扱う。

```rust
pub enum DanglingCommentGroupOr<T> {
    DanglingCommentGroup(DanglingCommentGroup),
    ItemGroup(T),
}
```

### 3. コンテナごとの要素グループ
| コンテナ | 要素グループ | API |
|---|---|---|
| Root | `KeyValueGroup` | `key_value_groups() -> impl Iterator<Item = DanglingCommentGroupOr<KeyValueGroup>>` |
| Table | `KeyValueGroup` | `key_value_groups() -> impl Iterator<Item = DanglingCommentGroupOr<KeyValueGroup>>` |
| ArrayOfTable | `KeyValueGroup` | `key_value_groups() -> impl Iterator<Item = DanglingCommentGroupOr<KeyValueGroup>>` |
| Array | `ValueWithCommaGroup` | `value_with_comma_groups() -> impl Iterator<Item = DanglingCommentGroupOr<ValueWithCommaGroup>>` |
| InlineTable | `KeyValueWithCommaGroup` | `key_value_with_comma_groups() -> impl Iterator<Item = DanglingCommentGroupOr<KeyValueWithCommaGroup>>` |

## パース規則

### 共通フロー
各コンテナのループで次を繰り返す:
1. 先頭 `LINE_BREAK` を消費
2. `Vec::<DanglingCommentGroup>::parse(p)`
3. `peek_leading_comments(p)` で終端/次セクション判定
4. 終端でなければ要素グループ (`KeyValueGroup` / `ValueWithCommaGroup` / `KeyValueWithCommaGroup`) をパース

### 要素グループ終端
- `KeyValueGroup::parse` / `ValueWithCommaGroup::parse` / `KeyValueWithCommaGroup::parse` は
  `is_group_separator(p)` を見た時点でグループを閉じる
- これにより、空行境界でグループが分割される

## コメントの所属

### leading comment
- `peek_leading_comments(p)` が拾うコメント
- 次の要素 (`KeyValue` / `Value`) 側に取り込まれる

### dangling comment group
- 要素に紐付かない独立ノードとして保持
- コンテナ直下に配置される

## ルール制御用ディレクティブの有効範囲

### `comment_directives()` が読む範囲（有効）
- Root: `dangling_comment_groups()`
- Table: `header_leading_comments()` + `header_trailing_comment()` + `dangling_comment_groups()`
- ArrayOfTable: `header_leading_comments()` + `header_trailing_comment()` + `dangling_comment_groups()`
- Array: `bracket_start_trailing_comment()` + `dangling_comment_groups()`
- InlineTable: `brace_start_trailing_comment()` + `dangling_comment_groups()`

### 無効（グループ境界）
`*_groups()` の途中に現れる `DanglingCommentGroupOr::DanglingCommentGroup` は、
フォーマッター/リンターのルール制御には使わない。

## Group Boundary Directive の扱い
- `document-tree` では `group_boundary_comment_directives` として保持する
- ルール制御には使わない（`comment_directives()` の対象外）
- LSP/バリデーションでは `TombiGroupBoundaryDirectiveContent` で解釈する
  - `GroupBoundaryFormatRules` / `GroupBoundaryLintRules` は空定義
  - ルールキーを書くと `KeyNotAllowed` 診断になる

## ソート規則
- ソート対象は `DanglingCommentGroupOr::ItemGroup` のみ
- `DanglingCommentGroup` はソート対象外
- グループをまたいだ移動は行わない

### 実装上の適用単位
- Root: 各 `KeyValueGroup` ごとに `table_keys_order`
- Table/ArrayOfTable: 各 `KeyValueGroup` ごとに `table_keys_order`
- Array: 各 `ValueWithCommaGroup` ごとに `array_values_order`
- InlineTable: 各 `KeyValueWithCommaGroup` ごとに `inline_table_keys_order`

## フォーマット規則

### `DanglingCommentGroup` の出力
- グループ内コメントは改行区切りでそのまま出力
- コメントスキップモード時は出力しない

### `Vec<DanglingCommentGroup>` の出力
- グループ間は常に 1 空行（改行 2 つ）
- 連続空行は 1 空行に正規化

### `Vec<DanglingCommentGroupOr<T>>` の出力
- `ItemGroup` と `ItemGroup` の間: 1 空行
- `DanglingCommentGroup` と次グループの間:
  - 直前に空行があった場合 (`has_empty_line_before == true`): 1 空行
  - なかった場合: 空行なし（改行 1 つ）

### テーブルヘッダ直後の扱い
- Table / ArrayOfTable では、ヘッダ直後に dangling group がある場合でも、
  最初の dangling group の前には空行を出さない（改行 1 つのみ）

## 互換性上の変更点
- 旧挙動: key/value 間のコメントが leading へマージされ、空行が消えるケースがあった
- 新挙動: `DANGLING_COMMENT_GROUP` として空行境界を保持し、
  出力時は最大 1 空行へ正規化して再現する

## 参照実装
- Parser
  - `crates/tombi-parser/src/parse/dangling_comment_group.rs`
  - `crates/tombi-parser/src/parse/root.rs`
  - `crates/tombi-parser/src/parse/table.rs`
  - `crates/tombi-parser/src/parse/array_of_table.rs`
  - `crates/tombi-parser/src/parse/array.rs`
  - `crates/tombi-parser/src/parse/inline_table.rs`
  - `crates/tombi-parser/src/parse/key_value_group.rs`
  - `crates/tombi-parser/src/parse/value_with_comma_group.rs`
  - `crates/tombi-parser/src/parse/key_value_with_comma_group.rs`
- AST
  - `crates/tombi-ast/src/node/dangling_comment_group.rs`
  - `crates/tombi-ast/src/node/dangling_comment_group_or.rs`
  - `crates/tombi-ast/src/support/comment.rs`
  - `crates/tombi-ast/src/impls/root.rs`
  - `crates/tombi-ast/src/impls/table.rs`
  - `crates/tombi-ast/src/impls/array_of_table.rs`
  - `crates/tombi-ast/src/impls/array.rs`
  - `crates/tombi-ast/src/impls/inline_table.rs`
- Formatter
  - `crates/tombi-formatter/src/format/comment.rs`
  - `crates/tombi-formatter/src/format/root.rs`
  - `crates/tombi-formatter/src/format/table.rs`
  - `crates/tombi-formatter/src/format/array_of_table.rs`
  - `crates/tombi-formatter/src/format/value/array.rs`
  - `crates/tombi-formatter/src/format/value/inline_table.rs`
- Document Tree / LSP / Validator
  - `crates/tombi-document-tree/src/root.rs`
  - `crates/tombi-document-tree/src/value/table.rs`
  - `crates/tombi-document-tree/src/value/array.rs`
  - `crates/tombi-comment-directive/src/value/group_boundary.rs`
  - `crates/tombi-lsp/src/comment_directive.rs`
  - `crates/tombi-validator/src/comment_directive/value.rs`
- AST Editor (sort)
  - `crates/tombi-ast-editor/src/edit/root.rs`
  - `crates/tombi-ast-editor/src/edit/table.rs`
  - `crates/tombi-ast-editor/src/edit/array_of_table.rs`
  - `crates/tombi-ast-editor/src/edit/array.rs`
  - `crates/tombi-ast-editor/src/edit/inline_table.rs`
