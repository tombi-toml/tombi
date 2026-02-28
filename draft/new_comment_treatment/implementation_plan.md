# 新しいコメント解釈ロジック実装計画（DANGLING_COMMENT_GROUP ノードアプローチ）

## 概要

`draft/new_comment_treatment/new_comment_treatment.md` の仕様に基づき、パーサーレベルで `DANGLING_COMMENT_GROUP` ノードを導入することで、空行で区切られたコメントグループを明示的にASTに保存する。これにより、情報の損失を防ぎ、フォーマッターとエディターの実装を簡略化する。

## 設計方針

### 従来のアプローチの問題点
- パーサーがコメントをトークンとして認識するだけ
- AST層で後からコメントをグループ化しようとするが、情報が失われる
- グループ境界の判定が複雑で、コメントが消える問題が発生

### 採用したアプローチ
- **パーサーレベルで `DANGLING_COMMENT_GROUP` ノードを作成**
  - 空行で区切られたコメントグループを明示的にパース
  - グループ境界の判定はパーサーが一度だけ行う
- **`DanglingCommentGroup` を `AstNode` として実装**
  - `syntax()` メソッドで子要素にアクセス可能
  - `comments()` メソッドで `DanglingComment` トークンを取得
- **`DanglingCommentGroupOr<T>` ジェネリック Enum で混合グループを表現**
  - `DanglingCommentGroup(DanglingCommentGroup)`: コメントだけのグループ
  - `ItemGroup(T)`: 要素のグループ（KeyValueGroup, ValueWithCommaGroup, etc.）
  - 各グループ型に対して統一的に適用可能

> **注**: 当初案では `KeyValueGroup` 自体を Enum にする方針だったが、`DanglingCommentGroupOr<T>` を使う方式に変更。これにより各グループ型の責務がシンプルに保たれる。

### 利点
1. ✅ 情報の損失を防ぐ（パーサーで一度判定した情報をASTに保存）
2. ✅ 責務の明確化（Parser: 判定、AST: 保持、Formatter: 出力）
3. ✅ 実装の簡略化（フォーマッターは `DanglingCommentGroupOr<T>` をパターンマッチ）
4. ✅ コメントが消える問題の解決
5. ✅ 複数のグループ型に再利用可能

---

## 実装フェーズ

- [x] Phase 1: DANGLING_COMMENT_GROUP ノードの導入
- [x] Phase 2: DanglingCommentGroupOr<T> と AST メソッドの実装
- [x] Phase 3: パーサーの更新
- [ ] Phase 4: フォーマッターの更新
- [ ] Phase 5: ソート処理の更新
- [ ] Phase 6: テストとドキュメント

---

## Phase 1: DANGLING_COMMENT_GROUP ノードの導入 ✅ 完了

### 実装内容

#### Task 1.1: SyntaxKind に DANGLING_COMMENT_GROUP を追加 ✅
**ファイル**: `crates/tombi-syntax/src/generated/syntax_kind.rs`

#### Task 1.2: DanglingCommentGroup AstNode の実装 ✅
**ファイル**: `crates/tombi-ast/src/node/dangling_comment_group.rs`

```rust
pub struct DanglingCommentGroup {
    pub(crate) syntax: SyntaxNode,
}

impl DanglingCommentGroup {
    pub fn comments(&self) -> impl Iterator<Item = DanglingComment> { ... }
    pub fn into_comments(self) -> impl Iterator<Item = DanglingComment> { ... }
    pub fn range(&self) -> tombi_text::Range { ... }
}

impl AstNode for DanglingCommentGroup { ... }
```

#### Task 1.3: DanglingCommentGroup パーサーの実装 ✅
**ファイル**: `crates/tombi-parser/src/parse/dangling_comment_group.rs`

