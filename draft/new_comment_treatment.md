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

## 提案: 空行を「境界ノード」として保持する

### 1. 空行を `Separator` として AST に埋め込む
- 連続する空行を `Separator(n)` として保持 (n は連続数。AST 上は空改行の数を保持)
- `Separator` はキーに紐付かず、**グループ境界**として扱う
- **フォーマット出力**: フォーマッタでは `Separator` を 0 行または 1 行の空行に圧縮して出力する (連続空行は最大 1 行に正規化)

### 2. ソートは「グループ単位」で実行
- `Separator` で分割された範囲をグループとみなし、各グループ内だけソート
- グループ間の順序は **元の順序を維持**
- コメントはグループ内のキーに紐付いたまま

### 3. グループ間移動を抑制するルール
自動ソートでキーが別グループに移動すると、
空行の意味 (視覚的区切り) が壊れるため禁止する。
結果として:
- `keyA` と `keyB` の順序を入れ替える必要がある場合、
  **同一グループ内**に存在していないと並び替えは実施しない
- これにより安定性を確保

### 4. コメントの紐付けはグループ内で維持
- コメントは既存のロジック通りノードに紐付ける
- `Separator` 自体にはコメントを紐付けない

### 5. Dangling comment の扱い (全コメント共通)
- **dangling comment の判定基準は「次ノードとの間の空行」**
- **前ノードとの間に空行が無ければ**前のグループに紐付ける
- **前ノードとの間に空行があれば**次のグループに紐付ける
- **テーブル間（または key-value 列の終端と次のセクションの間）の dangling**: ルートと [table] の間、[table] と次の key の間、[[item]] と次の [[item]] の間など、**その区間に key-value が存在しない**場合、その区間のコメントは「直前の key/value やテーブルに属する」のではなく、**そのスコープ（ルートやテーブル）の空の要素グループ**に対する dangling として扱う。つまり [1] や [7] と同様に「空の dangling comments グループ」とみなす
- **dangling コメントグループが連続する場合**: 空行で区切られた複数の dangling コメントグループが連続してあるとき、**最後のコメントグループ以外**は、要素が空のグループに対する dangling comment として保持する。**次の key-value グループに紐づくのは「最後の」コメントグループのみ**。ファイル先頭やテーブル先頭から key-value までの間も同じ（例: 先頭に [1] と [2] の 2 グループなら [1] は空の要素グループ、[2] が次のグループに紐づく）

### 6. コメントディレクティブの作用ノード
- ディレクティブは **「最も近い構文ノード」** に作用させる
- ただし `Separator` を跨いだ作用は禁止
- **ソート方法の優先順位**: 既存ドキュメント（`auto-sorting.mdx`）のとおり **Comment Directives > JSON Schema** である。グループにディレクティブが無い場合のみ、そのグループに JSON Schema のソート指示を適用する
- ディレクティブの紐付け・優先順位の詳細は既存ドキュメントに準拠する
  (`docs/src/routes/docs/comment-directive/tombi-value-directive.mdx` と
  `docs/src/routes/docs/formatter/auto-sorting.mdx`)
- 優先順位 (上から順に適用):
  1. **同一行 trailing**: その行のノード (同一行の key/value)
  2. **直前行の leading**: 次のノード
  3. **ファイル先頭の value ディレクティブ**: key-value の**直前に**あり、直後に空行を挟んで key-value が始まるコメントグループ内のディレクティブは、次の key-value グループに紐づく。ファイルの絶対先頭に別のコメントグループ（空行で区切られた）がある場合、その先頭のグループは「連続する dangling の最後以外」となり**空の要素グループ**に紐づく (document 用の `#:tombi` とは別。document レベルのコメントディレクティブは**最初のコメントグループにのみ**記述可能)
  4. **dangling**:
     - 次ノードとの間に空行なし: 前のグループに作用
     - 次ノードとの間に空行あり: 次のグループに作用
- **document コメントディレクティブ** (`#:tombi`): value 用の `# tombi:` とは指示形が異なるため本仕様の紐付けには影響しない。ただし、ドキュメントレベルのコメントディレクティブは**最初のコメントグループにしか**記述できない

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

