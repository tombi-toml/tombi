---
name: workspace-check
description: tombi で作業を完了する前に、Rust / Node / Python / docs / workflow のどの面に検証コマンドが必要かを切り分け、実行済みと未実行のギャップを残リスクとしてまとめる
---

# workspace-check

作業完了前の検証範囲を整理するための Skill。

## 手順

1. 変更した面を Rust / Node / Python / docs / workflow に分類する
2. 各面で最小限必要な確認コマンドを選ぶ
3. 実行したコマンドと未実行のギャップを分けて記録する

## 代表コマンド

- Rust: `cargo fmt --all`, `cargo check -p <crate>`, `cargo test -p <crate>`
- Workspace 全体: `cargo check --workspace --all-targets`, `cargo test --workspace`
- Node/docs: `pnpm format`, `pnpm lint`, package 単位コマンド
- Python: `uv run pytest`, `uv run ruff check`, `maturin build`

## 出力

- 実行したコマンド
- 未実行だが必要性を検討したコマンド
- 残るリスク
