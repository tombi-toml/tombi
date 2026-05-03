use tombi_test_lib::{cargo_feature_navigation_fixture_path, project_root_path};

macro_rules! test_references {
    (#[tokio::test] async fn $name:ident(
        $source:expr $(, $arg:expr )* $(,)?
    ) -> Ok([$($expected_file_path:expr),*$(,)?]);) => {
        #[tokio::test]
        async fn $name() -> Result<(), Box<dyn std::error::Error>> {
            use itertools::Itertools;
            use tombi_lsp::handler::{handle_did_open, handle_references};
            use tombi_lsp::Backend;
            use tower_lsp::{
                lsp_types::{
                    DidOpenTextDocumentParams, PartialResultParams, ReferenceContext,
                    ReferenceParams, TextDocumentIdentifier, TextDocumentItem,
                    TextDocumentPositionParams, Url, WorkDoneProgressParams,
                },
                LspService,
            };
            use tombi_text::IntoLsp;

            tombi_test_lib::init_log();

            #[allow(unused)]
            #[derive(Default)]
            struct TestArgs {
                source_file_path: Option<std::path::PathBuf>,
                config_file_path: Option<std::path::PathBuf>,
                include_declaration: bool,
                backend_options: tombi_lsp::backend::Options,
            }

            trait ApplyTestArg {
                fn apply(self, args: &mut TestArgs);
            }

            struct SourcePath(std::path::PathBuf);

            impl ApplyTestArg for SourcePath {
                fn apply(self, args: &mut TestArgs) {
                    args.source_file_path = Some(self.0);
                }
            }

            #[allow(unused)]
            struct ConfigPath(std::path::PathBuf);

            impl ApplyTestArg for ConfigPath {
                fn apply(self, args: &mut TestArgs) {
                    args.config_file_path = Some(self.0);
                }
            }

            #[allow(unused)]
            struct IncludeDeclaration(bool);

            impl ApplyTestArg for IncludeDeclaration {
                fn apply(self, args: &mut TestArgs) {
                    args.include_declaration = self.0;
                }
            }

            impl ApplyTestArg for tombi_lsp::backend::Options {
                fn apply(self, args: &mut TestArgs) {
                    args.backend_options = self;
                }
            }

            let mut args = TestArgs::default();
            $(ApplyTestArg::apply($arg, &mut args);)*

            let (service, _) = LspService::new(|client| {
                Backend::new(client, &args.backend_options)
            });

            let backend = service.inner();
            let source_path = args
                .source_file_path
                .as_ref()
                .ok_or("SourcePath must be provided for references tests")?;

            if let Some(config_file_path) = args.config_file_path.as_ref() {
                let config_content = std::fs::read_to_string(config_file_path).map_err(|e| {
                    format!(
                        "failed to read config file {}: {}",
                        config_file_path.display(),
                        e
                    )
                })?;
                let config: tombi_config::Config =
                    serde_tombi::from_str_async(&config_content).await.map_err(|e| {
                        format!(
                            "failed to parse config file {}: {}",
                            config_file_path.display(),
                            e
                        )
                    })?;

                let config_schema_store = backend
                    .config_manager
                    .config_schema_store_for_file(source_path)
                    .await;

                if let Some(config_path) = config_schema_store.config_path {
                    backend
                        .config_manager
                        .update_config_with_path(config, &config_path)
                        .await
                        .map_err(|e| {
                            format!(
                                "failed to update config {}: {}",
                                config_path.display(),
                                e
                            )
                        })?;
                } else {
                    backend.config_manager.update_editor_config(config).await;
                }
            }

            let toml_file_url = Url::from_file_path(source_path)
                .expect("failed to convert source file path to URL");

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

            let params = ReferenceParams {
                text_document_position: TextDocumentPositionParams {
                    text_document: TextDocumentIdentifier { uri: toml_file_url },
                    position: (tombi_text::Position::default()
                        + tombi_text::RelativePosition::of(&toml_text[..index]))
                    .into_lsp(&line_index),
                },
                context: ReferenceContext {
                    include_declaration: args.include_declaration,
                },
                work_done_progress_params: WorkDoneProgressParams::default(),
                partial_result_params: PartialResultParams::default(),
            };

            let Ok(result) = handle_references(&backend, params).await else {
                return Err("failed to handle references".into());
            };

            let expected_paths: Vec<std::path::PathBuf> = vec![$($expected_file_path.to_owned()),*];

            match result {
                Some(definition_links) => {
                    pretty_assertions::assert_eq!(
                        definition_links.into_iter().map(|link| link.uri.to_file_path().unwrap()).collect_vec(),
                        expected_paths,
                    );
                },
                None => {
                    if !expected_paths.is_empty() {
                        panic!("No references were returned, but expected paths: {:?}", expected_paths);
                    }
                }
            }

            Ok(())
        }
    };
}

mod references_tests {
    use super::*;

    mod cargo_schema {
        use super::*;

