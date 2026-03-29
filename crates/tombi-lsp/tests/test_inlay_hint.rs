use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
    sync::{Mutex, MutexGuard, OnceLock},
    time::Duration,
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
const WORKSPACE_PACKAGE_INHERITED_VALUE_TOOLTIP: &str = "Inherited value from workspace";

fn test_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

struct TestCacheHome {
    _guard: MutexGuard<'static, ()>,
    previous: Option<OsString>,
    _temp_dir: TempDir,
}

impl TestCacheHome {
    fn new() -> Self {
        let guard = test_lock()
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let temp_dir = tempfile::tempdir().unwrap();
        let previous = std::env::var_os("XDG_CACHE_HOME");
        unsafe {
            std::env::set_var("XDG_CACHE_HOME", temp_dir.path());
        }
        Self {
            _guard: guard,
            previous,
            _temp_dir: temp_dir,
        }
    }
}

impl Drop for TestCacheHome {
    fn drop(&mut self) {
        unsafe {
            if let Some(previous) = &self.previous {
                std::env::set_var("XDG_CACHE_HOME", previous);
            } else {
                std::env::remove_var("XDG_CACHE_HOME");
            }
        }
    }
}

async fn cached_remote_json_file_path(url: &str) -> PathBuf {
    let uri = tombi_uri::Uri::from_str(url).unwrap();
    tombi_cache::get_cache_file_path(&uri).await.unwrap()
}

async fn write_cached_response(url: &str, body: &str) {
    let cache_path = cached_remote_json_file_path(url).await;
    if let Some(parent) = cache_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&cache_path, body).unwrap();
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

    fn cargo(source: &str, cargo_lock: &str) -> Result<Self, Box<dyn std::error::Error>> {
        create_temp_fixture("Cargo.toml", source, vec![("Cargo.lock", cargo_lock)])
    }

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

fn new_service() -> LspService<Backend> {
    let (service, _) = LspService::new(|client| {
        Backend::new(
            client,
            &tombi_lsp::backend::Options {
                offline: Some(true),
                no_cache: Some(false),
            },
        )
    });

    service
}

fn inlay_hint_range(source: &str) -> Range {
    let lines = source.lines().collect::<Vec<_>>();
    let last_line = lines.len().saturating_sub(1) as u32;
    let last_column = lines.last().map_or(0, |line| line.len() as u32);

    Range::new(Position::new(0, 0), Position::new(last_line, last_column))
}

async fn collect_inlay_hints_with_backend(
    backend: &Backend,
    source: &str,
    source_path: PathBuf,
) -> Result<Option<Vec<InlayHint>>, Box<dyn std::error::Error>> {
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

    Ok(handle_inlay_hint(
        backend,
        InlayHintParams {
            text_document: TextDocumentIdentifier { uri: toml_file_url },
            range: inlay_hint_range(&toml_text),
            work_done_progress_params: WorkDoneProgressParams::default(),
        },
    )
    .await?)
}

async fn collect_inlay_hints(
    source: &str,
    source_path: PathBuf,
) -> Result<Option<Vec<InlayHint>>, Box<dyn std::error::Error>> {
    let service = new_service();
    let backend = service.inner();

    collect_inlay_hints_with_backend(backend, source, source_path).await
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

fn expected_default_features_tooltip(features: &[&str]) -> String {
    format!(
        "Default Features:\n{}",
        features
            .iter()
            .map(|feature| format!("- {feature:?}"))
            .collect::<Vec<_>>()
            .join("\n")
    )
}

macro_rules! source_files {
    ($(SourceFile { path = $path:expr, content = $content:expr $(,)? }),* $(,)?) => {
        vec![$(($path, $content)),*]
    };
}

macro_rules! source_fixture {
    (
        SourceFile { path = $source_path:expr, content = $source:expr $(,)? }
        $(,
            SourceFile { path = $path:expr, content = $content:expr $(,)? }
        )*
        $(,)?
    ) => {
        create_temp_fixture(
            $source_path,
            $source,
            source_files!(
                $(SourceFile {
                    path = $path,
                    content = $content,
                },)*
            ),
        )
    };
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
        async fn $name:ident(
            SourceFile { path = $source_path:expr, content = $source:expr $(,)? }
            $(,
                SourceFile { path = $path:expr, content = $content:expr $(,)? }
            )*
            $(,)?
        ) -> Ok($expected:expr);
    ) => {
        test_inlay_hint!(
            @run
            $name,
            source_fixture!(
                SourceFile {
                    path = $source_path,
                    content = $source,
                }
                $(,
                    SourceFile {
                        path = $path,
                        content = $content,
                    }
                )*
            ),
            $expected
        );
    };
}

