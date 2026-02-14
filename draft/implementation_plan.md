# 新しいコメント解釈ロジック実装計画

## 概要

`draft/new_comment_treatment.md` の仕様に基づき、空行をグループ境界として扱い、コメントとノードの紐付けを維持しながら自動ソートを行うロジックを実装します。

## 実装の目標

- [x] Phase 0: コードベース調査（完了）
- [ ] Phase 1: KeyValueGroup と ValueGroup 構造体の実装
- [ ] Phase 2: ソート処理の実装（グループ単位）
- [ ] Phase 3: フォーマッター層の実装
- [ ] Phase 4: コメントディレクティブの対応
- [ ] Phase 5: テストとドキュメント
- [ ] Phase 6: 既存ヘルパーメソッドの削除とマイグレーション

### 重要な設計方針

**Separator は論理的な概念**
- Grammar や AST の物理的な構造として追加しない
- 既存の `LINE_BREAK` の連続（2つ以上）を「グループ境界」として解釈するだけ
- AST からデータを取り出す際に、連続 LINE_BREAK を検出してグループ化する

**既存ロジックの活用**
- `crates/tombi-ast/src/support/node.rs:147-159` の `group_comments()` が既に空行でコメントをグループ化している
- このロジックを参考に、KeyValue や配列要素のグループ化を実装する

---

## Phase 1: KeyValueGroup 構造体の実装と AST メソッドの追加

### 目的
`KeyValueGroup` 構造体を作成し、AST レベルでグループを明示的に扱う。`Table::key_values()` の代わりに `Table::key_value_groups()` を追加し、グループ単位でダングリングコメントを管理できるようにする。

### 設計

#### KeyValueGroup 構造体
```rust
pub struct KeyValueGroup {
    // グループ内の KeyValue のリスト
    key_values: Vec<KeyValue>,
    // ダングリングコメントは rowan から動的に計算するため、フィールドとして持たない
}

impl KeyValueGroup {
    pub fn key_values(&self) -> &[KeyValue] { ... }

    // グループ先頭のダングリングコメント
    // rowan から動的に計算
    pub fn begin_dangling_comments(&self) -> impl Iterator<Item = DanglingComment> {
        // グループの最初の KeyValue の前のコメントを取得
    }

    // グループ末尾のダングリングコメント
    // rowan から動的に計算
    pub fn end_dangling_comments(&self) -> impl Iterator<Item = DanglingComment> {
        // グループの最後の KeyValue の後のコメントを取得
    }
}
```

#### AST メソッドの追加
- `Table::key_value_groups()` -> `impl Iterator<Item = KeyValueGroup>`
- `Root::key_value_groups()` -> `impl Iterator<Item = KeyValueGroup>`
- `Array::value_groups()` -> `impl Iterator<Item = ValueGroup>`
- `ArrayOfTable` も同様

### タスク

- [ ] **Task 1.1**: `KeyValueGroup` 構造体の定義
  - `crates/tombi-ast/src/node/key_value_group.rs` (新規) を作成
  - `crates/tombi-ast/src/node/table_or_array_of_table.rs` を参考に `can_cast` と `cast` で実装
  - `KeyValueGroup` 構造体を定義
  - `AstNode` trait を実装
  - `key_values()`, `begin_dangling_comments()`, `end_dangling_comments()` メソッドを実装

  ```rust
  // crates/tombi-ast/src/node/key_value_group.rs
  use crate::{AstNode, KeyValue, DanglingComment};
  use tombi_syntax::{SyntaxKind, SyntaxNode};

  #[derive(Debug, Clone)]
  pub struct KeyValueGroup {
      syntax: SyntaxNode,
  }

  impl KeyValueGroup {
      /// グループ内の KeyValue を取得
      pub fn key_values(&self) -> impl Iterator<Item = KeyValue> {
          // syntax から子ノードとして KeyValue を取得
          self.syntax
              .children()
              .filter_map(KeyValue::cast)
      }

      /// グループの最初の KeyValue の前のダングリングコメントグループを rowan から取得
      /// 空行で区切られた複数のコメントグループが存在する可能性がある
      pub fn begin_dangling_comments(&self) -> impl Iterator<Item = Vec<DanglingComment>> {
          // first_kv の前のコメントグループを rowan から取得
          // 既存の support::node::group_comments() ロジックを活用
          // 空行で区切られた複数のグループを返す
          std::iter::empty() // TODO: 実装
      }

      /// グループの最後の KeyValue の後のダングリングコメントを rowan から取得
      pub fn end_dangling_comments(&self) -> impl Iterator<Item = DanglingComment> {
          // last_kv の後のコメントを rowan から取得
          // 次のグループまたは次のセクションとの境界までのコメントを返す
          std::iter::empty() // TODO: 実装
      }
  }

  impl AstNode for KeyValueGroup {
      #[inline]
      fn can_cast(kind: SyntaxKind) -> bool {
          // KeyValueGroup が対応する SyntaxKind を判定
          // 実装時に決定
          todo!()
      }

      #[inline]
      fn cast(syntax: SyntaxNode) -> Option<Self> {
          if Self::can_cast(syntax.kind()) {
              Some(Self { syntax })
          } else {
              None
          }
      }

      #[inline]
      fn syntax(&self) -> &SyntaxNode {
          &self.syntax
      }
  }
  ```