        fn fixture_config_path(name: &str) -> std::path::PathBuf {
            project_root_path()
                .join("crates/tombi-lsp/tests/fixtures/extensions")
                .join(name)
                .join("tombi.toml")
        }

        test_references!(
            #[tokio::test]
            async fn feature_key_lists_same_file_and_workspace_usages(
                r#"
                [package]
                name = "provider"
                version = "0.1.0"
                edition = "2024"

                [features]
                jsonschema█ = []
                "#,
                SourcePath(
                    cargo_feature_navigation_fixture_path().join("workspace/provider/Cargo.toml")
                ),
            ) -> Ok([
                cargo_feature_navigation_fixture_path().join("workspace/Cargo.toml"),
                cargo_feature_navigation_fixture_path().join("workspace/consumer/Cargo.toml"),
                cargo_feature_navigation_fixture_path().join("workspace/consumer/Cargo.toml"),
                cargo_feature_navigation_fixture_path().join("workspace/renamed-consumer/Cargo.toml"),
                cargo_feature_navigation_fixture_path().join("workspace/renamed-consumer/Cargo.toml"),
                cargo_feature_navigation_fixture_path().join("workspace/weak-consumer/Cargo.toml"),
            ]);
        );

        test_references!(
            #[tokio::test]
            async fn cargo_references_use_references_setting_not_goto_definition(
                r#"
                [package]
                name = "provider"
                version = "0.1.0"
                edition = "2024"

                [features]
                jsonschema█ = []
                "#,
                SourcePath(
                    cargo_feature_navigation_fixture_path().join("workspace/provider/Cargo.toml")
                ),
                ConfigPath(fixture_config_path("cargo-references-only-enabled")),
            ) -> Ok([
                cargo_feature_navigation_fixture_path().join("workspace/Cargo.toml"),
                cargo_feature_navigation_fixture_path().join("workspace/consumer/Cargo.toml"),
                cargo_feature_navigation_fixture_path().join("workspace/consumer/Cargo.toml"),
                cargo_feature_navigation_fixture_path().join("workspace/renamed-consumer/Cargo.toml"),
                cargo_feature_navigation_fixture_path().join("workspace/renamed-consumer/Cargo.toml"),
                cargo_feature_navigation_fixture_path().join("workspace/weak-consumer/Cargo.toml"),
            ]);
        );

        test_references!(
            #[tokio::test]
            async fn include_declaration_appends_definition_location(
                r#"
                [package]
                name = "provider"
                version = "0.1.0"
                edition = "2024"

                [features]
                jsonschema█ = []
                "#,
                SourcePath(
                    cargo_feature_navigation_fixture_path().join("workspace/provider/Cargo.toml")
                ),
                IncludeDeclaration(true),
            ) -> Ok([
                cargo_feature_navigation_fixture_path().join("workspace/Cargo.toml"),
                cargo_feature_navigation_fixture_path().join("workspace/consumer/Cargo.toml"),
                cargo_feature_navigation_fixture_path().join("workspace/consumer/Cargo.toml"),
                cargo_feature_navigation_fixture_path().join("workspace/renamed-consumer/Cargo.toml"),
                cargo_feature_navigation_fixture_path().join("workspace/renamed-consumer/Cargo.toml"),
                cargo_feature_navigation_fixture_path().join("workspace/weak-consumer/Cargo.toml"),
                cargo_feature_navigation_fixture_path().join("workspace/provider/Cargo.toml"),
            ]);
        );

        test_references!(
            #[tokio::test]
            async fn server_references_setting_disables_references(
                r#"
                [package]
                name = "provider"
                version = "0.1.0"
                edition = "2024"

                [features]
                jsonschema█ = []
                "#,
                SourcePath(
                    cargo_feature_navigation_fixture_path().join("workspace/provider/Cargo.toml")
                ),
                ConfigPath(fixture_config_path("references-disabled")),
            ) -> Ok([]);
        );

        test_references!(
            #[tokio::test]
            async fn optional_dependency_lists_workspace_usages(
                r#"
                [package]
                name = "nagi-config"
                version = "0.1.0"
                edition = "2024"

                [dependencies]
                nagi_uri = { workspace = true, optional█ = true }

                [features]
                default = ["postgres", "serde"]
                postgres = []
                serde = ["dep:nagi_uri"]
                "#,
                SourcePath(cargo_feature_navigation_fixture_path().join("explicit/Cargo.toml")),
            ) -> Ok([
                cargo_feature_navigation_fixture_path().join("explicit/Cargo.toml"),
            ]);
        );

        test_references!(
            #[tokio::test]
            async fn target_optional_dependency_include_declaration_adds_dependency_key(
                r#"
                [package]
                name = "example"
                version = "0.1.0"

                [target.'cfg(unix)'.dependencies]
                schemars = { version = "1.0", optional█ = true }

                [features]
                bundle = ["dep:schemars"]
                "#,
                SourcePath(project_root_path().join("crates/example/Cargo.toml")),
                IncludeDeclaration(true),
            ) -> Ok([
                project_root_path().join("crates/example/Cargo.toml"),
                project_root_path().join("crates/example/Cargo.toml"),
            ]);
        );