fn bump_modified(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let file = fs::File::options().write(true).open(path)?;
    let modified = file.metadata()?.modified()? + Duration::from_secs(1);
    file.set_modified(modified)?;

    Ok(())
}

test_inlay_hint!(
    #[tokio::test]
    async fn inlay_hint_for_string_dependency_uses_the_resolved_lockfile_version(
        SourceFile {
            path = "Cargo.toml",
            content = r#"
            [package]
            name = "demo"
            version = "0.1.0"

            [dependencies]
            serde = "1.0.219"
            "#,
        },
        SourceFile {
            path = "Cargo.lock",
            content = r#"
            version = 4

            [[package]]
            name = "demo"
            version = "0.1.0"
            dependencies = ["serde 1.0.228"]

            [[package]]
            name = "serde"
            version = "1.0.228"
            "#,
        },
    ) -> Ok(Some(vec![expected_hint(
        tombi_text::Position::new(5, 17),
        r#" → "1.0.228""#,
        RESOLVED_VERSION_TOOLTIP,
    )]));
);

#[tokio::test]
async fn inlay_hint_for_registry_default_features_is_rendered()
-> Result<(), Box<dyn std::error::Error>> {
    let _cache_home = TestCacheHome::new();
    tombi_test_lib::init_log();

    write_cached_response(
        "https://crates.io/api/v1/crates/serde_json/1.0.140",
        r#"{"version":{"features":{"default":["std","alloc"],"preserve_order":["indexmap","std"],"std":["memchr/std","serde/std"]}}}"#,
    )
    .await;

    let fixture = source_fixture!(
        SourceFile {
            path = "Cargo.toml",
            content = r#"
            [workspace]
            members = ["crates/app"]

            [workspace.dependencies]
            serde_json = { version = "1.0.140", features = ["preserve_order", "std"] }
            "#,
        },
        SourceFile {
            path = "crates/app/Cargo.toml",
            content = r#"
            [package]
            name = "app"
            version = "0.1.0"
            "#,
        },
        SourceFile {
            path = "Cargo.lock",
            content = r#"
            version = 4

            [[package]]
            name = "app"
            version = "0.1.0"
            dependencies = ["serde_json 1.0.142"]

            [[package]]
            name = "serde_json"
            version = "1.0.142"
            "#,
        },
    )?;

    let result = collect_inlay_hints(&fixture.source, fixture.source_path).await?;

    pretty_assertions::assert_eq!(
        result,
        Some(vec![
            expected_hint(
                tombi_text::Position::new(4, 34),
                r#" → "1.0.142""#,
                RESOLVED_VERSION_TOOLTIP,
            ),
            expected_hint(
                tombi_text::Position::new(4, 72),
                r#" + ["alloc"]"#,
                &expected_default_features_tooltip(&["alloc", "std"]),
            ),
        ])
    );

    Ok(())
}