- [ ] **Task 1.2**: グループ境界検出ユーティリティの実装
  - `crates/tombi-ast/src/support/group.rs` (新規) を作成
  - `has_separator_between()` 関数を実装
    - 入力: 2つの SyntaxNode
    - 出力: `bool` (間に2つ以上の連続 LINE_BREAK があるか)
    - ロジック: 既存の `group_comments()` を参考に、ノード間のトークンを走査

  ```rust
  /// 2つのノード間に空行（2つ以上の連続 LINE_BREAK）があるかを判定
  pub fn has_separator_between(prev: &SyntaxNode, next: &SyntaxNode) -> bool {
      // prevの終わりからnextの始まりまでのトークンを走査
      // LINE_BREAKをカウントし、2つ以上あればtrue
  }
  ```

- [ ] **Task 1.3**: `Table::key_value_groups()` の実装
  - `crates/tombi-ast/src/impls/table.rs` に追加
  - `key_values()` を基に、グループ境界で分割
  - 各グループのダングリングコメントを抽出

  ```rust
  impl Table {
      pub fn key_value_groups(&self) -> impl Iterator<Item = KeyValueGroup> {
          // 1. key_values() を取得
          // 2. has_separator_between() でグループ境界を検出
          // 3. 各グループの begin/end dangling comments を抽出
          // 4. KeyValueGroup を構築して返す
      }
  }
  ```

- [ ] **Task 1.4**: `Root::key_value_groups()` の実装
  - `crates/tombi-ast/src/impls/root.rs` に追加
  - `Table::key_value_groups()` と同様のロジック

- [ ] **Task 1.5**: `ValueGroup` 構造体の定義と `Array::value_groups()` の実装
  - `crates/tombi-ast/src/node/value_group.rs` (新規) を作成
  - `KeyValueGroup` と同様に `can_cast` と `cast` で実装
  - `Array::value_groups()` を実装

  ```rust
  // crates/tombi-ast/src/node/value_group.rs
  use crate::{AstNode, Value, DanglingComment};
  use tombi_syntax::{SyntaxKind, SyntaxNode};

  #[derive(Debug, Clone)]
  pub struct ValueGroup {
      syntax: SyntaxNode,
  }

  impl ValueGroup {
      /// グループ内の Value を取得
      pub fn values(&self) -> impl Iterator<Item = Value> {
          // syntax から子ノードとして Value を取得
          self.syntax
              .children()
              .filter_map(Value::cast)
      }

      /// グループの最初の Value の前のダングリングコメントグループを rowan から取得
      pub fn begin_dangling_comments(&self) -> impl Iterator<Item = Vec<DanglingComment>> {
          std::iter::empty() // TODO: 実装
      }

      /// グループの最後の Value の後のダングリングコメントグループを rowan から取得
      pub fn end_dangling_comments(&self) -> impl Iterator<Item = DanglingComment> {
          std::iter::empty() // TODO: 実装
      }
  }

  impl AstNode for ValueGroup {
      #[inline]
      fn can_cast(kind: SyntaxKind) -> bool {
          // ValueGroup が対応する SyntaxKind を判定
          todo!()
      }

      #[inline]
      fn cast(syntax: SyntaxNode) -> Option<Self> {
          if Self::can_cast(syntax.kind()) {
              Some(Self { syntax })
          } else {
              None
          }
      }

      #[inline]
      fn syntax(&self) -> &SyntaxNode {
          &self.syntax
      }
  }
  ```

