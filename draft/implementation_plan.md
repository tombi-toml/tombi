# 新しいコメント解釈ロジック実装計画（DANGLING_COMMENTS ノードアプローチ）

## 概要

`draft/new_comment_treatment.md` の仕様に基づき、パーサーレベルで `DANGLING_COMMENTS` ノードを導入することで、空行で区切られたコメントグループを明示的にASTに保存します。これにより、情報の損失を防ぎ、フォーマッターとエディターの実装を大幅に簡略化します。

## 設計方針の転換

### 従来のアプローチの問題点
- パーサーがコメントをトークンとして認識するだけ
- AST層で後からコメントをグループ化しようとするが、情報が失われる
- グループ境界の判定が複雑で、コメントが消える問題が発生

### 新しいアプローチ
- **パーサーレベルで `DANGLING_COMMENTS` ノードを作成**
  - 空行で区切られたコメントグループを明示的にパース
  - グループ境界の判定はパーサーが一度だけ行う
- **`DANGLING_COMMENTS` を `AstNode` として実装**
  - `syntax()` メソッドで子要素にアクセス可能
  - `comment_groups()` メソッドでグループ化されたコメントを取得
- **KeyValueGroup / ValueGroup を Enum として実装**
  - `Elements(Vec<T>)`: 要素のグループ
  - `DanglingComments(DanglingComments)`: コメントだけのグループ

### 利点
1. ✅ 情報の損失を防ぐ（パーサーで一度判定した情報をASTに保存）
2. ✅ 責務の明確化（Parser: 判定、AST: 保持、Formatter: 出力）
3. ✅ 実装の簡略化（フォーマッターは単純にEnumをパターンマッチ）
4. ✅ コメントが消える問題の解決

---

## 実装フェーズ

- [ ] Phase 1: DANGLING_COMMENTS ノードの導入
- [ ] Phase 2: KeyValueGroup / ValueGroup の Enum 化
- [ ] Phase 3: パーサーの更新
- [ ] Phase 4: フォーマッターの更新
- [ ] Phase 5: ソート処理の更新
- [ ] Phase 6: テストとドキュメント

---

## Phase 1: DANGLING_COMMENTS ノードの導入

### 目的
パーサーレベルで空行で区切られたコメントグループを `DANGLING_COMMENTS` ノードとして明示的に認識し、ASTに保存する。

### タスク

#### Task 1.1: SyntaxKind に DANGLING_COMMENTS を追加

**ファイル**: `crates/tombi-syntax/src/syntax_kind.rs`

```rust
pub enum SyntaxKind {
    // ... existing kinds
    DANGLING_COMMENTS,  // 空行で区切られたコメントグループ
}
```

#### Task 1.2: DanglingComments AstNode の実装

**ファイル**: `crates/tombi-ast/src/impls/dangling_comments.rs` (新規)

```rust
use crate::{AstNode, Comment};
use tombi_syntax::{SyntaxKind::*, SyntaxNode};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DanglingComments {
    syntax: SyntaxNode,
}

impl AstNode for DanglingComments {
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if syntax.kind() == DANGLING_COMMENTS {
            Some(Self { syntax })
        } else {
            None
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

impl DanglingComments {
    /// Returns comment groups separated by empty lines
    ///
    /// Each inner Vec represents a group of consecutive comments
    /// separated by single line breaks. Groups are separated by
    /// empty lines (2+ consecutive LINE_BREAKs).
    pub fn comment_groups(&self) -> Vec<Vec<Comment>> {
        crate::support::node::group_comments(
            self.syntax()
                .children_with_tokens()
                .filter(|el| matches!(el.kind(), COMMENT | LINE_BREAK | WHITESPACE))
        )
    }

    /// Returns all comments (flattened)
    pub fn comments(&self) -> impl Iterator<Item = Comment> {
        crate::support::node::children(self.syntax())
    }
}
```

