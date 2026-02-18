## 目的
コメントとノードの紐付けを維持しつつ、`key = value` 間に空の改行を挿入できるようにする。

## 背景
Tombi の自動ソートは「ノードとコメントの紐付けが安定すること」を前提にしている。
一方で、可読性のために空行によるグルーピングが求められている。
ここでは、**安定性 (idempotent)** と **コメントの紐付け** を維持したまま、
空行を「意味を持つ区切り」として扱う方針を検討する。

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

## 提案: `DANGLING_COMMENTS` で dangling comment 情報を保持する

### 1. `DANGLING_COMMENTS` ノードを AST に埋め込む
- 空行由来の区切りは `DANGLING_COMMENTS` ノードから rowan 経由で LINE_BREAK が2回続いているかで判定する- `DANGLING_COMMENTS` は `crates/tombi-ast/src/node/dangling_comments.rs` を基準に扱う
- フォーマット出力では、空の改行が dangling comment の前にあるときだけ空行を 1 行出力する（連続空行は最大 1 行に正規化）
- ただし、table の先頭の dangling comment group は例外で空行を 0 行として出力する。

### 2. ソートは「グループ単位」で実行
- `KeyValueGroup` / `ValueGroup` で分割された範囲をグループとみなし、各グループ内だけソート
- dangling comment group は key_value を持たないため、ソート対象外である。
- グループ内でしか自動ソートしないため、グループ間の順序は **元の順序を維持** される。
- leading / trailing コメントはグループ内のキーに紐付いたまま自動ソートされる

### 3. グループ間移動を抑制するルール
自動ソートでキーが別グループに移動すると、
空行の意味 (視覚的区切り) が壊れるため禁止する。
結果として:
- `keyA` と `keyB` の順序を入れ替える必要がある場合、
  **同一グループ内**に存在していないと並び替えは実施しない
- これにより安定性を確保

### 4. コメントの紐付けはグループ内で維持
- leading / trailing コメントは既存ロジック通りノードに紐付ける
- dangling コメントはノードに紐付けず独立保持する（後述）
- `DANGLING_COMMENTS` 自体には作用ノードを紐付けない（table 先頭例外を除く）

### 5. Dangling comment の扱い (改訂)
- **判定**
  - **leading comment group**: コメントグループの直後に空行がなく、直後ノードが `key/value`（配列では value、テーブルでは key-value、またはテーブルヘッダ）であるもの
  - **dangling comment group**: コメントグループの直後に空行があるもの（またはスコープ終端にあるもの）
- **基本ルール**
  - dangling comment group は key_values グループに紐づけない
  - key_values グループの `end_dangling_comments` にも紐づけない
  - dangling comments は所属ノードを持たない独立要素として保持する
- **例外（テーブル先頭の dangling）**
  - 各テーブル（ルートテーブルを含む）で、先頭から最初の key_values group までにある dangling comment groups は **table 全体コメント**として扱う
  - この範囲の value directive は table に適用され、JSON Schema より優先される
  - ファイル先頭の dangling comment group も同ルールで、ルートテーブルへの directive として扱う
- **空行保持と出力**
  - dangling comment group ごとに `has_blank_line_before`（0/1）を保持する
  - 連続空行は 1 行に圧縮する
  - 各テーブルスコープで key_values group 手前にある **最初の dangling comment group は例外で `has_blank_line_before = 0`** として出力する
  - フォーマット時は `has_blank_line_before=1` の場合のみ空行を 1 行出力し、その後で comment directives（または通常コメント）を出力する

### 6. コメントディレクティブの作用ノード
- ディレクティブは **「最も近い構文ノード」** に作用させる
- ただし comment group 境界を跨いだ作用は禁止
- **ソート方法の優先順位**: 既存ドキュメント（`auto-sorting.mdx`）のとおり **Comment Directives > JSON Schema**
- ディレクティブの紐付け・優先順位の詳細は既存ドキュメントに準拠する
  (`docs/src/routes/docs/comment-directive/tombi-value-directive.mdx` と
  `docs/src/routes/docs/formatter/auto-sorting.mdx`)
- コメントディレクティブは、既存ロジックを踏襲しノードに紐づく全てのコメントを収集してバリデーションにかけられる（重複キーなどもバリデーションされる。バリデーション対象にノードに紐づく全てのコメントディレクティブを渡せば良い）:
  - leading comments
    ```toml
    # tombi: format.rules.table-keys-order.disabled = true
    key1 = "a"
    ```
  - trailing comment
    ```toml
    key1 = "a" # tombi: format.rules.table-keys-order.disabled = true
    ```
  - table header leading comment
    ```toml
    # tombi: format.rules.table-keys-order.disabled = true
    [table]
    ```
  - table header trailing comment
    ```toml
    [table] # tombi: format.rules.table-keys-order.disabled = true
    ```
  - table dangling comment
    ```toml
    [table]
    # tombi: format.rules.table-keys-order.disabled = true
    ```
- **document コメントディレクティブ**（`#:schema`, `#:tombi`）: root の table dangling commentsのみで定義できる制限を継続する
- ドキュメントが key value の leading comments から始まり、かつそのコメントの中に document コメントディレクティブがある場合、そのコメントディレクティブは root の table dangling comments として再フォーマット（key value の前に空の改行が追加）される。これは既存の挙動と同様。

#### 例: trailing
```toml
key1 = "a" # tombi: format.rules.table-keys-order.disabled = true
```
→ `key1` に作用

