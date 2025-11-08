use tombi_test_lib::project_root_path;

mod goto_definition_tests {
    use super::*;

    mod document_schema {
        use super::*;

        test_goto_definition!(
            #[tokio::test]
            async fn relative_schema_path(
                r#"
                #:schema ./json.schemastore.org/tombi.json█

                toml-version = "v1.0.0"
                "#,
                project_root_path().join("tombi.toml"),
            ) -> Ok([project_root_path().join("json.schemastore.org/tombi.json")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn ignores_http_schema_uri(
                r#"
                #:schema http://www.schemastore.org/tombi.json█

                toml-version = "v1.0.0"
                "#,
                project_root_path().join("tombi.toml"),
            ) -> Ok(["http://www.schemastore.org/tombi.json"]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn ignores_missing_relative_path(
                r#"
                #:schema schemas/does-not-exist.json█

                toml-version = "v1.0.0"
                "#,
                project_root_path().join("tombi.toml"),
            ) -> Ok([]);
        );
    }

    mod cargo_schema {
        use super::*;

        test_goto_definition!(
            #[tokio::test]
            async fn dependencies_serde_workspace(
                r#"
                [dependencies]
                serde = { workspace█ = true }
                "#,
                project_root_path().join("crates/test-crate/Cargo.toml"),
            ) -> Ok([project_root_path().join("Cargo.toml")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn dependencies_serde_with_workspace(
                r#"
                [dependencies]
                serde█ = { workspace = true }
                "#,
                project_root_path().join("crates/test-crate/Cargo.toml"),
            ) -> Ok([project_root_path().join("Cargo.toml")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn dependencies_tombi_ast_workspace(
                r#"
                [dependencies]
                tombi-ast = { workspace█ = true }
                "#,
                project_root_path().join("crates/test-crate/Cargo.toml"),
            ) -> Ok([project_root_path().join("Cargo.toml")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn dependencies_tombi_ast(
                r#"
                [dependencies]
                tombi-ast█ = { workspace = true }
                "#,
                project_root_path().join("crates/test-crate/Cargo.toml"),
            ) -> Ok([project_root_path().join("crates/tombi-ast/Cargo.toml")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn dev_dependencies_rstest_workspace(
                r#"
                [dev-dependencies]
                rstest = { workspace█ = true }
                "#,
                project_root_path().join("crates/test-crate/Cargo.toml"),
            ) -> Ok([project_root_path().join("Cargo.toml")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn build_dependencies_rstest_workspace(
                r#"
                [build-dependencies]
                serde = { workspace█ = true }
                "#,
                project_root_path().join("crates/test-crate/Cargo.toml"),
            ) -> Ok([project_root_path().join("Cargo.toml")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn dev_dependencies_tombi_ast_with_workspace(
                r#"
                [dev-dependencies]
                tombi-ast█ = { workspace = true }
                "#,
                project_root_path().join("crates/test-crate/Cargo.toml"),
            ) -> Ok([project_root_path().join("crates/tombi-ast/Cargo.toml")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn dev_dependencies_tombi_ast_workspace(
                r#"
                [dev-dependencies]
                tombi-ast = { workspace█ = true }
                "#,
                project_root_path().join("crates/test-crate/Cargo.toml"),
            ) -> Ok([project_root_path().join("Cargo.toml")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn build_dependencies_tombi_ast_workspace(
                r#"
                [build-dependencies]
                tombi-ast = { workspace█ = true }
                "#,
                project_root_path().join("crates/test-crate/Cargo.toml"),
            ) -> Ok([project_root_path().join("Cargo.toml")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn workspace_dependencies_serde(
                r#"
                [workspace.dependencies]
                serde█ = { version = "1.0.0" }
                "#,
                project_root_path().join("Cargo.toml"),
            ) -> Ok([]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn workspace_dependencies_tombi_ast(
                r#"
                [workspace.dependencies]
                tombi-ast = { path█ = "crates/tombi-ast" }
                "#,
                project_root_path().join("Cargo.toml"),
            ) -> Ok([project_root_path().join("crates/tombi-ast/Cargo.toml")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn workspace_dependencies_tombi_ast_editor(
                r#"
                [workspace]
                resolver = "2"
                members = ["crates/*"]

                [workspace.dependencies]
                tombi-ast-editor█ = { path = "crates/tombi-ast-editor" }
                "#,
                project_root_path().join("Cargo.toml"),
            ) -> Ok([
                project_root_path().join("crates/tombi-ast-editor/Cargo.toml"),
                project_root_path().join("crates/tombi-formatter/Cargo.toml"),
            ]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn workspace_dependencies_semver(
                r#"
                [workspace]
                resolver = "2"
                members = ["crates/*"]

                [workspace.dependencies]
                semver█ = { version = "1.0.23" }
                "#,
                project_root_path().join("Cargo.toml"),
            ) -> Ok([
                project_root_path().join("crates/tombi-lsp/Cargo.toml"),
            ]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn workspace_members_xtask(
                r#"
                [workspace]
                members = [
                    "xtask█"
                ]
                "#,
                project_root_path().join("Cargo.toml"),
            ) -> Ok([project_root_path().join("xtask/Cargo.toml")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn workspace_members_crate_tombi_ast(
                r#"
                [workspace]
                members = [
                    "crates/tombi-ast█"
                ]
                "#,
                project_root_path().join("Cargo.toml"),
            ) -> Ok([project_root_path().join("crates/tombi-ast/Cargo.toml")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn workspace_members_extension_tombi_extension_cargo(
                r#"
                [workspace]
                members = [
                    "extensions/*█"
                ]
                "#,
                project_root_path().join("Cargo.toml"),
            ) -> Ok([
                project_root_path().join("extensions/tombi-extension-cargo/Cargo.toml"),
                project_root_path().join("extensions/tombi-extension-tombi/Cargo.toml"),
                project_root_path().join("extensions/tombi-extension-uv/Cargo.toml"),
            ]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn bin_path_resolves_existing_file(
                r#"
                [[bin]]
                name = "profile"
                path = "src/bin/profile.rs█"
                "#,
                project_root_path().join("crates/tombi-glob/Cargo.toml"),
            ) -> Ok([project_root_path().join("crates/tombi-glob/src/bin/profile.rs")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn bin_path_missing_file_returns_none(
                r#"
                [[bin]]
                name = "missing"
                path = "src/bin/missing.rs█"
                "#,
                project_root_path().join("crates/tombi-glob/Cargo.toml"),
            ) -> Ok([]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn bin_path_multiple_entries_follow_active_table(
                r#"
                [[bin]]
                name = "primary"
                path = "src/bin/profile.rs"

                [[bin]]
                name = "secondary"
                path = "src/bin/profile.rs█"
                "#,
                project_root_path().join("crates/tombi-glob/Cargo.toml"),
            ) -> Ok([project_root_path().join("crates/tombi-glob/src/bin/profile.rs")]);
        );

        // Tests for platform specific dependencies (Issue #1192)
        test_goto_definition!(
            #[tokio::test]
            async fn target_dependencies_serde_workspace(
                r#"
                [target.'cfg(unix)'.dependencies]
                serde = { workspace█ = true }
                "#,
                project_root_path().join("crates/tombi-lsp/Cargo.toml"),
            ) -> Ok([project_root_path().join("Cargo.toml")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn target_dependencies_tombi_ast_workspace(
                r#"
                [target.'cfg(unix)'.dependencies]
                tombi-ast = { workspace█ = true }
                "#,
                project_root_path().join("crates/tombi-lsp/Cargo.toml"),
            ) -> Ok([project_root_path().join("Cargo.toml")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn target_dependencies_path(
                r#"
                [target.'cfg(unix)'.dependencies]
                tombi-ast = { path█ = "crates/tombi-ast" }
                "#,
                project_root_path().join("Cargo.toml"),
            ) -> Ok([project_root_path().join("crates/tombi-ast/Cargo.toml")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn target_dev_dependencies_workspace(
                r#"
                [target.'cfg(target_os = "linux")'.dev-dependencies]
                serde = { workspace█ = true }
                "#,
                project_root_path().join("crates/tombi-lsp/Cargo.toml"),
            ) -> Ok([project_root_path().join("Cargo.toml")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn target_build_dependencies_workspace(
                r#"
                [target.'cfg(windows)'.build-dependencies]
                serde = { workspace█ = true }
                "#,
                project_root_path().join("crates/tombi-lsp/Cargo.toml"),
            ) -> Ok([project_root_path().join("Cargo.toml")]);
        );
    }

    mod pyproject_uv_schema {
        use super::*;

        test_goto_definition!(
            #[tokio::test]
            async fn tool_uv_sources_package_with_workspace(
                r#"
                [tool.uv.sources]
                tombi-beta█ = { workspace = true }
                "#,
                project_root_path().join("python/tombi-beta/pyproject.toml"),
            ) -> Ok([project_root_path().join("python/tombi-beta/pyproject.toml")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn tool_uv_sources_package_workspace(
                r#"
                [tool.uv.sources]
                tombi-beta = { workspace█ = true }
                "#,
                project_root_path().join("python/tombi-beta/pyproject.toml"),
            ) -> Ok([project_root_path().join("pyproject.toml")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn tool_uv_workspace_members(
                r#"
                [tool.uv.workspace]
                members█ = ["python/tombi-beta"]
                "#,
                project_root_path().join("pyproject.toml"),
            ) -> Ok([project_root_path().join("python/tombi-beta/pyproject.toml")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn tool_uv_workspace_members_python_tombi_beta(
                r#"
                [tool.uv.workspace]
                members = ["python/tombi-beta█"]
                "#,
                project_root_path().join("pyproject.toml"),
            ) -> Ok([project_root_path().join("python/tombi-beta/pyproject.toml")]);
        );
    }

    mod pyproject_uv_workspace_dependencies {
        use super::*;

        fn pyproject_workspace_fixtures_path() -> std::path::PathBuf {
            project_root_path().join("crates/tombi-lsp/tests/fixtures/pyproject_workspace")
        }

        test_goto_definition!(
            #[tokio::test]
            async fn project_dependencies_inherit_workspace_version(
                r#"
                [project]
                name = "app"
                version = "0.1.0"
                dependencies = [
                    "pydantic█"
                ]
                "#,
                pyproject_workspace_fixtures_path().join("members/app/pyproject.toml"),
            ) -> Ok([
                pyproject_workspace_fixtures_path().join("pyproject.toml")
            ]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn workspace_dependencies_list_member_usages(
                r#"
                [project]
                name = "app"
                version = "0.1.0"
                dependencies = ["pydantic█"]

                [tool.uv.workspace]
                members = [
                    "members/app",
                    "members/app2",
                    "members/app3",
                ]
                "#,
                pyproject_workspace_fixtures_path().join("pyproject.toml"),
            ) -> Ok([
                pyproject_workspace_fixtures_path().join("members/app/pyproject.toml"),
                pyproject_workspace_fixtures_path().join("members/app2/pyproject.toml"),
            ]);
        );
    }

    mod tombi_schema {
        use super::*;

        test_goto_definition!(
            #[tokio::test]
            async fn schema_catalog_path(
                r#"
                [[schemas]]
                path = "█json.schemastore.org/tombi.json"
                "#,
                project_root_path().join("tombi.toml"),
            ) -> Ok([project_root_path().join("json.schemastore.org/tombi.json")]);
        );
    }

    #[macro_export]
    macro_rules! test_goto_definition {
        (#[tokio::test] async fn $name:ident(
            $source:expr,
            $file_path:expr,
        ) -> Ok([$($expected_file_path:expr),*$(,)?]);) => {
            #[tokio::test]
            async fn $name() -> Result<(), Box<dyn std::error::Error>> {
                use std::str::FromStr;

                use itertools::Itertools;
                use tombi_lsp::handler::{handle_did_open, handle_goto_definition};
                use tombi_lsp::Backend;
                use tower_lsp::{
                    lsp_types::{
                        DidOpenTextDocumentParams, GotoDefinitionParams,
                        PartialResultParams, TextDocumentIdentifier, TextDocumentItem,
                        TextDocumentPositionParams, Url, WorkDoneProgressParams,
                    },
                    LspService,
                };
                use tombi_text::IntoLsp;

                trait ToUri {
                    fn to_uri(&self) -> tombi_uri::Uri;
                }

                impl ToUri for tombi_uri::Uri {
                    fn to_uri(&self) -> tombi_uri::Uri {
                        self.clone()
                    }
                }

                impl ToUri for std::path::PathBuf {
                    fn to_uri(&self) -> tombi_uri::Uri {
                        tombi_uri::Uri::from_file_path(self).unwrap()
                    }
                }

                impl ToUri for &str {
                    fn to_uri(&self) -> tombi_uri::Uri {
                        tombi_uri::Uri::from_str(self).unwrap()
                    }
                }

                tombi_test_lib::init_tracing();

                let (service, _) = LspService::new(|client| {
                    Backend::new(client, &tombi_lsp::backend::Options::default())
                });

                let backend = service.inner();

                let toml_file_url = Url::from_file_path($file_path).expect("failed to convert file path to URL");

                let mut toml_text = textwrap::dedent($source).trim().to_string();
                let Some(index) = toml_text.as_str().find("█") else {
                    return Err("failed to find position marker (█) in the test data".into());
                };
                toml_text.remove(index);
                let line_index =
                tombi_text::LineIndex::new(&toml_text, tombi_text::EncodingKind::Utf16);

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

                let params = GotoDefinitionParams {
                    text_document_position_params: TextDocumentPositionParams {
                        text_document: TextDocumentIdentifier { uri: toml_file_url },
                        position: (tombi_text::Position::default()
                            + tombi_text::RelativePosition::of(&toml_text[..index]))
                        .into_lsp(&line_index),
                    },
                    work_done_progress_params: WorkDoneProgressParams::default(),
                    partial_result_params: PartialResultParams::default(),
                };

                let Ok(result) = handle_goto_definition(&backend, params).await else {
                    return Err("failed to handle goto_definition".into());
                };

                tracing::debug!("goto_definition result: {:#?}", result);

                let expected_paths: Vec<tombi_uri::Uri> = vec![$($expected_file_path.to_uri()),*];

                match result {
                    Some(definition_links) => {
                        pretty_assertions::assert_eq!(
                            definition_links.into_iter().map(|link| link.uri.to_uri()).collect_vec(),
                            expected_paths,
                        );
                    },
                    None => {
                        if !expected_paths.is_empty() {
                            panic!("No definition link was returned, but expected paths: {:?}", expected_paths);
                        }
                    }
                }

                Ok(())
            }
        };
    }
}
