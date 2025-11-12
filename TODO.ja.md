## TODO
### Milestone 1
- [x] parser を書き直す
  - [x] エラー出力のロジックの改善
  - [x] Red-Tree のPosition, Range のバグ修正
  - [x] Lexed の切り離し（events を inputs ではなく outputs の配列を参照させる）。
  - [x] dotted keys のサポート
- [x] Document のサポート。
- [x] https://github.com/toml-lang/toml-test に対応

### Milestone 2
- [x] JSON Schema のサポート
- [x] document site の立ち上げ

### Milestone 3
- [x] ast-editor の実装
    - [x] テーブルのキーの並び替えに対応
    - [x] 末尾カンマとコメントの関係の差し替え

### Milestone 4
- [x] serde-tombi を内部用に作成し、 TOML の Preview バージョンをパースできるように修正
- [x] JSON Schema への「定義へ移動」機能の追加

### Milestone ???
- [ ] WASM サポート & ドキュメントサイトの Playground 作成
- [x] Cargo.toml のなどの特別な機能追加

### Bugs
- [x] Local Date 型が誤って IntegerDec としてパースされる
- [x] Keys に float や int を使った場合、誤ってパースされる
    - [x] 3.14 を keys に使った場合、3 と 14 の key としてパースされる
    - [x] 3 を keys に使った場合、3 の key としてパースされる
    - [x] inf, nan を keys に使った場合、key としてパースされる
- [x] Array
    - [x] 複数行で最後にカンマがない場合、カンマを差し込む位置でコメントを考慮する
    - [x] Array のカンマと要素の末尾コメントの関係を見て、カンマの位置を移動
- [x] Inline Table
    - [x] 現行の v1.0.0 では複数行の Inline Table がサポートされていないのでエラーを出力させる。

### Refactor
- [ ] 各crateのエラー型の整理
- [ ] tokio::RwLock で読み取りロックを主にすることで、パフォーマンスを向上させる。
- [ ] `tombi format` | `tombi lint` のファイル探索の高速化


### Taplo の formatter のオプションの対応状況

| Progress    | Tombi Option                      | Taplo Option          | Description                                                                            | Default Value  |
|-------------|-----------------------------------|-----------------------|----------------------------------------------------------------------------------------|----------------|
| Done        | key-value-equal-alignment            | align_entries         | Align entries vertically. Entries that have table headers,                             | false          |
|             |                                   |                       | comments, or blank lines between them are not aligned.                                 |                |
| Not Yet     | key-value-align-trailing-comments | align_comments        | Align consecutive comments after entries and items vertically.                         | true           |
|             |                                   |                       | This applies to comments that are after entries or array items.                        |                |
| Not Planned |                                   | array_trailing_comma  | Put trailing commas for multiline arrays.                                              | true           |
| Not Planned |                                   | array_auto_expand     | Automatically expand arrays to multiple lines when they exceed characters.column_width | true           |
| Not Planned |                                   | array_auto_collapse   | Automatically collapse arrays if they fit in one line.                                 | true           |
| Done        | array-comma-space-width           | compact_arrays        | Omit whitespace padding inside single-line arrays.                                     | true           |
|             | array-bracket-space-width         |                       |                                                                                        |                |
| Done        | inline-table-comma-space-width    | compact_inline_tables | Omit whitespace padding inside inline tables.                                          | false          |
|             | inline-table-brace-space-width    |                       |                                                                                        |                |
| Not Planned |                                   | inline_table_expand   | Expand values (e.g. arrays) inside inline tables.                                      | true           |
| Done        | key-value-equal-space-width       | compact_entries       | Omit whitespace around .=                                                              | false          |
| Done        | line-width                        | column_width          | Target maximum column width after which arrays are expanded into new lines.            | 80             |
| Not Yet     |                                   | indent_tables         | Indent subtables if they come in order.                                                | false          |
| Done        | indent-table-key-values           | indent_entries        | Indent entries under tables.                                                           | false          |
| Done        | indent-width                      | indent_string         | Indentation to use, should be tabs or spaces but technically could be anything.        | 2 spaces (" ") |
| Not Planned |                                   | trailing_newline      | Add trailing newline to the source.                                                    | true           |
| Not Planned |                                   | reorder_keys          | Alphabetically reorder keys that are not separated by blank lines.                     | false          |
| Not Planned |                                   | reorder_arrays        | Alphabetically reorder array values that are not separated by blank lines.             | false          |
| Not Planned |                                   | reorder_inline_tables | Alphabetically reorder inline tables.                                                  | false          |
| Not Planned |                                   | allowed_blank_lines   | The maximum amount of consecutive blank lines allowed.                                 | 2              |
| Done        | line-ending                       | crlf                  | Use CRLF line endings.                                                                 | false          |
