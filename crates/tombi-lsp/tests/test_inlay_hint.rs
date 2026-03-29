use std::{
    fs,
    path::PathBuf,
    sync::{Mutex, OnceLock},
};

use tempfile::TempDir;
use tombi_extension::InlayHint;
use tower_lsp::{
    LspService,
    lsp_types::{
        DidOpenTextDocumentParams, InlayHintParams, Position, Range, TextDocumentIdentifier,
        TextDocumentItem, Url, WorkDoneProgressParams,
    },
};

use tombi_lsp::{
    Backend,
    handler::{handle_did_open, handle_inlay_hint},
};

const RESOLVED_VERSION_TOOLTIP: &str = "Resolved version in Cargo.lock";
const RESOLVED_UV_VERSION_TOOLTIP: &str = "Resolved version in uv.lock";

fn test_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

struct InlayHintFixture {
    _temp_dir: Option<TempDir>,
    source: String,
    source_path: PathBuf,
}

impl InlayHintFixture {
    fn new(temp_dir: Option<TempDir>, source: &str, source_path: PathBuf) -> Self {
        Self {
            _temp_dir: temp_dir,
            source: textwrap::dedent(source).trim().to_string(),
            source_path,
        }
    }

    #[allow(dead_code)]
    fn cargo(source: &str, cargo_lock: &str) -> Result<Self, Box<dyn std::error::Error>> {
        create_temp_fixture("Cargo.toml", source, vec![("Cargo.lock", cargo_lock)])
    }

    #[allow(dead_code)]
    fn cargo_workspace_root(
        workspace_cargo_toml: &str,
        member_manifest_path: &str,
        member_cargo_toml: &str,
        cargo_lock: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        create_temp_fixture(
            "Cargo.toml",
            workspace_cargo_toml,
            vec![
                (member_manifest_path, member_cargo_toml),
                ("Cargo.lock", cargo_lock),
            ],
        )
    }

    #[allow(dead_code)]
    fn cargo_workspace_member(
        workspace_cargo_toml: &str,
        member_manifest_path: &str,
        member_cargo_toml: &str,
        cargo_lock: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        create_temp_fixture(
            member_manifest_path,
            member_cargo_toml,
            vec![
                ("Cargo.toml", workspace_cargo_toml),
                ("Cargo.lock", cargo_lock),
            ],
        )
    }

    #[allow(dead_code)]
    fn pyproject(source: &str, uv_lock: &str) -> Result<Self, Box<dyn std::error::Error>> {
        create_temp_fixture("pyproject.toml", source, vec![("uv.lock", uv_lock)])
    }
}

fn create_temp_fixture(
    source_path: &str,
    source: &str,
    supporting_files: Vec<(&str, &str)>,
) -> Result<InlayHintFixture, Box<dyn std::error::Error>> {
    let temp_dir = tempfile::tempdir()?;
    let source_path = temp_dir.path().join(source_path);

    if let Some(parent) = source_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&source_path, textwrap::dedent(source).trim())?;

    for (path, content) in supporting_files {
        let path = temp_dir.path().join(path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, textwrap::dedent(content).trim())?;
    }

    Ok(InlayHintFixture::new(Some(temp_dir), source, source_path))
}

async fn collect_inlay_hints(
    source: &str,
    source_path: PathBuf,
) -> Result<Option<Vec<InlayHint>>, Box<dyn std::error::Error>> {
    let (service, _) = LspService::new(|client| {
        Backend::new(
            client,
            &tombi_lsp::backend::Options {
                offline: Some(true),
                no_cache: Some(false),
            },
        )
    });
    let backend = service.inner();

    let toml_text = textwrap::dedent(source).trim().to_string();
    let toml_file_url =
        Url::from_file_path(&source_path).expect("failed to convert source file path to URL");

    handle_did_open(
        backend,
        DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: toml_file_url.clone(),
                language_id: "toml".to_string(),
                version: 0,
                text: toml_text.clone(),
            },
        },
    )
    .await;

    let lines = toml_text.lines().collect::<Vec<_>>();
    let last_line = lines.len().saturating_sub(1) as u32;
    let last_column = lines.last().map_or(0, |line| line.len() as u32);

    Ok(handle_inlay_hint(
        backend,
        InlayHintParams {
            text_document: TextDocumentIdentifier { uri: toml_file_url },
            range: Range::new(Position::new(0, 0), Position::new(last_line, last_column)),
            work_done_progress_params: WorkDoneProgressParams::default(),
        },
    )
    .await?)
}

