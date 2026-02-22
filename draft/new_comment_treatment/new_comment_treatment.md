## 目的
コメントとノードの紐付けを維持しつつ、`key = value` 間に空の改行を挿入できるようにする。

## 背景
Tombi の自動ソートは「ノードとコメントの紐付けが安定すること」を前提にしている。
一方で、可読性のために空行によるグルーピングが求められている。
ここでは、**安定性 (idempotent)** と **コメントの紐付け** を維持したまま、
空行を「意味を持つ区切り」として扱う方針を検討する。

現行では key 間の空行で区切られたコメントは leading にマージされ空行は削除される。
本仕様では空行を保持し、グループ境界として扱う。

## 目標
- 空行をキー間の論理グループとして扱える
- 自動ソート後もコメントの紐付けが壊れない
- フォーマットの再実行で結果が変わらない (安定)

## 既存課題
- コメントは AST ノードに紐付くため、並び替えで関連が崩れる
- 空行は現在「情報を持たない whitespace」として捨てられる

## 用語

- **空行**: 二つ以上の LINE_BREAK が連続したもの（Whitespace は除く）。境界判定や「直前に空行があるか」の判定ではこの定義を用いる。
- **境界判定のノード**: 本仕様で「前ノード」「次ノード」として用いるノードは、**key-value**、**テーブルヘッダ**（`[table]` / `[[array of tables]]`）、**配列要素**とする。総合例の紐付けはこの範囲で一貫している。
- **グループセパレータ**: パーサーにおいて2つの連続する TS_LINE_END（LINE_BREAK | EOF）で判定する。`is_group_separator(p)` で実装。

## 設計: `DANGLING_COMMENT_GROUP` で dangling comment 情報を保持する

### 1. `DANGLING_COMMENT_GROUP` ノードをパーサーレベルで AST に埋め込む

パーサーレベルで空行により区切られたコメントグループを `DANGLING_COMMENT_GROUP` ノードとして明示的に認識し、AST に保存する。

#### 実装構造

- **`DANGLING_COMMENT_GROUP`** (`SyntaxKind`): パーサーが生成する構文ノード種別
- **`DanglingCommentGroup`** (`crates/tombi-ast/src/node/dangling_comment_group.rs`): AST ノード構造体
  ```rust
  pub struct DanglingCommentGroup {
      pub(crate) syntax: SyntaxNode,
  }
  ```
  - `comments()`: `DanglingComment` トークンのイテレータを返す
  - `into_comments()`: 所有権を消費してイテレータを返す
  - `range()`: テキスト範囲を返す

#### 空行情報の保持

空行情報は **`has_blank_line_before` フラグではなく**、構文木中の `LINE_BREAK` トークンの連続で表現される。

`DANGLING_COMMENT_GROUP` ノード間に `LINE_BREAK` が2つ以上連続していれば、空行があることを意味する。フォーマッタはこの情報を参照して空行の出力有無を決定する。

```
TABLE: {
    BRACKET_START: "[",
    ...
    BRACKET_END: "]",
    LINE_BREAK: "\n",          // ヘッダ後の改行
    DANGLING_COMMENT_GROUP: {  // group 1（直前に空行なし）
        COMMENT: "# comment 1"
    },
    LINE_BREAK: "\n",          // ← 2つ連続で空行を意味
    LINE_BREAK: "\n",          //
    DANGLING_COMMENT_GROUP: {  // group 2（直前に空行あり）
        COMMENT: "# comment 2"
    }
}
```

- フォーマット出力では、`DANGLING_COMMENT_GROUP` の直前に LINE_BREAK が2つ以上連続している場合のみ空行を 1 行出力する（連続空行は最大 1 行に正規化）
- ただし、table の先頭の dangling comment group は例外で空行を 0 行として出力する

#### パーサー実装

`Vec::<DanglingCommentGroup>::parse(p)` で dangling comment group のパースを行う（`crates/tombi-parser/src/parse/dangling_comment_group.rs`）。

**判定ロジック** (`dangling_comment_group_len`):
1. 現在位置が COMMENT トークンであること
2. COMMENT の直後が TS_LINE_END であること
3. その次が TS_DANGLING_COMMENT_GROUP_END（`}`, `]`, LINE_BREAK, EOF のいずれか）であれば、そこまでが1つのグループ
4. そうでなく COMMENT が続く場合は、ループして複数行コメントを同一グループに含める

