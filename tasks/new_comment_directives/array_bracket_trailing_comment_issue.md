# 配列の開きブラケット後のコメント処理問題

## 概要

配列の開きブラケット `[` の直後にtrailing commentがある場合の処理で、パースエラーが発生しています。この問題は、パーサーとフォーマッターの間でコメントの扱いが複雑に絡み合っており、解決に至っていません。

## 現在のエラー

### エラー内容
```
test comprehensive_example::comprehensive_full_example ... FAILED

Diagnostic {
  level: ERROR,
  code: "expected-value",
  message: "expected value",
  range: Range {
    start: Position { line: 56, column: 0 },
    end: Position { line: 57, column: 0 }
  }
}
```

### エラー箇所
テストファイル: `crates/tombi-formatter/tests/test_group_based_sorting.rs`
テストケース: `comprehensive_full_example`

入力TOMLの該当部分（行番号は0-indexed）:
```toml
# 行52: arr = [  # [19] array opening bracket trailing
# 行53: # [20] Array group 1 start dangling
# 行54: # tombi: format.rules.array-values-order.disabled = true
# 行55:
# 行56: "z",    ← エラーがこの行を指している
```

エラーは行56で発生していますが、実際の問題は行52-55のコメント処理にあります。

## 問題の根本原因

### 1. 開きブラケットのtrailing comment
配列の開きブラケット直後にtrailing commentがある場合:
```toml
arr = [  # bracket trailing
  1,
  2
]
```

このコメントを以下のどちらとして扱うべきか:
- **同じ行のtrailing comment**: `[ # comment` として同じ行に保持
- **次行のdangling comment**: 新しい行に移動

### 2. パーサーとフォーマッターの不整合

#### パーサー側
`crates/tombi-parser/src/parse/array.rs`の`Array::parse`で:
```rust
p.eat(T!['[']);
trailing_comment(p);  // ← これを追加するとパースは成功するが...

loop {
    while p.eat(LINE_BREAK) {}

    if should_parse_dangling_comments(p) {
        tombi_ast::DanglingComments::parse(p);
        continue;
    }
    // ...
}
```

- `trailing_comment(p)`を呼ぶと、WHITESPACEとCOMMENTトークンが消費される
- これらのトークンはArrayノードのシンタックスツリーに追加される
- しかし、`value_groups()`がこれらのトークンを再度検出し、dangling commentとして扱う

#### フォーマッター側
`crates/tombi-formatter/src/format/value/array.rs`の`format_multiline_array`で:
```rust
write!(f, "[")?;

// 開きブラケットのtrailing commentを書く
if let Some(trailing_comment) = array.bracket_start_trailing_comment() {
    write!(f, " {}", trailing_comment.text())?;
}

// value_groupsをフォーマット
// ← ここで同じコメントが再度フォーマットされてしまう
```

結果: コメントが重複して出力される（2-3回）

### 3. should_parse_dangling_commentsの判定

`crates/tombi-parser/src/parse/dangling_comments.rs`の`should_parse_dangling_comments`:
```rust
pub fn should_parse_dangling_comments(p: &Parser<'_>) -> bool {
    // 現在位置がCOMMENTで、かつ：
    // - 2つ以上の連続LINE_BREAK（空行）の後、または
    // - ファイルの先頭、または
    // - 単一LINE_BREAK + EOF
    // の場合にtrueを返す
}
```

開きブラケットのtrailing commentの後には通常1つのLINE_BREAKしかないため、次のコメントが`should_parse_dangling_comments`でfalseと判定される可能性があります。

## これまでの試行錯誤

### 試行1: パーサーで`trailing_comment(p)`を呼ぶ
```rust
p.eat(T!['[']);
trailing_comment(p);  // 追加
```

**結果**:
- パースは成功
- しかしフォーマット時にコメントが重複（2-3回出力）

### 試行2: フォーマッターでスキップロジックを追加
```rust
let start_group_idx = if array.bracket_start_trailing_comment().is_some() {
    if let Some(tombi_ast::ValueGroup::DanglingComments(_)) = value_groups.first() {
        1 // 最初のグループをスキップ
    } else {
        0
    }
} else {
    0
};

for (group_idx, group) in value_groups.iter().enumerate().skip(start_group_idx) {
    // ...
}
```