- [ ] **Task 1.6**: 既存ロジックとの整合性確認
  - `crates/tombi-ast/src/support/node.rs:147-159` の `group_comments()` を確認
  - 空行判定のロジックが一致していることを確認
  - 必要に応じて共通化

### 期待される変更ファイル
- `crates/tombi-ast/src/node/key_value_group.rs` (新規)
- `crates/tombi-ast/src/node/value_group.rs` (新規)
- `crates/tombi-ast/src/node/mod.rs` (module 追加)
- `crates/tombi-ast/src/support/group.rs` (新規)
- `crates/tombi-ast/src/support/mod.rs` (module 追加)
- `crates/tombi-ast/src/impls/table.rs`
- `crates/tombi-ast/src/impls/root.rs`
- `crates/tombi-ast/src/impls/array.rs`
- `crates/tombi-ast/src/lib.rs` (exports 追加)

### レビューポイント
- `KeyValueGroup` の設計が仕様を満たしているか
- グループ境界判定が既存のコメントグループ化と一致しているか
- ダングリングコメントの抽出が正しく動作するか
- Iterator の実装が効率的か

---

## Phase 2: ソート処理の実装 (グループ単位ソート)

### 目的
ソート処理を「グループ単位」に変更し、`KeyValueGroup` を使ってグループ内だけソートする。

### 前提条件
- Phase 1 で `KeyValueGroup` 構造体と `key_value_groups()` メソッドが実装済み

### タスク

- [ ] **Task 2.1**: `crates/tombi-ast-editor/src/rule/table_keys_order.rs:27-86` を修正
  - 現在: `table.key_values()` で全キーを一括ソート
  - 変更後: `table.key_value_groups()` でグループごとにソート

  ```rust
  // 疑似コード
  pub fn table_keys_order(...) -> Result<Vec<Change>> {
      let table = ...;

      // Step 1: グループを取得
      let groups = table.key_value_groups();

      let mut changes = Vec::new();

      // Step 2: 各グループ内でソート
      for group in groups {
          let key_values = group.key_values();

          // グループ内のキーをソート
          let sorted_group = get_sorted_accessors(
              value,
              &[],
              key_values.to_vec(),  // グループ内のキーのみ
              ...
          );

          // 変更を収集
          changes.extend(calculate_changes(key_values, sorted_group));
      }

      Ok(changes)
  }
  ```

- [ ] **Task 2.2**: ルートレベルのソート処理を更新
  - `crates/tombi-ast-editor/src/rule/root_table_keys_order.rs`
  - `root.key_value_groups()` を使用してグループ単位でソート

- [ ] **Task 2.3**: 配列要素のソート処理を更新
  - `crates/tombi-ast-editor/src/rule/array_values_order.rs`
  - `array.value_groups()` を使用してグループ単位でソート

- [ ] **Task 2.4**: テストの追加
  - グループ単位ソートのテストケース
  - グループ境界を跨いだソートが行われないことを確認
  - グループの begin/end dangling comments が保持されることを確認
  - 例:
    ```toml
    # 入力
    z = 3
    a = 1

    c = 2
    b = 1

    # 出力（グループごとにソート）
    a = 1
    z = 3

    b = 1
    c = 2
    ```

### 期待される変更ファイル
- `crates/tombi-ast-editor/src/rule/table_keys_order.rs`
- `crates/tombi-ast-editor/src/rule/root_table_keys_order.rs`
- `crates/tombi-ast-editor/src/rule/array_values_order.rs`
- テストファイル (複数)

### レビューポイント
- `key_value_groups()` を正しく使用しているか
- グループ単位でソートが行われるか
- グループ間の順序が維持されるか
- グループのダングリングコメントが保持されるか
- 既存のソートテストが通るか（または適切に更新されているか）

---

## Phase 3: フォーマッター層の実装 (グループ単位の出力)