#[tokio::test]
async fn inlay_hint_for_registry_default_features_is_not_rendered_when_disabled_explicitly()
-> Result<(), Box<dyn std::error::Error>> {
    let _cache_home = TestCacheHome::new();
    tombi_test_lib::init_log();

    write_cached_response(
        "https://crates.io/api/v1/crates/serde_json/1.0.140",
        r#"{"version":{"features":{"default":["std"],"preserve_order":["indexmap","std"],"std":["memchr/std","serde/std"]}}}"#,
    )
    .await;

    let fixture = source_fixture!(
        SourceFile {
            path = "Cargo.toml",
            content = r#"
            [workspace]
            members = ["crates/app"]

            [workspace.dependencies]
            serde_json = { version = "1.0.140", default-features = false, features = ["preserve_order"] }
            "#,
        },
        SourceFile {
            path = "crates/app/Cargo.toml",
            content = r#"
            [package]
            name = "app"
            version = "0.1.0"
            "#,
        },
        SourceFile {
            path = "Cargo.lock",
            content = r#"
            version = 4

            [[package]]
            name = "app"
            version = "0.1.0"
            dependencies = ["serde_json 1.0.142"]

            [[package]]
            name = "serde_json"
            version = "1.0.142"
            "#,
        },
    )?;

    let result = collect_inlay_hints(&fixture.source, fixture.source_path).await?;

    pretty_assertions::assert_eq!(
        result,
        Some(vec![expected_hint(
            tombi_text::Position::new(4, 34),
            r#" → "1.0.142""#,
            RESOLVED_VERSION_TOOLTIP,
        )])
    );

    Ok(())
}

#[tokio::test]
async fn cargo_inlay_hint_reloads_when_cargo_lock_changes() -> Result<(), Box<dyn std::error::Error>>
{
    let _guard = test_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    tombi_test_lib::init_log();

    let fixture = InlayHintFixture::cargo(
        r#"
        [package]
        name = "demo"
        version = "0.1.0"

        [dependencies]
        serde = "1.0.219"
        "#,
        r#"
        version = 4

        [[package]]
        name = "demo"
        version = "0.1.0"
        dependencies = ["serde 1.0.228"]

        [[package]]
        name = "serde"
        version = "1.0.228"
        "#,
    )?;
    let cargo_lock_path = fixture
        .source_path
        .parent()
        .expect("expected parent")
        .join("Cargo.lock");

    let service = new_service();
    let backend = service.inner();
    let first =
        collect_inlay_hints_with_backend(backend, &fixture.source, fixture.source_path.clone())
            .await?;
    pretty_assertions::assert_eq!(
        first,
        Some(vec![expected_hint(
            tombi_text::Position::new(5, 17),
            r#" → "1.0.228""#,
            RESOLVED_VERSION_TOOLTIP,
        )])
    );

    fs::write(
        &cargo_lock_path,
        textwrap::dedent(
            r#"
            version = 4

            [[package]]
            name = "demo"
            version = "0.1.0"
            dependencies = ["serde 1.0.229"]

            [[package]]
            name = "serde"
            version = "1.0.229"
            "#,
        )
        .trim(),
    )?;
    bump_modified(&cargo_lock_path)?;

    let second =
        collect_inlay_hints_with_backend(backend, &fixture.source, fixture.source_path.clone())
            .await?;
    pretty_assertions::assert_eq!(
        second,
        Some(vec![expected_hint(
            tombi_text::Position::new(5, 17),
            r#" → "1.0.229""#,
            RESOLVED_VERSION_TOOLTIP,
        )])
    );

    Ok(())
}

#[tokio::test]
async fn cargo_inlay_hint_uses_cached_value_when_reloading_invalid_lockfile()
-> Result<(), Box<dyn std::error::Error>> {
    let _guard = test_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    tombi_test_lib::init_log();

    let fixture = InlayHintFixture::cargo(
        r#"
        [package]
        name = "demo"
        version = "0.1.0"

        [dependencies]
        serde = "1.0.219"
        "#,
        r#"
        version = 4

        [[package]]
        name = "demo"
        version = "0.1.0"
        dependencies = ["serde 1.0.228"]

        [[package]]
        name = "serde"
        version = "1.0.228"
        "#,
    )?;
    let cargo_lock_path = fixture
        .source_path
        .parent()
        .expect("expected parent")
        .join("Cargo.lock");

    let service = new_service();
    let backend = service.inner();
    let first =
        collect_inlay_hints_with_backend(backend, &fixture.source, fixture.source_path.clone())
            .await?;

    fs::write(&cargo_lock_path, "not-a-lockfile = [")?;
    bump_modified(&cargo_lock_path)?;

    let second =
        collect_inlay_hints_with_backend(backend, &fixture.source, fixture.source_path.clone())
            .await?;

    pretty_assertions::assert_eq!(second, first);

    Ok(())
}