        test_references!(
            #[tokio::test]
            async fn workspace_dependency_key_lists_member_usages(
                r#"
                [workspace]
                resolver = "2"
                members = ["crates/*"]

                [workspace.dependencies]
                tombi-ast-editor█ = { path = "crates/tombi-ast-editor" }
                "#,
                SourcePath(project_root_path().join("Cargo.toml")),
            ) -> Ok([
                project_root_path().join("crates/tombi-formatter/Cargo.toml"),
            ]);
        );

        test_references!(
            #[tokio::test]
            async fn workspace_registry_dependency_key_lists_member_usages(
                r#"
                [workspace]
                resolver = "2"
                members = ["crates/*"]

                [workspace.dependencies]
                semver█ = { version = "1.0.23" }
                "#,
                SourcePath(project_root_path().join("Cargo.toml")),
            ) -> Ok([
                project_root_path().join("crates/tombi-lsp/Cargo.toml"),
            ]);
        );

        test_references!(
            #[tokio::test]
            async fn package_name_lists_workspace_and_member_dependency_usages(
                r#"
                [package]
                name = "provider█"
                version = "0.1.0"
                edition = "2024"

                [features]
                jsonschema = []
                "#,
                SourcePath(
                    cargo_feature_navigation_fixture_path().join("workspace/provider/Cargo.toml")
                ),
            ) -> Ok([
                cargo_feature_navigation_fixture_path().join("workspace/Cargo.toml"),
                cargo_feature_navigation_fixture_path().join("workspace/consumer/Cargo.toml"),
                cargo_feature_navigation_fixture_path().join("workspace/renamed-consumer/Cargo.toml"),
                cargo_feature_navigation_fixture_path().join("workspace/weak-consumer/Cargo.toml"),
            ]);
        );
    }

    mod pyproject_schema {
        use super::*;

        fn pyproject_workspace_fixtures_path() -> std::path::PathBuf {
            project_root_path().join("crates/tombi-lsp/tests/fixtures/pyproject_workspace")
        }

        fn fixture_config_path(name: &str) -> std::path::PathBuf {
            project_root_path()
                .join("crates/tombi-lsp/tests/fixtures/extensions")
                .join(name)
                .join("tombi.toml")
        }

        test_references!(
            #[tokio::test]
            async fn dependency_groups_group_name_lists_include_group_usages(
                r#"
                [dependency-groups]
                dev = [{ include-group = "ci" }]
                qa = [{ include-group = "ci" }]
                ci█ = ["ruff"]
                "#,
                SourcePath(project_root_path().join("pyproject.toml")),
            ) -> Ok([
                project_root_path().join("pyproject.toml"),
                project_root_path().join("pyproject.toml"),
            ]);
        );

        test_references!(
            #[tokio::test]
            async fn project_dependencies_workspace_usage_locations(
                r#"
                [project]
                name = "app1"
                version = "0.1.0"
                dependencies = [
                    "pydantic█"
                ]
                "#,
                SourcePath(pyproject_workspace_fixtures_path().join("members/app1/pyproject.toml")),
            ) -> Ok([
                pyproject_workspace_fixtures_path().join("pyproject.toml"),
            ]);
        );

        test_references!(
            #[tokio::test]
            async fn project_name_lists_workspace_and_member_dependency_usages(
                r#"
                [project]
                name = "app2█"
                version = "0.1.0"
                dependencies = [
                  "pydantic",
                ]
                "#,
                SourcePath(pyproject_workspace_fixtures_path().join("members/app2/pyproject.toml")),
            ) -> Ok([
                pyproject_workspace_fixtures_path().join("members/app3/pyproject.toml"),
                pyproject_workspace_fixtures_path().join("members/app3/pyproject.toml"),
            ]);
        );

        test_references!(
            #[tokio::test]
            async fn pyproject_references_use_references_setting_not_goto_definition(
                r#"
                [project]
                name = "app1"
                version = "0.1.0"
                dependencies = [
                    "pydantic█"
                ]
                "#,
                SourcePath(pyproject_workspace_fixtures_path().join("members/app1/pyproject.toml")),
                ConfigPath(fixture_config_path("pyproject-references-only-enabled")),
            ) -> Ok([
                pyproject_workspace_fixtures_path().join("pyproject.toml"),
            ]);
        );

        test_references!(
            #[tokio::test]
            async fn pyproject_include_declaration_adds_current_dependency_group(
                r#"
                [dependency-groups]
                dev = [{ include-group = "ci" }]
                qa = [{ include-group = "ci" }]
                ci█ = ["ruff"]
                "#,
                SourcePath(project_root_path().join("pyproject.toml")),
                IncludeDeclaration(true),
            ) -> Ok([
                project_root_path().join("pyproject.toml"),
                project_root_path().join("pyproject.toml"),
                project_root_path().join("pyproject.toml"),
            ]);
        );
    }
}