**TS_DANGLING_COMMENT_GROUP_END**: `}`, `]`, `LINE_BREAK`, `EOF` — dangling comment group の終端を示すトークン集合

### 2. `DanglingCommentGroupOr<T>` による混合グループの表現

`DANGLING_COMMENT_GROUP` と要素グループ（`KeyValueGroup` / `ValueWithCommaGroup` / `KeyValueWithCommaGroup`）を統一的に扱うために、ジェネリック enum `DanglingCommentGroupOr<T>` を使用する（`crates/tombi-ast/src/node/dangling_comment_group_or.rs`）。

```rust
pub enum DanglingCommentGroupOr<T> {
    DanglingCommentGroup(DanglingCommentGroup),
    ItemGroup(T),
}
```

> **注**: 当初の設計案では `KeyValueGroup` 自体を Enum（`KeyValues` / `DanglingComments`）にする方針だったが、実装では `KeyValueGroup` を素の構造体のまま維持し、外側の `DanglingCommentGroupOr<T>` で合成する方式を採用した。これにより `KeyValueGroup` の責務がシンプルに保たれ、他のグループ型（`ValueWithCommaGroup`, `KeyValueWithCommaGroup`）にも同じ `DanglingCommentGroupOr<T>` パターンを再利用できる。

### 3. グループノードの種類

| コンテナ | グループノード | 要素 | AST メソッド |
|----------|---------------|------|-------------|
| Root | `KeyValueGroup` (KEY_VALUE_GROUP) | KeyValue | `key_value_groups() -> impl Iterator<Item = DanglingCommentGroupOr<KeyValueGroup>>` |
| Table | `KeyValueGroup` (KEY_VALUE_GROUP) | KeyValue | `key_value_groups() -> impl Iterator<Item = DanglingCommentGroupOr<KeyValueGroup>>` |
| ArrayOfTable | 個別の KeyValue（グループノードなし） | KeyValue | `key_value_groups() -> impl Iterator<Item = DanglingCommentGroupOr<KeyValueGroup>>` |
| Array | `ValueWithCommaGroup` (VALUE_WITH_COMMA_GROUP) | Value + Comma | `value_with_comma_groups() -> impl Iterator<Item = DanglingCommentGroupOr<ValueWithCommaGroup>>` |
| InlineTable | `KeyValueWithCommaGroup` (KEY_VALUE_WITH_COMMA_GROUP) | KeyValue + Comma | `key_value_with_comma_groups() -> impl Iterator<Item = DanglingCommentGroupOr<KeyValueWithCommaGroup>>` |

> **注**: ArrayOfTable は現在、パーサーが個別の `KeyValue::parse(p)` を呼び出しており、`KeyValueGroup` ノードで包んでいない。AST の `key_value_groups()` メソッドでは `dangling_comment_group_or` ヘルパーを通じて論理的なグループ化を提供する。

### 4. ソートは「グループ単位」で実行
- `KeyValueGroup` / `ValueWithCommaGroup` / `KeyValueWithCommaGroup` で分割された範囲をグループとみなし、各グループ内だけソート
- `DanglingCommentGroup` は要素を持たないため、ソート対象外である
- グループ内でしか自動ソートしないため、グループ間の順序は **元の順序を維持** される
- leading / trailing コメントはグループ内のキーに紐付いたまま自動ソートされる

### 5. グループ間移動を抑制するルール
自動ソートでキーが別グループに移動すると、
空行の意味 (視覚的区切り) が壊れるため禁止する。
結果として:
- `keyA` と `keyB` の順序を入れ替える必要がある場合、
  **同一グループ内**に存在していないと並び替えは実施しない
- これにより安定性を確保

### 6. コメントの紐付けはグループ内で維持
- leading / trailing コメントは既存ロジック通りノードに紐付ける
- dangling コメントはノードに紐付けず独立保持する（後述）
- `DANGLING_COMMENT_GROUP` 自体には作用ノードを紐付けない（table 先頭例外を除く）

### 7. Dangling comment の扱い
- **判定**
  - **leading comment**: コメントの直後に空行がなく、直後ノードが `key/value`（配列では value、テーブルでは key-value、またはテーブルヘッダ）であるもの。パーサーの `peek_leading_comments` で先読みし、KeyValue ノードの子トークンとして取り込まれる
  - **dangling comment group**: コメントグループの直後に空行があるもの（またはスコープ終端にあるもの）。パーサーの `dangling_comment_group_len` で判定され、`DANGLING_COMMENT_GROUP` ノードとして生成される
