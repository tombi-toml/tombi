---
name: github-pr-resolve
description: "GitHub の PR URL / PR番号から、PR の head branch を checkout し、最新の main を取り込み、CI の失敗ジョブ確認・修正、AI / 人間を問わずレビューコメントと review thread を取得し、妥当な指摘の修正、不要または既対応の説明、必要な返信と resolve まで行う。tombi の Rust / cargo / pnpm / uv 運用に合わせて進める。トリガー: PR URL、'レビュー対応', 'AIレビュー対応', 'Copilotレビューをさばく', '人間レビュー対応', '会話を解決', 'CIも直して', '/pr-fix-review'"
metadata:
  short-description: PRのCI修正とレビュー解決
---

# GitHub PR Resolve

tombi リポジトリで、GitHub PR の CI 修正とレビュー会話の解決を最後まで進めるための統一スキル。AI reviewer と人間レビュアーを分けず、同じフローで扱う。

## 役割

- この Skill は PR 単位のオーケストレーションを担当する
- CI run / job の詳細解析が必要なときだけ [`references/ci-run-job-debug.md`](references/ci-run-job-debug.md) を補助的に読む
- この Skill から checkout / `origin/main` の merge / push / reply / resolve まで進める
- AI reviewer と人間レビュアーの thread / summary / top-level comment をまとめて処理する

## 使う場面

- PR URL / PR番号を起点に対応する
- GitHub Actions の失敗を直す
- AI / 人間を問わず review thread に返信して resolve する
- Copilot review を待ってから対応を始める
- 「妥当なら修正、不要なら理由を返す」を一連で処理する

## 前提

- `gh auth status` が成功する
- `gh` と `jq` の両方がインストール済み（review request 判定や thread 解析で `jq` を必須にする。macOS なら `brew install jq`、Debian/Ubuntu なら `apt-get install jq`）
- PR の head branch に push できる
- 返信前に、必要なローカル検証を終える
- thread は無言で resolve しない。必ず返信してから resolve する
- コード変更時は、このリポジトリの `AGENTS.md` と対象サブディレクトリの `AGENTS.md` / `.claude/rules/` に従う
- submodule / 認証まわりで詰まっても `git config --global` / `git config --local` / `git config --system` で Git 設定を書き換えて回避しない。PAT や `url.*.insteadOf` を Git 設定へ保存するのも禁止する

## 基本フロー

### 1. PR を開いて branch を合わせ、Copilot review request の有無を確認する

```bash
gh pr checkout <pr-url-or-number>
gh pr view <pr-url-or-number> --json number,title,url,headRefName,baseRefName,reviewDecision
gh pr checks <pr-url-or-number> --json name,state,bucket,link,workflow
```

- 変更範囲の把握が必要なら `gh pr view <pr> --json files`
- 既存の issue comment / review summary も見るなら `gh pr view <pr> --json comments,reviews`
- Copilot にレビュー依頼を出しているかは、review request を最初に確認する

```bash
gh api graphql \
  -f owner=tombi-toml \
  -f repo=tombi \
  -F number=<number> \
  -f query='
query($owner: String!, $repo: String!, $number: Int!) {
  repository(owner: $owner, name: $repo) {
    pullRequest(number: $number) {
      reviewRequests(first: 100) {
        nodes {
          requestedReviewer {
            __typename
            ... on Bot { login }
            ... on User { login }
            ... on Team { slug name }
          }
        }
      }
    }
  }
}' |
  jq '
    [
      .data.repository.pullRequest.reviewRequests.nodes[]?.requestedReviewer
      | .login? // .slug? // empty
      | ascii_downcase
    ]
    | any(test("copilot"))
  '
```

- `true` なら「Copilot review request が pending」とみなして Step 4 の待機判定に使う
- `false` なら、この PR では Copilot review request は確認できていないので、Copilot review 待ちのためのポーリングはしない

### 2. 最新の `main` を取り込む

checkout 後、head branch に最新の `origin/main` が入っているかを確認する。未取り込みなら、CI 修正や review 対応の前に merge して競合を解消する。

```bash
git fetch origin main
git merge-base --is-ancestor origin/main HEAD || git merge origin/main --no-edit
```

- `git merge-base --is-ancestor origin/main HEAD` が成功したら、最新の `origin/main` は取り込み済みなので何もしない
- merge が必要だった場合は、この段階で競合を解消してから次へ進む
- merge 後は、影響を受けるローカル検証をやり直してから CI 原因調査や review 返信に進む