#### 例: dangling (グループ境界)
```toml
key1 = "a"
# tombi: format.rules.table-keys-order.disabled = true

key2 = "b"
key3 = "c"
```
→ `key1` に紐付く (直前ノードとの間に空行が無い)

#### 例: dangling (空行を挟む)
```toml
key1 = "a"

# tombi: format.rules.table-keys-order.disabled = true
key2 = "b"
key3 = "c"
```
→ `key2` と `key3` のグループに紐付く
→ グループ先頭 (`key2`) の前に配置される

#### 例: グループ跨ぎ禁止
```toml
key1 = "a"
# tombi: format.rules.table-keys-order.disabled = true

key2 = "b"
```
→ `key1` に紐付く (直前ノードとの間に空行が無い)

#### 例: グループ末尾の dangling はソート対象外
空行を挟まないグループ末尾の dangling コメントは、
**自動ソートで移動しない** (グループに対する dangling として保持)。

```toml
b = 2
a = 1
# dangling: group tail

c = 3
```
↓ (同一グループ内のみソート)
```toml
a = 1
b = 2
# dangling: group tail

c = 3
```
→ ソート後も **グループ末尾の dangling** であることは変わらない

#### 例: グループ先頭のdangling コメントにディレクティブを書いた場合
グループ先頭の dangling に書かれたディレクティブは、
**グループ全体の自動ソートに作用**する。

先頭のグループの dangling コメントにディレクティブを書いた場合

```toml
# tombi: format.rules.table-keys-order.disabled = true

b = 2
a = 1

c = 3
```
↓ (このグループはソート無効)
```toml
# tombi: format.rules.table-keys-order.disabled = true

b = 2
a = 1

c = 3
```
→ ソート後も **グループ先頭の dangling** として保持される

#### 例: グループ末尾のdangling コメントにディレクティブを書いた場合

グループ末尾の dangling に書かれたディレクティブは、
**グループ全体の自動ソートに作用**する。

```toml
b = 2
a = 1
# tombi: format.rules.table-keys-order.disabled = true

c = 3
```
↓ (このグループはソート無効)
```toml
b = 2
a = 1
# tombi: format.rules.table-keys-order.disabled = true

c = 3
```

→ ソート後も **グループ末尾の dangling** として保持される

#### 例: 複数グループとディレクティブ
複数のグループがある場合、ディレクティブは**記載されたグループのみに作用**する。
各グループごとにディレクティブでソート方法を指定でき、
指定が無い場合は **JSON Schema のソート指示**を使う。
グループに作用させるディレクティブは **dangling comment** である必要があるため、ディレクティブの**直後に空行**を入れる。

```toml
# tombi: format.rules.table-keys-order.disabled = true

b = 2
a = 1

# tombi: format.rules.table-keys-order = "ascending"

z = 3
y = 2

d = 4
c = 5
```
↓ (1グループ目: 無効 / 2グループ目: 有効 / 3グループ目: JSON Schema のソート順)
```toml
# tombi: format.rules.table-keys-order.disabled = true

b = 2
a = 1

# tombi: format.rules.table-keys-order = "ascending"

y = 2
z = 3

c = 5
d = 4
```
→ 各ディレクティブは **該当グループのみ**に適用される。3 グループ目は Schema 指示でソート

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

## 総合例: コメントの紐付けと自動ソート（フルセット）

以下は **ルート / [table] / [[array of tables]] / inline table / array** をすべて含む 1 つの TOML で、各コメントの紐付けと自動ソートの適用をコードで示す。

### 入力（フォーマット前）

