# JSON Schema Compliance Design

対象: Tombi JSON Schema validation

## 1. 目的

この文書は、Tombi の JSON Schema 実装が従う設計方針を定義する。
ここで扱うのは「どの draft を対象にし、どう切り替え、どこまでを JSON Schema 標準挙動とし、どこからを Tombi 拡張とみなすか」である。

## 2. 対象 dialect

- 正式準拠対象は `draft-07` / `draft-2019-09` / `draft-2020-12` の 3 つとする。
- 実装順序は `draft-07` を基線とし、その互換性を壊さない範囲で `2019-09`、`2020-12` を積み上げる。
- 新規機能追加では、まず `draft-07` に対する退行がないことを確認したうえで、上位 dialect との差分語彙を追加する。

## 3. Dialect Resolution

- `$schema` が指定されている場合は、その URI から dialect を判定する。
- `$schema` 未指定時のデフォルト dialect は `draft-07` とする。
- 未知の `$schema` URI は未指定と同等に扱い、`draft-07` として評価する。
- dialect 判定はキーワード解釈、廃止判定、`format` や `unevaluated*` の有効性判定の入力として利用する。

## 4. 標準語彙と Tombi 拡張の境界

- `x-tombi-*` は Tombi 拡張語彙として扱い、JSON Schema assertion の真偽判定には含めない。
- `strict` は JSON Schema 標準ではなく Tombi 拡張挙動である。
- spec 準拠モードでは JSON Schema の既定挙動を優先し、たとえば `additionalProperties` 未指定は許可として扱う。
- 既存ユーザー互換のため、移行期間中は `strict` を維持しつつ、spec 準拠モードを明示的に切り替えられる構成を維持する。

## 5. Deprecated Keyword Policy

- 廃止キーワードは dialect ごとに `error` / `warning` / `compat` の 3 段階で扱う。
- 既定値は `warning` とし、互換実行を継続しつつ移行先キーワードを診断へ出す。
- `compat` は互換読込のみで追加診断を出さない。
- `error` は構文上受理しても検証失敗として扱う。

初期対象:

- `dependencies` -> `dependentRequired` / `dependentSchemas`
- tuple `items` / `additionalItems` -> `prefixItems` / 新 `items`
- `$recursiveAnchor` / `$recursiveRef` -> `$dynamicAnchor` / `$dynamicRef`
- `definitions` -> `$defs`

## 6. Validation Model

- validator の成功値は `EvaluatedLocations` とし、`unevaluatedProperties` / `unevaluatedItems` のための評価済み位置を返す。
- assertion の成否と diagnostics severity は分離する。
- `oneOf` / `anyOf` / `allOf` / `if-then-else` / `dependentSchemas` は、子 validator から返る `EvaluatedLocations` を合成して親へ伝播する。
- `Table` / `Array` の `unevaluated*` 判定は、局所的な静的推定ではなく、返却された評価済み位置を第一の根拠とする。

## 7. Compatibility Expectations

- 実装変更は dialect ごとの挙動差を明示的に保つこと。
- `draft-07`、`2019-09`、`2020-12` の回帰テストを継続して維持すること。
- `tombi-linter` の JSON Schema suite を、仕様差分と退行検知の基準セットとして扱う。

## 8. Non-Goals

- Tombi 拡張語彙を JSON Schema 標準語彙に見せかけることはしない。
- 未知 dialect を推測して部分対応することはしない。
- 廃止キーワードを silently rewrite して、元キーワードの利用事実を隠すことはしない。