**更新**: `crates/tombi-ast/src/impls/mod.rs` に `mod dangling_comments;` を追加
**更新**: `crates/tombi-ast/src/lib.rs` に `pub use impls::dangling_comments::DanglingComments;` を追加

#### Task 1.3: DanglingComments パーサーの実装

**ファイル**: `crates/tombi-parser/src/parse/dangling_comments.rs` (新規)

```rust
use tombi_syntax::{SyntaxKind::*, T};
use crate::{
    parse::Parse,
    parser::Parser,
};

impl Parse for tombi_ast::DanglingComments {
    fn parse(p: &mut Parser<'_>) {
        let m = p.start();

        // Consume comments and line breaks until we hit a group separator
        // or a non-comment/line-break token
        loop {
            if p.at(COMMENT) {
                p.bump(COMMENT);
            } else if p.at(LINE_BREAK) {
                // Check if this is a group separator (2+ consecutive LINE_BREAKs)
                let mut lookahead = 1;
                while p.nth_at(lookahead, WHITESPACE) {
                    lookahead += 1;
                }

                if p.nth_at(lookahead, LINE_BREAK) {
                    // This is a group separator, stop here
                    p.bump(LINE_BREAK);  // Consume the first LINE_BREAK
                    break;
                }

                p.bump(LINE_BREAK);
            } else if p.at(WHITESPACE) {
                p.bump(WHITESPACE);
            } else {
                break;
            }
        }

        m.complete(p, DANGLING_COMMENTS);
    }
}

/// Check if we should parse a DANGLING_COMMENTS node
/// Returns true if current position has comments/line breaks that form
/// a dangling comments group (separated by empty line from next element)
pub(crate) fn should_parse_dangling_comments(p: &Parser<'_>) -> bool {
    if !p.at(COMMENT) && !p.at(LINE_BREAK) {
        return false;
    }

    // Look ahead to see if this forms a complete dangling group
    let mut n = 0;
    let mut has_comment = false;

    loop {
        if p.nth_at(n, COMMENT) {
            has_comment = true;
            n += 1;
        } else if p.nth_at(n, LINE_BREAK) {
            n += 1;
            // Check for group separator
            let mut m = n;
            while p.nth_at(m, WHITESPACE) {
                m += 1;
            }
            if p.nth_at(m, LINE_BREAK) {
                // Found group separator
                return has_comment;
            }
        } else if p.nth_at(n, WHITESPACE) {
            n += 1;
        } else {
            break;
        }
    }

    false
}
```

**更新**: `crates/tombi-parser/src/parse/mod.rs` に `pub mod dangling_comments;` を追加
**更新**: `crates/tombi-parser/src/parse.rs` に `pub use dangling_comments::should_parse_dangling_comments;` を追加

---

## Phase 2: KeyValueGroup / ValueGroup の Enum 化

### 目的
現在の構造体ベースの `KeyValueGroup` と `ValueGroup` を Enum に変更し、要素グループとコメントグループを明確に区別する。

### タスク

#### Task 2.1: KeyValueGroup を Enum に変更

**ファイル**: `crates/tombi-ast/src/node/key_value_group.rs`