#### 例: leading (次ノードに作用)
```toml
# tombi: format.rules.table-keys-order.disabled = true
key2 = "b"
```
→ `key2` に作用

#### 例: dangling (直後空行あり)
```toml
key1 = "a"

# tombi: format.rules.table-keys-order.disabled = true

key2 = "b"
```
→ dangling は `key2` グループに紐づけない
→ 直前が開業で終わっているため、フォーマッタは出力時に空行を 1 行復元してからコメントを出力する

#### 例: dangling (直前空行なし)
```toml
key1 = "a"
# tombi: format.rules.table-keys-order.disabled = true

key2 = "b"
```
→ dangling はどの key_values グループにも紐づけない

#### 例: table 先頭 dangling は table に作用
```toml
[table]

# tombi: format.rules.table-keys-order.disabled = true

key_b = 2
key_a = 1
```
→ この dangling は table 先頭範囲にあるため、`[table]` に作用する
→ `table-keys-order.disabled = true` が有効になる

```toml
[table]


key_b = 2

# tombi: format.rules.table-keys-order.disabled = true

key_a = 1
```

この dangling は key values の後にあるため、どのノードにも紐づかない。

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

### 判定

| コメント | 扱い | `has_blank_line_before` | ソートへの影響 |
|----------|------|-------------------------|----------------|
| table's dangling comment group 1 | table 先頭 dangling（例外） | **0（強制）** | `[table]` に適用 |
| table's dangling comment group 2 | table 先頭 dangling（例外） | 1 | `[table]` に適用 |
| key value group's dangling comment group 1 | 独立 dangling | 0 | なし |
| key value group's dangling comment group 2 | 独立 dangling | 1（連続空行は 1 に圧縮） | なし |

- `key_values` グループの直前/直後 dangling は `key_values` へ紐づけない
- `end_dangling_comments` にも入れない
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

→ `group 1` は「最初の dangling comment group」例外で空行 0。
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

## 実装上のポイント
- パーサで `DANGLING_COMMENTS` と `has_blank_line_before` を保持する必要がある
- フォーマッタのソート関数に「グループ境界」を渡す
- dangling comment には `has_blank_line_before` を保持する
- 保持レイヤーは `KeyValueGroup` と同階層とし、`crates/tombi-ast/src/node/key_value_group.rs` の `KeyValueGroup::DanglingComments` を基準に扱う
- 既存の comment attachment を壊さないよう、キーの移動はグループ内のみ（dangling は独立保持）

## スコープ: テーブル以外のソート対象

### 共通方針
- ソート方法の優先順位は **(ノードに作用可能な) Comment Directives > JSON Schema**（既存ドキュメント通り）。グループ単位で適用する
- 以下、inline table / array of tables / array それぞれで、本仕様（dangling の独立保持・空行復元）をどう当てはめるかを提案する

### Array（format.rules.array-values-order）
- **対象**: 配列の**要素**の並び。複数行にわたる配列では、要素と要素の間に空行を入れ得る
- **グループ**: 要素列を ValueGroup（`Values` / `DanglingComments`）で分割した区間を 1 グループとする。グループ内の要素のみソートし、グループ間の順序は変えない
- **コメントディレクティブ**:
  - **trailing**: その要素行の末尾 → その要素に作用
  - **leading**: 次の要素の直前行 → 次の要素に作用
  - **dangling**: 要素やグループには紐づけず独立保持。`has_blank_line_before` のみ保持し、出力時に空行復元の有無を決める
- グループに（leading/trailing で）ディレクティブが無い場合は JSON Schema の `x-tombi-array-values-order` 等に従う

### Inline table（format.rules.table-keys-order）
- **対象**: `key = { k1 = v1, k2 = v2 }` のようなインライン表の**キー**の並び
- **制約**: TOML のインライン表は 1 行で書くため、**構文上キー間に空行は存在しない**。よってインライン表内のグループは常に **1 グループのみ**
- **コメントディレクティブ**:
  - **trailing**: そのキー行の末尾、または `}` の直前の trailing → そのキーまたはインライン表全体に作用（現行どおり）
  - **leading**: インライン表の直前行、または `{` の直後 → 次のキーまたはインライン表に作用
  - インライン表内部では dangling をグループ制御には使わない。ディレクティブは leading/trailing で**そのインライン表（またはキー）**に付く
- ソート無効・ソート方法は、そのインライン表に付いたディレクティブで指定。無ければ Schema に従う

### Array of tables（format.rules.table-keys-order）
- **対象**: 各 `[[section]]` 内の key-value 列。通常の `[table]` と同じ並び
- **グループ**: 各 `[[section]]` の key-values のあいだを KeyValueGroup（`KeyValues` / `DanglingComments`）で区切った区間をグループとする。ルートや通常テーブルと同じルール
- **コメントディレクティブ**:
  - **trailing / leading**: その key-value または次の key-value に作用（既存どおり）
  - **dangling**: key-values グループには紐づけず独立保持。`has_blank_line_before` を使って出力時の空行復元だけを行う
  - ただし各 `[[section]]` の先頭から最初の key-values group までの dangling は、その `[[section]]` テーブル全体コメントとして扱う
- テーブル間（`[[a]]` と `[[b]]` の間）に key-value が無い区間の dangling も、いずれの key_values グループにも紐づけない独立 dangling として扱う
- グループに（leading/trailing で）ディレクティブが無い場合は JSON Schema に従う
