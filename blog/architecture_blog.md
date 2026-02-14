# Tombi のアーキテクチャ：TOML ファイルを解釈する 4 段階のデータ変換

## はじめに

TOML ファイルを編集する際、エディタが自動的にフォーマットしてくれたり、エラーを教えてくれたり、コード補完が効いたりすることがあります。これらの機能はどのように実現されているのでしょうか？

この記事では、TOML 向けの開発ツールキット「Tombi」がどのようにファイルを解釈し、様々な機能を提供しているのかを、初心者にもわかりやすく解説します。

## Tombi とは

Tombi は、TOML ファイルのための総合的な開発ツールキットです。以下の3つの機能を提供します：

- **Formatter（フォーマッター）**: コードを整形する
- **Linter（リンター）**: エラーや問題点を検出する
- **Language Server（言語サーバー）**: エディタでコード補完やホバー情報などを提供する

Tombi は [Taplo](https://taplo.tamasfe.dev/) というツールに触発されて開発され、AST（抽象構文木）の実装には [Rust Analyzer](https://rust-analyzer.github.io/) の設計を参考にしています。

## Tombi が TOML テキストを解釈する仕組み

Tombi は TOML ファイルを読み込んで解釈する際、段階的に異なるデータ構造へ変換していきます。各段階で異なる目的に特化したデータ構造を使うことで、Formatter、Linter、Language Server といった様々な機能を効率的に実現しています。

```
TOML テキスト → Token[] → AST → DocumentTree → Document
```

それぞれの段階を順番に見ていきましょう。

### 1. Token[]（トークン配列）

**役割**: テキストを最小単位に分解する

TOML ファイルのテキストを読み込むと、まず**字句解析**という処理が行われます。これは、テキストを「意味のある最小単位」に分解する作業です。

例えば、`name = "tombi"` というテキストは、以下のようなトークンに分解されます：

- `name`（識別子）
- `=`（等号）
- `"tombi"`（文字列）

各トークンには、**ファイル内のどこに書かれているか**という位置情報も含まれます。

### 2. AST（抽象構文木）

**役割**: TOML の構文構造を木構造で表現する

トークン配列を構文解析して、TOML の文法に従った**木構造**を作ります。これを抽象構文木（Abstract Syntax Tree）と呼びます。

AST では、以下のような TOML の構文要素がノードとして表現されます：

- キーと値のペア
- テーブル定義
- 配列
- コメント

AST にも位置情報が含まれており、「このキーはファイルの何行目に書かれている」といった情報を保持しています。

**AST が適している処理**: テキスト編集に関わる処理（フォーマット、シンタックスハイライトなど）

### 3. DocumentTree

**役割**: TOML の内容を JSON に近い形式で表現する

AST をさらに変換して、**JSON のようなデータ構造**に近づけたものが DocumentTree です。

なぜこの変換が必要なのでしょうか？それは、Tombi が **JSON Schema** を使ってバリデーション（検証）を行うためです。JSON Schema は広く使われている検証の仕組みで、これを活用するには JSON に近い構造にする必要があります。

DocumentTree の特徴：

- JSON Schema によるバリデーションが可能
- 位置情報を保持している
- **Incomplete（不完全）な状態**を表現できる

最後の点が重要です。コード補完などの機能は、まだ書きかけの不完全な TOML でも動作する必要があります。DocumentTree は、エラーがあっても処理を続けられる設計になっています。

```rust
pub enum Value {
    Boolean(Boolean),
    Integer(Integer),
    Float(Float),
    String(String),
    OffsetDateTime(OffsetDateTime),
    LocalDateTime(LocalDateTime),
    LocalDate(LocalDate),
    LocalTime(LocalTime),
    Array(Array),
    Table(Table),
    Incomplete { range: tombi_text::Range },  // 不完全な状態を表現
}
```

**DocumentTree が適している処理**: スキーマを使った検証、コード補完、ホバー情報など

### 4. Document

**役割**: 完全にデシリアライズされたデータ

DocumentTree をさらに変換したものが Document です。これは、位置情報を持たない純粋なデータ構造で、`serde_json::Value` のように Rust のデータ構造として TOML を読み取る際に使用されます。

例えば、`tombi.toml` という設定ファイルを Rust のプログラムで読み込む場合に利用されます。

## 各機能での利用方法

### Formatter（フォーマッター）

フォーマッターは、コードを整形する機能です。主に **AST** を使用します。

```rust
fn format(source: &str, config: &Config) -> Result<String, Vec<Error>> {
    let ast = tombi_parser::parse(source)?;
    // AST Editor がスキーマ情報を使って AST を編集（自動ソートなど）
    let edited_ast = edit_with_schema(ast, schema_context)?;
    // 編集された AST を使って整形処理を行う
    format_ast(edited_ast)
}
```

Tombi の Formatter は、JSON Schema からキーの順序情報を取得し、**AST Editor** が AST のノードを並び替えます。この自動ソート機能により、スキーマで定義された順序でキーやテーブルを並べ替えることができます。

ただし、この自動ソートには技術的な課題があります。AST はテキストの記述順序を保持しているため、ノードを並び替えるとコメントディレクティブの適用順序も変わってしまいます。

```toml
[aaa]
# tombi: format.rules.table-keys-order = "preserve"
[aaa.ccc]

# tombi: format.rules.table-keys-order = "ascending"
[aaa.bbb]
```

このような場合、自動ソートで `[aaa.bbb]` が `[aaa.ccc]` より前に移動すると、コメントディレクティブの適用順序も変わり、期待しない動作になる可能性があります。これは、AST の「記述順序」を保持する性質と、データとしての「論理的な構造」のギャップから生じる問題です。

### Linter（リンター）

リンターは、エラーや問題点を検出する機能です。主に **DocumentTree** を使用します。

```rust
fn lint(source: &str) -> Result<(), Vec<Diagnostic>> {
    let ast = tombi_parser::parse(source)?;
    let document_tree = tombi_document::try_from(&ast).map_err(|errors|
      errors.into_iter().map(Diagnostic::from).collect_vec()
    )?;

    validate(&document_tree)  // JSON Schema でバリデーション
}
```

JSON Schema を使った検証を行うため、DocumentTree への変換が必須です。

### Language Server（言語サーバー）

Language Server は、エディタでコード補完やホバー情報などを提供する機能です。**AST、DocumentTree、スキーマ情報**を使い分けます。

各機能で使用するデータ構造：

| LSP 機能               | 利用データ構造  |
|------------------------|-----------------|
| `textDocument/formatting`          | AST, スキーマ情報 |
| `textDocument/diagnostic`          | DocumentTree      |
| `textDocument/completion`          | DocumentTree      |
| `textDocument/hover`               | DocumentTree      |
| `textDocument/definition`          | DocumentTree      |
| `textDocument/semanticTokens/full` | AST               |

**使い分けのポイント**：

- テキスト編集を行う場合 → **AST**
- スキーマをもとに判断する必要がある場合 → **DocumentTree**
- スキーマ情報を使った AST 編集が必要な場合 → **AST + スキーマ情報**

例えば、シンタックスハイライト（`semanticTokens/full`）はテキストの装飾なので AST を使い、コード補完（`completion`）はスキーマから候補を探すので DocumentTree を使います。フォーマット（`formatting`）はスキーマに基づいてキーを並べ替えた後に AST を整形します。

## 不完全な構文への対応

Language Server の大きな課題は、**まだ書きかけのコードでも動作する**必要があることです。

例えば、以下のような不完全な TOML を考えてみましょう：

```toml
[package]
name =
version = "0.1.0"
```

name に対応する値が入っていません。しかし、エディタではこの状態でもコード補完やエラー表示が期待されます。

Tombi はこの問題を、DocumentTree の **Incomplete** 状態で解決しています。DocumentTree への変換中にエラーが発生しても、エラーを記録しつつ処理を続行します。これにより、不完全なノードの状態でもホバー情報やコード補完を提供できます。

## データ構造の設計思想：2つの世界の行き来

Tombi の設計の核心は、**AST の世界**と**データの世界**という2つの異なる世界を行き来する点にあります。

### AST の世界：記述順序を保持する構文の世界

AST は、テキストファイルとしての TOML を忠実に表現します。重要な特徴は：

- **記述順序を保持**: ファイルに書かれた順序そのままを保持
- **位置情報を保持**: すべてのノードがファイル内の位置を持つ
- **高い表現力**: コメントの位置関係など、細かい情報まで保持可能

例えば、以下のような TOML では、複数のコメントの位置関係を正確に保持できます：

```toml
# comment1
[aaa]

# comment2
[aaa.bbb]
ccc = 1

# comment3
[aaa.bbb.ddd]
```

この表現力の高さにより、フォーマット機能では元のテキスト構造を保ちながら整形できます。

### データの世界：論理的な構造を表現する世界

一方、DocumentTree と Document は、TOML を JSON に近いデータ構造として表現します：

- **論理的な構造**: キーと値の階層関係を表現
- **スキーマ検証に最適**: JSON Schema による検証が可能
- **情報の圧縮**: コメント情報そのものではなく、解析済みのコメントディレクティブのみを保持

データモデルでは、上記の例の `comment1`、`comment2`、`comment3` をどこに帰属させるかが曖昧になります。このため、DocumentTree では**コメントディレクティブとして解析済みの情報のみ**を保持し、表現力を抑える代わりにスキーマ検証を可能にしています。

### 2つの世界のギャップと課題

この2つの世界を行き来することで、様々な機能を実現できる一方、ギャップによる課題も存在します。

最も顕著な例が、Formatter の自動ソート機能です。AST の記述順序を変更すると、コメントディレクティブの適用順序も変わってしまいます（前述の Formatter セクション参照）。

また、データモデルからテキストへの逆変換（シリアライズ）では、コメントをどの位置に配置すべきか判断できません。そのため、Tombi では AST からのフォーマットを基本とし、データモデルはバリデーションと読み込みに特化させています。

このように、Tombi は2つの世界それぞれの強みを活かしながら、ギャップによる制約を理解した上で機能を提供しています。

## まとめ

Tombi は、**AST の世界**と**データの世界**という2つの異なる世界を行き来することで、Formatter、Linter、Language Server という多様な機能を提供しています。

- **AST の世界**: 記述順序と位置情報を保持し、テキスト編集に強い
- **データの世界**: 論理的な構造を表現し、スキーマ検証に強い。不完全な構文にも対応
- **2つの世界のギャップ**: 自動ソートにおけるコメントディレクティブの適用順序問題など、課題も存在する
- **各機能の設計**: それぞれの世界の強みを活かし、制約を理解した上で実装されている

この設計により、Tombi は高速で堅牢な TOML 開発体験を提供しています。一方で、2つの世界のギャップから生じる技術的な課題も含め、実装の詳細が興味深いアーキテクチャとなっています。

開発当初は 5 段階のレイヤーがありましたが、改良を重ねて現在の 4 段階構成に落ち着きました。AST とデータという2つの世界を明確に分離しつつ、それらを効果的に連携させる設計は、TOMLツールキットとしての完成度の高さを示しています。

---

**参考**:
- Tombi は [Taplo](https://taplo.tamasfe.dev/) に触発されて開発されました
- AST の実装は [Rust Analyzer](https://rust-analyzer.github.io/) を参考にしています
- JSON Schema による検証には [JSON Schema Store](https://www.schemastore.org/) を活用しています