- **dangling comment の所属レベル**
  - `KeyValueGroup` / `ValueWithCommaGroup` / `KeyValueWithCommaGroup` / `DanglingCommentGroup` 自体には dangling comment は付与されない。これらのグループノードは dangling comment を保持しない
  - dangling comment group は **コンテナレベル**（Root / Table / ArrayOfTable / Array / InlineTable）の子ノードとして存在する
  - これにより、従来と同様のレベル（コンテナ単位）でのコメントディレクティブの適用が可能となる。各コンテナの `comment_directives()` メソッドで `dangling_comment_groups()` からディレクティブを収集する
- **基本ルール**
  - dangling comment group は key_values グループに紐づけない
  - dangling comments は所属ノードを持たない独立要素として、コンテナの直接の子として保持する
- **例外（テーブル先頭の dangling）**
  - 各テーブル（ルートテーブルを含む）で、先頭から最初の key_values group までにある dangling comment groups は **table 全体コメント**として扱う
  - この範囲の value directive は table に適用され、JSON Schema より優先される
  - ファイル先頭の dangling comment group も同ルールで、ルートテーブルへの directive として扱う
- **空行保持と出力**
  - dangling comment group 間の空行は LINE_BREAK トークンの連続で構文木に保持される
  - 連続空行は 1 行に圧縮する
  - 各テーブルスコープでヘッダ直後にある **最初の dangling comment group** は例外で空行なしとして出力する
  - フォーマット時は `DANGLING_COMMENT_GROUP` の直前に LINE_BREAK が2つ以上連続している場合のみ空行を 1 行出力し、その後でコメントを出力する

## 仕様案

### フォーマット挙動
```toml
key1 = "a"
key3 = "c"

key2 = "b"
```
↓ (key1, key3 は同グループなのでソート対象)
```toml
key1 = "a"
key3 = "c"

key2 = "b"
```

### 例2: グループを跨ぐ移動は禁止
```toml
key1 = "a"

key2 = "b"
key3 = "c"
```
↓ (key2, key3 は同グループ内でソート)
```toml
key1 = "a"

key2 = "b"
key3 = "c"
```
`key1` と `key2` は別グループのため並び替えなし

## 総合例: dangling の空行復元（改訂）

`table` 先頭例外と、`key_values` グループ非紐付けを同時に示す。

### 入力（フォーマット前）

```toml
[table]

# table's dangling comment group 1
# tombi: format.rules.table-keys-order.disabled = true

# table's dangling comment group 2

key_b = "value"
# key value group's dangling comment group 1

key_a = "value"


# key value group's dangling comment group 2
```

### パーサー出力（構文木）

```
TABLE: {
    BRACKET_START: "[",
    KEYS: { BARE_KEY: { BARE_KEY: "table" } },
    BRACKET_END: "]",
    LINE_BREAK: "\n",
    LINE_BREAK: "\n",
    DANGLING_COMMENT_GROUP: {
        COMMENT: "# table's dangling comment group 1",
        LINE_BREAK: "\n",
        COMMENT: "# tombi: format.rules.table-keys-order.disabled = true"
    },
    LINE_BREAK: "\n",
    LINE_BREAK: "\n",
    DANGLING_COMMENT_GROUP: {
        COMMENT: "# table's dangling comment group 2"
    },
    LINE_BREAK: "\n",
    LINE_BREAK: "\n",
    KEY_VALUE_GROUP: {
        KEY_VALUE: {
            KEYS: { BARE_KEY: { BARE_KEY: "key_b" } },
            ...
        }
    },
    LINE_BREAK: "\n",
    DANGLING_COMMENT_GROUP: {
        COMMENT: "# key value group's dangling comment group 1"
    },
    LINE_BREAK: "\n",
    LINE_BREAK: "\n",
    KEY_VALUE_GROUP: {
        KEY_VALUE: {
            KEYS: { BARE_KEY: { BARE_KEY: "key_a" } },
            ...
        }
    },
    LINE_BREAK: "\n",
    LINE_BREAK: "\n",
    LINE_BREAK: "\n",
    DANGLING_COMMENT_GROUP: {
        COMMENT: "# key value group's dangling comment group 2"
    }
}
```

### 判定