#[tokio::test]
async fn pyproject_inlay_hint_reloads_when_uv_lock_changes()
-> Result<(), Box<dyn std::error::Error>> {
    let _guard = test_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    tombi_test_lib::init_log();

    let fixture = InlayHintFixture::pyproject(
        r#"
        [project]
        name = "demo"
        version = "0.1.0"
        dependencies = ["pytest>=8.0"]
        "#,
        r#"
        version = 1

        [[package]]
        name = "demo"
        version = "0.1.0"
        dependencies = [{ name = "pytest" }]

        [[package]]
        name = "pytest"
        version = "8.3.3"
        "#,
    )?;
    let uv_lock_path = fixture
        .source_path
        .parent()
        .expect("expected parent")
        .join("uv.lock");

    let service = new_service();
    let backend = service.inner();
    let first =
        collect_inlay_hints_with_backend(backend, &fixture.source, fixture.source_path.clone())
            .await?;
    pretty_assertions::assert_eq!(
        first,
        Some(vec![expected_hint(
            tombi_text::Position::new(3, 29),
            r#" → "8.3.3""#,
            RESOLVED_UV_VERSION_TOOLTIP,
        )])
    );

    fs::write(
        &uv_lock_path,
        textwrap::dedent(
            r#"
            version = 1

            [[package]]
            name = "demo"
            version = "0.1.0"
            dependencies = [{ name = "pytest" }]

            [[package]]
            name = "pytest"
            version = "8.3.4"
            "#,
        )
        .trim(),
    )?;
    bump_modified(&uv_lock_path)?;

    let second =
        collect_inlay_hints_with_backend(backend, &fixture.source, fixture.source_path.clone())
            .await?;
    pretty_assertions::assert_eq!(
        second,
        Some(vec![expected_hint(
            tombi_text::Position::new(3, 29),
            r#" → "8.3.4""#,
            RESOLVED_UV_VERSION_TOOLTIP,
        )])
    );

    Ok(())
}

test_inlay_hint!(
    #[tokio::test]
    async fn inlay_hint_for_dotted_version_key_is_rendered_after_the_version_literal(
        SourceFile {
            path = "Cargo.toml",
            content = r#"
            [package]
            name = "demo"
            version = "0.1.0"

            [dependencies]
            addr.version = "0.15.5"
            "#,
        },
        SourceFile {
            path = "Cargo.lock",
            content = r#"
            version = 4

            [[package]]
            name = "demo"
            version = "0.1.0"
            dependencies = ["addr 0.15.6"]

            [[package]]
            name = "addr"
            version = "0.15.6"
            "#,
        },
    ) -> Ok(Some(vec![expected_hint(
        tombi_text::Position::new(5, 23),
        r#" → "0.15.6""#,
        RESOLVED_VERSION_TOOLTIP,
    )]));
);

test_inlay_hint!(
    #[tokio::test]
    async fn inlay_hint_for_inline_table_version_uses_the_resolved_lockfile_version(
        SourceFile {
            path = "Cargo.toml",
            content = r#"
            [package]
            name = "demo"
            version = "0.1.0"

            [dependencies]
            tokio = { version = "1.45.0", features = ["fs"] }
            "#,
        },
        SourceFile {
            path = "Cargo.lock",
            content = r#"
            version = 4

            [[package]]
            name = "demo"
            version = "0.1.0"
            dependencies = ["tokio 1.47.1"]

            [[package]]
            name = "tokio"
            version = "1.47.1"
            "#,
        },
    ) -> Ok(Some(vec![expected_hint(
        tombi_text::Position::new(5, 28),
        r#" → "1.47.1""#,
        RESOLVED_VERSION_TOOLTIP,
    )]));
);