fn expected_hint(position: tombi_text::Position, label: &str, tooltip: &str) -> InlayHint {
    InlayHint {
        position,
        label: label.to_string(),
        kind: Some(tower_lsp::lsp_types::InlayHintKind::TYPE),
        tooltip: Some(tooltip.to_string()),
        padding_left: Some(true),
        padding_right: Some(false),
    }
}

macro_rules! test_inlay_hint {
    (@run $name:ident, $fixture:expr, $expected:expr) => {
        #[tokio::test]
        async fn $name() -> Result<(), Box<dyn std::error::Error>> {
            let _guard = test_lock()
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            tombi_test_lib::init_log();

            let fixture = $fixture?;
            let result = collect_inlay_hints(&fixture.source, fixture.source_path).await?;

            pretty_assertions::assert_eq!(result, $expected);

            Ok(())
        }
    };
    (
        #[tokio::test]
        async fn $name:ident($fixture:expr) -> Ok($expected:expr);
    ) => {
        test_inlay_hint!(@run $name, $fixture, $expected);
    };
    (
        #[tokio::test]
        async fn $name:ident(
            $source:expr,
            CargoFile {
                $(cargo_lock = $cargo_lock:expr)?
                $(,)?
            }
        ) -> Ok($expected:expr);
    ) => {
        test_inlay_hint!(
            @run
            $name,
            create_temp_fixture("Cargo.toml", $source, {
                let mut files = Vec::new();
                $(files.push(("Cargo.lock", $cargo_lock));)?
                files
            }),
            $expected
        );
    };
    (
        #[tokio::test]
        async fn $name:ident(
            $source:expr,
            WorkspaceRootFile {
                member_path = $member_path:expr,
                member_context = $member_context:expr
                $(, cargo_lock = $cargo_lock:expr)?
                $(,)?
            }
        ) -> Ok($expected:expr);
    ) => {
        test_inlay_hint!(
            @run
            $name,
            create_temp_fixture("Cargo.toml", $source, {
                let mut files = vec![($member_path, $member_context)];
                $(files.push(("Cargo.lock", $cargo_lock));)?
                files
            }),
            $expected
        );
    };
    (
        #[tokio::test]
        async fn $name:ident(
            $source:expr,
            WorkspaceFile {
                path = $path:expr,
                context = $context:expr
                $(, cargo_lock = $cargo_lock:expr)?
                $(, uv_lock = $uv_lock:expr)?
                $(,)?
            }
        ) -> Ok($expected:expr);
    ) => {
        test_inlay_hint!(
            @run
            $name,
            create_temp_fixture($path, $source, {
                let context = $context;
                let mut files = Vec::new();
                if !context.is_empty() {
                    files.push(("Cargo.toml", context));
                }
                $(files.push(("Cargo.lock", $cargo_lock));)?
                $(files.push(("uv.lock", $uv_lock));)?
                files
            }),
            $expected
        );
    };
    (
        #[tokio::test]
        async fn $name:ident(
            $source:expr,
            PyprojectFile {
                $(uv_lock = $uv_lock:expr)?
                $(,)?
            }
        ) -> Ok($expected:expr);
    ) => {
        test_inlay_hint!(
            @run
            $name,
            create_temp_fixture("pyproject.toml", $source, {
                let mut files = Vec::new();
                $(files.push(("uv.lock", $uv_lock));)?
                files
            }),
            $expected
        );
    };
}

test_inlay_hint!(
    #[tokio::test]
    async fn inlay_hint_for_string_dependency_uses_the_resolved_lockfile_version(
        r#"
        [package]
        name = "demo"
        version = "0.1.0"

        [dependencies]
        serde = "1.0.219"
        "#,
        CargoFile {
            cargo_lock = r#"
            version = 4

            [[package]]
            name = "demo"
            version = "0.1.0"
            dependencies = ["serde 1.0.228"]

            [[package]]
            name = "serde"
            version = "1.0.228"
            "#,
        }
    ) -> Ok(Some(vec![expected_hint(
        tombi_text::Position::new(5, 17),
        r#" → "1.0.228""#,
        RESOLVED_VERSION_TOOLTIP,
    )]));
);

test_inlay_hint!(
    #[tokio::test]
    async fn inlay_hint_for_dotted_version_key_is_rendered_after_the_version_literal(
        r#"
        [package]
        name = "demo"
        version = "0.1.0"

        [dependencies]
        addr.version = "0.15.5"
        "#,
        CargoFile {
            cargo_lock = r#"
            version = 4

            [[package]]
            name = "demo"
            version = "0.1.0"
            dependencies = ["addr 0.15.6"]

            [[package]]
            name = "addr"
            version = "0.15.6"
            "#,
        }
    ) -> Ok(Some(vec![expected_hint(
        tombi_text::Position::new(5, 23),
        r#" → "0.15.6""#,
        RESOLVED_VERSION_TOOLTIP,
    )]));
);