```rust
use crate::{DanglingComments, KeyValue};

#[derive(Debug, Clone)]
pub enum KeyValueGroup {
    /// Key-values with their dangling comments
    KeyValues {
        key_values: Vec<KeyValue>,
        begin_dangling_comments: Vec<Vec<crate::BeginDanglingComment>>,
        end_dangling_comments: Vec<Vec<crate::EndDanglingComment>>,
    },
    /// Dangling comments without any key-values (empty element group)
    DanglingComments(DanglingComments),
}

impl KeyValueGroup {
    pub fn new(key_values: Vec<KeyValue>) -> Self {
        Self::KeyValues {
            key_values,
            begin_dangling_comments: Vec::new(),
            end_dangling_comments: Vec::new(),
        }
    }

    pub fn with_dangling_comments(
        key_values: Vec<KeyValue>,
        begin_dangling_comments: Vec<Vec<crate::BeginDanglingComment>>,
        end_dangling_comments: Vec<Vec<crate::EndDanglingComment>>,
    ) -> Self {
        Self::KeyValues {
            key_values,
            begin_dangling_comments,
            end_dangling_comments,
        }
    }

    pub fn from_dangling_comments(comments: DanglingComments) -> Self {
        Self::DanglingComments(comments)
    }

    /// Returns true if this is an empty element group (no key-values)
    pub fn is_empty(&self) -> bool {
        match self {
            Self::KeyValues { key_values, .. } => key_values.is_empty(),
            Self::DanglingComments(_) => true,
        }
    }

    /// Returns true if this group has key-values
    pub fn has_elements(&self) -> bool {
        !self.is_empty()
    }

    /// Returns key-values if this is a KeyValues variant
    pub fn key_values(&self) -> &[KeyValue] {
        match self {
            Self::KeyValues { key_values, .. } => key_values,
            Self::DanglingComments(_) => &[],
        }
    }

    /// Returns begin dangling comments if this is a KeyValues variant
    pub fn begin_dangling_comments(&self) -> Vec<Vec<crate::BeginDanglingComment>> {
        match self {
            Self::KeyValues { begin_dangling_comments, .. } => begin_dangling_comments.clone(),
            Self::DanglingComments(_) => Vec::new(),
        }
    }

    /// Returns end dangling comments if this is a KeyValues variant
    pub fn end_dangling_comments(&self) -> Vec<Vec<crate::EndDanglingComment>> {
        match self {
            Self::KeyValues { end_dangling_comments, .. } => end_dangling_comments.clone(),
            Self::DanglingComments(_) => Vec::new(),
        }
    }
}
```

#### Task 2.2: ValueGroup を Enum に変更

**ファイル**: `crates/tombi-ast/src/node/value_group.rs`

同様のパターンで `ValueGroup` を Enum に変更する。

---

## Phase 3: パーサーの更新

### 目的
テーブル、配列、array of table のパーサーを更新し、`DANGLING_COMMENTS` ノードを適切にパースする。

### タスク

#### Task 3.1: テーブルパーサーの更新

**ファイル**: `crates/tombi-parser/src/parse/table.rs`

```rust
impl Parse for tombi_ast::Table {
    fn parse(p: &mut Parser<'_>) {
        let m = p.start();

        // ... header parsing ...

        begin_dangling_comments(p);

        loop {
            while p.eat(LINE_BREAK) {}

            // Check if we have a dangling comments group
            if should_parse_dangling_comments(p) {
                tombi_ast::DanglingComments::parse(p);
                continue;
            }

            let n = peek_leading_comments(p);
            if p.nth_at_ts(n, TS_NEXT_SECTION) {
                break;
            }

            tombi_ast::KeyValue::parse(p);

            if !p.at_ts(TS_LINE_END) {
                invalid_line(p, ExpectedLineBreak);
            }
        }

        end_dangling_comments(p, false);

        while p.eat(LINE_BREAK) {}

        m.complete(p, TABLE);
    }
}
```

#### Task 3.2: 配列パーサーの更新

**ファイル**: `crates/tombi-parser/src/parse/array.rs`

同様のパターンで配列パーサーを更新する。

#### Task 3.3: Array of table パーサーの更新

**ファイル**: `crates/tombi-parser/src/parse/array_of_table.rs`

同様のパターンで array of table パーサーを更新する。

---

## Phase 4: フォーマッターの更新

### 目的
フォーマッターを更新し、Enum の各バリアントに応じて適切に出力する。

### タスク

#### Task 4.1: テーブルフォーマッターの更新

**ファイル**: `crates/tombi-formatter/src/format/table.rs`

