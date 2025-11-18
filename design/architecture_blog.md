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

## 4 段階のデータ変換フロー

Tombi の最大の特徴は、TOML ファイルを段階的に異なるデータ構造へ変換していく点です。このアプローチにより、各機能に最適なデータ構造を選択できます。

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
    // AST を使って整形処理を行う
}
```

ただし、Tombi は JSON Schema から自動ソートの情報を取得するため、**DocumentTree も同時に生成**します。これにより、スキーマで定義された順序でキーをソートすることができます。

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

Language Server は、エディタでコード補完やホバー情報などを提供する機能です。**AST と DocumentTree の両方**を使い分けます。

各機能で使用するデータ構造：

| LSP 機能               | 利用データ構造  |
|------------------------|-----------------|
| `textDocument/formatting`          | AST, DocumentTree |
| `textDocument/diagnostic`          | DocumentTree      |
| `textDocument/completion`          | DocumentTree      |
| `textDocument/hover`               | DocumentTree      |
| `textDocument/definition`          | DocumentTree      |
| `textDocument/semanticTokens/full` | AST               |

**使い分けのポイント**：

- テキスト編集を行う場合 → **AST**
- スキーマをもとに判断する必要がある場合 → **DocumentTree**

例えば、シンタックスハイライト（`semanticTokens/full`）はテキストの装飾なので AST を使い、コード補完（`completion`）はスキーマから候補を探すので DocumentTree を使います。

## 不完全な構文への対応

Language Server の大きな課題は、**まだ書きかけのコードでも動作する**必要があることです。

例えば、以下のような不完全な TOML を考えてみましょう：

```toml
[package]
name = "tom
```

最後の文字列が閉じられていません。しかし、エディタではこの状態でもコード補完やエラー表示が期待されます。

Tombi はこの問題を、DocumentTree の **Incomplete** 状態で解決しています。DocumentTree への変換中にエラーが発生しても、エラーを記録しつつ処理を続行します。これにより、不完全なノードの状態でもホバー情報やコード補完を提供できます。

## データ構造の設計思想

Tombi が 4 段階のデータ構造を採用している理由をまとめます：

1. **Token[]**: テキストの最小単位。すべての始まり
2. **AST**: 構文構造を保持。テキスト編集に最適
3. **DocumentTree**: JSON に近い構造。スキーマ検証に最適。不完全な状態も表現可能
4. **Document**: 純粋なデータ。設定ファイルの読み込みに最適

各機能が必要とするデータ構造だけを使用することで、効率的な実装を実現しています。

## まとめ

Tombi は、TOML ファイルを段階的に変換することで、Formatter、Linter、Language Server という 3 つの機能を効率的に提供しています。

- **AST** はテキスト編集に強い
- **DocumentTree** はスキーマ検証に強く、不完全な構文にも対応できる
- 各機能は最適なデータ構造を選択して実装されている

この設計により、Tombi は高速で堅牢な TOML 開発体験を提供しています。

開発当初は 5 段階のレイヤーがありましたが、改良を重ねて現在の 4 段階構成に落ち着いたとのこと。シンプルさと機能性のバランスが取れた、洗練されたアーキテクチャと言えるでしょう。

---

**参考**:
- Tombi は [Taplo](https://taplo.tamasfe.dev/) に触発されて開発されました
- AST の実装は [Rust Analyzer](https://rust-analyzer.github.io/) を参考にしています
- JSON Schema による検証には [JSON Schema Store](https://www.schemastore.org/) を活用しています