test_inlay_hint!(
    #[tokio::test]
    async fn inlay_hint_for_workspace_dependencies_uses_workspace_members_lockfile_resolution(
        SourceFile {
            path = "Cargo.toml",
            content = r#"
            [workspace]
            members = ["crates/app"]

            [workspace.dependencies]
            serde = "1.0.219"
            "#,
        },
        SourceFile {
            path = "crates/app/Cargo.toml",
            content = r#"
            [package]
            name = "app"
            version = "0.1.0"
            "#,
        },
        SourceFile {
            path = "Cargo.lock",
            content = r#"
            version = 4

            [[package]]
            name = "app"
            version = "0.1.0"
            dependencies = ["serde 1.0.228"]

            [[package]]
            name = "serde"
            version = "1.0.228"
            "#,
        },
    ) -> Ok(Some(vec![expected_hint(
        tombi_text::Position::new(4, 17),
        r#" → "1.0.228""#,
        RESOLVED_VERSION_TOOLTIP,
    )]));
);

test_inlay_hint!(
    #[tokio::test]
    async fn inlay_hint_for_workspace_inline_table_dependency_uses_the_resolved_lockfile_version(
        SourceFile {
            path = "Cargo.toml",
            content = r#"
            [workspace]
            members = ["crates/app"]

            [workspace.dependencies]
            serde_json = { version = "1.0.140", default-features = false, features = ["preserve_order"] }
            "#,
        },
        SourceFile {
            path = "crates/app/Cargo.toml",
            content = r#"
            [package]
            name = "app"
            version = "0.1.0"
            "#,
        },
        SourceFile {
            path = "Cargo.lock",
            content = r#"
            version = 4

            [[package]]
            name = "app"
            version = "0.1.0"
            dependencies = ["serde_json 1.0.142"]

            [[package]]
            name = "serde_json"
            version = "1.0.142"
            "#,
        },
    ) -> Ok(Some(vec![
        expected_hint(
            tombi_text::Position::new(4, 34),
            r#" → "1.0.142""#,
            RESOLVED_VERSION_TOOLTIP,
        ),
    ]));
);

test_inlay_hint!(
    #[tokio::test]
    async fn inlay_hint_for_workspace_inheritance_is_rendered_even_when_versions_match(
        SourceFile {
            path = "crates/app/Cargo.toml",
            content = r#"
            [package]
            name = "app"
            version = "0.1.0"

            [dependencies]
            addr.workspace = true
            "#,
        },
        SourceFile {
            path = "Cargo.toml",
            content = r#"
            [workspace]
            members = ["crates/app"]
            "#,
        },
        SourceFile {
            path = "Cargo.lock",
            content = r#"
            version = 4

            [[package]]
            name = "app"
            version = "0.1.0"
            dependencies = ["addr 0.15.6"]

            [[package]]
            name = "addr"
            version = "0.15.6"
            "#,
        },
    ) -> Ok(Some(vec![expected_hint(
        tombi_text::Position::new(5, 21),
        r#" → "0.15.6""#,
        RESOLVED_VERSION_TOOLTIP,
    )]));
);

test_inlay_hint!(
    #[tokio::test]
    async fn inlay_hint_for_workspace_default_features_is_not_rendered_without_member_override(
        SourceFile {
            path = "crates/app/Cargo.toml",
            content = r#"
            [package]
            name = "app"
            version = "0.1.0"

            [dependencies]
            serde_json = { workspace = true, features = ["preserve_order"] }
            "#,
        },
        SourceFile {
            path = "Cargo.toml",
            content = r#"
            [workspace]
            members = ["crates/app"]

            [workspace.dependencies]
            serde_json = { version = "1.0.140", default-features = false }
            "#,
        },
        SourceFile {
            path = "Cargo.lock",
            content = r#"
            version = 4

            [[package]]
            name = "app"
            version = "0.1.0"
            dependencies = ["serde_json 1.0.142"]

            [[package]]
            name = "serde_json"
            version = "1.0.142"
            "#,
        },
    ) -> Ok(Some(vec![expected_hint(
        tombi_text::Position::new(5, 31),
        r#" → "1.0.142""#,
        RESOLVED_VERSION_TOOLTIP,
    )]));
);

