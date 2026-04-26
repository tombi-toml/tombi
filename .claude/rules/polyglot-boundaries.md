---
paths: "**/*"
---

# Polyglot Boundaries

- Rust の仕様変更を docs や editor plugin だけで吸収しない。source of truth はまず core 実装側で直す
- docs、editor integration、Python packaging は core の変更結果を反映する層として扱う
- `crates/`、`extensions/`、`rust/`、`python/`、`docs/`、`editors/` をまたぐ変更では、どこが source of truth かを最初に決める
- release / packaging 変更では、CLI、editor extension、PyPI/npm/homebrew など複数配布面への波及を意識する
