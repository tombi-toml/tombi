---
paths: "**/*"
---

# Generated Artifacts

- 生成物や派生ファイルを更新する場合は、まず source generator / source data / build entrypoint を確認する
- generated file だけを手編集して済ませない。やむを得ず手編集した場合でも、再生成手順を確認し、再生成で壊れない状態に戻す
- schema、fixture、manifest、packaging metadata を変えるときは、その消費側の Rust / docs / editor integration への影響も確認する
- 生成手順が不明な場合は推測で増築せず、README、xtask、workflow、既存スクリプトを読んでから着手する