| コメント | 扱い | 直前の空行 | ソートへの影響 |
|----------|------|-----------|----------------|
| table's dangling comment group 1 | table 先頭 dangling（例外） | LINE_BREAK ×2 → **あるが先頭例外で 0 出力** | `[table]` に適用 |
| table's dangling comment group 2 | table 先頭 dangling（例外） | LINE_BREAK ×2 → 空行 1 | `[table]` に適用 |
| key value group's dangling comment group 1 | 独立 dangling | LINE_BREAK ×1 → 空行なし | なし |
| key value group's dangling comment group 2 | 独立 dangling | LINE_BREAK ×3 → 空行 1（圧縮） | なし |

- `key_values` グループの直前/直後 dangling は `key_values` へ紐づけない
- table 先頭範囲の dangling directives のみ、table ノードに適用される

### 出力（フォーマット後）

```toml
[table]
# table's dangling comment group 1
# tombi: format.rules.table-keys-order.disabled = true

# table's dangling comment group 2

key_b = "value"
# key value group's dangling comment group 1

key_a = "value"

# key value group's dangling comment group 2
```

→ `group 1` は「テーブルヘッダ直後の最初の dangling comment group」例外で空行 0。
→ `group 2` と `key value group's dangling comment group 2` は空行 1 を復元。
→ `table-keys-order.disabled` は table 先頭 dangling なので有効、`key_b`/`key_a` は並び替えない。

---

### ミニ例: 同じキーが別グループにあるときは並び替えない

```toml
# グループ1
name = "first"
version = "0.1.0"

# グループ2
version = "0.2.0"
name = "second"
```

| グループ | ソート結果 | 理由 |
|----------|------------|------|
| グループ1 | name, version のまま | グループ内で既に昇順的なら変更なし。Schema が昇順なら name → version の順に揃う |
| グループ2 | version, name のまま or name, version に | グループ内のみソート。**グループ1の name とグループ2の name は入れ替わらない** |

フォーマット後（Schema で昇順の場合）:

```toml
# グループ1
name = "first"
version = "0.1.0"

# グループ2
name = "second"
version = "0.2.0"
```

→ グループ間の順序（グループ1 → グループ2）は不変。各グループ内だけキー順が変わる。

## 代替案 (非推奨)

### A. 空行をコメントと同等に扱う
- コメント再配置と同様の問題が発生するため不適

### B. 空行の数を保持せず最大値だけ保持
- グループ意図が失われるため不可

### C. KeyValueGroup を Enum にする（当初案、不採用）
- `KeyValueGroup::KeyValues { ... }` / `KeyValueGroup::DanglingComments(...)` として Enum にする案
- 不採用理由: `DanglingCommentGroupOr<T>` ジェネリック enum のほうが各グループ型（`KeyValueGroup`, `ValueWithCommaGroup`, `KeyValueWithCommaGroup`）に対して統一的に適用でき、個々のグループ型の責務をシンプルに保てるため

## 実装上のポイント

### パーサー層

- 各コンテナ（Root, Table, ArrayOfTable, Array, InlineTable）のパースループ内で `Vec::<DanglingCommentGroup>::parse(p)` を呼び、`DANGLING_COMMENT_GROUP` ノードを生成する
- `is_group_separator(p)` は `p.at_ts(TS_LINE_END) && p.nth_at_ts(1, TS_LINE_END)` で判定する
- `KeyValueGroup::parse` / `ValueWithCommaGroup::parse` は `is_group_separator` でグループの終端を検出し、グループ単位でノードを生成する
- leading comment の先読みは `peek_leading_comments(p)` で行い、KeyValue / Value の子トークンとして取り込む

### AST 層

- `DanglingCommentGroup` は `AstNode` を実装し、`DANGLING_COMMENT_GROUP` からキャスト可能
- `DanglingCommentGroupOr<T>` は `AstNode` を実装し、`DanglingCommentGroup` と `T` の両方からキャスト可能
- ヘルパー関数（`crates/tombi-ast/src/support/comment.rs`）:
  - `dangling_comment_groups(iter)`: イテレータから `DanglingCommentGroup` を抽出
  - `dangling_comment_group_or<T>(iter)`: イテレータから `DanglingCommentGroupOr<T>` を抽出
  - `leading_comments(iter)`: イテレータから `LeadingComment` を抽出
  - `trailing_comment(iter, end)`: 指定トークンの後の `TrailingComment` を抽出

### フォーマッタ層

- フォーマッタのソート関数に「グループ境界」を渡す
- `DanglingCommentGroupOr<T>` をパターンマッチし:
  - `ItemGroup(group)` → グループ内でソート処理
  - `DanglingCommentGroup(comments)` → コメントをそのまま出力