```toml
# ========== ルート ==========
# [1] ファイル先頭の 1 つ目のコメントグループ。直後に空行があり [2] が別グループ → 「連続する dangling の最後以外」なので**空の要素グループ**に紐づく（[7] と同じ扱い）

# [2] ルート・グループAの直前にあり直後に空行あり → 「最後の」コメントグループなので**ルート・グループA**に紐づく（グループ先頭の dangling）。ここに value ディレクティブを書けばグループAに作用
# tombi: format.rules.table-keys-order.disabled = true

root_z = 3
root_y = 1
root_x = 2  # [3] ルート・グループAの trailing（root_x の行）
# [4] ルート・グループAの末尾の dangling。次ノードとの間に空行あり → 次のグループBに紐づく

# [5] ルート・グループBの先頭の dangling（ディレクティブ）。直後に空行あり → グループB全体に作用
# tombi: format.rules.table-keys-order = "ascending"

root_b = 2
root_a = 1
root_c = 3
# [6] ルート・グループBの末尾の dangling。次ノードとの間に空行なし → グループBに紐づく（ソートで動かない）

# [7] 空の要素グループに対する dangling（連続 dangling の「最後以外」）

# [8] ルート・グループCの先頭の dangling。ディレクティブなし → グループCは Schema に従う

root_m = 2
root_k = 1
root_n = 3

# [9] ルートと [foo] の間に key-value が無いため、**ルートの空の dangling comments グループ**（ルートの空の要素グループに対する dangling）

# [10] [foo] の leading（テーブルヘッダの直前行）→ [foo] に作用
[foo]
# [11] [foo] グループ1の先頭の dangling（ディレクティブ）。直後に空行あり → [foo] のグループ1全体に作用
# tombi: format.rules.table-keys-order.disabled = true

foo_z = 3
foo_y = 1
foo_x = 2  # [12] [foo] グループ1の trailing
# [13] [foo] グループ1の末尾の dangling。次ノード（foo_b）との間に空行が無い → グループ1に紐づく（ソートで動かない）

# [14] [foo] グループ2の先頭の dangling。直前に空行あり・直後に空行なし（foo_b が続く）。ディレクティブなし → グループ2は Schema に従う

foo_b = 2
foo_a = 1
foo_c = 3
# [15] [foo] グループ2の末尾の dangling。次（inline_val）との間に空行あり → 次の key の前の位置だが、[foo] の末尾として保持

# [16] inline table の leading → 次の key（inline_val）に作用
inline_val = { b = 2, a = 1 }  # [17] inline table の trailing → この key（inline_val）に作用。1 グループのみなのでキー順はディレクティブ or Schema に従う

# [18] array の leading → 次の key（arr）に作用
arr = [  # [19] array の開始ブラケット `[` の trailing。配列全体（arr）に作用
  # [20] 配列・グループ1の先頭の dangling（ディレクティブ）。直後に空行あり → 配列のグループ1全体に作用
  # tombi: format.rules.array-values-order.disabled = true

  "z",
  "y",
  "x"  # [21] 配列・グループ1の trailing（要素 "x" の行）

  # [22] 配列・グループ2の先頭の dangling。ディレクティブなし → グループ2は Schema に従う

  "b",
  "a",
  "c"  # [23] 配列・グループ2の末尾の dangling（最後の要素の次）
  # [24] 配列・グループ2の末尾の dangling。次（閉じ `]`）との間に空行が無い → グループ2に紐づく

  # [25] 配列の空の要素グループに対する dangling comments グループ
]  # [26] array の閉じブラケット `]` の trailing。配列全体（arr）に作用

# [27] ルートと [[item]] の間に key-value が無いため、**ルートの空の dangling comments グループ**

# [28] 1 つ目の [[item]] の leading（ヘッダの直前行）→ この [[item]] に作用
[[item]]
# [29] [[item]] グループ1の先頭の dangling（ディレクティブ）。直後に空行あり → この [[item]] 内グループ1全体に作用
# tombi: format.rules.table-keys-order = "ascending"

item_b = 2
item_a = 1
item_c = 3
# [30] 1 つ目と 2 つ目の [[item]] の間に key-value が無いため、**1 つ目の [[item]] の空の dangling comments グループ**

# [31] 同上（1 つ目の [[item]] の空の dangling comments グループ。[30] と同一グループまたは連続する空グループ）

# [32] 2 つ目の [[item]] の leading（ヘッダの直前行）
[[item]]
# [33] 2 つ目の [[item]] 内の先頭の dangling（このテーブルは 1 グループのみ）
item_k = 1
item_m = 2
item_n = 3

# [34] 2 つ目の [[item]] の空の要素グループに対する dangling comments グループ（要素の直後。次との間に空行なし）

# [35] 2 つ目の [[item]] の空の要素グループに対する dangling comments グループ。[34] と [35] の間に空行あり → 別コメントグループ
```