test_inlay_hint!(
    #[tokio::test]
    async fn inlay_hint_is_not_rendered_when_the_version_already_matches_the_lockfile(
        SourceFile {
            path = "Cargo.toml",
            content = r#"
            [package]
            name = "demo"
            version = "0.1.0"

            [dependencies]
            serde = "1.0.228"
            "#,
        },
        SourceFile {
            path = "Cargo.lock",
            content = r#"
            version = 4

            [[package]]
            name = "demo"
            version = "0.1.0"
            dependencies = ["serde 1.0.228"]

            [[package]]
            name = "serde"
            version = "1.0.228"
            "#,
        },
    ) -> Ok(None);
);

test_inlay_hint!(
    #[tokio::test]
    async fn inlay_hint_for_workspace_path_crate_is_rendered_from_the_lockfile(
        SourceFile {
            path = "crates/app/Cargo.toml",
            content = r#"
            [package]
            name = "app"
            version = "0.1.0"

            [dependencies]
            tombi-text.workspace = true
            "#,
        },
        SourceFile {
            path = "Cargo.toml",
            content = r#"
            [workspace]
            members = ["crates/app"]
            "#,
        },
        SourceFile {
            path = "Cargo.lock",
            content = r#"
            version = 4

            [[package]]
            name = "app"
            version = "0.1.0"
            dependencies = ["tombi-text 0.0.0-dev"]

            [[package]]
            name = "tombi-text"
            version = "0.0.0-dev"
            "#,
        },
    ) -> Ok(Some(vec![expected_hint(
        tombi_text::Position::new(5, 27),
        r#" → "0.0.0-dev""#,
        RESOLVED_VERSION_TOOLTIP,
    )]));
);

test_inlay_hint!(
    #[tokio::test]
    async fn inlay_hint_for_member_package_workspace_inheritance_uses_workspace_package_value(
        SourceFile {
            path = "crates/app/Cargo.toml",
            content = r#"
            [package]
            name = "app"
            version = { workspace = true }
            authors.workspace = true
            "#,
        },
        SourceFile {
            path = "Cargo.toml",
            content = r#"
            [workspace]
            members = ["crates/app"]

            [workspace.package]
            version = "0.0.0-dev"
            authors = ["Tombi", "Cargo"]
            "#,
        },
        SourceFile {
            path = "Cargo.lock",
            content = "",
        },
    ) -> Ok(Some(vec![
        expected_hint(
            tombi_text::Position::new(2, 28),
            r#" → "0.0.0-dev""#,
            WORKSPACE_PACKAGE_INHERITED_VALUE_TOOLTIP,
        ),
        expected_hint(
            tombi_text::Position::new(3, 24),
            r#" → ["Tombi", "Cargo"]"#,
            WORKSPACE_PACKAGE_INHERITED_VALUE_TOOLTIP,
        ),
    ]));
);

test_inlay_hint!(
    #[tokio::test]
    async fn inlay_hint_for_root_package_workspace_inheritance_uses_same_manifest_workspace_package_value(
        SourceFile {
            path = "Cargo.toml",
            content = r#"
            [package]
            name = "app"
            version = { workspace = true }

            [workspace]
            members = ["."]

            [workspace.package]
            version = "0.0.0-dev"
            "#,
        },
        SourceFile {
            path = "Cargo.lock",
            content = "",
        },
    ) -> Ok(Some(vec![expected_hint(
        tombi_text::Position::new(2, 28),
        r#" → "0.0.0-dev""#,
        WORKSPACE_PACKAGE_INHERITED_VALUE_TOOLTIP,
    )]));
);