```rust
for (group_idx, group) in key_value_groups.iter().enumerate() {
    // Insert empty line between groups
    if group_idx > 0 {
        write!(f, "{}", f.line_ending())?;
        write!(f, "{}", f.line_ending())?;
    }

    match group {
        KeyValueGroup::KeyValues { key_values, begin_dangling_comments, end_dangling_comments } => {
            // Format begin dangling comments
            for comment_group in begin_dangling_comments {
                for comment in comment_group {
                    f.write_indent()?;
                    comment.format(f)?;
                    write!(f, "{}", f.line_ending())?;
                }
            }

            // Format key-values
            for (i, key_value) in key_values.iter().enumerate() {
                if i > 0 {
                    write!(f, "{}", f.line_ending())?;
                }
                key_value.format(f)?;
            }

            // Format end dangling comments
            for comment_group in end_dangling_comments {
                for comment in comment_group {
                    write!(f, "{}", f.line_ending())?;
                    f.write_indent()?;
                    comment.format(f)?;
                }
            }
        }
        KeyValueGroup::DanglingComments(dangling) => {
            // Format dangling comments
            for comment_group in dangling.comment_groups() {
                for comment in comment_group {
                    f.write_indent()?;
                    comment.format(f)?;
                    write!(f, "{}", f.line_ending())?;
                }
            }
        }
    }
}
```

#### Task 4.2: 配列フォーマッターの更新

**ファイル**: `crates/tombi-formatter/src/format/value/array.rs`

同様のパターンで配列フォーマッターを更新する。

#### Task 4.3: Array of table フォーマッターの更新

**ファイル**: `crates/tombi-formatter/src/format/array_of_table.rs`

同様のパターンで array of table フォーマッターを更新する。

---

## Phase 5: ソート処理の更新

### 目的
ソート処理を Enum に対応させ、DanglingComments グループからディレクティブを抽出する。

### タスク

#### Task 5.1: グループごとのソート処理

**ファイル**: `crates/tombi-ast-editor/src/edit/table.rs`

```rust
for group in table.key_value_groups() {
    match group {
        KeyValueGroup::KeyValues { key_values, .. } => {
            // グループのディレクティブを取得
            let group_directives = group.comment_directives();
            let merged_directives = merge_directives(header_directives, group_directives);

            // ソート処理
            let sorted = sort_key_values(key_values, &merged_directives)?;
            changes.extend(sorted);
        }
        KeyValueGroup::DanglingComments(dangling) => {
            // コメントのみのグループはソート不要
            // ただしディレクティブがあれば記録（次のグループに影響する可能性）
        }
    }
}
```

---

## Phase 6: テストとドキュメント

### 目的
総合的なテストを行い、ドキュメントを更新する。

### タスク

#### Task 6.1: comprehensive_full_example テストの修正

**ファイル**: `crates/tombi-formatter/tests/test_group_based_sorting.rs`

既存のテストを実行し、全て通ることを確認する。

#### Task 6.2: エッジケースのテスト追加

- 空のグループのみのファイル
- 連続する空行の正規化
- 複雑なコメントディレクティブの組み合わせ

#### Task 6.3: ドキュメントの更新

- 実装の概要と設計方針をドキュメント化
- ユーザー向けドキュメントの更新（必要に応じて）

---

## 実装の優先順位

1. **Phase 1**: DANGLING_COMMENTS ノードの基礎実装（最優先）
2. **Phase 2**: Enum 化（Phase 1 完了後すぐ）
3. **Phase 3**: パーサー更新（一つずつ、テーブルから開始）
4. **Phase 4**: フォーマッター更新（パーサーと並行可能）
5. **Phase 5**: ソート処理（フォーマッター完了後）
6. **Phase 6**: テストとドキュメント（最後）

---

## 成功基準

- [ ] comprehensive_full_example テストが通る
- [ ] 既存の全テストが通る
- [ ] コメントが消える問題が解決される
- [ ] フォーマット結果が idempotent である
- [ ] ソートがグループ単位で正しく適用される

---

## 次のステップ

Phase 1 の Task 1.1 から実装を開始します。
