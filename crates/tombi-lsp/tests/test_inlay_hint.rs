use std::{
    fs,
    path::PathBuf,
    sync::{Mutex, OnceLock},
};

use tempfile::TempDir;
use tombi_extension::InlayHint;
use tombi_test_lib::project_root_path;
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

fn create_temp_cargo_project(
    cargo_toml: &str,
    cargo_lock: &str,
) -> Result<(TempDir, PathBuf), Box<dyn std::error::Error>> {
    let temp_dir = tempfile::tempdir()?;
    let cargo_toml_path = temp_dir.path().join("Cargo.toml");
    fs::write(&cargo_toml_path, textwrap::dedent(cargo_toml).trim())?;
    fs::write(
        temp_dir.path().join("Cargo.lock"),
        textwrap::dedent(cargo_lock).trim(),
    )?;

    Ok((temp_dir, cargo_toml_path))
}

fn create_temp_pyproject(
    pyproject_toml: &str,
    uv_lock: &str,
) -> Result<(TempDir, PathBuf), Box<dyn std::error::Error>> {
    let temp_dir = tempfile::tempdir()?;
    let pyproject_toml_path = temp_dir.path().join("pyproject.toml");
    fs::write(
        &pyproject_toml_path,
        textwrap::dedent(pyproject_toml).trim(),
    )?;
    fs::write(
        temp_dir.path().join("uv.lock"),
        textwrap::dedent(uv_lock).trim(),
    )?;

    Ok((temp_dir, pyproject_toml_path))
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

#[tokio::test]
async fn inlay_hint_for_string_dependency_uses_the_resolved_lockfile_version()
-> Result<(), Box<dyn std::error::Error>> {
    let _guard = test_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    tombi_test_lib::init_log();

    let result = collect_inlay_hints(
        r#"
        [package]
        name = "tombi-lsp"
        version.workspace = true

        [dependencies]
        serde = "1.0.219"
        "#,
        project_root_path().join("crates/tombi-lsp/Cargo.toml"),
    )
    .await?;

    pretty_assertions::assert_eq!(
        result,
        Some(vec![InlayHint {
            position: tombi_text::Position::new(5, 17),
            label: r#" → "1.0.228""#.to_string(),
            kind: Some(tower_lsp::lsp_types::InlayHintKind::TYPE),
            tooltip: Some(RESOLVED_VERSION_TOOLTIP.to_string()),
            padding_left: Some(true),
            padding_right: Some(false),
        }])
    );

    Ok(())
}

#[tokio::test]
async fn inlay_hint_for_dotted_version_key_is_rendered_after_the_version_literal()
-> Result<(), Box<dyn std::error::Error>> {
    let _guard = test_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    tombi_test_lib::init_log();

    let result = collect_inlay_hints(
        r#"
        [package]
        name = "tombi-validator"
        version.workspace = true

        [dependencies]
        addr.version = "0.15.5"
        "#,
        project_root_path().join("crates/tombi-validator/Cargo.toml"),
    )
    .await?;

    pretty_assertions::assert_eq!(
        result,
        Some(vec![InlayHint {
            position: tombi_text::Position::new(5, 23),
            label: r#" → "0.15.6""#.to_string(),
            kind: Some(tower_lsp::lsp_types::InlayHintKind::TYPE),
            tooltip: Some(RESOLVED_VERSION_TOOLTIP.to_string()),
            padding_left: Some(true),
            padding_right: Some(false),
        }])
    );

    Ok(())
}

#[tokio::test]
async fn inlay_hint_for_inline_table_version_uses_the_resolved_lockfile_version()
-> Result<(), Box<dyn std::error::Error>> {
    let _guard = test_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    tombi_test_lib::init_log();

    let result = collect_inlay_hints(
        r#"
        [package]
        name = "tombi-cli"
        version.workspace = true

        [dependencies]
        tokio = { version = "1.45.0", features = ["fs"] }
        "#,
        project_root_path().join("rust/tombi-cli/Cargo.toml"),
    )
    .await?;

    pretty_assertions::assert_eq!(
        result,
        Some(vec![InlayHint {
            position: tombi_text::Position::new(5, 28),
            label: r#" → "1.47.1""#.to_string(),
            kind: Some(tower_lsp::lsp_types::InlayHintKind::TYPE),
            tooltip: Some(RESOLVED_VERSION_TOOLTIP.to_string()),
            padding_left: Some(true),
            padding_right: Some(false),
        }])
    );

    Ok(())
}

#[tokio::test]
async fn inlay_hint_for_workspace_dependencies_uses_workspace_members_lockfile_resolution()
-> Result<(), Box<dyn std::error::Error>> {
    let _guard = test_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    tombi_test_lib::init_log();

    let result = collect_inlay_hints(
        r#"
        [workspace]
        members = ["crates/tombi-lsp"]

        [workspace.dependencies]
        serde = "1.0.219"
        "#,
        project_root_path().join("Cargo.toml"),
    )
    .await?;

    pretty_assertions::assert_eq!(
        result,
        Some(vec![InlayHint {
            position: tombi_text::Position::new(4, 17),
            label: r#" → "1.0.228""#.to_string(),
            kind: Some(tower_lsp::lsp_types::InlayHintKind::TYPE),
            tooltip: Some(RESOLVED_VERSION_TOOLTIP.to_string()),
            padding_left: Some(true),
            padding_right: Some(false),
        }])
    );

    Ok(())
}

#[tokio::test]
async fn inlay_hint_for_workspace_inline_table_dependency_uses_the_resolved_lockfile_version()
-> Result<(), Box<dyn std::error::Error>> {
    let _guard = test_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    tombi_test_lib::init_log();

    let result = collect_inlay_hints(
        r#"
        [workspace]
        members = ["crates/tombi-lsp"]

        [workspace.dependencies]
        serde_json = { version = "1.0.140", features = ["preserve_order"] }
        "#,
        project_root_path().join("Cargo.toml"),
    )
    .await?;

    pretty_assertions::assert_eq!(
        result,
        Some(vec![InlayHint {
            position: tombi_text::Position::new(4, 34),
            label: r#" → "1.0.142""#.to_string(),
            kind: Some(tower_lsp::lsp_types::InlayHintKind::TYPE),
            tooltip: Some(RESOLVED_VERSION_TOOLTIP.to_string()),
            padding_left: Some(true),
            padding_right: Some(false),
        }])
    );

    Ok(())
}

#[tokio::test]
async fn inlay_hint_for_workspace_inheritance_is_rendered_even_when_versions_match()
-> Result<(), Box<dyn std::error::Error>> {
    let _guard = test_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    tombi_test_lib::init_log();

    let result = collect_inlay_hints(
        r#"
        [package]
        name = "tombi-validator"
        version.workspace = true

        [dependencies]
        addr.workspace = true
        "#,
        project_root_path().join("crates/tombi-validator/Cargo.toml"),
    )
    .await?;

    pretty_assertions::assert_eq!(
        result,
        Some(vec![InlayHint {
            position: tombi_text::Position::new(5, 21),
            label: r#" → "0.15.6""#.to_string(),
            kind: Some(tower_lsp::lsp_types::InlayHintKind::TYPE),
            tooltip: Some(RESOLVED_VERSION_TOOLTIP.to_string()),
            padding_left: Some(true),
            padding_right: Some(false),
        }])
    );

    Ok(())
}

#[tokio::test]
async fn inlay_hint_is_not_rendered_when_the_version_already_matches_the_lockfile()
-> Result<(), Box<dyn std::error::Error>> {
    let _guard = test_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    tombi_test_lib::init_log();

    let result = collect_inlay_hints(
        r#"
        [package]
        name = "tombi-lsp"
        version.workspace = true

        [dependencies]
        serde = "1.0.228"
        "#,
        project_root_path().join("crates/tombi-lsp/Cargo.toml"),
    )
    .await?;

    pretty_assertions::assert_eq!(result, None);

    Ok(())
}

#[tokio::test]
async fn inlay_hint_for_workspace_path_crate_is_rendered_from_the_lockfile()
-> Result<(), Box<dyn std::error::Error>> {
    let _guard = test_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    tombi_test_lib::init_log();

    let result = collect_inlay_hints(
        r#"
        [package]
        name = "tombi-validator"
        version.workspace = true

        [dependencies]
        tombi-text.workspace = true
        "#,
        project_root_path().join("crates/tombi-validator/Cargo.toml"),
    )
    .await?;

    pretty_assertions::assert_eq!(
        result,
        Some(vec![InlayHint {
            position: tombi_text::Position::new(5, 27),
            label: r#" → "0.0.0-dev""#.to_string(),
            kind: Some(tower_lsp::lsp_types::InlayHintKind::TYPE),
            tooltip: Some(RESOLVED_VERSION_TOOLTIP.to_string()),
            padding_left: Some(true),
            padding_right: Some(false),
        }])
    );

    Ok(())
}

#[tokio::test]
async fn inlay_hint_for_path_dependency_is_rendered_after_the_path_value()
-> Result<(), Box<dyn std::error::Error>> {
    let _guard = test_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    tombi_test_lib::init_log();

    let source = r#"
        [package]
        name = "demo"
        version = "0.1.0"

        [dependencies]
        serde = { path = "vendor/serde" }
    "#;
    let lock = r#"
        version = 4

        [[package]]
        name = "demo"
        version = "0.1.0"
        dependencies = ["serde 1.0.228"]

        [[package]]
        name = "serde"
        version = "1.0.228"
    "#;
    let (_temp_dir, cargo_toml_path) = create_temp_cargo_project(source, lock)?;

    let result = collect_inlay_hints(source, cargo_toml_path).await?;

    pretty_assertions::assert_eq!(
        result,
        Some(vec![InlayHint {
            position: tombi_text::Position::new(5, 31),
            label: r#" → "1.0.228""#.to_string(),
            kind: Some(tower_lsp::lsp_types::InlayHintKind::TYPE),
            tooltip: Some(RESOLVED_VERSION_TOOLTIP.to_string()),
            padding_left: Some(true),
            padding_right: Some(false),
        }])
    );

    Ok(())
}

#[tokio::test]
async fn inlay_hint_for_git_dependency_is_rendered_after_the_git_value()
-> Result<(), Box<dyn std::error::Error>> {
    let _guard = test_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    tombi_test_lib::init_log();

    let source = r#"
        [package]
        name = "demo"
        version = "0.1.0"

        [dependencies]
        serde = { git = "https://github.com/serde-rs/serde" }
    "#;
    let lock = r#"
        version = 4

        [[package]]
        name = "demo"
        version = "0.1.0"
        dependencies = ["serde 1.0.228"]

        [[package]]
        name = "serde"
        version = "1.0.228"
        source = "git+https://github.com/serde-rs/serde"
    "#;
    let (_temp_dir, cargo_toml_path) = create_temp_cargo_project(source, lock)?;

    let result = collect_inlay_hints(source, cargo_toml_path).await?;

    pretty_assertions::assert_eq!(
        result,
        Some(vec![InlayHint {
            position: tombi_text::Position::new(5, 51),
            label: r#" → "1.0.228""#.to_string(),
            kind: Some(tower_lsp::lsp_types::InlayHintKind::TYPE),
            tooltip: Some(RESOLVED_VERSION_TOOLTIP.to_string()),
            padding_left: Some(true),
            padding_right: Some(false),
        }])
    );

    Ok(())
}

#[tokio::test]
async fn pyproject_inlay_hint_uses_uv_lock_for_project_dependencies()
-> Result<(), Box<dyn std::error::Error>> {
    let _guard = test_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    tombi_test_lib::init_log();

    let source = r#"
        [project]
        name = "demo"
        version = "0.1.0"
        dependencies = ["pytest>=8.0"]

        [dependency-groups]
        dev = ["ruff>=0.7.0"]
    "#;
    let lock = r#"
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
    "#;
    let (_temp_dir, pyproject_toml_path) = create_temp_pyproject(source, lock)?;

    let result = collect_inlay_hints(source, pyproject_toml_path).await?;

    pretty_assertions::assert_eq!(
        result,
        Some(vec![
            InlayHint {
                position: tombi_text::Position::new(3, 29),
                label: r#" → "8.3.3""#.to_string(),
                kind: Some(tower_lsp::lsp_types::InlayHintKind::TYPE),
                tooltip: Some(RESOLVED_UV_VERSION_TOOLTIP.to_string()),
                padding_left: Some(true),
                padding_right: Some(false),
            },
            InlayHint {
                position: tombi_text::Position::new(6, 20),
                label: r#" → "0.7.4""#.to_string(),
                kind: Some(tower_lsp::lsp_types::InlayHintKind::TYPE),
                tooltip: Some(RESOLVED_UV_VERSION_TOOLTIP.to_string()),
                padding_left: Some(true),
                padding_right: Some(false),
            }
        ])
    );

    Ok(())
}

#[tokio::test]
async fn pyproject_inlay_hint_finds_uv_lock_from_ancestor_directory()
-> Result<(), Box<dyn std::error::Error>> {
    let _guard = test_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    tombi_test_lib::init_log();

    let temp_dir = tempfile::tempdir()?;
    let member_dir = temp_dir.path().join("members/app");
    fs::create_dir_all(&member_dir)?;

    let source = r#"
        [project]
        name = "app"
        version = "0.1.0"
        dependencies = ["pytest>=8.0"]
    "#;
    fs::write(
        member_dir.join("pyproject.toml"),
        textwrap::dedent(source).trim(),
    )?;
    fs::write(
        temp_dir.path().join("uv.lock"),
        textwrap::dedent(
            r#"
            version = 1

            [[package]]
            name = "app"
            version = "0.1.0"
            dependencies = [{ name = "pytest" }]

            [[package]]
            name = "pytest"
            version = "8.3.3"
            "#,
        )
        .trim(),
    )?;

    let result = collect_inlay_hints(source, member_dir.join("pyproject.toml")).await?;

    pretty_assertions::assert_eq!(
        result,
        Some(vec![InlayHint {
            position: tombi_text::Position::new(3, 29),
            label: r#" → "8.3.3""#.to_string(),
            kind: Some(tower_lsp::lsp_types::InlayHintKind::TYPE),
            tooltip: Some(RESOLVED_UV_VERSION_TOOLTIP.to_string()),
            padding_left: Some(true),
            padding_right: Some(false),
        }])
    );

    Ok(())
}
