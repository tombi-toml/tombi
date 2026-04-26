---
name: review
description: tombi の差分レビュー時に correctness / 回帰リスク / 検証不足 / source of truth の整合を優先順位付きで点検する標準フロー
---

# review

差分レビューをするときの既定フロー。

## 目的

- correctness、回帰リスク、検証不足を優先して洗い出す
- 保守性や重複は、致命的な問題の後に扱う

## 手順

1. `git diff origin/main...` か対象差分を確認する
2. 変更された領域の source of truth を特定する
3. 既存の test macro、workflow、生成手順から外れていないかを確認する
4. findings を重要度順に整理する
5. 問題がなければ、未確認の検証範囲だけを残リスクとして記録する

## 観点

- Rust core と editor / docs の整合
- generated artifact の更新漏れ
- 既存 test macro を無視したテスト実装
- packaging / workflow への影響漏れ
