---
paths: "**/*.rs"
---

# Rust Workspace Practices

- Rust を編集したら `cargo fmt --all` を前提に整える
- 検証は変更範囲に最も近い crate / package から始め、必要に応じて `cargo check --workspace --all-targets` や `cargo test --workspace` へ広げる
- `cargo xtask` が既存フローとして使われている領域では、同等の ad hoc スクリプトを増やす前に xtask を確認する
- warning は放置しない。`#[allow(...)]` は最後の手段として扱う
- panic 回避より先に型と境界条件を整理する。テスト以外での安易な `unwrap()` 追加は避ける