### 目的
フォーマッター出力時に `key_value_groups()` を使用し、グループ単位でフォーマットする。グループ間に空行を挿入（最大1行に正規化）。

### 前提条件
- Phase 1 で `KeyValueGroup` 構造体と `key_value_groups()` メソッドが実装済み

### タスク

- [ ] **Task 3.1**: `crates/tombi-formatter/src/format/root.rs` を修正
  - `root.key_values()` の代わりに `root.key_value_groups()` を使用
  - グループ単位でフォーマット
  - グループ間に空行を挿入

  ```rust
  // 疑似コード
  pub fn format_root(root: &Root, f: &mut Formatter) -> Result<()> {
      let groups = root.key_value_groups();

      for (group_idx, group) in groups.enumerate() {
          // グループ間の空行
          if group_idx != 0 {
              write!(f, "{}", f.line_ending())?;
          }

          // グループ先頭のダングリングコメントグループ（複数グループの可能性）
          for comment_group in group.begin_dangling_comments() {
              format_comment_group(&comment_group, f)?;
          }

          // グループ内のキーバリュー（空の場合もある）
          // key_values が空の場合は、dangling comments のみが出力される
          let mut has_key_value = false;
          for key_value in group.key_values() {
              has_key_value = true;
              write!(f, "{}", f.line_ending())?;
              format_key_value(key_value, f)?;
          }

          if has_key_value {
              // グループ末尾のダングリングコメント
              for comment in group.end_dangling_comments() {
                  format_comment(&comment, f)?;
              }
          }
      }

      Ok(())
  }
  ```

- [ ] **Task 3.2**: `crates/tombi-formatter/src/format/table.rs` を修正
  - `table.key_value_groups()` を使用してグループ単位でフォーマット

- [ ] **Task 3.3**: `crates/tombi-formatter/src/format/array.rs` を修正
  - `array.value_groups()` を使用してグループ単位でフォーマット

- [ ] **Task 3.4**: テストの追加
  - 空行が保持されることを確認
  - 複数の連続空行が1行に正規化されることを確認
  - グループのダングリングコメントが正しく出力されることを確認
  - **空の要素グループ**（key_values が空のグループ）のダングリングコメントが正しく出力されることを確認
  - idempotent（再実行で変わらない）ことを確認
  - 例:
    ```toml
    # 入力（3つの空行）
    a = 1



    b = 2

    # 出力（1つの空行に正規化）
    a = 1

    b = 2
    ```
  - 空の要素グループの例:
    ```toml
    # 入力（空の要素グループに対するコメント）
    # [1] ファイル先頭のコメント

    # [2] 次のグループに紐づく

    a = 1

    # 出力（コメントが保持される）
    # [1] ファイル先頭のコメント

    # [2] 次のグループに紐づく

    a = 1
    ```

### 期待される変更ファイル
- `crates/tombi-formatter/src/format/root.rs`
- `crates/tombi-formatter/src/format/table.rs`
- `crates/tombi-formatter/src/format/array.rs`
- テストファイル (複数)

### レビューポイント
- `key_value_groups()` を正しく使用しているか
- 空行が正しく出力されるか（最大1行に正規化）
- グループのダングリングコメントが正しく出力されるか
- 既存のフォーマッターテストが通るか
- フォーマット結果が idempotent か

---

## Phase 4: コメントディレクティブの対応

### 目的
コメントディレクティブの作用範囲を「グループ単位」に変更する。`KeyValueGroup` の `begin_dangling_comments()` と `end_dangling_comments()` からディレクティブを抽出し、グループ全体に適用する。

### 前提条件
- Phase 1 で `KeyValueGroup` 構造体が実装済み
- Phase 2 でグループ単位ソートが実装済み

### タスク

- [ ] **Task 4.1**: グループ単位でのディレクティブ抽出
  - `KeyValueGroup` からディレクティブを抽出する関数を実装
  - **重要**: `begin_dangling_comments()` と `end_dangling_comments()` を `chain()` で繋げてから解析
  - これにより、同じグループ内で競合するディレクティブを検出できる

  ```rust
  // 疑似コード
  fn extract_directive_from_group(group: &KeyValueGroup) -> Option<Directive> {
      // begin と end のコメントを繋げてから解析
      // これにより、同じグループ内で競合するディレクティブを検出できる
      let all_comments: Vec<_> = group.begin_dangling_comments()
          .chain(group.end_dangling_comments())
          .collect();

      // 既存の parse_directive がグループ内の競合を検出する
      if let Some(directive) = parse_directive(&all_comments) {
          return Some(directive);
      }

      None
  }
  ```

