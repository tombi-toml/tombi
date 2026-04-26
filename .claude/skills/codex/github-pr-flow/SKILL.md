---
name: github-pr-flow
description: "Codex 専用。必要に応じて `github-pr-create`（または user-global `my-github-pr-create`）で PR を作成し、その後 `gh` で Copilot review を依頼し、統一 skill `github-pr-resolve` に待機と対応を委譲する。PR 作成手順そのものは PR-create skill を source of truth として利用する。トリガー: 'github-pr-flow', 'PR作成してCopilotレビュー依頼', 'PR作成からレビュー対応まで', '/github-pr-flow'"
metadata:
  short-description: PR作成からCopilot依頼、即時引き継ぎまで
---

# GitHub PR Flow

現在のローカル差分をもとに、必要なら別 Skill で PR を作成し、その後の Copilot レビュー依頼を `gh pr edit --add-reviewer @copilot` で行い、待機とレビュー対応を統一 Skill [`../../github-pr-resolve/SKILL.md`](../../github-pr-resolve/SKILL.md) に委譲する Codex 専用 Skill。

## Use This Skill When

- 現在の差分から PR 作成と Copilot レビュー依頼をまとめて進めたい
- 現在の差分で新規 PR を作り、その直後に Copilot へレビュー依頼したい
- Copilot にレビュー依頼した後、そのまま待機と対応まで続けて進めたい
- すべての review thread を解決した状態で PR を引き渡す

## Do Not Use This Skill When

- 新規 PR を作るだけでよい → `github-pr-create` または user-global `my-github-pr-create`
- Copilot review を使わずに PR URL を返せば十分
- 既存 PR の review 対応だけをしたい → [`../../github-pr-resolve/SKILL.md`](../../github-pr-resolve/SKILL.md)

## Preconditions

- `gh auth status` が成功する
- push 権限がある
- このリポジトリの `AGENTS.md` と対象領域の追加ルールを確認済み
- `gh --version` で `gh >= 2.89.0` を満たしている
- `@copilot` reviewer の可否確認は `gh pr edit <PR番号またはURL> --add-reviewer @copilot` を実行し、未対応ならその正確なエラー内容を確認できる
- 対象リポジトリが `github.com` 上にある（GitHub Enterprise Server の `@copilot` reviewer は未対応）
- 非対話コマンドを使う

## Workflow

### 1. 必要なら PR 作成 skill で PR URL 取得まで完了する

PR 作成そのものは別 skill に従う。優先順位:

1. リポジトリ内に `github-pr-create` skill が存在すればそれを使う
2. 無ければ user-global の `my-github-pr-create` を使う
3. どちらも使えない場合は手動で:
   - ブランチ命名（`<type>/<purpose-kebab>`）
   - `git fetch origin main` と取り込み
   - 必要な検証（tombi: `cargo fmt --all`、Rust 変更時は `cargo clippy --workspace --all-targets --locked -- -D warnings`、`cargo nextest run --workspace --locked --no-fail-fast`、`cargo test --workspace --doc --locked`）
   - commit / `git push -u origin <branch>`
   - `gh pr create --base main --head <branch> --title ... --body ... --label ...`

PR URL がまだ無い場合は、この Skill の中で PR 作成まで進めてから戻る。

### 2. `gh` で Copilot にレビュー依頼する

1. 対象 PR を PR 番号、PR URL、または head branch で特定する
2. `gh pr edit <PR番号|URL|ブランチ名> --add-reviewer @copilot` を実行する
3. reviewer 追加に成功したら、そのまま統一 Skill に委譲する

依頼できた場合:

- 以後の待機と review 対応は統一 Skill に委譲する
- reviewer UI 確認のために browser を開かない

依頼できない場合:

- `gh --version` が `2.89.0` 未満なら、先に `gh` を更新する。Homebrew 管理なら `brew upgrade gh` を使う
- `gh auth status` が失敗する場合は認証を修復してから再試行する
- `gh pr edit ... --add-reviewer @copilot` が機能未提供、権限不足、または Copilot code review 未有効を示す場合は、その正確なエラー内容をユーザーへ報告して止まる
- 通常フローとして browser 操作へフォールバックしない。ユーザーが明示的に UI での手動依頼を選んだ場合だけ案内する

### 3. 統一 Skill に直ちに委譲する

Copilot へのレビュー依頼が終わったら、この Skill では待たずに統一 Skill [`../../github-pr-resolve/SKILL.md`](../../github-pr-resolve/SKILL.md) を使う。

- 入力は PR URL または PR番号を使う
- Copilot review が未着なら最大 15 分ポーリングして待つ処理も、その Skill の責務とする
- 採用 / 不採用 / 既対応の判断、必要な修正、CI 修正、全返信、thread resolve はその Skill の責務とする
- この Skill 側では review 対応ロジックを重複実装しない

### 4. 最終状態を整理して終了する

- PR はマージしない
- PR URL、head branch、最終未解決 thread 数、残タスクの有無を記録する
- 呼び出し元が連続フロー skill の場合、merge 待機と最終 merge 実行は呼び出し元の責務として継続する
- merge や deploy などの最終判断は、この Skill 単体では扱わない

## Notes

- 使い分け:
  - 新規 PR 作成だけなら `github-pr-create` / `my-github-pr-create`
  - PR 作成後に Copilot review 依頼と thread 解決まで求められたら `github-pr-flow`
- `github-pr-flow` は PR 作成機能も持つが、その手順は PR-create skill をそのまま利用する
- Copilot レビュー依頼は `gh pr edit --add-reviewer @copilot` を通常フローとする
- `gh --version` が `2.89.0` 未満なら先に更新し、未対応時は実コマンドのエラー内容で判断する
- `@copilot` reviewer は `github.com` でのみ使う。GHES は対象外
- review 対応は統一 Skill に集約し、`github-pr-flow` 側に重複手順を書かない
- reviewer 追加や Copilot レビュー依頼のために browser を開かない
- 連続フロー skill から呼ばれた場合でも、この Skill 自体は merge しない。呼び出し元は未マージを理由に終了せず、待機と merge を続ける

## Output Checklist

- 利用した PR-create skill 名と PR 作成時の必須項目（branch、検証、commit、PR URL、label）
- Copilot レビュー依頼の成否
- 返信した thread 数
- 修正不要として解決した thread 数
- 最終未解決 thread 数
- PR は未マージのまま引き渡したこと