### 各コメントの扱い（紐付け）

| ラベル | 種類 | 紐づく先 | 備考 |
|--------|------|----------|------|
| [1] | 空の要素グループに対する dangling | **要素が空のグループ** | ファイル先頭から key-value までに [1] と [2] の 2 グループあるため「最後以外」→ 空の要素グループ（[7] と同様） |
| [2] | ルート・グループA 先頭の dangling（ディレクティブ） | **ルート・グループA全体** | key-value の直前にあり直後に空行あり＝「最後の」コメントグループ。グループAに紐づき、ソートを無効化 |
| [3] | trailing | **root_x** | 同一行の key-value に作用 |
| [4] | dangling（次に空行あり） | **ルート・グループB**（先頭の前に配置） | 次のグループに紐づく |
| [5] | ルート・グループB 先頭の dangling（ディレクティブ） | **ルート・グループB全体** | ascending を適用 |
| [6] | ルート・グループB 末尾の dangling | **ルート・グループB** | 次に空行なし。ソートでも位置維持 |
| [7] | dangling（空の要素グループ） | **要素が空のグループ** | 連続 dangling の「最後以外」 |
| [8] | ルート・グループC 先頭の dangling | **ルート・グループC** | ディレクティブなし → Schema に従う |
| [9] | ルートの空の dangling comments グループ | **ルートの空の要素グループ** | ルートと [foo] の間に key-value が無いため。 [1][7] と同様の「空の要素グループ」 |
| [10] | leading | **[foo]**（テーブルヘッダ） | 次のノード（[foo]）に作用 |
| [11] | [foo] グループ1 先頭の dangling（ディレクティブ） | **[foo] グループ1全体** | ソート無効 |
| [12] | trailing | **foo_x** | 同一行の key-value |
| [13] | [foo] グループ1 末尾の dangling | **[foo] グループ1** | 次ノード（foo_b）との間に空行が無い → グループ1に紐づく。ソートでもこの位置を維持 |
| [14] | [foo] グループ2 先頭の dangling | **[foo] グループ2**（先頭の前に配置） | 直前に空行あり（Separator でグループ境界）。ディレクティブなし → グループ2は Schema に従う |
| [15] | [foo] グループ2 末尾の dangling | **[foo] グループ2** | 次（inline_val）との間に空行あり。ここでは [foo] 末尾として保持 |
| [16] | leading | **inline_val**（次の key） | inline table の直前行 |
| [17] | trailing | **inline_val**（この key） | inline table は 1 グループのみ。キー順はディレクティブ or Schema |
| [18] | leading | **arr**（次の key） | 配列の直前行 |
| [19] | trailing | **arr**（配列全体） | array の開始ブラケット `[` の同一行。配列全体に作用 |
| [20] | 配列・グループ1 先頭の dangling（ディレクティブ） | **配列のグループ1全体** | array-values-order を無効化 |
| [21] | trailing | **要素 "x"** | 同一行の配列要素 |
| [22] | 配列・グループ2 先頭の dangling | **配列のグループ2** | ディレクティブなし → Schema |
| [23] | 配列・グループ2 末尾の dangling | **配列のグループ2** | 最後の要素の次 |
| [24] | 配列・グループ2 末尾の dangling | **配列のグループ2** | 次（閉じ `]`）との間に空行が無い → グループ2に紐づく |
| [25] | 配列の空の要素グループに対する dangling comments グループ | **配列の空の要素グループ** | 閉じ `]` との間に空行あり。配列内に要素が無い区間の dangling（[1][7] と同様） |
| [26] | trailing | **arr**（配列全体） | array の閉じブラケット `]` の同一行。配列全体に作用 |
| [27] | ルートの空の dangling comments グループ | **ルートの空の要素グループ** | ルートと [[item]] の間に key-value が無いため |
| [28] | leading | **1 つ目の [[item]]** | ヘッダの直前行 |
| [29] | [[item]] グループ1 先頭の dangling（ディレクティブ） | **1 つ目の [[item]] 内グループ1全体** | ascending |
| [30] | 1 つ目の [[item]] の空の dangling comments グループ | **1 つ目の [[item]] の空の要素グループ** | 1 つ目と 2 つ目の [[item]] の間に key-value が無いため |
| [31] | 同上 | **1 つ目の [[item]] の空の要素グループ** | [30] と同区間（空行で区切られていれば別グループ） |
| [32] | leading | **2 つ目の [[item]]** | ヘッダの直前行 |
| [33] | 2 つ目の [[item]] 先頭の dangling | **2 つ目の [[item]]**（1 グループのみ） | ディレクティブなし → Schema |
| [34] | 2 つ目の [[item]] の空の要素グループに対する dangling comments グループ | **2 つ目の [[item]] の空の要素グループ** | 要素（item_k, item_m, item_n）の直後。次（[35]）との間に空行が無いので 1 つ目のコメントグループ |
| [35] | 2 つ目の [[item]] の空の要素グループに対する dangling comments グループ | **2 つ目の [[item]] の空の要素グループ** | [34] との間に空行あり → 別コメントグループ。ファイル末尾側の空のグループ |

