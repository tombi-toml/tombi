---
name: github-pr-resolve
description: "GitHub の PR URL / PR番号から head branch を checkout し、最新の main を取り込み、CI の失敗を修正し、AI / 人間を問わず既存の review comment と review thread を取得して、妥当な指摘の修正、不要または既対応の説明、返信、resolve まで行う。tombi の Rust / cargo / pnpm / uv 運用に合わせる。トリガー: PR URL、'レビュー対応'、'AIレビュー対応'、'人間レビュー対応'、'会話を解決'、'CIも直して'、'/pr-fix-review'"
---

# GitHub PR Resolve

tombi の GitHub PR に既に存在する CI 失敗とレビュー会話を解決する。AI reviewer と人間レビュアーを同じ基準で扱う。この Skill から reviewer を追加せず、未着レビューを待つためのポーリングもしない。

## 前提

- `gh auth status` が成功する
- `gh` と `jq` がインストール済みである
- PR の head branch に push できる
- 返信前に必要なローカル検証を終える
- thread は必ず返信してから resolve する
- コード変更時は `AGENTS.md` と対象領域の `.claude/rules/` に従う
- 認証回避のために Git の global / local / system config を書き換えたり、PAT や `url.*.insteadOf` を保存したりしない

## ワークフロー

### 1. PR と branch を確認する

```bash
gh pr checkout <pr-url-or-number>
gh pr view <pr-url-or-number> --json number,title,url,headRefName,headRefOid,headRepository,headRepositoryOwner,baseRefName,reviewDecision,comments,reviews
gh pr checks <pr-url-or-number> --json name,state,bucket,link,workflow
```

必要なら `gh pr view <pr> --json files` で変更範囲も確認する。review request の追加や AI reviewer の応答待ちは行わず、現在取得できる CI、reviews、comments、threads を処理する。

この Skill は `main` base 専用である。取得した `baseRefName` が `main` 以外なら、`origin/main` を merge せず未対応 base として停止し、ユーザーへ報告する。

fork PR では `origin` を head remote とみなさない。`headRepository.nameWithOwner`、`headRepositoryOwner.login`、`headRefName` から `https://github.com/<owner>/<repo>.git` の head ref を特定し、`git ls-remote <head-repository-url> refs/heads/<headRefName>` を PR の `headRefOid` と比較する。push 権限または head repository を特定できなければ変更を push せず報告する。

### 2. 最新の `main` を取り込む

```bash
git fetch origin main
git merge-base --is-ancestor origin/main HEAD || git merge origin/main --no-edit
```

merge が必要なら競合を解消し、影響を受けるローカル検証をやり直してから進む。

### 3. 現在の full PR diff のレビュー証拠を確立する

処理開始時に、コード変更の有無にかかわらず、現在の PR head に対応する review base OID、patch hash、review fingerprint、exact head OID を確立する。

`github-pr-flow` などの呼び出し元から承認済み review base OID、review fingerprint、exact head OID が渡された場合は、remote base、local HEAD、remote head、PR head、`git diff --binary origin/main HEAD` の hash を再取得する。すべて一致する場合だけレビュー結果を引き継ぐ。

証拠が渡されていない standalone 実行、または1項目でも一致しない場合は、現在の full PR diff に必要なローカル検証を実行し、`<review-base-oid>:<patch-hash>` を fingerprint として記録してから、Codex または Claude Code の `review-quality` サブエージェントへ読み取り専用レビューを依頼する。actionable finding があれば修正フローへ進み、有効な判定を得られなければ fail closed とする。

### 4. CI の失敗を処理する

- `gh pr checks <pr> --json ...` の `bucket == "fail"` を優先する
- failed check の `link` から run ID を取り、推測せず job log を確認する
- job の rerun には URL の `jobs/<number>` ではなく `databaseId` を使う

