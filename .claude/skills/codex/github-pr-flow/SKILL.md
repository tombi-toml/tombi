---
name: github-pr-flow
description: "現在のローカル差分を検証し、Codex または Claude Code の review-quality サブエージェントによる独立レビューを PR 作成前に完了してから、commit、push、GitHub PR 作成、CI と既存 review thread の対応まで進める。トリガー: 'github-pr-flow'、'PR作成からレビュー対応まで'、'ローカルレビューしてPR作成'、'/github-pr-flow'"
---

# GitHub PR Flow

現在の変更をローカルで独立レビューしてから PR を作成し、作成後の CI と既存 review thread を [`../../github-pr-resolve/SKILL.md`](../../github-pr-resolve/SKILL.md) で処理する。GitHub 上の AI reviewer にはレビューを依頼しない。

## 前提

- `gh auth status` が成功する
- push 権限がある
- リポジトリと変更対象領域の `AGENTS.md` / `.claude/rules/` を確認済みである
- Codex または Claude Code のサブエージェント機能を利用できる
- 非対話コマンドを使う

## ワークフロー

### 1. PR に含める差分を確定する

1. `git fetch origin main` を実行し、必要なら `origin/main` を取り込む
2. `git rev-parse origin/main` の出力を review base OID として記録する
3. [`../../workspace-check/SKILL.md`](../../workspace-check/SKILL.md) で変更面を分類し、必要なローカル検証を選択・実行して、実行済み項目と残るギャップを記録する
4. PR に含めるファイルだけを stage する
5. `git diff --cached --check` と `git status --short` で、意図しない差分や未追跡ファイルがないことを確認する
6. `git diff --cached --binary origin/main | shasum -a 256` の出力を patch hash として記録し、`<review-base-oid>:<patch-hash>` を review fingerprint とする

差分が空なら PR を作成しない。未追跡ファイルは stage されるまでレビュー対象に含まれないため、必ず先に対象を確定する。

### 2. ローカルの独立サブエージェントにレビューを依頼する

実行中のハーネスに応じて、Codex または Claude Code のサブエージェント機能で `review-quality` エージェントを起動する。親エージェント自身の自己レビューだけで代替しない。

サブエージェントには次を渡す。

- base: `origin/main`
- review base OID
- review fingerprint
- 対象: `git diff --cached origin/main` と変更ファイルの関連実装・テスト
- 実行済み検証と結果
- 読み取り専用でレビューし、ファイルを変更しないこと
- correctness、回帰、検証不足、source of truth の不整合を優先し、finding は file / line と根拠を付けること

実装方針の正当化や期待する結論を先に教えない。サブエージェントが独立に差分と周辺コードを確認できる情報だけを渡す。

### 3. finding を解消し、同じ差分を再確認する

- actionable finding がなければ、review fingerprint が変わっていないことを確認して Step 4 へ進む
- actionable finding があれば親エージェントが修正し、対象検証、stage、fingerprint 採取をやり直す
- fingerprint が変わった場合、以前のレビュー結果を再利用せず、新しいサブエージェントで Step 2 をやり直す
- 仕様判断が必要で解消できない finding は、PR を作成せずユーザーに確認する

レビュー結果が空、失敗、権限不足、タイムアウトなどで有効な判定を得られない場合も fail closed とし、PR を作成しない。

### 4. PR を作成する

PR 作成そのものは次の優先順位で既存 skill に従う。

1. リポジトリ内の `github-pr-create`
2. user-global の `my-github-pr-create`
3. どちらも使えない場合は、次の manual fallback を使う

manual fallback では、現在 branch が `main` または detached HEAD でないことを確認する。該当する場合は `<type>/<purpose-kebab>` 形式の topic branch を作成してから commit する。push 先は現在の GitHub アカウントが書き込める明示的な remote とし、base repository の保護された `main` へ直接 push しない。

```bash
git commit -m '<type>: <summary>'
git push -u <writable-remote> HEAD:refs/heads/<topic-branch>
gh pr create \
  --repo tombi-toml/tombi \
  --base main \
  --head '<owner>:<topic-branch>' \
  --title '<title>' \
  --body-file <body-file> \
  --label '<label>'
```

`<writable-remote>` の URL と `<owner>` が同じ fork を指すこと、push 後の remote OID が local HEAD と一致することを確認する。引数なしの対話的な `gh pr create` は使わない。

commit 直前に `git fetch origin main` を再実行し、`origin/main` の OID と staged patch hash が承認済み review fingerprint と一致することを確認する。commit 後は `git diff --binary origin/main HEAD | shasum -a 256` で同じ tree 差分であることに加え、`git status --porcelain` が空であることを確認する。commit hook、formatter、base drift のいずれかで fingerprint が変わるか未commit差分が残った場合は、Step 1 から検証とレビューをやり直す。

`gh pr create` の直前にも `git ls-remote origin refs/heads/main` で remote base OID を直接確認する。承認済み review base OID と異なる場合は PR を作成せず、`origin/main` を更新して Step 1 からやり直す。

PR 作成時は branch、検証結果、commit、PR URL、必要な label を記録する。GitHub 上の AI reviewer 追加コマンドは実行しない。

### 5. 作成後の状態を確認する

PR URL、承認済み review base OID、review fingerprint、commit 後の exact head OID を入力として [`../../github-pr-resolve/SKILL.md`](../../github-pr-resolve/SKILL.md) を使う。委譲先は現在の remote base、local / remote / PR head、full PR diff が渡された証拠と一致する場合だけ、そのレビュー結果を引き継ぐ。

- CI の失敗を調査・修正する
- 既に投稿された AI / 人間の review thread があれば通常どおり対応する
- review request の追加や AI reviewer の応答待ちは行わない
- PR はマージしない

PR 作成後の `main` 取り込み、CI 修正、review 対応などで head tree を変える場合は、push 前に新しい full PR diff を Step 1〜3 と同じ検証・fingerprint・独立サブエージェントレビューへ通す。以前の review fingerprint は再利用しない。

最終 handoff 前に remote base OID を再取得する。base または head tree が最後に承認された fingerprint から変わっていれば、最新の組み合わせで検証と独立レビューをやり直す。その後、現在の head、CI、reviews、comments、全ページの review threads、mergeability を同じ状態で取り直す。

## 出力

- 利用したローカル review subagent
- 承認済み review base OID、review fingerprint、finding の有無
- 実行した検証
- branch、commit、PR URL、label
- 対応した thread 数と最終未解決 thread 数
- PR を未マージで引き渡したこと
