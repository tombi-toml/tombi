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