```bash
gh run view <run-id> --json jobs
gh run view <run-id> --json jobs --jq '.jobs[] | {name, databaseId, conclusion}'
gh run view <run-id> --log-failed
gh run view <run-id> --job <job-database-id> --log
```

詳細解析が必要なら [`references/ci-run-job-debug.md`](references/ci-run-job-debug.md) を読む。修正中は失敗ジョブ相当の最小検証を優先し、変更面に応じて次を追加する。

- Rust: `cargo build --verbose --locked`、`cargo nextest run --workspace --locked --no-fail-fast`、`cargo test --workspace --doc --locked`、`cargo shear`
- format / lint: `cargo fmt --all --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`
- editor / docs / blog: `pnpm format`、`pnpm lint`
- Python packaging: `uv run pytest`、`uv run ruff check`、必要なら `maturin build`
- toml-test: 対象を絞った `cargo nextest run -p toml-test ...`

push 後は CI を再確認する。

```bash
gh run rerun <run-id> --failed
gh pr checks <pr-url-or-number> --watch --interval 10
```

新規 PR も含め、check suite がまだ現れていない場合は開始を待って再取得する。`bucket == "pending"` が1件でもあれば terminal になるまで待ち、`fail` または `cancel` があれば原因を処理する。全対象 check が同じ head で `pass` または意図された `skipping` になったことを確認するまで handoff しない。待機が環境の上限に達した場合は未完了として報告し、成功扱いにしない。

### 5. 全ページの review thread を取得する

`gh pr view --json reviews,comments` では thread 単位の resolve 状態が取れないため、GraphQL の `reviewThreads` を使う。

```bash
gh api graphql \
  -f owner=tombi-toml \
  -f repo=tombi \
  -F number=<number> \
  -f query='
query($owner: String!, $repo: String!, $number: Int!, $after: String) {
  repository(owner: $owner, name: $repo) {
    pullRequest(number: $number) {
      id
      reviewThreads(first: 100, after: $after) {
        totalCount
        pageInfo { hasNextPage endCursor }
        nodes {
          id
          isResolved
          isOutdated
          viewerCanReply
          viewerCanResolve
          path
          line
          comments(first: 100) {
            totalCount
            pageInfo { hasNextPage endCursor }
            nodes {
              id
              body
              publishedAt
              url
              author { login }
              pullRequestReview { id state url }
            }
          }
        }
      }
    }
  }
}'
```

- 初回は `after` を省略し、`pageInfo.hasNextPage == true` なら `-f after='<endCursor>'` を追加して次ページを取得する。`hasNextPage == false` になるまで繰り返す
- 各 thread の `comments.pageInfo.hasNextPage == true` なら、thread ID と comments cursor を使う次の query を `hasNextPage == false` まで繰り返す
- 各 GraphQL 応答の `.errors` が空であること、`endCursor` が非空かつ前ページから進行すること、ページ継続中に空 nodes が返らないこと、thread / comment ID が重複しないことを確認する
- `reviewThreads.totalCount` と取得した unique thread ID 数、および各 `comments.totalCount` と取得した unique comment ID 数が一致しなければ fail closed とする
- 実行環境のページ上限へ達した場合は完全取得とみなさず、未完了として報告する
- `isResolved == false` の thread を処理する
- `isOutdated == true` でも指摘の意図が残るなら無視しない
- top-level comment は必要に応じて `gh pr comment` で返信する
- `viewerCanResolve == false` なら返信まで行い、権限不足を報告する

```bash
gh api graphql \
  -F threadId='<thread-id>' \
  -f after='<comments-endCursor>' \
  -f query='
query($threadId: ID!, $after: String) {
  node(id: $threadId) {
    ... on PullRequestReviewThread {
      comments(first: 100, after: $after) {
        totalCount
        pageInfo { hasNextPage endCursor }
        nodes {
          id
          body
          publishedAt
          url
          author { login }
          pullRequestReview { id state url }
        }
      }
    }
  }
}'
```