```rust
impl Parse for Vec<tombi_ast::DanglingCommentGroup> {
    fn parse(p: &mut Parser<'_>) {
        loop {
            while p.eat(LINE_BREAK) {}
            let Some(group_len) = dangling_comment_group_len(p) else { break; };
            let m = p.start();
            (0..group_len).for_each(|_| p.bump_any());
            m.complete(p, DANGLING_COMMENT_GROUP);
        }
    }
}
```

判定ロジック (`dangling_comment_group_len`):
- COMMENT で始まり、COMMENT + TS_LINE_END のペアが連続
- 次のトークンが TS_DANGLING_COMMENT_GROUP_END (`}`, `]`, LINE_BREAK, EOF) であればグループ終了

---

## Phase 2: DanglingCommentGroupOr<T> と AST メソッドの実装 ✅ 完了

### 実装内容

> **当初案との変更点**: KeyValueGroup / ValueGroup を Enum 化する代わりに、`DanglingCommentGroupOr<T>` ジェネリック Enum を導入。各グループ型は構造体のまま維持。

#### Task 2.1: DanglingCommentGroupOr<T> の実装 ✅
**ファイル**: `crates/tombi-ast/src/node/dangling_comment_group_or.rs`

```rust
pub enum DanglingCommentGroupOr<T> {
    DanglingCommentGroup(DanglingCommentGroup),
    ItemGroup(T),
}

impl<T: AstNode> AstNode for DanglingCommentGroupOr<T> { ... }
```

#### Task 2.2: AST ヘルパー関数の実装 ✅
**ファイル**: `crates/tombi-ast/src/support/comment.rs`

```rust
pub fn dangling_comment_groups<I>(iter: I) -> impl Iterator<Item = DanglingCommentGroup> { ... }
pub fn dangling_comment_group_or<T, I>(iter: I) -> impl Iterator<Item = DanglingCommentGroupOr<T>> { ... }
pub fn leading_comments<I>(iter: I) -> impl Iterator<Item = LeadingComment> { ... }
pub fn trailing_comment<I>(iter: I, end: SyntaxKind) -> Option<TrailingComment> { ... }
```

#### Task 2.3: ValueWithCommaGroup の実装 ✅
**ファイル**: `crates/tombi-ast/src/node/value_with_comma_group.rs`

```rust
pub struct ValueWithCommaGroup {
    pub(crate) syntax: SyntaxNode,
}

impl ValueWithCommaGroup {
    pub fn values(&self) -> impl Iterator<Item = Value> { ... }
    pub fn values_with_comma(&self) -> impl Iterator<Item = (Value, Option<Comma>)> { ... }
}
```

#### Task 2.4: 各コンテナの AST メソッド追加 ✅

**Root** (`crates/tombi-ast/src/impls/root.rs`):
- `dangling_comment_groups()` → `impl Iterator<Item = DanglingCommentGroup>`
- `key_value_groups()` → `impl Iterator<Item = DanglingCommentGroupOr<KeyValueGroup>>`
- `comment_directives()` → `impl Iterator<Item = TombiValueCommentDirective>`
- `schema_document_comment_directive()`, `tombi_document_comment_directives()`

**Table** (`crates/tombi-ast/src/impls/table.rs`):
- `dangling_comment_groups()` → `impl Iterator<Item = DanglingCommentGroup>`
- `key_value_groups()` → `impl Iterator<Item = DanglingCommentGroupOr<KeyValueGroup>>`
- `comment_directives()` → `impl Iterator<Item = TombiValueCommentDirective>`
- `header_leading_comments()`, `header_trailing_comment()`

**ArrayOfTable** (`crates/tombi-ast/src/impls/array_of_table.rs`):
- `dangling_comment_groups()` → `impl Iterator<Item = DanglingCommentGroup>`
- `key_value_groups()` → `impl Iterator<Item = DanglingCommentGroupOr<KeyValueGroup>>`
- `comment_directives()` → `impl Iterator<Item = TombiValueCommentDirective>`
- `header_leading_comments()`, `header_trailing_comment()`

