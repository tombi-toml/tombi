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
- 連続する空行を `Separator(n)` として保持 (n は連続数)
- `Separator` はキーに紐付かず、**グループ境界**として扱う

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
- **dangling comment の紐付け先は「直前ノード or 次グループ」**
- **直前ノードとの間に空行が無ければ**直前ノードに紐付ける
- **直前ノードとの間に空行があれば**次グループに紐付ける
- 次グループに紐付く場合は、**グループ先頭ノードの直前**に配置

### 6. コメントディレクティブの作用ノード
- ディレクティブは **「最も近い構文ノード」** に作用させる
- ただし `Separator` を跨いだ作用は禁止
- 優先順位 (上から順に適用):
  1. **同一行 trailing**: その行のノード (同一行の key/value)
  2. **直前行の inline/leading**: 次のノード
  3. **dangling**:
     - 空行なし: 直前ノードに作用
     - 空行あり: 次グループ先頭ノードに作用

#### 例: trailing
```toml
key1 = "a" # tombi: format.disabled
```
→ `key1` に作用

#### 例: leading (次ノードに作用)
```toml
# tombi: format.disabled
key2 = "b"
```
→ `key2` に作用

#### 例: dangling (グループ境界)
```toml
key1 = "a"
# tombi: format.disabled

key2 = "b"
key3 = "c"
```
→ `key1` に紐付く (直前ノードとの間に空行が無い)

#### 例: dangling (空行を挟む)
```toml
key1 = "a"

# tombi: format.disabled
key2 = "b"
key3 = "c"
```
→ `key2` と `key3` のグループに紐付く
→ グループ先頭 (`key2`) の前に配置される

#### 例: グループ跨ぎ禁止
```toml
key1 = "a"
# tombi: format.disabled

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
↓ (1グループ目: 無効 / 2グループ目: 有効 / 3グループ目: Schema 指示)
```toml
# tombi: format.rules.table-keys-order.disabled = true
b = 2
a = 1

# tombi: format.rules.table-keys-order = "ascending"
y = 2
z = 3

<schema-order>
```
→ 各ディレクティブは **該当グループのみ**に適用される

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

## 代替案 (非推奨)

### A. 空行をコメントと同等に扱う
- コメント再配置と同様の問題が発生するため不適

### B. 空行の数を保持せず最大値だけ保持
- グループ意図が失われるため不可

## 実装上のポイント
- パーサで空行を `Separator` として保持する必要がある
- フォーマッタのソート関数に「グループ境界」を渡す
- 既存の comment attachment を壊さないよう、キーの移動はグループ内のみ

## 未解決の論点
- schema 側で定義されるソート順とグループ分割の整合
- 複合キー (inline table, array of tables) でのグループ判定
- 連続空行数の正規化 (最大 1 で良いか)