### 3. CI の失敗を処理する

- `gh pr checks <pr> --json ...` で `bucket == "fail"` を優先する
- failed check の `link` から run ID を取り、ログを確認する
- job の rerun が必要なときは、URL の `jobs/<number>` ではなく `databaseId` を使う

```bash
gh run view <run-id> --json jobs
gh run view <run-id> --json jobs --jq '.jobs[] | {name, databaseId, conclusion}'
gh run view <run-id> --log-failed
gh run view <run-id> --job <job-database-id> --log
```

- 深掘りが必要なら [`references/ci-run-job-debug.md`](references/ci-run-job-debug.md) を読む
- 修正中は失敗していたジョブ相当のローカル検証を優先する
- Rust 変更時の主な検証コマンド（tombi の CI と整合）:

```bash
cargo build --verbose --locked
cargo nextest run --workspace --locked --no-fail-fast
cargo test --workspace --doc --locked
cargo shear
```

- 必要に応じて `cargo fmt --all --check` と `cargo clippy --workspace --all-targets --locked -- -D warnings` も実行する（CI には含まれないが、warning は放置しない方針）
- editor / docs (`editors/`、`docs/`、`blog/`) の変更時: `pnpm format`、`pnpm lint`
- Python packaging 変更時: `uv run pytest`、`uv run ruff check`、必要なら `maturin build`
- toml-test 関連の変更時: `cargo nextest run -p toml-test ...` など対象 crate に絞った実行

push 後の再確認:

```bash
gh run rerun <run-id> --failed
gh pr checks <pr-url-or-number> --watch --fail-fast
```

### 4. Copilot review request が pending で、まだコメント未着なら最大 15 分待つ

Step 1 で Copilot review request が `true` だった場合だけ、Copilot から review / thread / top-level comment が投稿済みかを確認する。Copilot review request 自体が無いなら待たずに進む。request があり、かつ Copilot からの反応がまだ 1 件も無い場合は、`gh` で最大 15 分ポーリングして待つ。

```bash
# 最大待機回数と間隔を決める（15 秒 × 60 回 = 最大 15 分）
max_attempts=60
interval=15
attempt=0

while true; do
  attempt=$((attempt + 1))

  echo "[copilot-wait] $(date -u "+%Y-%m-%dT%H:%M:%SZ") (${attempt}/${max_attempts}) Copilot review を確認中..."

  copilot_requested=$(
    gh api graphql \
      -f owner=tombi-toml \
      -f repo=tombi \
      -F number=<pr-number> \
      -f query='
query($owner: String!, $repo: String!, $number: Int!) {
  repository(owner: $owner, name: $repo) {
    pullRequest(number: $number) {
      reviewRequests(first: 100) {
        nodes {
          requestedReviewer {
            __typename
            ... on Bot { login }
            ... on User { login }
            ... on Team { slug name }
          }
        }
      }
    }
  }
}' |
      jq '
        [
          .data.repository.pullRequest.reviewRequests.nodes[]?.requestedReviewer
          | .login? // .slug? // empty
          | ascii_downcase
        ]
        | any(test("copilot"))
      '
  )

  if [ "$copilot_requested" != "true" ]; then
    echo "[copilot-wait] Copilot review request は pending ではありません。待機を終了します。"
    break
  fi

  copilot_has_feedback=$(
    gh pr view <pr-url-or-number> --json reviews,comments |
      jq '
        [
          (.reviews[]? | .author.login),
          (.comments[]? | .author.login)
        ]
        | flatten
        | map(select(. != null) | ascii_downcase)
        | any(test("copilot"))
      '
  )

  copilot_has_thread=$(
    gh api graphql \
      -f query='...reviewThreads...' \
      -F owner=tombi-toml \
      -F repo=tombi \
      -F number=<pr-number> |
      jq '
        [
          .. | objects | select(has("author")) | .author.login?
        ]
        | map(select(. != null) | ascii_downcase)
        | any(test("copilot"))
      '
  )

  if [ "$copilot_has_feedback" = "true" ] || [ "$copilot_has_thread" = "true" ]; then
    echo "[copilot-wait] Copilot からの review / comment / thread を検知したため待機を終了します。"
    break
  fi

  if [ "$attempt" -ge "$max_attempts" ]; then
    echo "[copilot-wait] 最大待機回数 (${max_attempts}) に到達しました。"
    echo "[copilot-wait] 現時点で Copilot からの review / comment / thread は確認できていません。"
    break
  fi

  if [ $((attempt % 5)) -eq 0 ]; then
    echo "[copilot-wait] まだ Copilot からの review / comment はありません。引き続き待機します..."
  fi

  sleep "$interval"
done
```