### 自動ソートの適用（スコープ別）

| スコープ | ソート | 理由 |
|----------|--------|------|
| **ルート・グループA** (root_z, root_y, root_x) | **しない** | [2] で `table-keys-order.disabled = true` |
| **ルート・グループB** (root_b, root_a, root_c) | **昇順** | [5] で `table-keys-order = "ascending"` → root_a, root_b, root_c |
| **ルート・グループC** (root_m, root_k, root_n) | **Schema に従う** | ディレクティブなし（例: 昇順なら root_k, root_m, root_n） |
| **[foo] グループ1** (foo_z, foo_y, foo_x) | **しない** | [11] で disabled |
| **[foo] グループ2** (foo_b, foo_a, foo_c) | **Schema に従う** | ディレクティブなし（例: 昇順なら foo_a, foo_b, foo_c） |
| **inline_val**（inline table） | **Schema に従う** または trailing [17] で指定 | 1 グループのみ。ディレクティブが無ければ Schema（例: 昇順なら a, b） |
| **arr グループ1** ("z", "y", "x") | **しない** | [20] で `array-values-order.disabled = true` |
| **arr グループ2** ("b", "a", "c") | **Schema に従う** | ディレクティブなし（例: 昇順なら "a", "b", "c"） |
| **1 つ目の [[item]] グループ1** (item_b, item_a, item_c) | **昇順** | [29] で ascending → item_a, item_b, item_c |
| **2 つ目の [[item]]** (item_k, item_m, item_n) | **Schema に従う** | ディレクティブなし（例: 昇順なら item_k, item_m, item_n） |

### 出力（フォーマット後）

```toml
# ========== ルート ==========
# [1]

# [2]
# tombi: format.rules.table-keys-order.disabled = true

root_z = 3
root_y = 1
root_x = 2  # [3]

# [4]

# [5]
# tombi: format.rules.table-keys-order = "ascending"

root_a = 1
root_b = 2
root_c = 3
# [6]

# [7]

# [8]

root_k = 1
root_m = 2
root_n = 3

# [9]

# [10]
[foo]
# [11]
# tombi: format.rules.table-keys-order.disabled = true

foo_z = 3
foo_y = 1
foo_x = 2  # [12]

# [13]

# [14]

foo_a = 1
foo_b = 2
foo_c = 3
# [15]

# [16]
inline_val = { a = 1, b = 2 }  # [17]

# [18]
arr = [  # [19]
  # [20]
  # tombi: format.rules.array-values-order.disabled = true

  "z",
  "y",
  "x"  # [21]

  # [22]

  "a",
  "b",
  "c"  # [23]
  # [24]
  # [25]
]  # [26]

# [27]

# [28]
[[item]]
# [29]
# tombi: format.rules.table-keys-order = "ascending"

item_a = 1
item_b = 2
item_c = 3
# [30]

# [31]

# [32]
[[item]]
# [33]
item_k = 1
item_m = 2
item_n = 3
# [34]

# [35]
```

