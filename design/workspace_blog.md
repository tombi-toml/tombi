本記事では、私が開発している TOML ツールキット [Tombi](https://github.com/tombi-toml/tombi)のワークスペースに関連する機能を紹介します。
Tombi は TOML のための Formatter, Linter, Language Server を包含した統合ツールです。

私たちのチームでは、パッケージ管理ツールを [Poetry](https://python-poetry.org/) から [uv](https://docs.astral.sh/uv/) へ移行したことを機に、[VSCode Multi-root Workspaces](https://code.visualstudio.com/docs/editing/workspaces/multi-root-workspaces) から [uvのワークスペース機能](https://docs.astral.sh/uv/concepts/projects/workspaces/) を利用したモノレポ開発に切り替えました。
この記事を参考にすることで、依存関係のバージョン管理をルートの pyproject.toml に集約し、さらに Tombi の機能を用いることで快適なワークスペース管理ができるようになります。

## ワークスペース機能とは

ワークスペース機能とは、モノレポ（単一リポジトリ）開発において、レポジトリに存在する複数のプロジェクトを管理するための仕組みです。

VSCode などエディタレベルでサポートするものもありますが、最近では各プログラミング言語のパッケージマネージャーレベルで採用される傾向が増えています。
TOMLを設定ファイルに採用し、ワークスペース機能を備えた言語として、RustとPythonが有名です。

- **Rust：**Cargo が[サポート](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html)。ルートの Cargo.toml でワークスペースを定義し、その依存関係をメンバークレートが継承することで、データの共通化を容易にします。
- **Python：**uv が[サポート](https://docs.astral.sh/uv/concepts/projects/workspaces/)。Cargo を参考に開発されており、ルートの pyproject.toml でワークスペースを定義し、リポジトリ内の相互依存パッケージを統一された仮想環境で扱うことができます。

今回は Python を中心に説明しますが、 Cargo でも同様のことができます。

### uvのワークスペース機能を利用したディレクトリ構成

それでは、Python でのワークスペースを用いたモノレポ開発のディレクトリ構成について簡単に説明しましょう。
ワークスペース機能を利用したディレクトリ構造は、次のようにルートにワークスペース全体の設定をするための pyproject.toml を用意し、 packages に自分たちのパッケージを配置します。
uv のワークスペース機能を利用した場合、プロジェクトのルートに唯一の uv.lock が作成され、そこで全てのパッケージが管理されます。また、この uv.lock に基づいた一つの仮想環境がワークスペース上に作成されます。

```plaintext
my_project
├── packages
│ ├── app1
│ │ └── pyproject.toml
│ └── app2
│   └── pyproject.toml
├── uv.lock
└── pyproject.toml
```

次に、各 pyproject.toml を見てみましょう。まずはルートの pyproject.toml です。
ここでは全てのメンバーのパッケージのパスと、利用する全てのサードパーティ製のパッケージのバージョンが管理されます。
`tool.uv.workspace.members` にワークスペース上で利用するメンバーのパッケージを指定します。

```toml
[project]
name = "my_project"
version = "0.1.0"
dependencies = ["pandas>=2.2.3", "pydantic>=2.10"]

[tool.uv.workspace]
members = ["packages/*"]
```

続いて、各メンバーパッケージの pyproject.toml です。
`tool.uv.sources` にワークスペースから利用したい他のメンバーパッケージを指定します。

```toml
[project]
name = "app1"
version = "0.1.0"
dependencies = ["app2", "pydantic"]

[tool.uv.sources]
app2 = { workspace = true }
```

```toml
[project]
name = "app2"
version = "0.1.0"
dependencies = ["pandas", "pydantic"]
```

ここで重要なのは、メンバーの pyproject.toml では**サードパーティ製のパッケージのバージョンを指定しない**ことです。バージョンはルートの pyproject.toml で指定されているので、 uv はそのバージョン指定を元にバージョンを決定できます。
もし、メンバーでもバージョン指定を記述しているとバージョン制約の不整合が起こり、依存関係の解決が失敗しやすくなります。なるべくワークスペース側でバージョンを管理するようにしましょう。

## Tombiによるワークスペース開発の強力なサポート

Tombi は上記の構成に従ったワークスペース開発の編集を快適にする、2つの主要機能を提供します。

### コードアクション（Code Action）

ワークスペース内での依存関係を整理するための自動変換機能です。メンバーパッケージの開発中に新しくサードパーティのパッケージを追加したときに利用します。

- **"Add to Workspace and Use Dependency"**: メンバーパッケージにのみ存在するサードパーティ製のパッケージを**自動でワークスペースルートに追加**し、その上でメンバーファイルの記述をワークスペース参照形式に変換します。これにより、依存関係の中央集権化を支援します。

![Add to Workspace and Use Dependency Code Action](./workspace_blog/images/uv_CodeAction_AddToWorkspaceAndUseWorkspaceDependency.gif)

- **"Use Workspace Dependency"**: これは、すでにサードパーティ製のパッケージがワークスペースで管理されている場合に利用でき、ワークスペース側を参照する形に自動変換します。

![Use Workspace Dependency Code Action](./workspace_blog/images/uv_CodeAction_UseWorkspaceDependency.gif)

### 定義へ移動（Goto Definition）

Tombi はワークスペースパッケージ↔︎メンバーパッケージの定義移動をサポートしており、パッケージの定義元、参照元への移動を容易にしています。また、この機能でサードパーティ製のライブラリがどれだけ利用されているかも確認することができます。

![Goto Definitoin](./workspace_blog/images/uv_GoToDefinition.png)

## まとめ

Tombiは、Cargoとuvのワークスペース機能を活用したモノレポ開発において、「ワークスペース依存関係への自動変換」と「定義へ移動」という強力なツールサポートを提供します。
また、その他にも [JSON Schema Store](https://www.schemastore.org/) に基づいた診断機能・補完機能・[自動ソート機能](https://tombi-toml.github.io/tombi/docs/formatter/auto-sorting)などを提供しています。

Tombi を応援したい方は是非 [By Me a Coffee](https://buymeacoffee.com/tombi) や [GitHub Sponsor](https://github.com/sponsors/tombi-toml) でサポートをお願いします ☕️