- `reviews[].author.login`、thread comment の `author.login`、top-level comment を見て Copilot からの最初の反応が付いたかを確認する
- pending の Copilot review request が無い PR では待たない
- Copilot からの反応が見えたらポーリングを止め、通常の review 対応フローに進む
- ポーリング中に review request が取り消された / 消化済みで pending ではなくなった場合も待機を止める
- 待機は `gh` だけで行い、review 確認のために browser を開かない
- 15 分待っても返ってこない場合は、その時点の状態と確認時刻をユーザーに報告する

### 5. unresolved review thread を取る

`gh pr view --json reviews,comments` では thread 単位の resolve 状態が取れない。review thread は GraphQL で取得する。

```bash
gh api graphql \
  -f owner=tombi-toml \
  -f repo=tombi \
  -F number=<number> \
  -f query='
query($owner: String!, $repo: String!, $number: Int!) {
  repository(owner: $owner, name: $repo) {
    pullRequest(number: $number) {
      id
      reviewThreads(first: 100) {
        pageInfo {
          hasNextPage
          endCursor
        }
        nodes {
          id
          isResolved
          isOutdated
          viewerCanReply
          viewerCanResolve
          path
          line
          comments(first: 20) {
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

- `isResolved == false` の thread を順に処理する
- `pageInfo.hasNextPage == true` なら追加ページも取って、未処理 thread を残さない
- `isOutdated == true` でも、指摘の意図が残っていれば無視しない
- top-level PR comment は resolve できないため、必要なら `gh pr comment` で別途返信する
- `viewerCanResolve == false` の thread は返信までは行い、権限不足をユーザーに報告する
- AI reviewer / 人間レビュアーを分けず、対象にした unresolved thread を残さない

### 6. 指摘の扱いを決める

- 妥当: 修正し、必要なテストを通し、push してから返信する
- 不要: 修正しない。コードや仕様に基づく理由を返信する
- 既対応: 対応した commit / file / test を示して返信する

対象は AI reviewer と人間レビュアーの両方を含む。技術判断で完結するものは自分で決めて進める。曖昧な仕様判断だけユーザー確認を検討する。

### 7. thread に返信して resolve する

返信は review thread に付ける。top-level の PR comment で代用しない。

```bash
gh api graphql \
  -F threadId='THREAD_ID' \
  -F body=@/tmp/review-thread-reply.md \
  -f query='
mutation($threadId: ID!, $body: String!) {
  addPullRequestReviewThreadReply(
    input: {
      pullRequestReviewThreadId: $threadId
      body: $body
    }
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

- `pullRequestReviewThreadId` と `resolveReviewThread.input.threadId` は別名に見えても、どちらも thread ID を使う
- 返信後に resolve し、無言 resolve はしない

top-level PR comment への返信:

```bash
gh pr comment <pr-url-or-number> --body-file /tmp/pr-comment-reply.md
```

## 返信の基準

### 修正したとき

- 何を変えたか
- どこで変えたか
- 何で確認したか

例:

```text
対応しました。`crates/tombi-formatter/src/...rs` の分岐を修正し、`cargo nextest run -p tombi-formatter --locked` と `cargo clippy --workspace --all-targets --locked -- -D warnings` で確認しています。
```

### 修正不要のとき

- なぜ不要か
- 既存コード / 仕様 / テストのどれが根拠か
- 必要なら代替案

例:

```text
この件は現状のままとしました。ここは TOML の `...` 仕様に合わせた分岐で、`crates/tombi-formatter/tests/...` でも同じ前提を固定しています。今回の PR では変更対象にしません。
```

## 完了条件

- 必要なコード修正が push 済み
- 失敗していた CI の原因を潰している
- 対象にした unresolved review thread すべてに返信済み
- resolve 可能な thread はすべて resolve 済み
- top-level comment / review summary があれば、必要な説明を PR comment で返している
- ユーザーには、修正内容・実行した検証・未解決事項の有無だけを短く報告する
