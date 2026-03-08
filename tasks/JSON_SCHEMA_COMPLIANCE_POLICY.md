# JSON Schema 準拠ポリシー

決定日: 2026-03-08
対象: Tombi JSON Schema validation

## 1. 準拠対象バージョン

- 正式準拠対象は `draft-07` / `draft-2019-09` / `draft-2020-12` の 3 段階とする。
- 新規実装は `draft-07` 互換を壊さないことを前提に、`2019-09`、`2020-12` の順で適用する。

## 2. dialect 切替ポリシー

- `$schema` が指定されている場合は、その URI から dialect を判定して適用する。
- `$schema` 未指定時のデフォルト dialect は `draft-07` とする。
- 未知の `$schema` URI は「未指定と同等」に扱い、`draft-07` として評価する。

## 3. Tombi 拡張と spec 準拠モードの分離

- `x-tombi-*` は Tombi 拡張語彙として扱い、JSON Schema assertion の真偽判定には混ぜない。
- `strict` は Tombi 拡張挙動として扱い、spec 準拠モードでは JSON Schema 既定（例: `additionalProperties` 未指定は許可）を優先する。
- 既存ユーザー互換性のため、移行期間中は `strict` を維持しつつ、spec 準拠モードを明示的に切替可能にする。

## 4. 廃止キーワードの扱い

- 廃止キーワードは dialect ごとに `error` / `warning` / `compat` の 3 段階ポリシーで扱う。
- 既定値は `warning` とし、互換実行を継続しつつ移行先キーワードを診断メッセージに出す。
- `compat` は互換読込のみ（追加診断なし）、`error` は検証失敗として扱う。
- 初期対象:
  - `dependencies` -> `dependentRequired` / `dependentSchemas`
  - tuple `items` / `additionalItems` -> `prefixItems` / 新 `items`
  - `$recursiveAnchor` / `$recursiveRef` -> `$dynamicAnchor` / `$dynamicRef`
  - `definitions` -> `$defs`