**Array** (`crates/tombi-ast/src/impls/array.rs`):
- `dangling_comment_groups()` → `impl Iterator<Item = DanglingCommentGroup>`
- `value_with_comma_groups()` → `impl Iterator<Item = DanglingCommentGroupOr<ValueWithCommaGroup>>`
- `comment_directives()` → `impl Iterator<Item = TombiValueCommentDirective>`
- `bracket_start_trailing_comment()`

**InlineTable** (`crates/tombi-ast/src/impls/inline_table.rs`):
- `dangling_comment_groups()` → `impl Iterator<Item = DanglingCommentGroup>`
- `key_value_with_comma_groups()` → `impl Iterator<Item = DanglingCommentGroupOr<KeyValueWithCommaGroup>>`
- `comment_directives()` → `impl Iterator<Item = TombiValueCommentDirective>`
- `brace_start_trailing_comment()`

---

## Phase 3: パーサーの更新 ✅ 完了

### 実装内容

全コンテナのパーサーで `Vec::<DanglingCommentGroup>::parse(p)` をパースループに組み込み済み。

#### Task 3.1: Root パーサー ✅
**ファイル**: `crates/tombi-parser/src/parse/root.rs`

```rust
// key-values ループ
loop {
    while p.eat(LINE_BREAK) {}
    Vec::<tombi_ast::DanglingCommentGroup>::parse(p);
    let n = peek_leading_comments(p);
    if p.nth_at_ts(n, TS_NEXT_SECTION) { break; }
    tombi_ast::KeyValueGroup::parse(p);
    ...
}
```

#### Task 3.2: Table パーサー ✅
**ファイル**: `crates/tombi-parser/src/parse/table.rs`

Root と同じパターン。ヘッダパース後に `Vec::<DanglingCommentGroup>::parse(p)` → `KeyValueGroup::parse(p)` のループ。

#### Task 3.3: ArrayOfTable パーサー ✅
**ファイル**: `crates/tombi-parser/src/parse/array_of_table.rs`

Table と同じパターンだが、`KeyValueGroup::parse` ではなく個別の `KeyValue::parse` を使用。

#### Task 3.4: Array パーサー ✅
**ファイル**: `crates/tombi-parser/src/parse/array.rs`

`Vec::<DanglingCommentGroup>::parse(p)` → `ValueWithCommaGroup::parse(p)` のループ。

#### Task 3.5: InlineTable パーサー ✅
**ファイル**: `crates/tombi-parser/src/parse/inline_table.rs`

`Vec::<DanglingCommentGroup>::parse(p)` → `KeyValueWithCommaGroup::parse(p)` のループ。

#### Task 3.6: グループセパレータ検出 ✅
**ファイル**: `crates/tombi-parser/src/parse.rs`

```rust
fn is_group_separator(p: &mut Parser<'_>) -> bool {
    p.at_ts(TS_LINE_END) && p.nth_at_ts(1, TS_LINE_END)
}
```

#### Task 3.7: KeyValueGroup パーサー ✅
**ファイル**: `crates/tombi-parser/src/parse/key_value_group.rs`

```rust
impl Parse for tombi_ast::KeyValueGroup {
    fn parse(p: &mut Parser<'_>) {
        let m = p.start();
        loop {
            if is_group_separator(p) { break; }
            tombi_ast::KeyValue::parse(p);
            if !p.at(LINE_BREAK) { break; }
            let n = peek_leading_comments(p);
            if !p.nth_at_ts(n, TS_KEY_FIRST) { break; }
        }
        m.complete(p, KEY_VALUE_GROUP);
    }
}
```

#### Task 3.8: ValueWithCommaGroup パーサー ✅
**ファイル**: `crates/tombi-parser/src/parse/value_with_comma_group.rs`

KeyValueGroup と同じパターンで、`Value::parse` + `Comma::parse` を使用。

---

## Phase 4: フォーマッターの更新（未完了）

### 目的
フォーマッターを更新し、`DanglingCommentGroupOr<T>` の各バリアントに応じて適切に出力する。