test_inlay_hint!(
    #[tokio::test]
    async fn inlay_hint_for_path_dependency_is_rendered_after_the_path_value(
        SourceFile {
            path = "Cargo.toml",
            content = r#"
            [package]
            name = "demo"
            version = "0.1.0"

            [dependencies]
            serde = { path = "vendor/serde" }
            "#,
        },
        SourceFile {
            path = "Cargo.lock",
            content = r#"
            version = 4

            [[package]]
            name = "demo"
            version = "0.1.0"
            dependencies = ["serde 1.0.228"]

            [[package]]
            name = "serde"
            version = "1.0.228"
            "#,
        },
    ) -> Ok(Some(vec![expected_hint(
        tombi_text::Position::new(5, 31),
        r#" → "1.0.228""#,
        RESOLVED_VERSION_TOOLTIP,
    )]));
);

test_inlay_hint!(
    #[tokio::test]
    async fn inlay_hint_for_default_features_is_rendered_after_the_features_array(
        SourceFile {
            path = "Cargo.toml",
            content = r#"
            [package]
            name = "demo"
            version = "0.1.0"

            [dependencies]
            dep = { path = "vendor/dep", features = ["default1", "extra"] }
            "#,
        },
        SourceFile {
            path = "vendor/dep/Cargo.toml",
            content = r#"
            [package]
            name = "dep"
            version = "0.2.0"

            [features]
            default = ["default1", "default2"]
            extra = []
            "#,
        },
        SourceFile {
            path = "Cargo.lock",
            content = r#"
            version = 4

            [[package]]
            name = "demo"
            version = "0.1.0"
            dependencies = ["dep 0.2.0"]

            [[package]]
            name = "dep"
            version = "0.2.0"
            "#,
        },
    ) -> Ok(Some(vec![
        expected_hint(
            tombi_text::Position::new(5, 27),
            r#" → "0.2.0""#,
            RESOLVED_VERSION_TOOLTIP,
        ),
        expected_hint(
            tombi_text::Position::new(5, 61),
            r#" + ["default2"]"#,
            &expected_default_features_tooltip(&["default1", "default2"]),
        ),
    ]));
);

test_inlay_hint!(
    #[tokio::test]
    async fn inlay_hint_for_default_features_is_not_rendered_when_disabled_explicitly(
        SourceFile {
            path = "Cargo.toml",
            content = r#"
            [package]
            name = "demo"
            version = "0.1.0"

            [dependencies]
            dep = { path = "vendor/dep", default-features = false, features = ["extra"] }
            "#,
        },
        SourceFile {
            path = "vendor/dep/Cargo.toml",
            content = r#"
            [package]
            name = "dep"
            version = "0.2.0"

            [features]
            default = ["default1", "default2"]
            extra = []
            "#,
        },
        SourceFile {
            path = "Cargo.lock",
            content = r#"
            version = 4

            [[package]]
            name = "demo"
            version = "0.1.0"
            dependencies = ["dep 0.2.0"]

            [[package]]
            name = "dep"
            version = "0.2.0"
            "#,
        },
    ) -> Ok(Some(vec![expected_hint(
        tombi_text::Position::new(5, 27),
        r#" → "0.2.0""#,
        RESOLVED_VERSION_TOOLTIP,
    )]));
);

test_inlay_hint!(
    #[tokio::test]
    async fn inlay_hint_for_git_dependency_is_rendered_after_the_git_value(
        SourceFile {
            path = "Cargo.toml",
            content = r#"
            [package]
            name = "demo"
            version = "0.1.0"

            [dependencies]
            serde = { git = "https://github.com/serde-rs/serde" }
            "#,
        },
        SourceFile {
            path = "Cargo.lock",
            content = r#"
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
        },
    ) -> Ok(Some(vec![expected_hint(
        tombi_text::Position::new(5, 51),
        r#" → "1.0.228""#,
        RESOLVED_VERSION_TOOLTIP,
    )]));
);