→ ルート: グループA 不変、グループB 昇順、グループC Schema（昇順例）。[foo]: グループ1 不変（[13] はグループ1末尾のまま）、グループ2 Schema（昇順例）で [14] はグループ2先頭の前に維持。[15] はグループ2末尾。inline_val: Schema で a, b にソート。arr: [19][26] は配列全体の trailing、[24] はグループ2末尾、[25] は配列の空の要素グループに対する dangling comments グループ。グループ1 不変、グループ2 Schema（昇順例）。1 つ目の [[item]]: 昇順。2 つ目の [[item]]: Schema（昇順例）。[9][27] はルートの空の要素グループ、[30][31] は 1 つ目の [[item]] の空の要素グループ、[34][35] は 2 つ目の [[item]] の空の要素グループに紐づくため、フォーマット後もそれらの位置が維持される。

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
- パーサで空行を `Separator` として保持する必要がある
- フォーマッタのソート関数に「グループ境界」を渡す
- 既存の comment attachment を壊さないよう、キーの移動はグループ内のみ

## スコープ: テーブル以外のソート対象

### 共通方針
- ソート方法の優先順位は **Comment Directives > JSON Schema**（既存ドキュメント通り）。グループ単位で適用する
- 以下、inline table / array of tables / array それぞれで、本仕様（Separator・dangling・グループ先頭/末尾のディレクティブ）をどう当てはめるかを提案する

### Array（format.rules.array-values-order）
- **対象**: 配列の**要素**の並び。複数行にわたる配列では、要素と要素の間に空行を入れ得る
- **グループ**: 要素列を Separator（空行）で分割した区間を 1 グループとする。グループ内の要素のみソートし、グループ間の順序は変えない
- **コメントディレクティブ**:
  - **trailing**: その要素行の末尾 → その要素に作用
  - **leading**: 次の要素の直前行 → 次の要素に作用
  - **dangling**: 前の要素と空行なし → 前のグループに作用。前の要素と空行あり（＝グループ先頭の dangling）→ そのグループ全体の array-values-order に作用。ディレクティブの直後に空行が必要
  - グループ末尾の dangling にディレクティブがあれば、そのグループ全体に作用
- グループにディレクティブが無い場合は JSON Schema の `x-tombi-array-values-order` 等に従う

### Inline table（format.rules.table-keys-order）
- **対象**: `key = { k1 = v1, k2 = v2 }` のようなインライン表の**キー**の並び
- **制約**: TOML のインライン表は 1 行で書くため、**構文上キー間に空行は存在しない**。よってインライン表内のグループは常に **1 グループのみ**
- **コメントディレクティブ**:
  - **trailing**: そのキー行の末尾、または `}` の直前の trailing → そのキーまたはインライン表全体に作用（現行どおり）
  - **leading**: インライン表の直前行、または `{` の直後 → 次のキーまたはインライン表に作用
  - グループ分割がないため「グループ先頭の dangling」は発生しない。ディレクティブは leading/trailing で**そのインライン表（またはキー）**に付く
- ソート無効・ソート方法は、そのインライン表に付いたディレクティブで指定。無ければ Schema に従う

### Array of tables（format.rules.table-keys-order）
- **対象**: 各 `[[section]]` 内の key-value 列。通常の `[table]` と同じ並び
- **グループ**: 各 `[[section]]` の key-values のあいだを Separator（空行）で区切った区間をグループとする。ルートや通常テーブルと同じルール
- **コメントディレクティブ**:
  - **trailing / leading**: その key-value または次の key-value に作用（既存どおり）
  - **dangling**: 前 key-value と空行なし → 前のグループ。空行あり → 次のグループ（グループ先頭の dangling）。グループ先頭の dangling の直後に空行があれば、そのグループ全体の table-keys-order に作用
  - グループ末尾の dangling にディレクティブがあれば、そのグループ全体に作用
- テーブル間（`[[a]]` と `[[b]]` の間）に key-value が無い区間の dangling は、**直前のテーブルの空の要素グループ**に対する dangling（そのテーブルの空の dangling comments グループ）として扱う
- グループにディレクティブが無い場合は JSON Schema に従う
