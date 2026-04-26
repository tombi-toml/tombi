# `.claude/`

Claude Code 用の補助設定。`.codex/` はこのディレクトリへの symlink で構成され、Codex からも同じ内容が参照される。

## 構成

```text
.claude/
├── settings.json   # Claude Code 用 permissions / MCP
├── rules/          # 常時適用ルール（先頭に `paths:` frontmatter）
├── skills/         # 複数ステップ手順（SKILL.md は name/description frontmatter）
├── agents/         # subagent 定義（name/description/tools/model frontmatter）
└── hooks/          # Claude Code event hook（Codex は実行しない）
```

## 参照側

- Claude Code は `.claude/` を直接読む
- Codex は `.codex/rules/`、`.codex/skills/`、`.codex/agents/`、`.codex/hooks/` を読む（実体は `.claude/` 配下）