- [ ] **Task 4.2**: ソート処理でのディレクティブ適用
  - `crates/tombi-ast-editor/src/rule/table_keys_order.rs` を更新
  - 各グループのディレクティブを確認し、グループごとにソート方法を決定

  ```rust
  // 疑似コード
  for group in table.key_value_groups() {
      // グループのディレクティブを取得
      let directive = extract_directive_from_group(&group);

      // ディレクティブに基づいてソート
      if let Some(directive) = directive {
          if directive.is_disabled() {
              // ソートしない
              continue;
          }
          // その他のディレクティブ処理
      } else {
          // JSON Schema に従ってソート
      }
  }
  ```

- [ ] **Task 4.3**: 連続する dangling コメントグループの処理
  - 「最後のコメントグループ以外」は空の要素グループに紐づく
  - ファイル先頭の1グループ目も空のグループと解釈
  - この処理は `key_value_groups()` の実装内で行う

- [ ] **Task 4.4**: テストの追加
  - 各種ディレクティブのテストケース（draft/new_comment_treatment.md の例を参考）
  - グループ先頭/末尾のディレクティブがグループ全体に作用することを確認
  - **競合するディレクティブの検出**: 同じグループ内で異なるディレクティブがある場合、適切に警告が出ることを確認
  - 例:
    ```toml
    # グループ1に作用
    # tombi: format.rules.table-keys-order.disabled = true

    z = 3
    a = 1

    # グループ2はソートされる
    c = 2
    b = 1
    ```
  - 競合の例:
    ```toml
    # tombi: format.rules.table-keys-order.disabled = true

    z = 3
    a = 1
    # tombi: format.rules.table-keys-order = "ascending"  # 競合！
    ```

### 期待される変更ファイル
- `crates/tombi-comment-directive/src/lib.rs`
- `crates/tombi-ast-editor/src/rule/table_keys_order.rs`
- `crates/tombi-ast-editor/src/rule/root_table_keys_order.rs`
- `crates/tombi-ast-editor/src/rule/array_values_order.rs`
- 関連するテストファイル

### レビューポイント
- グループのダングリングコメントからディレクティブを抽出できるか
- ディレクティブがグループ全体に作用するか
- グループ間でディレクティブが混在しないか
- 既存のコメントディレクティブテストが通るか（または適切に更新されているか）

---

## Phase 5: テストとドキュメント

### 目的
総合的なテストを行い、ドキュメントを更新する。

### タスク

- [ ] **Task 5.1**: 総合例のテスト実装
  - `draft/new_comment_treatment.md` の「総合例: コメントの紐付けと自動ソート（フルセット）」をテストケース化
  - 入力 TOML → フォーマット後の出力が期待通りか確認

- [ ] **Task 5.2**: 各種エッジケースのテスト
  - 空のグループ
  - 連続する空行（2行、3行など → 1行に正規化）
  - コメントのみのファイル
  - ディレクティブとグループの組み合わせ
  - グループ境界とコメントの組み合わせ

- [ ] **Task 5.3**: パフォーマンステスト
  - 大規模ファイルでのパフォーマンス確認
  - グループ抽出のオーバーヘッド測定
  - メモリ使用量の確認

- [ ] **Task 5.4**: ドキュメントの更新
  - 既存の自動ソートドキュメント (`docs/src/routes/docs/formatter/auto-sorting.mdx`) を更新
  - コメントディレクティブドキュメント (`docs/src/routes/docs/comment-directive/`) を更新
  - 新しいグループ概念を説明
  - 空行の扱いを説明

- [ ] **Task 5.5**: CHANGELOG の更新
  - 破壊的変更があれば記載
  - 新機能として「空行によるグループ化とグループ単位ソート」を記載

### 期待される変更ファイル
- テストファイル (複数)
- `docs/src/routes/docs/formatter/auto-sorting.mdx`
- `docs/src/routes/docs/comment-directive/` 配下のファイル
- `CHANGELOG.md`

