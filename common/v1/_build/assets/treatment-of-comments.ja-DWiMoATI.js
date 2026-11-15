import{M as u}from"./index-6OQr06v0.js";import{b as n,k as l}from"./web-DCz9_uPG.js";function o(i){const e={a:"a",br:"br",code:"code",h1:"h1",h2:"h2",h3:"h3",li:"li",p:"p",strong:"strong",ul:"ul",...u(),...i.components},{CodeBlock:r,PageHeading:a}=e;return r||t("CodeBlock"),a||t("PageHeading"),[n(a,{title:"Tombi",description:"a TOML toolkit that provides Formatter, Linter, and Language Server.",og_url:"https://tombi-toml.github.io/tombi/common/v1/docs/formatter/treatment-of-comments.ja"}),`
`,n(e.h1,{children:"コメントの扱い"}),`
`,n(e.p,{children:"Tombi は、フォーマットの安定性と自動ソートの予測可能性を保つために、コメントをグループ単位で扱います。"}),`
`,n(e.p,{children:"このページでは、実装詳細ではなく、TOML の具体例で挙動を説明します。"}),`
`,n(e.h2,{children:"要点"}),`
`,n(e.ul,{get children(){return[`
`,n(e.li,{children:"Tombi は、空行で分かれたコメントブロックを意味のある境界として扱います。"}),`
`,n(e.li,{children:"ソートは各グループ内でのみ行われ、グループ境界をまたいで並び替えません。"}),`
`,n(e.li,{get children(){return[n(e.code,{children:"# tombi:"})," ディレクティブは、適用対象が明確な位置に置かれた場合にのみ formatter/linter のルール制御に使われます。"]}}),`
`]}}),`
`,n(e.h2,{id:"toml",className:"group relative heading-with-anchor",children:"TOML におけるコメントの役割"}),`
`,n(e.h3,{id:"leading-comment",className:"group relative heading-with-anchor",children:"leading comment"}),`
`,n(e.p,{children:"直下の要素に直接ひも付くコメントです。"}),`
`,n(r,{code:`# leading comment 1
# leading comment 2
key = "value"`,language:"toml"}),`
`,n(e.h3,{id:"trailing-comment",className:"group relative heading-with-anchor",children:"trailing comment"}),`
`,n(e.p,{children:"同じ行の末尾にあるコメントです。"}),`
`,n(r,{code:'key = "value" # trailing comment',language:"toml"}),`
`,n(e.h3,{id:"dangling-comment-group",className:"group relative heading-with-anchor",children:"dangling comment group"}),`
`,n(e.p,{children:"空行で区切られたコメントブロックです。視覚的・論理的な境界として機能します。"}),`
`,n(r,{code:`# dangling comment group A line 1
# dangling comment group A line 2

# dangling comment group B line 1
# dangling comment group B line 2`,language:"toml"}),`
`,n(e.h2,{children:"空行は境界を定義する"}),`
`,n(e.p,{children:"空行はグループ間の境界として扱われます。"}),`
`,n(e.p,{children:"この境界が重要なのは、Tombi がソート時に値やキーをこの境界の外へ移動させないためです。"}),`
`,n(r,{code:`[tool.demo]
b = 2
a = 1

z = 26
y = 25`,language:"toml"}),`
`,n(e.p,{get children(){return["Tombi は各グループ内（",n(e.code,{children:"b/a"})," と ",n(e.code,{children:"z/y"}),"）で並び替えることはありますが、境界をまたいで結合・移動はしません。"]}}),`
`,n(e.h2,{id:"tombi",className:"group relative heading-with-anchor",get children(){return[n(e.code,{children:"# tombi:"})," ディレクティブが有効になる場所"]}}),`
`,n(e.p,{children:"ルールはシンプルです。"}),`
`,n(e.ul,{get children(){return[`
`,n(e.li,{children:"有効: そのコンテナの最初の要素より前"}),`
`,n(e.li,{children:"ルール制御としては無効: 要素出現後のグループ境界上"}),`
`]}}),`
`,n(e.p,{children:"以下に具体例を示します。"}),`
`,n(e.h3,{id:"root",className:"group relative heading-with-anchor",children:"Root レベルの例"}),`
`,n(e.p,{children:"Root で有効（最初のキーより前）:"}),`
`,n(r,{code:`# root の directive: ルートテーブルに有効
# tombi: format.rules.table-keys-order = "ascending"

b = 2
a = 1`,language:"toml"}),`
`,n(e.p,{children:"Root で無効（すでに root の要素が出現した後）:"}),`
`,n(r,{code:`b = 2
a = 1

# root の group-boundary directive:
# 空の key-value group に対する directive として解釈される
# （ルール制御には使われない）
# tombi: format.rules.table-keys-order = "ascending"

z = 26
y = 25`,language:"toml"}),`
`,n(e.p,{get children(){return["この位置では、Tombi はこの directive を境界上の ",n(e.strong,{children:"空の key-value group"})," に属するものとして解釈します。",n(e.br,{}),`
`,"そのためツール側では認識されますが、formatter/linter のルール制御には影響しません。"]}}),`
`,n(e.h3,{id:"table",className:"group relative heading-with-anchor",children:"通常の Table の例"}),`
`,n(e.p,{get children(){return[n(e.code,{children:"[package]"})," に有効（この table の最初のキーより前）:"]}}),`
`,n(r,{code:`[package]
# package の directive: この table に有効
# tombi: format.rules.table-keys-order = "ascending"

name = "demo"
version = "1.0.0"`,language:"toml"}),`
`,n(e.p,{get children(){return[n(e.code,{children:"[package]"})," に無効（キー出現後のグループ境界）:"]}}),`
`,n(r,{code:`[package]
name = "demo"
version = "1.0.0"

# package の group-boundary directive: ルール制御には使われない
# tombi: format.rules.table-keys-order = "ascending"

description = "example"
license = "MIT"`,language:"toml"}),`
`,n(e.h3,{id:"array",className:"group relative heading-with-anchor",children:"Array の例"}),`
`,n(e.p,{children:"Array に有効（最初の値より前）:"}),`
`,n(r,{code:`items = [
  # array の directive: この array に有効
  # tombi: format.rules.array-values-order = "ascending"

  "banana",
  "apple",
]`,language:"toml"}),`
`,n(e.p,{children:"Array に無効（値出現後のグループ境界）:"}),`
`,n(r,{code:`items = [
  "banana",
  "apple",

  # array の group-boundary directive: ルール制御には使われない
  # tombi: format.rules.array-values-order = "ascending"

  "date",
  "cherry",
]`,language:"toml"}),`
`,n(e.h3,{id:"inline-table",className:"group relative heading-with-anchor",children:"Inline Table の例"}),`
`,n(e.p,{children:"Inline Table に有効（その inline table 内の最初の key-value より前）:"}),`
`,n(r,{code:`point = {
  # inline table の directive: この inline table に有効
  # tombi: format.rules.table-keys-order = "ascending"

  y = 20,
  x = 10,
}`,language:"toml"}),`
`,n(e.p,{children:"Inline Table に無効（key-value 出現後のグループ境界）:"}),`
`,n(r,{code:`point = {
  y = 20,
  x = 10,

  # inline table の group-boundary directive: ルール制御には使われない
  # tombi: format.rules.table-keys-order = "ascending"

  z = 30,
}`,language:"toml"}),`
`,n(e.h3,{id:"array-of-tables",className:"group relative heading-with-anchor",children:"Array of Tables の例"}),`
`,n(e.p,{get children(){return["各 ",n(e.code,{children:"[[servers]]"})," ブロックで有効なのは、そのブロックの最初のキーより前に置いた場合のみです。"]}}),`
`,n(r,{code:`[[servers]]
# array of tables の directive: この table item に有効
# tombi: format.rules.table-keys-order = "ascending"

port = 8080
host = "localhost"`,language:"toml"}),`
`,n(e.p,{children:"キー出現後のグループ境界に置いた場合は無効です。"}),`
`,n(r,{code:`[[servers]]
port = 8080
host = "localhost"

# array of tables の group-boundary directive: ルール制御には使われない
# tombi: format.rules.table-keys-order = "ascending"

protocol = "http"`,language:"toml"}),`
`,n(e.h2,{children:"無効な位置のディレクティブはどうなるか"}),`
`,n(e.p,{children:"グループ境界上（ルール制御として無効な位置）にディレクティブを置いた場合:"}),`
`,n(e.ul,{get children(){return[`
`,n(e.li,{children:"Formatter のルール制御: 利用できるルールがない"}),`
`,n(e.li,{children:"Linter のルール制御: 利用できるルールがない"}),`
`,n(e.li,{children:"エディター支援（completion/hover/validation）: 利用できる"}),`
`]}}),`
`,n(e.p,{children:"つまり、Language Server の補助は得られますが、フォーマット/リントの挙動は変わりません。"}),`
`,n(e.h2,{children:"注釈付きの通し例"}),`
`,n(r,{code:`# [root の directive: 有効]
# tombi: format.rules.table-keys-order = "ascending"

# [root dangling comment group]

[package]
# [table の directive: package table に有効]
# tombi: format.rules.table-keys-order = "ascending"

name = "demo"
version = "1.0.0"

# [table の group-boundary directive: ルール制御には無効]
# tombi: format.rules.table-keys-order = "descending"

description = "example"

items = [
  # [array の directive: この array に有効]
  # tombi: format.rules.array-values-order = "ascending"

  "b",
  "a",

  # [array の group-boundary directive: ルール制御には無効]
  # tombi: format.rules.array-values-order = "descending"

  "d",
  "c",
]`,language:"toml"}),`
`,n(e.p,{children:"読み方:"}),`
`,n(e.ul,{get children(){return[`
`,n(e.li,{children:"「有効」と書かれた directive コメントはルール制御に使われます。"}),`
`,n(e.li,{children:"「group-boundary directive」と書かれたコメントは境界コメントとして扱われ、formatter/linter のルール制御には使われません。"}),`
`]}}),`
`,n(e.p,{get children(){return["ディレクティブの構文や利用可能なオプションは、",n(e.a,{href:"/tombi/common/v1/docs/comment-directive/",children:"Comment Directive"})," を参照してください。"]}})]}function m(i={}){const{wrapper:e}={...u(),...i.components};return e?n(e,l(i,{get children(){return n(o,i)}})):o(i)}function t(i,e){throw new Error("Expected component `"+i+"` to be defined: you likely forgot to import, pass, or provide it.")}export{m as default};
