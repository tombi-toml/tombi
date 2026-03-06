# JSON Schema 完全準拠ギャップ TODO

調査日: 2026-03-01  
調査対象: JSON Schema draft-07 / 2019-09 / 2020-12（core, applicator, validation, unevaluated, meta-data, format, content）  
目的: Tombi の不足機能を仕様バージョン単位で管理し、廃止キーワードを実装計画に明示する

## 0. 方針確定

- [ ] 準拠対象を `draft-07` / `2019-09` / `2020-12` の 3 段階で正式決定する
- [ ] `$schema` による dialect 切替ポリシーを決める（未指定時デフォルト含む）
- [ ] Tombi 拡張（`x-tombi-*`, strict mode）を JSON Schema 準拠モードと分離する
- [ ] 廃止キーワードの扱いを決める（`error` / `warning` / `compat` 自動変換）

## 1. バージョン共通 TODO（draft-07 / 2019-09 / 2020-12）

- [ ] ルートが boolean schema（`true` / `false`）の JSON Schema を受理する
- [ ] サブスキーマ位置でも boolean schema を受理する（`items`, `properties`, `patternProperties`, `additionalProperties` など）
- [ ] `$id` による base URI 解決を実装する
- [ ] 相対 `$ref`（例: `./defs.json#...`）を base URI 基準で解決する
- [ ] 外部 `$ref` + fragment（例: `https://example.com/schema.json#/$defs/x`）を正しく解決する
- [ ] `$comment` を annotation として取得可能にする
- [ ] `anyOf` / `oneOf` / `allOf` の成功判定を「assertion 真偽」と「diagnostics severity」で分離する
- [ ] `uniqueItems` を複合値（配列/オブジェクト）まで含めた deep-equality 判定に拡張する
- [ ] `multipleOf` の浮動小数誤差対策（decimal/big rational 等）を行う
- [ ] JSON-Schema-Test-Suite を導入し、draft ごとの pass rate を計測する
- [ ] CI に compliance レポート（dialect 別 pass rate + 未対応 keyword）を追加する

## 2. draft-07 TODO（基線）

- [x] `if` / `then` / `else` を実装する
- [x] `propertyNames` を実装する（2026-03-03 対応）
- [x] `contains` を実装する（draft-07 の意味論）（2026-03-05 対応）
- [x] `dependencies` を実装する（2026-03-05 対応）
  備考: 2019-09 以降は `dependentRequired` / `dependentSchemas` へ置換（将来版では legacy 扱い）
- [x] `items` の tuple 形式（配列）を実装する（2026-03-05 対応）
  備考: 2020-12 で廃止、`prefixItems` に置換
- [x] `additionalItems` を実装する（2026-03-05 対応）
  備考: 2020-12 で廃止、新しい `items` 意味論へ置換
- [x] object に対する `const` / `enum` 検証を実装する（`TableSchema` 側）（2026-03-06 対応）
- [x] array に対する `const` / `enum` 検証を実装する（`ArraySchema` 側）（2026-03-06 対応）
- [ ] `readOnly` / `writeOnly` annotation を保持・公開する
- [ ] `contentEncoding` / `contentMediaType` / `contentSchema` を annotation として扱う
- [ ] `format` サポート範囲を仕様語彙（`uri-reference`, `ipv4`, `ipv6`, `iri`, `json-pointer` など）へ拡張する

## 3. draft-2019-09 TODO（移行レイヤ）

- [ ] `$vocabulary` の解釈と未対応 vocabulary 検出を実装する
- [ ] `$anchor` を実装する
- [ ] `$recursiveAnchor` / `$recursiveRef` を実装する  
  備考: 2020-12 で `$dynamicAnchor` / `$dynamicRef` に置換（将来版では compatibility のみ）
- [ ] `dependentRequired` を実装する（`dependencies` 置換）
- [ ] `dependentSchemas` を実装する（`dependencies` 置換）
- [ ] `unevaluatedProperties` を実装する
- [ ] `unevaluatedItems` を実装する
- [ ] `minContains` / `maxContains` を実装する

## 4. draft-2020-12 TODO（目標準拠）