### レビューポイント
- すべてのテストが通るか
- ドキュメントが最新の仕様を反映しているか
- 破壊的変更が適切に文書化されているか
- パフォーマンスへの影響が許容範囲か

---

## 実装の注意点

### 互換性の維持
- 既存の TOML ファイルが正しくパース・フォーマットされることを確認
- 空行が無い従来のファイルでも正常に動作すること

### パフォーマンス
- グループ抽出のオーバーヘッドを最小限に
- 既存の処理速度に影響が出ないよう注意

### テスト駆動
- 各 Phase で対応するテストを先に書く（可能であれば）
- 既存テストの互換性を確認

---

## 想定される課題と対策

### 課題1: 既存コードへの影響範囲
- **対策**: 段階的な実装とレビュー。各 Phase でテストを実行

### 課題2: グループ境界の判定精度
- **対策**: 既存の `group_comments()` ロジックと整合性を保つ

### 課題3: パフォーマンスへの影響
- **対策**: ベンチマークを取り、必要に応じて最適化

---

## レビューポイント（全体）

各 Phase 完了時に以下を確認：
- [ ] 実装が仕様 (`draft/new_comment_treatment.md`) に準拠しているか
- [ ] 既存のテストが通るか
- [ ] 新しいテストが追加されているか
- [ ] コードの可読性が保たれているか
- [ ] パフォーマンスへの影響が許容範囲か

---

## 次のステップ

Phase 1 から順に実装を開始します。各 Phase 完了後、ユーザーにレビューを依頼します。

---

## Phase 6: 既存ヘルパーメソッドの削除とマイグレーション

### 目的
すべての機能（ソート、フォーマット、ディレクティブ）が `key_value_groups()` と `value_groups()` で正常に動作することを確認した後、既存のヘルパーメソッドを削除し、完全に新しい API に移行する。

### 前提条件
- Phase 1-5 がすべて完了し、テストが通ることを確認済み
- 自動ソートが正常に動作することを確認済み
- フォーマット結果が idempotent であることを確認済み

### タスク

- [ ] **Task 6.1**: 既存ヘルパーメソッドの使用箇所を特定
  - `key_values_begin_dangling_comments()` の使用箇所をすべて検索
  - `key_values_end_dangling_comments()` の使用箇所をすべて検索
  - `key_values_dangling_comments()` の使用箇所をすべて検索
  - 影響範囲をリストアップ

- [ ] **Task 6.2**: 使用箇所を新しい API に移行
  - フォーマッター層での使用箇所を `key_value_groups()` に変更
  - AST エディター層での使用箇所を `key_value_groups()` に変更
  - その他の参照箇所を更新

- [ ] **Task 6.3**: 既存ヘルパーメソッドの削除
  - `crates/tombi-ast/src/impls/table.rs` から削除:
    - `key_values_begin_dangling_comments()` (または類似名)
    - `key_values_end_dangling_comments()` (または類似名)
    - `key_values_dangling_comments()` (または類似名)
  - `crates/tombi-ast/src/impls/root.rs` から同様のメソッドを削除
  - `crates/tombi-ast/src/impls/array.rs` から同様のメソッドを削除（存在する場合）

- [ ] **Task 6.4**: 全テストの実行と確認
  - すべてのテストが通ることを確認
  - リグレッションがないことを確認
  - フォーマット結果が変わっていないことを確認

- [ ] **Task 6.5**: ドキュメントの更新
  - API の変更を CHANGELOG に記載（破壊的変更）
  - マイグレーションガイドを作成（必要に応じて）

### 期待される変更ファイル
- `crates/tombi-ast/src/impls/table.rs`
- `crates/tombi-ast/src/impls/root.rs`
- `crates/tombi-ast/src/impls/array.rs`
- フォーマッター層の関連ファイル
- AST エディター層の関連ファイル
- `CHANGELOG.md`

### レビューポイント
- すべてのテストが通るか
- 既存機能に影響がないか
- 破壊的変更が適切に文書化されているか
- マイグレーションが容易か

---

## 次のステップ

Phase 1 から順に実装を開始します。各 Phase 完了後、ユーザーにレビューを依頼します。Phase 6 は最後に、すべての機能が正常に動作することを確認してから実行します。
