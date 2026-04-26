---
name: rust-change
description: tombi の Rust workspace (crates/, extensions/, rust/, toml-test/, xtask/) に変更を入れるとき、対象 crate の特定から最小範囲の cargo 検証までの標準フローを示す
---

# rust-change

Rust workspace に変更を入れるときの標準フロー。

## 手順

1. 対象 crate と責務を特定する
2. 近い既存実装と既存テストマクロを確認する
3. 実装を変更する
4. 必要なら `cargo fmt --all` を実行する
5. 最小範囲の `cargo check -p <crate>` または `cargo test -p <crate>` で検証する
6. 変更が広い場合だけ `cargo check --workspace --all-targets` や `cargo test --workspace` へ広げる

## 注意

- 新しいテスト helper を足す前に、既存の `test_...!` マクロを再利用できるか確認する
- `xtask` で提供済みのフローがあるなら、それを再利用する