- [ ] `$dynamicAnchor` / `$dynamicRef` を実装する
- [x] `prefixItems` を実装する（2026-03-05 対応）
- [x] 2020-12 の `items`（旧 `additionalItems` 相当の役割）を実装する（2026-03-05 対応）
- [ ] `format` を `format-annotation` / `format-assertion` で切替可能にする
- [ ] 2020-12 モードでは `dependencies` を廃止キーワードとして扱い、`dependent*` へ移行ガイドを出す
- [ ] 2020-12 モードでは `additionalItems` と tuple `items` を廃止キーワードとして扱い、`prefixItems` + 新 `items` へ移行ガイドを出す
- [ ] 2020-12 モードでは `$recursiveAnchor` / `$recursiveRef` を legacy 扱いにし、`$dynamic*` への移行を明示する
- [ ] spec 準拠モードでは `additionalProperties` 未指定時を `true` 相当で扱う（現 strict 拡張挙動と分離）

## 5. 廃止・置換キーワード一覧（実装計画に明示する項目）

| キーワード | draft-07 | 2019-09 | 2020-12 | 実装計画での扱い |
| --- | --- | --- | --- | --- |
| `dependencies` | 有効 | 置換推奨 | 廃止扱い（`dependent*`） | 互換読込 + deprecation warning + 自動移行ヒント |
| tuple `items`（配列） | 有効 | 有効 | 廃止 | 2020-12 では `prefixItems` へマップ案内 |
| `additionalItems` | 有効 | 有効 | 廃止 | 2020-12 では新 `items` へマップ案内 |
| `$recursiveAnchor` / `$recursiveRef` | なし | 有効 | 置換（`$dynamic*`） | 互換読込 + `\$dynamic*` 推奨 warning |
| `definitions` | 有効 | `\$defs` 推奨 | `\$defs` を使用 | 旧キーワード受理時は `\$defs` 移行ヒント |

## 6. 実装計画（廃止事項を含む）

### Phase A: Dialect 基盤

- [x] `schema.options` と `SchemaStore::Options` に dialect 設定を追加する（2026-03-03 対応）
- [x] キーワード判定を `dialect x vocabulary` ベースへ変更する（2026-03-03 対応）
- [x] 廃止キーワード判定フック（`deprecated_in`）を導入する（2026-03-03 対応）

### Phase B: draft-07 準拠を固定

- [ ] draft-07 必須キーワードを実装し、まず draft-07 pass rate を安定化する
- [ ] 将来廃止キーワード（`dependencies`, tuple `items`, `additionalItems`）に移行メッセージを追加する

### Phase C: 2019-09 互換レイヤを追加

- [ ] `dependentRequired` / `dependentSchemas` / `unevaluated*` / `$vocabulary` / `$recursive*` を実装する
- [ ] `dependencies` を互換実装に落とし、deprecation warning を標準化する
- [ ] 検証結果に evaluated locations を保持する（`unevaluated*` の前提）

### Phase D: 2020-12 ネイティブ実装

- [ ] `$dynamicAnchor` / `$dynamicRef`, `prefixItems` + 新 `items` を実装する
- [ ] 2020-12 モードで廃止キーワードの取り扱いを明示（warning/error を設定可能化）
- [ ] `format-annotation` / `format-assertion` の切替実装を投入する

### Phase E: 継続運用

- [ ] dialect ごとの回帰テスト（`draft-07`, `2019-09`, `2020-12`）を CI に追加する
- [ ] 準拠率ダッシュボードに「廃止キーワード利用件数」を追加する

## 7. 構造改修評価（2026-03-01 追記）

結論: TODO を「完全準拠（特に 2020-12 の `$dynamicRef` / `unevaluated*` と廃止キーワード管理含む）」まで実現するには、局所実装では足りず、スキーマ表現と検証結果モデルの大規模改修が必要。

### 大規模改修が必要な理由（具体コード）

1. ルート boolean schema を受理できない

```rust
// crates/tombi-schema-store/src/store.rs
let object = match self.fetch_schema_value(schema_uri).await? {
    Some(tombi_json::ValueNode::Object(object)) => object,
    Some(_) => {
        return Err(crate::Error::SchemaMustBeObject { schema_uri: schema_uri.to_owned() });
    }
    None => return Ok(None),
};
```

`draft-07` / `2020-12` では schema 自体が boolean でも合法だが、現状は object 固定で拒否する。

2. スキーマ AST が object 前提

```rust
// crates/tombi-schema-store/src/schema/document_schema.rs
pub fn new(object: tombi_json::ObjectNode, schema_uri: SchemaUri) -> Self
```

`DocumentSchema` 自体が `ObjectNode` を要求しており、boolean schema を表現する型が無い。

3. `$ref` 解決が「文字列参照 + 単純分岐」で、base URI / anchor 系スコープ情報を持たない

