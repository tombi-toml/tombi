# Tombi のデータ構造

## LSP とは
説明を書く。

## Formatter/Linter/LSP に要求される
### Formatter
単一のファイルのみで処理できるため、ファイルを横断した依存グラフのデータベースを持つ必要がなく楽
公式のパーサーを再利用して実装するものも多い。
ただし、公式パーサがコメントをパース結果から除外する場合は、独自のコメント処理が必要になる。

### Linter
対応したい言語の公式パーサはコメントを除外するものが一定数存在する。
コメントディレクティブなどを考慮しなければ、既存のパーサでも再利用できる。

### LSP 
コード補完など、不完全な構文でも動作する必要があるため、従来のパーサとは異なる実装が必要になる場合が多い。

Rust のように、コンパイル用のパーサーと、LSP用のパーサーが別に実装されている言語もあるが、
新し目の言語では一つのパーサーで両方を賄うものが存在する。

## Tombi のデータ構造

Tombi は Taplo に触発されて開発された Formatter/Linter/Language Server を包含したツールキットです。

AST の実装を学習するため、 Rust Analyzer を参考に作成されました。

Tombi は下記のような段階的なデータ構造のリレーによって、データをシリアライズ化でき、Language Server は各機能が求めるデータ構造を利用して機能を提供しています。

Token[] -> AST -> DocumentTree -> Document

開発当初は、レイヤーがもう一つありましたが、縮退させて現在の4段階構成に落ち着いています。

各データは下記のとおりです。

| データ構造 | 説明 |
|-------------|------|
| Token[]     | 字句解析の結果得られる最小単位のデータ。位置情報を含む。 |
| AST         | 抽象構文木。TOML の構文要素をノードとして持つ。位置情報を含む。 |
| DocumentTree| AST を基に、TOML のドキュメント構造を表現したデータ。位置情報を含む JSON データのようなもの。 |
| Document    | DocumentTree を基に、コメントやホワイトスペースを含む完全なドキュメント構造を表現したデータ。位置情報を含まない。JSON データのようなもの |

Tombi の LSP の機能はほとんどが２種類のデータ構造、 AST と DocumentTree を利用して機能を提供しています。
※ Document は `tombi.toml` のような TOML テキストを Rust のデータ構造として読み取るために利用されます（`serde_json::Value` のようなもの）。

この利用するデータ構造を利用する視点は、テキスト編集を行う場合は AST、スキーマをもとに判断する必要がある場合は DocumentTree を利用する、というものです。

LSP の機能表と利用しているデータ構造の対応表は下記

| LSP 機能               | 利用データ構造  |
|------------------------|-----------------|
| `textDocument/formatting`          | AST, DocumentTree |
| `textDocument/diagnostic`          | DocumentTree      |
| `textDocument/completion`          | DocumentTree      |
| `textDocument/hover`               | DocumentTree      |
| `textDocument/definition`          | DocumentTree      |
| `textDocument/semanticTokens/full` | AST               |


これは、背景として Taplo と同様に JSON Schema Store から JSON Schema を取得し、TOML ドキュメントのスキーマバリデーションを行う関係上、
JSON に近いデータ構造に変換しないとバリデーションの適用ができないため、
より冗長な表現を持つ AST ではなく、 DocumentTree に変換してから JSON Schema を適用する実情が影響しています。

### Tombi Formatter

Fotmatter はテキストの整形なので AST を基本とした実装が行われます。

ただし、Tombi は JSON Schema の情報から自動ソートの情報を取得するため、
DocumentTree も同時に生成し、利用しています。

簡単に説明すると、下記のような関数シグネチャになります。

```rust
mod tombi_formatter {
  fn format(source: &str, config: &Config) -> Result<String, Vec<Error>> {
    let ast = tombi_parser::parse(source)?;
    ...
  }
}
```

### Tombi Linter

Linter はスキーマバリデーションを行うため、 DocumentTree を作成して実装されています。

DocumentTree への変換でエラーが発生した場合は、その時点でエラーを返し、
変換に成功した場合は、 JSON Schema を利用したバリデーションを行います。

```rust
mod tombi_linter {
  fn lint(source: &str) -> Result<(), Vec<Diagnostic>> {
    let ast = tombi_parser::parse(source)?;
    let document_tree = tombi_document::try_from(&ast).map_err(|errors|
      errors.into_iter().map(Diagnostic::from).collect_vec()
    )?;

    validate(&document_tree)
  }
}
```

### Tombi Language Server
コード補完のような機能は、TOML として不完全な構文でも動作する必要があるため、 DocumentTree への変換が失敗した状態でも動作するように作られています。

これは、 DocumentTree そのものがエラーが発生した場合のケースを想定した設計になっており、 Incomplete という状態を持つことができるためです。

```rust
#[derive(Debug, Clone, PartialEq)]
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
    Incomplete { range: tombi_text::Range },
}
```

DocuentTree への変換の途中でエラーが発生しても、エラーを追加した上で処理は続けられ、エラーが０件でなければ不完全な TOML ドキュメントであることがわかります。

これにより、不完全なノードの状態でもホバー情報やコード補完を提供できます。