test_inlay_hint!(
    #[tokio::test]
    async fn pyproject_inlay_hint_uses_uv_lock_for_project_dependencies(
        SourceFile {
            path = "pyproject.toml",
            content = r#"
            [project]
            name = "demo"
            version = "0.1.0"
            dependencies = ["pytest>=8.0"]

            [dependency-groups]
            dev = ["ruff>=0.7.0"]
            "#,
        },
        SourceFile {
            path = "uv.lock",
            content = r#"
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
        },
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
    async fn pyproject_inlay_hint_supports_tool_uv_dependency_lists(
        SourceFile {
            path = "pyproject.toml",
            content = r#"
            [project]
            name = "demo"
            version = "0.1.0"

            [tool.uv]
            dev-dependencies = ["ruff>=0.7.0"]
            constraint-dependencies = ["pytest<9"]
            override-dependencies = ["werkzeug==2.2.3"]
            build-constraint-dependencies = ["setuptools==59.0.0"]
            "#,
        },
        SourceFile {
            path = "uv.lock",
            content = r#"
            version = 1

            [[package]]
            name = "demo"
            version = "0.1.0"

            [package.dev-dependencies]
            dev = [{ name = "ruff" }]

            [[package]]
            name = "ruff"
            version = "0.7.4"

            [[package]]
            name = "pytest"
            version = "8.3.3"

            [[package]]
            name = "werkzeug"
            version = "2.3.0"

            [[package]]
            name = "setuptools"
            version = "60.0.0"
            "#,
        },
    ) -> Ok(Some(vec![
        expected_hint(
            tombi_text::Position::new(5, 33),
            r#" → "0.7.4""#,
            RESOLVED_UV_VERSION_TOOLTIP,
        ),
        expected_hint(
            tombi_text::Position::new(6, 37),
            r#" → "8.3.3""#,
            RESOLVED_UV_VERSION_TOOLTIP,
        ),
        expected_hint(
            tombi_text::Position::new(7, 42),
            r#" → "2.3.0""#,
            RESOLVED_UV_VERSION_TOOLTIP,
        ),
        expected_hint(
            tombi_text::Position::new(8, 53),
            r#" → "60.0.0""#,
            RESOLVED_UV_VERSION_TOOLTIP,
        ),
    ]));
);

test_inlay_hint!(
    #[tokio::test]
    async fn pyproject_inlay_hint_uv_dependency_lists_without_project_table(
        SourceFile {
            path = "pyproject.toml",
            content = r#"
            [tool.uv]
            constraint-dependencies = ["pytest<9"]
            override-dependencies = ["werkzeug==2.2.3"]
            "#,
        },
        SourceFile {
            path = "uv.lock",
            content = r#"
            version = 1

            [[package]]
            name = "pytest"
            version = "8.3.3"

            [[package]]
            name = "werkzeug"
            version = "2.3.0"
            "#,
        },
    ) -> Ok(Some(vec![
        expected_hint(
            tombi_text::Position::new(2, 37),
            r#" → "8.3.3""#,
            RESOLVED_UV_VERSION_TOOLTIP,
        ),
        expected_hint(
            tombi_text::Position::new(3, 42),
            r#" → "2.3.0""#,
            RESOLVED_UV_VERSION_TOOLTIP,
        ),
    ]));
);

test_inlay_hint!(
    #[tokio::test]
    async fn pyproject_inlay_hint_finds_uv_lock_from_ancestor_directory(
        SourceFile {
            path = "members/app/pyproject.toml",
            content = r#"
            [project]
            name = "app"
            version = "0.1.0"
            dependencies = ["pytest>=8.0"]
            "#,
        },
        SourceFile {
            path = "uv.lock",
            content = r#"
            version = 1

            [[package]]
            name = "app"
            version = "0.1.0"
            dependencies = [{ name = "pytest" }]

            [[package]]
            name = "pytest"
            version = "8.3.3"
            "#,
        },
    ) -> Ok(Some(vec![expected_hint(
        tombi_text::Position::new(3, 29),
        r#" → "8.3.3""#,
        RESOLVED_UV_VERSION_TOOLTIP,
    )]));
);
