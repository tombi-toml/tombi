# CI Run / Job Debug

PR 対応中に GitHub Actions の failed run / job を深掘りするときの補助メモ。checkout / push / review thread 返信は `SKILL.md` 側で扱い、このファイルはログ解析とローカル再現に絞る。

## 使う入力

- run URL
- job URL
- run ID

例:

```text
https://github.com/tombi-toml/tombi/actions/runs/<run-id>/job/<job-id>?pr=<pr-number>
```

## 1. run / job を特定する

```bash
url="https://github.com/tombi-toml/tombi/actions/runs/<run-id>/job/<job-id>?pr=<pr-number>"
run_id=$(echo "$url" | grep -oE '/runs/([0-9]+)' | grep -oE '[0-9]+')
job_id=$(echo "$url" | grep -oE '/job/([0-9]+)' | grep -oE '[0-9]+')
repo=$(echo "$url" | grep -oE 'github.com/([^/]+/[^/]+)' | sed 's|github.com/||')
```

PR 起点なら先に `gh pr checks <pr> --json name,state,bucket,link,workflow` で failed check を出し、`link` から run ID を取る。

## 2. run 全体の状態を見る

```bash
gh run view "$run_id" --repo "$repo"
gh run view "$run_id" --repo "$repo" --json jobs
gh run view "$run_id" --repo "$repo" --json jobs --jq '.jobs[] | {name, databaseId, conclusion}'
```

- `bucket == "fail"` または `conclusion == "failure"` を優先する
- job 単位で掘るなら `databaseId` を使う

## 3. 失敗ログを切る

```bash
gh run view "$run_id" --repo "$repo" --log-failed
gh run view "$run_id" --repo "$repo" --job <job-database-id> --log
gh run view "$run_id" --repo "$repo" --log 2>&1 | grep -i "error\\|failed" | head -50
```

## 4. tombi の workflow パターン別に切り分ける

tombi の主な workflow:

- `ci_rust.yml`: `cargo build --verbose --locked` + `cargo nextest run --verbose --locked` + `cargo shear`
- `ci_python.yml`: pyproject 経由の Python テスト
- `ci_vscode_extensions.yml`: VS Code extension のビルド / lint
- `ci_intellij_plugin.yml`: IntelliJ plugin
- `ci_install_script.yml`: `docs/public/install.sh` のインストール検証
- `ci_nix.yml`: Nix ビルド
- `toml-test.yml`: TOML 仕様適合性テスト
- `deploy_docs.yml`、`release_*.yml`: 配布面

### `cargo build` 失敗

```bash
cargo build --verbose --locked
cargo check --workspace --all-targets --locked
```

- `E0425`: 未定義の変数 / 関数
- `E0599`: メソッドや trait import 不足
- `E0308`: 型不一致
- `E0277`: trait bound 不足

### `cargo nextest run` 失敗

```bash
cargo nextest run --workspace --locked --no-fail-fast
cargo test --workspace --doc --locked
```

- failure name から対象 crate と test を特定する
- 該当 crate に絞って再実行: `cargo nextest run -p <crate> --locked`
- `tombi-formatter` 系の差分テストは fixture / ungrammar 更新が原因のことが多い

### `cargo shear` 失敗

```bash
cargo shear
```

- 未使用 dep を検出。指摘された crate を `Cargo.toml` から削除する
- `cargo shear --fix` で自動削除も可能

### `fmt` / `clippy`（tombi の CI には未設定だが手元で出やすい）

```bash
cargo fmt --all
cargo fmt --all --check
git diff --stat
cargo clippy --workspace --all-targets --locked -- -D warnings
```

### Node / TypeScript（editors / docs）

```bash
pnpm format
pnpm lint
pnpm -C editors/vscode <task>
pnpm -C docs <task>
```

- `biome` ベース。`pnpm format` / `pnpm lint` は `--write` 付き

### Python packaging

```bash
uv run pytest
uv run ruff check
maturin build  # 必要な場合のみ
```

### toml-test 仕様適合

```bash
cargo nextest run -p toml-test --locked
```

仕様差分が原因なら、`toml-test/` の fixture と `crates/tombi-parser` / `crates/tombi-formatter` の整合を確認する。

## 5. ローカル検証セット

必要に応じて次を順に打つ。

```bash
cargo build --verbose --locked
cargo nextest run --workspace --locked --no-fail-fast
cargo test --workspace --doc --locked
cargo shear
```

warning ベースの追加検証:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
```

GitHub Actions 本番で submodule / 認証絡みのエラーが出ても `git config --global` / `git config --local` / `git config --system` で Git 設定を書き換えてはいけない。`url.*.insteadOf` や credential helper に PAT / 認証情報を保存して回避することも禁止する。

`git@github.com:` の rewrite がグローバル / システム設定にあるだけなら、`env GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null ...` で一時的に無効化して再現する。それでも進まない場合は、Git 設定を書き換えずにその時点の失敗内容を報告して止まる。

## 6. この reference の完了条件

- failed run / job を特定できている
- 根拠ログを示せている
- ローカル再現コマンドが整理できている
- commit / push / review reply は `SKILL.md` 側へ戻して続行できる