- 空行出力: `DANGLING_COMMENT_GROUP` ノードの直前の LINE_BREAK トークン数で空行の有無を判定
- 既存の comment attachment を壊さないよう、キーの移動はグループ内のみ（dangling は独立保持）

## スコープ: テーブル以外のソート対象

### 共通方針
- ソート方法の優先順位は **(ノードに作用可能な) Comment Directives > JSON Schema**（既存ドキュメント通り）。グループ単位で適用する
- 以下、inline table / array of tables / array それぞれで、本仕様（dangling の独立保持・空行復元）をどう当てはめるかを示す

### Array（format.rules.array-values-order）
- **対象**: 配列の**要素**の並び。複数行にわたる配列では、要素と要素の間に空行を入れ得る
- **グループノード**: `ValueWithCommaGroup` (VALUE_WITH_COMMA_GROUP)
- **グループ取得**: `Array::value_with_comma_groups() -> impl Iterator<Item = DanglingCommentGroupOr<ValueWithCommaGroup>>`
- **コメントディレクティブ**: `Array::comment_directives()` で `bracket_start_trailing_comment()` + `dangling_comment_groups()` から収集
- **パーサー**: `Vec::<DanglingCommentGroup>::parse(p)` → `ValueWithCommaGroup::parse(p)` のループ
- **ソートルール**: グループ内の要素のみソートし、グループ間の順序は変えない
  - **trailing**: その要素行の末尾 → その要素に作用
  - **leading**: 次の要素の直前行 → 次の要素に作用
  - **dangling**: 要素やグループには紐づけず独立保持。空行復元の有無は LINE_BREAK の連続数で決定
- グループに（leading/trailing で）ディレクティブが無い場合は JSON Schema の `x-tombi-array-values-order` 等に従う

### Inline table（format.rules.table-keys-order）
- **対象**: `key = { k1 = v1, k2 = v2 }` のようなインライン表の**キー**の並び
- **グループノード**: `KeyValueWithCommaGroup` (KEY_VALUE_WITH_COMMA_GROUP)
- **グループ取得**: `InlineTable::key_value_with_comma_groups() -> impl Iterator<Item = DanglingCommentGroupOr<KeyValueWithCommaGroup>>`
- **コメントディレクティブ**: `InlineTable::comment_directives()` で `brace_start_trailing_comment()` + `dangling_comment_groups()` から収集
- **パーサー**: `Vec::<DanglingCommentGroup>::parse(p)` → `KeyValueWithCommaGroup::parse(p)` のループ
- **制約**: TOML v1.0.0 のインライン表は 1 行で書くため、**構文上キー間に空行は存在しない**。よってインライン表内のグループは常に **1 グループのみ**。TOML v1.1.0 では複数行インライン表が可能で、空行によるグループ分割も適用される
- ソート無効・ソート方法は、そのインライン表に付いたディレクティブで指定。無ければ Schema に従う

### Array of tables（format.rules.table-keys-order）
- **対象**: 各 `[[section]]` 内の key-value 列。通常の `[table]` と同じ並び
- **グループ取得**: `ArrayOfTable::key_value_groups() -> impl Iterator<Item = DanglingCommentGroupOr<KeyValueGroup>>`
- **コメントディレクティブ**: `ArrayOfTable::comment_directives()` で `header_leading_comments()` + `header_trailing_comment()` + `dangling_comment_groups()` から収集
- **パーサー**: `Vec::<DanglingCommentGroup>::parse(p)` → `KeyValue::parse(p)` のループ（**注**: Table と異なり、パーサーは `KeyValueGroup` ノードで包まず個別の `KeyValue` をパースする。AST 層の `key_value_groups()` で論理的なグループ化を提供）
- **ソートルール**: ルートや通常テーブルと同じ
  - **trailing / leading**: その key-value または次の key-value に作用（既存どおり）
  - **dangling**: key-values グループには紐づけず独立保持。LINE_BREAK の連続数で出力時の空行復元を決定
  - ただし各 `[[section]]` の先頭から最初の key-values group までの dangling は、その `[[section]]` テーブル全体コメントとして扱う
- テーブル間（`[[a]]` と `[[b]]` の間）に key-value が無い区間の dangling も、いずれの key_values グループにも紐づけない独立 dangling として扱う
- グループに（leading/trailing で）ディレクティブが無い場合は JSON Schema に従う
