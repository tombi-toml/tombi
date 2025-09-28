# コメントディレクティブのサポート

コメントディレクティブの実装は非常に複雑なものになっている。

これはコメントディレクティブに対しても補完やバリデーションが効くように設計されているためでもあるが、
根本的には、 TOML が一意にデータ構造を定めることができない表現方法を持っているため、コメントの適用範囲の優先度を決定することが困難なためである。

## はじめに
TOML は一つのデータ構造に対して複数の表現方法を持つ。

```toml
# Pattern 1
aaa.bbb.ccc = true

# Pattern 2
[aaa]
bbb.ccc = true

# Pattern 3
[aaa.bbb]
ccc = true

# Pattern 4
aaa = { bbb = { ccc = true } }

# Pattern 5
aaa = { bbb.ccc = true }
```

そのため、 Tombi では、TOML の表現構造を保持する tombi-ast と、
TOML のデータ構造を保持する tombi-document-tree の2つの構造を利用することで、
様々な機能の提供を行っている。

コメントディレクティブを利用することで、 Toml のバリデーションエラーを無視することができるが、
バリデーションは JSON Schema によって行われるため、 document-tree の構造を利用する必要がある。

この２種類の構造の表現力の違いが、コメントディレクティブの設計において問題を発生させる。

## テーブルの親キーにはどのコメントディレクティブを適用すべきか？
tombi-ast は tombi-document-tree よりも多くのコメント情報を保持するため、
どのコメントが、どの document-node に適用されるかを判断する必要がある。

この処理に複雑さが生じる。

```toml
# tombi: lint.rules.table-min-keys.disabled = true
[aaa.bbb.ccc]
ddd.eee.fff = true
```

この場合、 `aaa.bbb.ccc` に適用されるコメントは、どのテーブルに対して効果を及ぼすのだろうか？

- `aaa`
- `bbb`
- `ccc`

最も確実なのは、 `ccc` である。そして、暗黙的に `aaa` と `bbb` にも適用されることが期待される。

では次はどうか？

```toml
# tombi: lint.rules.table-min-keys.disabled = true
[aaa.bbb.ccc]
ddd.eee.fff = true

# tombi: lint.rules.table-max-keys.disabled = true
[aaa.bbb.ggg]
hhh.iii.jjj = true
```

この場合、 `ccc` に table-min-keys が、 `aaa.bbb.ggg` に table-max-keys で適用されることは許可してよかろう。

では、 `aaa` と `bbb` は何を適用すべきだろうか？

上の話題を拡張すると、 table-min-keys と table-max-keys の両方を適用すべきであろう。

別のケースとして、複数のテーブルに同じコメントディレクティブが指定された場合、コメントディレクティブは結合する必要がある。

幸い、現在の設定では異なる矛盾した指示を Linter で指定することはできない。
（ `disabled = true` と `disabled = false` のような指示はできない。バリデーションは true のみを許可するからだ）

## 親キーに対してコメントディレクティブを適用したくない場合

先ほどの例を再度取り上げる。

```toml
# tombi: lint.rules.table-min-keys.disabled = true
[aaa.bbb.ccc]
ddd.eee.fff = true
```

これは、 `aaa`, `bbb`, `ccc` に対してコメントディレクティブを適用しているが、
`aaa` と `bbb` には適用したくない場合がある。

その場合はどうするか。

これは、以下のように記述することで解決する。

```toml
[aaa.bbb]

# tombi: lint.rules.table-min-keys.disabled = true
[aaa.bbb.ccc]
ddd.eee.fff = true
```

これは、より親の階層である `[aaa.bbb]` のノードではコメントディレクティブを適用しないようにすることで、コメントディレクトリが適用されないようにすることができる。
（空のコメントディレクトリの行を記述する）

こうすることで、親ノードへのコメントディレクティブの伝搬を打ち切ることができる。

## 自動ソートの場合どうする？

例えば、次のような状況を考える。

```toml
# tombi: lint.rules.table-max-keys.disabled = true
[[aaa.bbb]]
order = "2"

[[aaa.bbb]]
order = "1"
```

これは１つ目の `bbb` のテーブルに対してコメントディレクティブを適用しているため、
２つ目の `bbb` のテーブルにはコメントディレクティブを記述する必要がない。
先勝ちのためそうなる。

しかし、このファイルの JSON Schema が `bbb` のテーブルを `order` でソートする場合、結果は次のようになる。

```toml
[[aaa.bbb]]
order = "1"

# tombi: lint.rules.table-max-keys.disabled = true
[[aaa.bbb]]
order = "2"
```

自動ソートの結果は正しいが、先勝ちのアルゴリズムでコメントディレクティブを適用する場合、自動ソートの結果バリデーションエラーが発生する。

先勝ちではコメントディレクティブが発生しないためである。

そのため、コメントディレクティブは、同レベルの階層（keys が同じもの）に対しては順序に依存してはいけない。

順番に依存してはならないことは、非常に強い制約である。
これは、現状コメントディレクティブが矛盾のある指事を指定できないため実現可能である。

format.rules は自動ソートの順番を矛盾のある形で指定できるが、
lint のコメントディレクティブの指定方法とは異なる適用範囲を持つように設計されているため、問題を回避できることに注意。

例えば、

```toml
# tombi: format.rules.table-keys-order = "ascending"
[[aaa.bbb]]
order = "2"

# tombi: format.rules.table-keys-order = "descending"
[[aaa.bbb]]
order = "1"

# tombi: format.rules.table-keys-order = "version-sort"
[[aaa.bbb]]
order = "3"
```

これは、矛盾した指定のように見えるが、このコメントは各テーブルに対して適用される。

すなわち、実際の解釈としては

```toml
[[aaa.bbb]]
# tombi: format.rules.table-keys-order = "ascending"

order = "2"

[[aaa.bbb]]
# tombi: format.rules.table-keys-order = "descending"

order = "1"

[[aaa.bbb]]
# tombi: format.rules.table-keys-order = "version-sort"

order = "3"
```

であるため、コメントディレクティブの競合は発生しない。

Table や Array of Tables に対してコメントディレクティブによる自動ソートをしたい場合は、ルートに対してコメントディレクティブを指定する必要がある。

```toml
# tombi: format.rules.table-keys-order = "version-sort"

[[aaa.bbb]]
order = "2"

[[aaa.bbb]]
order = "1"

[[aaa.bbb]]
order = "3"
```

このため、コメントディレクティブの format には矛盾する指定項目があるが、実際には競合が発生しないのである。