### タスク

#### Task 4.1: テーブルフォーマッターの更新

**ファイル**: `crates/tombi-formatter/src/format/table.rs`

```rust
for group in table.key_value_groups() {
    match group {
        DanglingCommentGroupOr::ItemGroup(key_value_group) => {
            // グループ間の空行出力
            // KeyValueGroup 内のソート処理
            // key-values のフォーマット
        }
        DanglingCommentGroupOr::DanglingCommentGroup(dangling) => {
            // 直前の LINE_BREAK 数に応じて空行出力
            // dangling comments のフォーマット
        }
    }
}
```

#### Task 4.2: Root フォーマッターの更新
同じ `DanglingCommentGroupOr<KeyValueGroup>` パターン。

#### Task 4.3: Array フォーマッターの更新
`DanglingCommentGroupOr<ValueWithCommaGroup>` パターン。

#### Task 4.4: InlineTable フォーマッターの更新
`DanglingCommentGroupOr<KeyValueWithCommaGroup>` パターン。

#### Task 4.5: ArrayOfTable フォーマッターの更新
`DanglingCommentGroupOr<KeyValueGroup>` パターン。

---

## Phase 5: ソート処理の更新（未完了）

### 目的
ソート処理を `DanglingCommentGroupOr<T>` に対応させ、グループ単位でソートを実行する。

### タスク

#### Task 5.1: グループごとのソート処理

```rust
for group in table.key_value_groups() {
    match group {
        DanglingCommentGroupOr::ItemGroup(key_value_group) => {
            let directives = /* collect directives from group and table */;
            sort_key_values(key_value_group.key_values(), &directives);
        }
        DanglingCommentGroupOr::DanglingCommentGroup(_) => {
            // ソート不要、ディレクティブ記録のみ
        }
    }
}
```

---

## Phase 6: テストとドキュメント（未完了）

### タスク

#### Task 6.1: パーサーテスト ✅ 完了
以下のテストが追加済み:
- Root: `parses_root_dangling_comment`, `parses_root_dangling_comment_groups`, `parses_root_key_value_group_and_dangling_comment_groups`
- Table: `parses_table_dangling_comment`, `parses_table_dangling_comment_group`, `parses_table_dangling_comment_groups`, `parses_table_key_value_group_and_dangling_comment_groups`, `keeps_key_value_leading_comments_as_non_dangling`
- ArrayOfTable: 同様のテストセット
- InlineTable: `inline_table_dangling_comment`, `inline_table_dangling_comment_groups`, `inline_table_key_value_with_comma_group_and_dangling_comment_groups`

#### Task 6.2: フォーマッターテスト（未完了）
- comprehensive_full_example テストの修正
- 空行正規化のテスト
- ソート + グループ境界のテスト

#### Task 6.3: エッジケーステスト（未完了）
- 空のグループのみのファイル
- 連続する空行の正規化
- 複雑なコメントディレクティブの組み合わせ

---

## クリーンアップ（要対応）

以下の未使用コードを削除する必要がある:

- `crates/tombi-parser/src/support/comment.rs`:
  - `begin_dangling_comments()` — 未使用
  - `end_dangling_comments()` — 未使用
  - `dangling_comments()` — 未使用
- `crates/tombi-parser/src/token_set.rs`:
  - `TS_DANGLING_COMMENTS_KINDS` — 未使用

これらは旧アプローチの名残であり、新しい `Vec::<DanglingCommentGroup>::parse(p)` と `DANGLING_COMMENT_GROUP` ノードベースのアプローチに置き換えられた。

---

## 成功基準

- [ ] comprehensive_full_example テストが通る
- [ ] 既存の全テストが通る
- [ ] コメントが消える問題が解決される
- [ ] フォーマット結果が idempotent である
- [ ] ソートがグループ単位で正しく適用される
- [ ] 未使用コード（旧 begin_dangling_comments / end_dangling_comments）が削除される