**結果**:
- コメントは依然として重複して出力される
- スキップロジックが正しく機能していない（条件が一致しない？）

### 試行3: trailing_commentでWHITESPACEをスキップ
`crates/tombi-parser/src/parse.rs`の`trailing_comment`を修正:
```rust
pub fn trailing_comment(p: &mut crate::parser::Parser<'_>) {
    // WHITESPACEをスキップしてからCOMMENTを探す
    while p.at(WHITESPACE) {
        p.bump_any();
    }
    while p.eat_ts(TS_TAILING_COMMENT_KINDS) {}
}
```

**結果**:
- コメントの消費は改善
- しかし重複問題は解決せず

### 試行4: 全ての変更を元に戻す
開きブラケットのtrailing comment対応を諦め、元の実装に戻しました。

**結果**:
- パースエラーが再発（現在の状態）

## 実施した変更

### 1. カンマのパース処理を簡素化
**ファイル**: `crates/tombi-parser/src/parse/array.rs`

**変更前**:
複雑な空行検出ロジックで、値の後のカンマを探していた

**変更後**:
```rust
// 値の後のカンマをチェック
let mut check_n = 0;
let mut found_comma = false;

// LINE_BREAK, WHITESPACE, COMMENTをスキップしてカンマを探す
while p.nth_at(check_n, LINE_BREAK) || p.nth_at(check_n, WHITESPACE) || p.nth_at(check_n, COMMENT) {
    check_n += 1;
}

if p.nth_at(check_n, T![,]) {
    // カンマが見つかったら、現在位置からカンマまでの全てのトークンをCommaノードに含める
    let comma_marker = p.start();

    while !p.at(T![,]) && !p.at(EOF) && !p.at(T![']']) {
        if p.at(LINE_BREAK) || p.at(WHITESPACE) || p.at(COMMENT) {
            p.bump_any();
        } else {
            break;
        }
    }

    p.eat(T![,]);
    trailing_comment(p);
    comma_marker.complete(p, T!(,));
    found_comma = true;
}
```

**理由**:
- 空行検出の再実装が複雑で、`peek_leading_comments`の結果と整合性が取れていなかった
- シンプルなスキップロジックに変更

### 2. trailing_commentの修正
**ファイル**: `crates/tombi-parser/src/parse.rs`

**変更**:
```rust
pub fn trailing_comment(p: &mut crate::parser::Parser<'_>) {
    // WHITESPACEをスキップ追加
    while p.at(WHITESPACE) {
        p.bump_any();
    }
    while p.eat_ts(TS_TAILING_COMMENT_KINDS) {}
}
```

**理由**:
- `trailing_comment(p)`がWHITESPACEを処理していなかったため、コメントを正しく消費できなかった

### 3. 配列ASTへのメソッド追加（使用せず）
**ファイル**: `crates/tombi-ast/src/impls/array.rs`

追加したメソッド（現在は使用されていない）:
```rust
pub fn bracket_start_trailing_comment(&self) -> Option<crate::TrailingComment> {
    support::node::trailing_comment(self.syntax().children_with_tokens(), T!('['))
}
```

### 4. フォーマッターの変更（元に戻した）
**ファイル**: `crates/tombi-formatter/src/format/value/array.rs`

一時的に追加したが、元に戻した:
- 開きブラケット後にtrailing commentを出力
- `AstToken`トレイトのインポート
- `start_group_idx`によるグループスキップロジック

## デバッグ情報

### テストファイル
`/tmp/test_array_bracket_comment.toml`:
```toml
arr = [  # bracket trailing
  1,
  2
]
```

このファイルで検証可能:
```bash
cargo run --bin tombi -- format /tmp/test_array_bracket_comment.toml
cat /tmp/test_array_bracket_comment.toml
```

### パース検証
`/tmp/test_parse_comprehensive`:
```bash
cd /tmp/test_parse_comprehensive
cargo run  # 現在は "expected-value" エラー
```

