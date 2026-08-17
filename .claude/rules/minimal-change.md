---
paths: "**/*"
---

# Minimal Change

- 変更前に対象の実行経路と既存実装を確認し、既存 helper / pattern、標準ライブラリ、platform 機能、導入済み dependency の順に再利用を検討する
- 新しい abstraction、設定、dependency、将来用 scaffold は、現在の要件で必要な場合だけ追加する
- bug report は症状として扱い、変更対象の caller と同じ経路を使う箇所を検索して、可能なら共有された根因を一度だけ修正する
- 最小差分は調査後に選ぶ。trust boundary の validation、data loss を防ぐ error handling、security、accessibility、明示された要件は簡略化しない
- 非自明な変更には、既存の test macro / test pattern を使った最小の回帰確認を残す
