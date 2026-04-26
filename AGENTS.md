# Tombi

TOML toolkit (parser / formatter / linter / LSP) の polyglot repository。Rust workspace を中心に editor integration、Python packaging、docs を同居させている。

## 必須ルール

- Think in English, but generate responses in Japanese
- 自動テストには既存の宣言的テストマクロ（例: `test_format!`）を必ず再利用する

## 補助設定

- Claude Code: `.claude/`
- Codex: `.codex/`（`.claude/` 配下を symlink）