```rust
// crates/tombi-schema-store/src/schema/referable_schema.rs
pub enum Referable<T> {
    Resolved { schema_uri: Option<SchemaUri>, value: Arc<T> },
    Ref { reference: String, title: Option<String>, description: Option<String>, deprecated: Option<bool> },
}
```

```rust
// crates/tombi-schema-store/src/schema/referable_schema.rs
} else if is_json_pointer(reference) {
    // local pointer
} else if let Ok(schema_uri) = SchemaUri::from_str(reference) {
    // absolute URI 前提
} else {
    return Err(crate::Error::UnsupportedReference { ... });
}
```

相対 `$ref` を base URI で解決する情報、`$id` による base 更新、`$anchor`/`$dynamicRef` の動的解決コンテキストを保持していない。

4. 検証 API が pass/fail + diagnostics のみで、`unevaluated*` に必要な「評価済み位置集合」を返せない

```rust
// crates/tombi-validator/src/validate.rs
pub trait Validate {
    fn validate(...) -> BoxFuture<'b, Result<(), crate::Error>>;
}
```

`unevaluatedProperties` / `unevaluatedItems` は applicator 実行後に「どの key/index が評価済みか」を合成して判定する必要があるため、返却モデルの拡張が必須。

5. `oneOf` 成立判定が warning を成功扱いにする設計

```rust
// crates/tombi-validator/src/validate.rs
fn is_success_or_warning(result: &Result<(), crate::Error>) -> bool {
    match result {
        Ok(()) => true,
        Err(error) => !has_error_level_diagnostics(error),
    }
}
```

```rust
// crates/tombi-validator/src/validate/one_of.rs
if is_success_or_warning(&result) {
    valid_count += 1;
}
```

JSON Schema assertion の真偽と、lint severity を混在させているため、仕様準拠モードでは判定系を分離する再設計が必要。

6. `strict` 拡張が既定で有効で、spec 既定（`additionalProperties` 未指定は許可）と不一致

```rust
// crates/tombi-schema-store/src/store.rs
pub fn strict(&self) -> bool {
    self.options.strict.unwrap_or(true)
}
```

```rust
// crates/tombi-schema-store/src/schema/table_schema.rs
pub fn allows_additional_properties(&self, strict: bool) -> bool {
    self.additional_properties.unwrap_or(!strict)
}
```

仕様準拠モードと Tombi 拡張モードを切り分ける設定レイヤが必要。

7. `uniqueItems` がリテラル値のみ比較で、複合値 deep-equality を満たさない

```rust
// crates/tombi-validator/src/validate/array.rs
let literal_values = array_value
    .values()
    .iter()
    .filter_map(Option::<LiteralValueRef>::from)
    .counts();
```

`LiteralValueRef` は `Array` / `Table` を扱わないため、仕様要求の配列・オブジェクト比較が未実装。

8. dialect 設定が `strict` 以外ほぼ存在せず、廃止キーワードを version ごとに制御できない

```rust
// crates/tombi-schema-store/src/options.rs
pub struct Options {
    pub strict: Option<bool>,
    pub offline: Option<bool>,
    pub cache: Option<tombi_cache::Options>,
}
```

```rust
// crates/tombi-config/src/schema.rs
pub struct SchemaOverviewOptions {
    pub enabled: Option<BoolDefaultTrue>,
    pub strict: Option<BoolDefaultTrue>,
    pub catalog: Option<SchemaCatalog>,
}
```

`draft-07` と `2020-12` で有効キーワードが異なるため、dialect を持たない設計では「廃止キーワード warning/error」の一貫運用ができない。

### 中規模拡張で対応可能な領域（構造維持で進めやすい）

- `if/then/else`, `dependentRequired`, `contains`, `minContains/maxContains` の追加
- object/array の `const` / `enum` 検証追加
- `format` サポート拡張、`readOnly`/`writeOnly`/content 系 annotation の保持

### 推奨する改修順序（廃止キーワード移行を含む）

1. 仕様準拠モード（dialect + strict policy + deprecated keyword policy）を設定層に追加
2. 検証結果モデルを `Result<(), Error>` から「assertion + annotations + evaluated locations」へ拡張
3. 参照解決器を `$id` / relative `$ref` / `$anchor` / `$dynamicRef` 対応に再設計
4. 未対応キーワードを draft 順に投入し、同時に廃止キーワードの移行診断を実装
5. JSON-Schema-Test-Suite により dialect 別準拠率と廃止利用件数を継続計測
