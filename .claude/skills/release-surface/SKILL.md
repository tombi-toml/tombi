---
name: release-surface
description: tombi の配布面 (GitHub Actions / CLI / Python wheel / npm / editor marketplace) に影響する変更を扱うとき、波及確認の手順を提供する
---

# release-surface

配布面に影響する変更を扱うときの確認メモ。

## 対象

- GitHub Actions
- CLI packaging
- Python wheel / sdist
- npm package
- editor marketplace artifacts

## 手順

1. どの配布面が影響を受けるか列挙する
2. 既存 workflow の relevant-changes 条件を確認する
3. version / artifact / publish 手順を壊していないか確認する
4. 必要なら対象 surface の最小ビルドや lint を実行する