### 6. 指摘を判定する

- 妥当: 修正し、必要なテストと次のローカルレビューゲートを通し、push してから返信する
- 不要: コードまたは仕様に基づく理由を返信する
- 既対応: 対応した commit、file、test を示して返信する

AI reviewer と人間レビュアーを区別せず、技術判断で完結するものは進める。曖昧な仕様判断だけユーザー確認を検討する。

#### コード変更を push する前のローカルレビューゲート

`main` の取り込み、CI 修正、review 対応などで head tree を変える場合は、次を push 前に実行する。

1. `git fetch origin main` を実行する
2. `git merge-base --is-ancestor origin/main HEAD || git merge origin/main --no-edit` で最新の `origin/main` を必ず取り込む
3. merge 後の tree に対して必要なローカル検証を完了し、PR に含める変更だけを stage する
4. `git rev-parse origin/main` を review base OID として記録する
5. `git diff --cached --binary origin/main | shasum -a 256` を patch hash とし、`<review-base-oid>:<patch-hash>` を review fingerprint として記録する
6. Codex または Claude Code の `review-quality` サブエージェントに、full PR diff、fingerprint、検証結果を渡して読み取り専用レビューを依頼する
7. actionable finding があれば修正・検証・stage・fingerprint・独立レビューをやり直す
8. finding がなくても、commit 直前に remote base OID と staged full PR diff が承認済み fingerprint と一致することを確認する
9. commit 後に `git diff --binary origin/main HEAD | shasum -a 256` が同じ tree 差分を示し、`git status --porcelain` が空であることを確認してから push する

base OID、commit hook、formatter などで head tree が変われば以前のレビュー結果を失効させ、検証と独立レビューをやり直す。サブエージェントから有効な判定を得られない場合も commit / push しない。

push は Step 1 で特定した head repository の `<headRefName>` を明示的な対象にする。push 後は同じ repository URL を `git ls-remote` し、その OID と local HEAD、PR の `headRefOid` が一致するまで確認する。

### 7. thread に返信して resolve する

返信は top-level comment ではなく対象 review thread に付ける。

```bash
gh api graphql \
  -F threadId='THREAD_ID' \
  -F body=@/tmp/review-thread-reply.md \
  -f query='
mutation($threadId: ID!, $body: String!) {
  addPullRequestReviewThreadReply(
    input: { pullRequestReviewThreadId: $threadId, body: $body }
  ) {
    comment { url }
  }
}'
```

```bash
gh api graphql \
  -F threadId='THREAD_ID' \
  -f query='
mutation($threadId: ID!) {
  resolveReviewThread(input: { threadId: $threadId }) {
    thread { id isResolved }
  }
}'
```

返信後に resolve する。top-level comment には `gh pr comment <pr> --body-file <file>` で返信する。

## 返信内容

修正した場合は、変更内容、変更箇所、確認コマンドを書く。修正しない場合は、不要と判断した理由と根拠となるコード、仕様、テストを書く。

## 完了条件

- 必要なコード修正が push 済みである
- 失敗していた CI の原因を解消している
- check suite が開始済みで、pending / failed / cancelled check がなく、全対象 check が同じ exact head で pass または意図された skipping になっている
- 対象にした unresolved thread すべてに返信済みである
- resolve 可能な thread はすべて resolve 済みである
- top-level comment / review summary に必要な説明を返している
- handoff 直前に remote `main` の OID、local HEAD、remote head、PR head、承認済み review fingerprint を再取得している
- remote base OID と承認済み review base OID が一致し、local / remote / PR head が同一で、current full PR diff の hash が承認済み patch hash と一致している
- base または head tree が変わっていれば、最新の組み合わせで merge、検証、fingerprint、独立レビューをやり直している
- 同じ exact head / base で CI、reviews、comments、全ページの threads、mergeability を再取得している
- 修正内容、検証、未解決事項だけを短く報告する