test_inlay_hint!(
    #[tokio::test]
    async fn inlay_hint_for_inline_table_version_uses_the_resolved_lockfile_version(
        r#"
        [package]
        name = "demo"
        version = "0.1.0"

        [dependencies]
        tokio = { version = "1.45.0", features = ["fs"] }
        "#,
        CargoFile {
            cargo_lock = r#"
            version = 4

            [[package]]
            name = "demo"
            version = "0.1.0"
            dependencies = ["tokio 1.47.1"]

            [[package]]
            name = "tokio"
            version = "1.47.1"
            "#,
        }
    ) -> Ok(Some(vec![expected_hint(
        tombi_text::Position::new(5, 28),
        r#" → "1.47.1""#,
        RESOLVED_VERSION_TOOLTIP,
    )]));
);

test_inlay_hint!(
    #[tokio::test]
    async fn inlay_hint_for_workspace_dependencies_uses_workspace_members_lockfile_resolution(
        r#"
        [workspace]
        members = ["crates/app"]

        [workspace.dependencies]
        serde = "1.0.219"
        "#,
        WorkspaceRootFile {
            member_path = "crates/app/Cargo.toml",
            member_context = r#"
            [package]
            name = "app"
            version = "0.1.0"
            "#,
            cargo_lock = r#"
            version = 4

            [[package]]
            name = "app"
            version = "0.1.0"
            dependencies = ["serde 1.0.228"]

            [[package]]
            name = "serde"
            version = "1.0.228"
            "#,
        }
    ) -> Ok(Some(vec![expected_hint(
        tombi_text::Position::new(4, 17),
        r#" → "1.0.228""#,
        RESOLVED_VERSION_TOOLTIP,
    )]));
);

test_inlay_hint!(
    #[tokio::test]
    async fn inlay_hint_for_workspace_inline_table_dependency_uses_the_resolved_lockfile_version(
        r#"
        [workspace]
        members = ["crates/app"]

        [workspace.dependencies]
        serde_json = { version = "1.0.140", features = ["preserve_order"] }
        "#,
        WorkspaceRootFile {
            member_path = "crates/app/Cargo.toml",
            member_context = r#"
            [package]
            name = "app"
            version = "0.1.0"
            "#,
            cargo_lock = r#"
            version = 4

            [[package]]
            name = "app"
            version = "0.1.0"
            dependencies = ["serde_json 1.0.142"]

            [[package]]
            name = "serde_json"
            version = "1.0.142"
            "#,
        }
    ) -> Ok(Some(vec![expected_hint(
        tombi_text::Position::new(4, 34),
        r#" → "1.0.142""#,
        RESOLVED_VERSION_TOOLTIP,
    )]));
);

test_inlay_hint!(
    #[tokio::test]
    async fn inlay_hint_for_workspace_inheritance_is_rendered_even_when_versions_match(
        r#"
        [package]
        name = "app"
        version = "0.1.0"

        [dependencies]
        addr.workspace = true
        "#,
        WorkspaceFile {
            path = "crates/app/Cargo.toml",
            context = r#"
            [workspace]
            members = ["crates/app"]
            "#,
            cargo_lock = r#"
            version = 4

            [[package]]
            name = "app"
            version = "0.1.0"
            dependencies = ["addr 0.15.6"]

            [[package]]
            name = "addr"
            version = "0.15.6"
            "#,
        }
    ) -> Ok(Some(vec![expected_hint(
        tombi_text::Position::new(5, 21),
        r#" → "0.15.6""#,
        RESOLVED_VERSION_TOOLTIP,
    )]));
);

test_inlay_hint!(
    #[tokio::test]
    async fn inlay_hint_is_not_rendered_when_the_version_already_matches_the_lockfile(
        r#"
        [package]
        name = "demo"
        version = "0.1.0"

        [dependencies]
        serde = "1.0.228"
        "#,
        CargoFile {
            cargo_lock = r#"
            version = 4

            [[package]]
            name = "demo"
            version = "0.1.0"
            dependencies = ["serde 1.0.228"]

            [[package]]
            name = "serde"
            version = "1.0.228"
            "#,
        }
    ) -> Ok(None);
);

