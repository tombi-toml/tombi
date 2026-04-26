# Hooks

Claude Code の event hook（`UserPromptSubmit` / `PreToolUse` / `PostToolUse` など）置き場。

## 適用範囲

- このディレクトリのスクリプトは Claude Code でのみ自動実行される。
- Codex / 他の AI agent はこれらを実行しないため、hook が課す制約や挙動は必ず [`AGENTS.md`](../../AGENTS.md) または [`.claude/rules/`](../rules/) に同等の規約として書くこと。
- そうすることで、Claude が hook で自動化する内容と Codex が手動で守るべき内容が一致する。

## 現状

- 現在は実行スクリプトを持たない。`AGENTS.md` と `.claude/rules/` で同等の規約を提供しているため、UserPromptSubmit での context 注入は冗長になる。
- 将来 hook を追加するときは、`settings.json` の `hooks` フィールドに登録した上で、上記の通り規約をミラーリングする。

## 追加例

```jsonc
// .claude/settings.json
{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "Edit|Write|MultiEdit",
        "hooks": [
          {
            "type": "command",
            "command": "cd \"$CLAUDE_PROJECT_DIR\" && python3 .claude/hooks/example_post_edit.py"
          }
        ]
      }
    ]
  }
}
```

参考: <https://docs.claude.com/en/docs/claude-code/hooks>
