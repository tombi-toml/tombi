use std::{
    ffi::OsString,
    fs,
    path::PathBuf,
    str::FromStr,
    sync::{Mutex, MutexGuard, OnceLock},
};

use tempfile::TempDir;

fn test_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

struct TestCacheHome {
    _guard: MutexGuard<'static, ()>,
    previous_tombi: Option<OsString>,
    previous_xdg: Option<OsString>,
    _temp_dir: TempDir,
}

impl TestCacheHome {
    fn new() -> Self {
        let guard = test_lock()
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let temp_dir = tempfile::tempdir().unwrap();
        let previous_tombi = std::env::var_os("TOMBI_CACHE_HOME");
        let previous_xdg = std::env::var_os("XDG_CACHE_HOME");
        unsafe {
            std::env::remove_var("TOMBI_CACHE_HOME");
            std::env::set_var("XDG_CACHE_HOME", temp_dir.path());
        }
        Self {
            _guard: guard,
            previous_tombi,
            previous_xdg,
            _temp_dir: temp_dir,
        }
    }
}

impl Drop for TestCacheHome {
    fn drop(&mut self) {
        unsafe {
            if let Some(previous) = &self.previous_tombi {
                std::env::set_var("TOMBI_CACHE_HOME", previous);
            } else {
                std::env::remove_var("TOMBI_CACHE_HOME");
            }

            if let Some(previous) = &self.previous_xdg {
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

#[derive(Clone)]
pub struct CachedResponseSpec {
    url: &'static str,
    body: &'static str,
}

impl CachedResponseSpec {
    pub const fn new(url: &'static str, body: &'static str) -> Self {
        Self { url, body }
    }
}

pub struct UseCacheResponses(pub Vec<CachedResponseSpec>);

mod refactor_rewrite {
    mod common {
        use tombi_lsp::code_action::CodeActionRefactorRewriteName;

        use crate::test_code_action_refactor_rewrite;

        test_code_action_refactor_rewrite! {
            #[tokio::test]
            async fn dotted_keys_to_inline_table(
                r#"
            foo.bar█ = 1
            "#,
                Select(CodeActionRefactorRewriteName::DottedKeysToInlineTable),
            ) -> Ok(Some(
                r#"
            foo = { bar = 1 }
            "#
            ));
        }

        test_code_action_refactor_rewrite! {
            #[tokio::test]
            async fn dotted_keys_to_inline_table_with_comment(
                r#"
            foo.bar█ = 1 # comment
            "#,
                Select(CodeActionRefactorRewriteName::DottedKeysToInlineTable),
            ) -> Ok(Some(
                r#"
            foo = { bar = 1 } # comment
            "#
            ));
        }

        test_code_action_refactor_rewrite! {
            #[tokio::test]
            async fn nested_dotted_keys_to_inline_table_with_comment(
                r#"
                foo.bar█.baz = 1 # comment
                "#,
                Select(CodeActionRefactorRewriteName::DottedKeysToInlineTable),
            ) -> Ok(Some(
                r#"
                foo = { bar.baz = 1 } # comment
                "#
            ));
        }

        test_code_action_refactor_rewrite! {
            #[tokio::test]
            async fn inline_table_to_dotted_keys(
                r#"
            foo = { bar = █1 }
            "#,
                Select(CodeActionRefactorRewriteName::InlineTableToDottedKeys),
            ) -> Ok(Some(
                r#"
            foo.bar = 1
            "#
            ));
        }

        test_code_action_refactor_rewrite! {
            #[tokio::test]
            async fn inline_table_to_dotted_keys_with_comment(
                r#"
            foo = { bar = █1 } # comment
            "#,
                Select(CodeActionRefactorRewriteName::InlineTableToDottedKeys),
            ) -> Ok(Some(
                r#"
            foo.bar = 1 # comment
            "#
            ));
        }

        test_code_action_refactor_rewrite! {
            #[tokio::test]
            async fn nested_inline_table_to_dotted_keys_with_comment(
                r#"
            foo = { bar█.baz = 1 } # comment
            "#,
                Select(CodeActionRefactorRewriteName::InlineTableToDottedKeys),
            ) -> Ok(Some(
                r#"
            foo.bar.baz = 1 # comment
            "#
            ));
        }

        test_code_action_refactor_rewrite! {
            #[tokio::test]
            async fn inline_table_array_to_dotted_keys_with_comment(
                r#"
            foo = { bar = █[1, 2, 3] } # comment
            "#,
                Select(CodeActionRefactorRewriteName::InlineTableToDottedKeys),
            ) -> Ok(Some(
                r#"
            foo.bar = [1, 2, 3] # comment
            "#
            ));
        }

        test_code_action_refactor_rewrite! {
            #[tokio::test]
            async fn inline_table_multiline_array_to_dotted_keys_with_comment(
                r#"
            foo = { bar = █[
              1,
              2,
              3,
            ] } # comment
            "#,
                Select(CodeActionRefactorRewriteName::InlineTableToDottedKeys),
            ) -> Ok(Some(
                r#"
            foo.bar = [
              1,
              2,
              3,
            ] # comment
            "#
            ));
        }

        test_code_action_refactor_rewrite! {
            #[tokio::test]
            async fn inline_table_has_other_keys(
                r#"
            foo = { bar = █1, baz = 2 }
            "#,
            ) -> Ok(None);
        }

        test_code_action_refactor_rewrite! {
            #[tokio::test]
            async fn inline_table_has_other_keys_with_comment(
                r#"
            foo = { bar = █1, baz = 2 } # comment
            "#,
            ) -> Ok(None);
        }
    }

    mod cargo_toml {
        use tombi_extension_cargo::CodeActionRefactorRewriteName;
        use tombi_test_lib::project_root_path;

        use crate::{CachedResponseSpec, UseCacheResponses, test_code_action_refactor_rewrite};

        test_code_action_refactor_rewrite! {
            #[tokio::test]
            async fn cargo_toml_package_version(
                r#"
                [package]
                version█ = "1.0"
                "#,
                Select(CodeActionRefactorRewriteName::InheritFromWorkspace),
                project_root_path().join("crates/subcrate/Cargo.toml"),
            ) -> Ok(Some(
                r#"
                [package]
                version.workspace = true
                "#
            ));
        }

        test_code_action_refactor_rewrite! {
            #[tokio::test]
            async fn cargo_toml_package_version_workspace(
                r#"
                [package]
                version.workspace█ = true
                "#,
                Select(CodeActionRefactorRewriteName::InheritFromWorkspace),
                project_root_path().join("crates/subcrate/Cargo.toml"),
            ) -> Ok(None);
        }

        test_code_action_refactor_rewrite! {
            #[tokio::test]
            async fn cargo_toml_dependencies_serde_dotted_version(
                r#"
                [dependencies]
                serde.version█ = "1.0"
                "#,
                Select(CodeActionRefactorRewriteName::InheritDependencyFromWorkspace),
                project_root_path().join("crates/subcrate/Cargo.toml"),
            ) -> Ok(Some(
                r#"
                [dependencies]
                serde.workspace = true
                "#
            ));
        }

        test_code_action_refactor_rewrite! {
            #[tokio::test]
            async fn cargo_toml_dependencies_serde_dotted_version_with_comment(
                r#"
                [dependencies]
                serde.version█ = "1.0" # comment
                "#,
                Select(CodeActionRefactorRewriteName::InheritDependencyFromWorkspace),
                project_root_path().join("crates/subcrate/Cargo.toml"),
            ) -> Ok(Some(
                r#"
                [dependencies]
                serde.workspace = true # comment
                "#
            ));
        }

        test_code_action_refactor_rewrite! {
            #[tokio::test]
            async fn cargo_toml_dependencies_serde_inline_table_version(
                r#"
                [dependencies]
                serde = { version█ = "1.0" }
                "#,
                Select(CodeActionRefactorRewriteName::InheritDependencyFromWorkspace),
                project_root_path().join("crates/subcrate/Cargo.toml"),
            ) -> Ok(Some(
                r#"
                [dependencies]
                serde = { workspace = true }
                "#
            ));
        }

        test_code_action_refactor_rewrite! {
            #[tokio::test]
            async fn cargo_toml_dependencies_serde_inline_table_version_with_comment(
                r#"
                [dependencies]
                serde = { version█ = "1.0" } # comment
                "#,
                Select(CodeActionRefactorRewriteName::InheritDependencyFromWorkspace),
                project_root_path().join("crates/subcrate/Cargo.toml"),
            ) -> Ok(Some(
                r#"
                [dependencies]
                serde = { workspace = true } # comment
                "#
            ));
        }

        test_code_action_refactor_rewrite! {
            #[tokio::test]
            async fn cargo_toml_dependencies_serde_inline_table_version_with_other_keys(
                r#"
                [dependencies]
                serde = { version█ = "1.0", features = ["derive"] }
                "#,
                Select(CodeActionRefactorRewriteName::InheritDependencyFromWorkspace),
                project_root_path().join("crates/subcrate/Cargo.toml"),
            ) -> Ok(Some(
                r#"
                [dependencies]
                serde = { workspace = true, features = ["derive"] }
                "#
            ));
        }

        test_code_action_refactor_rewrite! {
            #[tokio::test]
            async fn cargo_toml_dependencies_serde_inline_table_version_with_other_keys_and_comment(
                r#"
                [dependencies]
                serde = { version█ = "1.0", features = ["derive"] } # comment
                "#,
                Select(CodeActionRefactorRewriteName::InheritDependencyFromWorkspace),
                project_root_path().join("crates/subcrate/Cargo.toml"),
            ) -> Ok(Some(
                r#"
                [dependencies]
                serde = { workspace = true, features = ["derive"] } # comment
                "#
            ));
        }

        test_code_action_refactor_rewrite! {
            #[tokio::test]
            async fn cargo_toml_dependencies_serde_dotted_keys_workspace(
                r#"
                [dependencies]
                serde.workspace█ = true
                "#,
                Select(CodeActionRefactorRewriteName::InheritDependencyFromWorkspace),
                project_root_path().join("crates/subcrate/Cargo.toml"),
            ) -> Ok(None);
        }

        test_code_action_refactor_rewrite! {
            #[tokio::test]
            async fn cargo_toml_dependencies_serde_inline_table_workspace(
                r#"
                [dependencies]
                serde = { workspace█ = true }
                "#,
                Select(CodeActionRefactorRewriteName::InheritDependencyFromWorkspace),
                project_root_path().join("crates/subcrate/Cargo.toml"),
            ) -> Ok(None);
        }

        test_code_action_refactor_rewrite! {
            #[tokio::test]
            async fn cargo_toml_dependencies_serde_table_version(
                r#"
                [dependencies.serde]
                version█ = "1.0"
                "#,
                Select(CodeActionRefactorRewriteName::InheritDependencyFromWorkspace),
                project_root_path().join("crates/subcrate/Cargo.toml"),
            ) -> Ok(Some(
                r#"
                [dependencies.serde]
                workspace = true
                "#
            ));
        }

        test_code_action_refactor_rewrite! {
            #[tokio::test]
            async fn cargo_toml_dependencies_serde_table_version_with_other_keys(
                r#"
                [dependencies.serde]
                version█ = "1.0"
                default-features = false
                "#,
                Select(CodeActionRefactorRewriteName::InheritDependencyFromWorkspace),
                project_root_path().join("crates/subcrate/Cargo.toml"),
            ) -> Ok(Some(
                r#"
                [dependencies.serde]
                workspace = true
                default-features = false
                "#
            ));
        }

        test_code_action_refactor_rewrite! {
            #[tokio::test]
            async fn cargo_toml_dependencies_serde_table_version_into_table(
                r#"
                [dependencies]
                serde█ = "1.0"
                "#,
                Select(CodeActionRefactorRewriteName::ConvertDependencyToTableFormat),
                project_root_path().join("crates/subcrate/Cargo.toml"),
            ) -> Ok(Some(
                r#"
                [dependencies]
                serde = { version = "1.0" }
                "#
            ));
        }

        test_code_action_refactor_rewrite! {
            #[tokio::test]
            async fn cargo_toml_dependencies_serde_table_version_into_table_with_comment(
                r#"
                [dependencies]
                serde█ = "1.0" # comment
                "#,
                Select(CodeActionRefactorRewriteName::ConvertDependencyToTableFormat),
                project_root_path().join("crates/subcrate/Cargo.toml"),
            ) -> Ok(Some(
                r#"
                [dependencies]
                serde = { version = "1.0" } # comment
                "#
            ));
        }

        // Tests for platform specific dependencies (Issue #1192)
        test_code_action_refactor_rewrite! {
            #[tokio::test]
            async fn cargo_toml_target_dependencies_convert_to_table_format(
                r#"
                [target.'cfg(not(target_arch = "wasm32"))'.dependencies]
                egui-winit█ = "0.33.0"
                "#,
                Select(CodeActionRefactorRewriteName::ConvertDependencyToTableFormat),
                project_root_path().join("crates/subcrate/Cargo.toml"),
            ) -> Ok(Some(
                r#"
                [target.'cfg(not(target_arch = "wasm32"))'.dependencies]
                egui-winit = { version = "0.33.0" }
                "#
            ));
        }

        test_code_action_refactor_rewrite! {
            #[tokio::test]
            async fn cargo_toml_target_dependencies_inherit_from_workspace(
                r#"
                [target.'cfg(unix)'.dependencies]
                serde█ = "1.0"
                "#,
                Select(CodeActionRefactorRewriteName::InheritDependencyFromWorkspace),
                project_root_path().join("crates/subcrate/Cargo.toml"),
            ) -> Ok(Some(
                r#"
                [target.'cfg(unix)'.dependencies]
                serde.workspace = true
                "#
            ));
        }

        test_code_action_refactor_rewrite! {
            #[tokio::test]
            async fn cargo_toml_target_dependencies_inline_table_inherit_from_workspace(
                r#"
                [target.'cfg(unix)'.dependencies]
                serde = { version█ = "1.0" }
                "#,
                Select(CodeActionRefactorRewriteName::InheritDependencyFromWorkspace),
                project_root_path().join("crates/subcrate/Cargo.toml"),
            ) -> Ok(Some(
                r#"
                [target.'cfg(unix)'.dependencies]
                serde = { workspace = true }
                "#
            ));
        }

        test_code_action_refactor_rewrite! {
            #[tokio::test]
            async fn cargo_toml_target_dependencies_inline_table_with_features(
                r#"
                [target.'cfg(unix)'.dependencies]
                serde = { version█ = "1.0", features = ["derive"] }
                "#,
                Select(CodeActionRefactorRewriteName::InheritDependencyFromWorkspace),
                project_root_path().join("crates/subcrate/Cargo.toml"),
            ) -> Ok(Some(
                r#"
                [target.'cfg(unix)'.dependencies]
                serde = { workspace = true, features = ["derive"] }
                "#
            ));
        }

        test_code_action_refactor_rewrite! {
            #[tokio::test]
            async fn cargo_toml_target_dev_dependencies_convert_to_table_format(
                r#"
                [target.'cfg(target_os = "linux")'.dev-dependencies]
                tokio█ = "1.0"
                "#,
                Select(CodeActionRefactorRewriteName::ConvertDependencyToTableFormat),
                project_root_path().join("crates/subcrate/Cargo.toml"),
            ) -> Ok(Some(
                r#"
                [target.'cfg(target_os = "linux")'.dev-dependencies]
                tokio = { version = "1.0" }
                "#
            ));
        }

        test_code_action_refactor_rewrite! {
            #[tokio::test]
            async fn cargo_toml_target_build_dependencies_convert_to_table_format(
                r#"
                [target.'cfg(windows)'.build-dependencies]
                cc█ = "1.0"
                "#,
                Select(CodeActionRefactorRewriteName::ConvertDependencyToTableFormat),
                project_root_path().join("crates/subcrate/Cargo.toml"),
            ) -> Ok(Some(
                r#"
                [target.'cfg(windows)'.build-dependencies]
                cc = { version = "1.0" }
                "#
            ));
        }

        test_code_action_refactor_rewrite! {
            #[tokio::test]
            async fn cargo_toml_dependencies_update_to_latest_version(
                r#"
                [dependencies]
                serde = "1.0█"
                "#,
                project_root_path().join("crates/subcrate/Cargo.toml"),
                Select(CodeActionRefactorRewriteName::UpdateDependencyToLatestVersion),
                tombi_lsp::backend::Options {
                    offline: Some(true),
                    no_cache: Some(false),
                },
                UseCacheResponses(vec![CachedResponseSpec::new(
                    "https://crates.io/api/v1/crates/serde",
                    r#"{
                        "crate": {
                            "max_version": "1.0.228"
                        }
                    }"#,
                )]),
            ) -> Ok(Some(
                r#"
                [dependencies]
                serde = "1.0.228"
                "#
            ));
        }

        test_code_action_refactor_rewrite! {
            #[tokio::test]
            async fn cargo_toml_dependencies_inline_table_update_to_latest_version(
                r#"
                [dependencies]
                serde = { version = "1.0█", features = ["derive"] }
                "#,
                Select(CodeActionRefactorRewriteName::UpdateDependencyToLatestVersion),
                project_root_path().join("crates/subcrate/Cargo.toml"),
                tombi_lsp::backend::Options {
                    offline: Some(true),
                    no_cache: Some(false),
                },
                UseCacheResponses(vec![CachedResponseSpec::new(
                    "https://crates.io/api/v1/crates/serde",
                    r#"{
                        "crate": {
                            "max_version": "1.0.228"
                        }
                    }"#,
                )]),
            ) -> Ok(Some(
                r#"
                [dependencies]
                serde = { version = "1.0.228", features = ["derive"] }
                "#
            ));
        }
    }

    mod pyproject_toml {
        use tombi_extension_pyproject::CodeActionRefactorRewriteName;
        use tombi_test_lib::project_root_path;

        use crate::{CachedResponseSpec, UseCacheResponses, test_code_action_refactor_rewrite};

        test_code_action_refactor_rewrite! {
            #[tokio::test]
            async fn pyproject_dependencies_update_to_latest_version(
                r#"
                [project]
                dependencies = ["requests█>=2.0; python_version < '3.13'"]
                "#,
                Select(CodeActionRefactorRewriteName::UpdateDependencyToLatestVersion),
                project_root_path().join("pyproject.toml"),
                tombi_lsp::backend::Options {
                    offline: Some(true),
                    no_cache: Some(false),
                },
                UseCacheResponses(vec![CachedResponseSpec::new(
                    "https://pypi.org/pypi/requests/json",
                    r#"{
                        "info": {
                            "version": "2.33.1"
                        }
                    }"#,
                )]),
            ) -> Ok(Some(
                r#"
                [project]
                dependencies = ["requests==2.33.1; python_version < '3.13'"]
                "#
            ));
        }

        test_code_action_refactor_rewrite! {
            #[tokio::test]
            async fn pyproject_dependencies_update_to_latest_version_noop(
                r#"
                [project]
                dependencies = ["requests█==2.33.1; python_version < '3.13'"]
                "#,
                Select(CodeActionRefactorRewriteName::UpdateDependencyToLatestVersion),
                project_root_path().join("pyproject.toml"),
                tombi_lsp::backend::Options {
                    offline: Some(true),
                    no_cache: Some(false),
                },
                UseCacheResponses(vec![CachedResponseSpec::new(
                    "https://pypi.org/pypi/requests/json",
                    r#"{
                        "info": {
                            "version": "2.33.1"
                        }
                    }"#,
                )]),
            ) -> Ok(None);
        }

        test_code_action_refactor_rewrite! {
            #[tokio::test]
            async fn pyproject_workspace_root_updates_dependency_to_latest_version(
                r#"
                [tool.uv.workspace]
                members = ["member1"]

                [project]
                dependencies = ["requests█>=2.0"]
                "#,
                Select(CodeActionRefactorRewriteName::UpdateDependencyToLatestVersion),
                project_root_path().join("pyproject.toml"),
                tombi_lsp::backend::Options {
                    offline: Some(true),
                    no_cache: Some(false),
                },
                UseCacheResponses(vec![CachedResponseSpec::new(
                    "https://pypi.org/pypi/requests/json",
                    r#"{
                        "info": {
                            "version": "2.33.1"
                        }
                    }"#,
                )]),
            ) -> Ok(Some(
                r#"
                [tool.uv.workspace]
                members = ["member1"]

                [project]
                dependencies = ["requests==2.33.1"]
                "#
            ));
        }
    }

    #[macro_export]
    macro_rules! test_code_action_refactor_rewrite {
        (
            #[tokio::test]
            async fn $name:ident($source:expr $(, $arg:expr )* $(,)?) -> Ok(source);
        ) => {
            test_code_action_refactor_rewrite! {
                #[tokio::test]
                async fn $name($source $(, $arg)*) -> Ok($source);
            }
        };

        (
            #[tokio::test]
            async fn $name:ident($source:expr $(, $arg:expr )* $(,)?) -> Ok($expected:expr);
        ) => {
            #[tokio::test]
            async fn $name() -> Result<(), Box<dyn std::error::Error>> {
                use itertools::Itertools;
                use tombi_lsp::Backend;
                use tombi_lsp::handler::handle_code_action;
                use tombi_lsp::handler::handle_did_open;
                use tombi_text::IntoLsp;
                use tower_lsp::LspService;
                use tower_lsp::lsp_types::{CodeActionParams, TextDocumentIdentifier, Url};
                use tower_lsp::lsp_types::{DidOpenTextDocumentParams, TextDocumentItem};

                tombi_test_lib::init_log();

                #[allow(unused)]
                #[derive(Default)]
                pub struct TestArgs {
                    select: Option<String>,
                    toml_file_path: Option<std::path::PathBuf>,
                    backend_options: tombi_lsp::backend::Options,
                    cached_responses: Vec<$crate::CachedResponseSpec>,
                }

                #[allow(unused)]
                pub trait ApplyTestArg {
                    fn apply(self, args: &mut TestArgs);
                }

                /// Code action title to select in assertions.
                #[allow(unused)]
                pub struct Select<T>(pub T);

                impl<T: ToString> ApplyTestArg for Select<T> {
                    fn apply(self, args: &mut TestArgs) {
                        args.select = Some(self.0.to_string());
                    }
                }

                impl ApplyTestArg for tombi_lsp::backend::Options {
                    fn apply(self, args: &mut TestArgs) {
                        args.backend_options = self;
                    }
                }

                impl ApplyTestArg for std::path::PathBuf {
                    fn apply(self, args: &mut TestArgs) {
                        args.toml_file_path = Some(self);
                    }
                }

                impl ApplyTestArg for $crate::UseCacheResponses {
                    fn apply(self, args: &mut TestArgs) {
                        args.cached_responses = self.0;
                    }
                }

                #[allow(unused_mut)]
                let mut args = TestArgs::default();
                $(ApplyTestArg::apply($arg, &mut args);)*

                let (service, _) = LspService::new(|client| {
                    Backend::new(client, &args.backend_options)
                });
                let backend = service.inner();
                let _cache_home =
                    (!args.cached_responses.is_empty()).then($crate::TestCacheHome::new);
                for response in &args.cached_responses {
                    $crate::write_cached_response(response.url, response.body).await;
                }
                let temp_file = tempfile::NamedTempFile::with_suffix_in(
                    ".toml",
                    std::env::current_dir().expect("failed to get current directory"),
                )?;

                let mut toml_text = textwrap::dedent($source).trim().to_string();
                let Some(index) = toml_text.find("█") else {
                    return Err(
                        "failed to find code action position marker (█) in the test data".into(),
                    );
                };
                toml_text.remove(index);
                log::debug!("test toml text: {:?}", toml_text);
                log::debug!("test toml text index: {:?}", index);

                let line_index =
                    tombi_text::LineIndex::new(&toml_text, tombi_text::EncodingKind::Utf16);

                let toml_file_url = args
                    .toml_file_path
                    .as_ref()
                    .map(|path| Url::from_file_path(path).expect("failed to convert file path to URL"))
                    .unwrap_or_else(|| {
                        Url::from_file_path(temp_file.path())
                            .expect("failed to convert temp file path to URL")
                    });

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

                let params = CodeActionParams {
                    text_document: TextDocumentIdentifier {
                        uri: toml_file_url.clone(),
                    },
                    range: tombi_text::Range::at(
                        (tombi_text::Position::default()
                            + tombi_text::RelativePosition::of(&toml_text[..index])),
                    )
                    .into_lsp(&line_index),
                    context: Default::default(),
                    work_done_progress_params: Default::default(),
                    partial_result_params: Default::default(),
                };

                let Ok(actions) = handle_code_action(backend, params).await else {
                    return Err("failed to get code actions".into());
                };

                log::debug!("code actions found: {:?}", actions);

                match (actions, $expected) {
                    (Some(actions), Some(expected)) => {
                        let Some(selected) = args.select.as_ref() else {
                            return Err("no code action selection provided via Select(..)".into());
                        };
                        let Some(action) = actions.into_iter().find_map(|a| match a {
                            tower_lsp::lsp_types::CodeActionOrCommand::CodeAction(ca)
                                if ca.title == *selected =>
                            {
                                Some(ca)
                            }
                            _ => None,
                        }) else {
                            return Err(format!(
                                "failed to find the selected code action '{}'.",
                                selected
                            )
                            .into());
                        };
                        let Some(edit) = action.edit else {
                            return Err("selected code action has no edit".into());
                        };

                        let mut new_text = toml_text.clone();

                        if let Some(tower_lsp::lsp_types::DocumentChanges::Edits(edits)) =
                            edit.document_changes
                        {
                            let mut all_edits: Vec<_> =
                                edits.into_iter().flat_map(|e| e.edits).collect();
                            // Sort by range.start in descending order to apply edits from the end of the text.
                            all_edits.sort_by(|a, b| {
                                let a = match a {
                                    tower_lsp::lsp_types::OneOf::Left(e) => &e.range.start,
                                    _ => return std::cmp::Ordering::Equal,
                                };
                                let b = match b {
                                    tower_lsp::lsp_types::OneOf::Left(e) => &e.range.start,
                                    _ => return std::cmp::Ordering::Equal,
                                };
                                b.line.cmp(&a.line).then(b.character.cmp(&a.character))
                            });
                            // Apply all edits using a single string buffer and byte offsets.
                            let mut line_offsets = Vec::new();
                            let mut acc = 0;
                            for line in new_text.lines() {
                                line_offsets.push(acc);
                                acc += line.len() + 1; // +1 for '\n'
                            }
                            let mut text = new_text.clone();
                            for text_edit in all_edits {
                                if let tower_lsp::lsp_types::OneOf::Left(edit) = text_edit {
                                    let start_line = edit.range.start.line as usize;
                                    let start_char = edit.range.start.character as usize;
                                    let end_line = edit.range.end.line as usize;
                                    let end_char = edit.range.end.character as usize;
                                    let start =
                                        line_offsets.get(start_line).copied().unwrap_or(0) + start_char;
                                    let end =
                                        line_offsets.get(end_line).copied().unwrap_or(0) + end_char;
                                    text.replace_range(start..end, &edit.new_text);
                                    // Recalculate line offsets after each edit to ensure correct byte positions.
                                    line_offsets.clear();
                                    acc = 0;
                                    for line in text.lines() {
                                        line_offsets.push(acc);
                                        acc += line.len() + 1;
                                    }
                                }
                            }
                            new_text = text;
                        }
                        pretty_assertions::assert_eq!(new_text, textwrap::dedent(expected).trim());
                        Ok(())
                    }
                    (None, None) => {
                        if args.select.is_some() {
                            return Err("Select(..) should not be provided when expecting no code actions (Ok(None))".into());
                        };
                        log::debug!("no code actions found, as expected");
                        Ok(())
                    }
                    (Some(actions), None) => {
                        let Some(selected) = args.select.as_ref() else {
                            return Err("no code action selection provided via Select(..)".into());
                        };
                        let None = actions.iter().find_map(|a| match a {
                            tower_lsp::lsp_types::CodeActionOrCommand::CodeAction(ca)
                                if ca.title == *selected =>
                            {
                                Some(ca)
                            }
                            _ => None,
                        }) else {
                            return Err(format!(
                                "expected '{}' but not included in {:?}",
                                selected,
                                actions
                                    .iter()
                                    .filter_map(|a| match a {
                                        tower_lsp::lsp_types::CodeActionOrCommand::CodeAction(ca) =>
                                            Some(ca.title.clone()),
                                        _ => None,
                                    })
                                    .collect_vec()
                            )
                            .into());
                        };
                        Ok(())
                    }
                    (None, Some(_)) => {
                        return Err("expected code actions, but found none".into());
                    }
                }
            }
        };
    }
}