test_inlay_hint!(
    #[tokio::test]
    async fn inlay_hint_for_workspace_path_crate_is_rendered_from_the_lockfile(
        r#"
        [package]
        name = "app"
        version = "0.1.0"

        [dependencies]
        tombi-text.workspace = true
        "#,
        WorkspaceFile {
            path = "crates/app/Cargo.toml",
            context = r#"
            [workspace]
            members = ["crates/app"]
            "#,
            cargo_lock = r#"
            version = 4

            [[package]]
            name = "app"
            version = "0.1.0"
            dependencies = ["tombi-text 0.0.0-dev"]

            [[package]]
            name = "tombi-text"
            version = "0.0.0-dev"
            "#,
        }
    ) -> Ok(Some(vec![expected_hint(
        tombi_text::Position::new(5, 27),
        r#" → "0.0.0-dev""#,
        RESOLVED_VERSION_TOOLTIP,
    )]));
);

test_inlay_hint!(
    #[tokio::test]
    async fn inlay_hint_for_path_dependency_is_rendered_after_the_path_value(
        r#"
        [package]
        name = "demo"
        version = "0.1.0"

        [dependencies]
        serde = { path = "vendor/serde" }
        "#,
        CargoFile {
            cargo_lock = r#"
            version = 4

            [[package]]
            name = "demo"
            version = "0.1.0"
            dependencies = ["serde 1.0.228"]

            [[package]]
            name = "serde"
            version = "1.0.228"
            "#,
        }
    ) -> Ok(Some(vec![expected_hint(
        tombi_text::Position::new(5, 31),
        r#" → "1.0.228""#,
        RESOLVED_VERSION_TOOLTIP,
    )]));
);

test_inlay_hint!(
    #[tokio::test]
    async fn inlay_hint_for_git_dependency_is_rendered_after_the_git_value(
        r#"
        [package]
        name = "demo"
        version = "0.1.0"

        [dependencies]
        serde = { git = "https://github.com/serde-rs/serde" }
        "#,
        CargoFile {
            cargo_lock = r#"
            version = 4

            [[package]]
            name = "demo"
            version = "0.1.0"
            dependencies = ["serde 1.0.228"]

            [[package]]
            name = "serde"
            version = "1.0.228"
            source = "git+https://github.com/serde-rs/serde"
            "#,
        }
    ) -> Ok(Some(vec![expected_hint(
        tombi_text::Position::new(5, 51),
        r#" → "1.0.228""#,
        RESOLVED_VERSION_TOOLTIP,
    )]));
);

test_inlay_hint!(
    #[tokio::test]
    async fn pyproject_inlay_hint_uses_uv_lock_for_project_dependencies(
        r#"
        [project]
        name = "demo"
        version = "0.1.0"
        dependencies = ["pytest>=8.0"]

        [dependency-groups]
        dev = ["ruff>=0.7.0"]
        "#,
        PyprojectFile {
            uv_lock = r#"
            version = 1

            [[package]]
            name = "demo"
            version = "0.1.0"
            dependencies = [{ name = "pytest" }]

            [package.dev-dependencies]
            dev = [{ name = "ruff" }]

            [[package]]
            name = "pytest"
            version = "8.3.3"

            [[package]]
            name = "ruff"
            version = "0.7.4"
            "#,
        }
    ) -> Ok(Some(vec![
        expected_hint(
            tombi_text::Position::new(3, 29),
            r#" → "8.3.3""#,
            RESOLVED_UV_VERSION_TOOLTIP,
        ),
        expected_hint(
            tombi_text::Position::new(6, 20),
            r#" → "0.7.4""#,
            RESOLVED_UV_VERSION_TOOLTIP,
        ),
    ]));
);

test_inlay_hint!(
    #[tokio::test]
    async fn pyproject_inlay_hint_finds_uv_lock_from_ancestor_directory(
        r#"
        [project]
        name = "app"
        version = "0.1.0"
        dependencies = ["pytest>=8.0"]
        "#,
        WorkspaceFile {
            path = "members/app/pyproject.toml",
            context = "",
            uv_lock = r#"
            version = 1

            [[package]]
            name = "app"
            version = "0.1.0"
            dependencies = [{ name = "pytest" }]

            [[package]]
            name = "pytest"
            version = "8.3.3"
            "#,
        }
    ) -> Ok(Some(vec![expected_hint(
        tombi_text::Position::new(3, 29),
        r#" → "8.3.3""#,
        RESOLVED_UV_VERSION_TOOLTIP,
    )]));
);