### 関連テスト
```bash
# グループベースのソートテスト
cargo test --test test_group_based_sorting

# ライブラリテスト
cargo test --lib  # 23個のlinterテストが失敗
```

## 推奨される次のステップ

### アプローチA: 開きブラケットのtrailing commentを別の行に移動
最も簡単な解決策は、開きブラケットのtrailing commentを常に次の行に移動することです。

1. パーサーでは特別な処理をしない
2. フォーマッターで開きブラケット後のコメントをdangling commentとして扱う
3. テストの期待値を更新:
   ```toml
   arr = [
     # bracket trailing
     1,
     2
   ]
   ```

**利点**:
- 実装が簡単
- コメントの重複問題が発生しない

**欠点**:
- 元のフォーマットが変わる

### アプローチB: value_groups()の実装を修正
`value_groups()`が開きブラケットのtrailing commentを除外するように修正します。

**ファイル**: `crates/tombi-ast/src/impls/array.rs`の`value_groups()`

**必要な変更**:
1. 開きブラケット直後のCOMMENTトークンを識別
2. それらを`ValueGroup`に含めないようにする
3. または、専用の`BracketTrailingComment`グループ型を追加

**利点**:
- フォーマットを元の通りに保てる
- コメントの重複問題が根本的に解決

**欠点**:
- AST層の変更が必要
- 複雑で影響範囲が大きい

### アプローチC: 新しいASTノード型を導入
開きブラケットのtrailing comment専用のASTノード型を導入します。

**必要な変更**:
1. `BracketTrailingComment`ノード型を定義
2. パーサーで明示的にこのノードを作成
3. フォーマッターで専用の処理を実装

**利点**:
- 意図が明確
- 型安全

**欠点**:
- 大規模な変更が必要
- AST構造の変更

### アプローチD: 段階的な解決
1. **まず**: カンマの処理と`trailing_comment`の修正を確実に動作させる
2. **次に**: 開きブラケットのtrailing commentは一旦次の行に移動（アプローチA）
3. **最後に**: 将来的にアプローチBまたはCで改善

## 関連ファイル

### パーサー
- `crates/tombi-parser/src/parse/array.rs` - 配列のパース処理
- `crates/tombi-parser/src/parse.rs` - `trailing_comment`と`peek_leading_comments`
- `crates/tombi-parser/src/parse/dangling_comments.rs` - `should_parse_dangling_comments`

### AST
- `crates/tombi-ast/src/impls/array.rs` - Arrayの実装、`value_groups()`
- `crates/tombi-ast/src/generated/ast_node.rs` - Array構造体定義
- `crates/tombi-ast/src/support/node.rs` - `trailing_comment`ヘルパー
- `crates/tombi-ast/src/support/group.rs` - グルーピングロジック

### フォーマッター
- `crates/tombi-formatter/src/format/value/array.rs` - 配列のフォーマット処理

### テスト
- `crates/tombi-formatter/tests/test_group_based_sorting.rs` - `comprehensive_full_example`テスト
- `/tmp/test_array_bracket_comment.toml` - 簡易テストファイル

## その他の注意事項

### 現在のテスト状況
- **ライブラリテスト**: 68 passed, 23 failed (linterテスト)
- **グループベースソートテスト**: 12 passed, 1 failed (comprehensive_full_example)

### 既知の問題
1. 開きブラケットのtrailing comment問題（本ドキュメントの主題）
2. linterテスト23個の失敗（詳細未調査）

### コミット状態
現在のブランチ: `add_new_comment_treatment_draft`

最新のコミット:
```
a3733313 docs: add draft for new comment treatment implementation readiness
```

## まとめ

開きブラケットのtrailing comment問題は、パーサー・AST・フォーマッターの3層にまたがる複雑な問題です。最も現実的な解決策は、**アプローチA（コメントを次の行に移動）** または **アプローチD（段階的な解決）** です。

完璧な解決にはAST構造の見直しが必要ですが、まずは動作する状態を作ることを優先すべきです。
