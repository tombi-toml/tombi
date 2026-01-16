use tombi_test_lib::project_root_path;

mod goto_declaration_tests {
    use super::*;

    mod cargo_schema {
        use super::*;

        test_goto_declaration!(
            #[tokio::test]
            async fn dependencies_serde_workspace(
                r#"
                [dependencies]
                serde = { workspace█ = true }
                "#,
                SourcePath(project_root_path().join("crates/test-crate/Cargo.toml")),
            ) -> Ok([project_root_path().join("Cargo.toml")]);
        );

        test_goto_declaration!(
            #[tokio::test]
            async fn dependencies_serde(
                r#"
                [dependencies]
                serde█ = { workspace = true }
                "#,
                SourcePath(project_root_path().join("crates/test-crate/Cargo.toml")),
            ) -> Ok([project_root_path().join("Cargo.toml")]);
        );

        test_goto_declaration!(
            #[tokio::test]
            async fn workspace_dependencies_tombi_ast(
                r#"
                [workspace.dependencies]
                tombi-ast = { path█ = "crates/tombi-ast" }
                "#,
                SourcePath(project_root_path().join("Cargo.toml")),
            ) -> Ok([]);
        );

        test_goto_declaration!(
            #[tokio::test]
            async fn workspace_members_xtask(
                r#"
                [workspace]
                members = [
                    "xtask█"
                ]
                "#,
                SourcePath(project_root_path().join("Cargo.toml")),
            ) -> Ok([]);
        );
    }

    mod pyproject_uv_schema {
        use super::*;

        fn pyproject_workspace_fixtures_path() -> std::path::PathBuf {
            project_root_path().join("crates/tombi-lsp/tests/fixtures/pyproject_workspace")
        }

        test_goto_declaration!(
            #[tokio::test]
            async fn tool_uv_sources_tombi_beta(
                r#"
                [tool.uv.sources]
                tombi-beta█ = { workspace = true }
                "#,
                SourcePath(project_root_path().join("python/tombi-beta/pyproject.toml")),
            ) -> Ok([project_root_path().join("pyproject.toml")]);
        );

        test_goto_declaration!(
            #[tokio::test]
            async fn tool_uv_sources_tombi_beta_workspace(
                r#"
                [tool.uv.sources]
                tombi-beta = { workspace█ = true }
                "#,
                SourcePath(project_root_path().join("python/tombi-beta/pyproject.toml")),
            ) -> Ok([project_root_path().join("pyproject.toml")]);
        );

        test_goto_declaration!(
            #[tokio::test]
            async fn tool_uv_sources_path_dependency(
                r#"
                [tool.uv.sources]
                tombi-beta = { path = "members/app█" }
                "#,
                SourcePath(project_root_path().join("python/tombi-beta/pyproject.toml")),
            ) -> Ok([project_root_path().join("python/tombi-beta/pyproject.toml")]);
        );

        test_goto_declaration!(
            #[tokio::test]
            async fn project_dependencies_workspace_jump(
                r#"
                [project]
                name = "app"
                version = "0.1.0"
                dependencies = [
                    "pydantic█"
                ]
                "#,
                SourcePath(pyproject_workspace_fixtures_path().join("members/app/pyproject.toml")),
            ) -> Ok([
                pyproject_workspace_fixtures_path().join("pyproject.toml"),
            ]);
        );

        test_goto_declaration!(
            #[tokio::test]
            async fn dependency_groups_member_candidates(
                r#"
                [project]
                name = "workspace"
                version = "0.1.0"
                dependencies = ["anyio>=4.0"]

                [tool.uv.workspace]
                members = [
                    "members/app",
                    "members/app2",
                    "members/app3",
                ]

                [dependency-groups]
                extras = ["pydantic█"]
                "#,
                SourcePath(pyproject_workspace_fixtures_path().join("pyproject.toml")),
            ) -> Ok([]);
        );

        test_goto_declaration!(
            #[tokio::test]
            async fn tool_uv_workspace_members_jump_to_member_project(
                r#"
                [tool.uv.workspace]
                members = ["members/app█"]
                "#,
                SourcePath(pyproject_workspace_fixtures_path().join("pyproject.toml")),
            ) -> Ok([
                pyproject_workspace_fixtures_path().join("members/app/pyproject.toml"),
            ]);
        );

        test_goto_declaration!(
            #[tokio::test]
            async fn tool_uv_workspace_members_glob_multiple_candidates(
                r#"
                [tool.uv.workspace]
                members = ["members/app█*"]
                "#,
                SourcePath(pyproject_workspace_fixtures_path().join("pyproject.toml")),
            ) -> Ok([
                pyproject_workspace_fixtures_path().join("members/app/pyproject.toml"),
                pyproject_workspace_fixtures_path().join("members/app2/pyproject.toml"),
                pyproject_workspace_fixtures_path().join("members/app3/pyproject.toml"),
            ]);
        );
    }

    #[macro_export]
    macro_rules! test_goto_declaration {
        (#[tokio::test] async fn $name:ident(
            $source:expr $(, $arg:expr )* $(,)?
        ) -> Ok([$($expected_file_path:expr),*$(,)?]);) => {
            #[tokio::test]
            async fn $name() -> Result<(), Box<dyn std::error::Error>> {
                use itertools::Itertools;
                use tombi_lsp::handler::{handle_did_open, handle_goto_declaration};
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

                tombi_test_lib::init_tracing();

                #[allow(unused)]
                #[derive(Default)]
                struct TestConfig {
                    source_file_path: Option<std::path::PathBuf>,
                    schema_file_path: Option<std::path::PathBuf>,
                    subschemas: Vec<SubSchemaPath>,
                    backend_options: tombi_lsp::backend::Options,
                }

                #[allow(unused)]
                trait ApplyTestArg {
                    fn apply(self, config: &mut TestConfig);
                }

                #[allow(unused)]
                struct SourcePath(std::path::PathBuf);

                impl ApplyTestArg for SourcePath {
                    fn apply(self, config: &mut TestConfig) {
                        config.source_file_path = Some(self.0);
                    }
                }

                #[allow(unused)]
                struct SchemaPath(std::path::PathBuf);

                impl ApplyTestArg for SchemaPath {
                    fn apply(self, config: &mut TestConfig) {
                        config.schema_file_path = Some(self.0);
                    }
                }

                #[allow(unused)]
                struct SubSchemaPath {
                    pub root: String,
                    pub path: std::path::PathBuf,
                }

                impl ApplyTestArg for SubSchemaPath {
                    fn apply(self, config: &mut TestConfig) {
                        config.subschemas.push(self);
                    }
                }

                impl ApplyTestArg for tombi_lsp::backend::Options {
                    fn apply(self, config: &mut TestConfig) {
                        config.backend_options = self;
                    }
                }

                #[allow(unused_mut)]
                let mut config = TestConfig::default();
                $(ApplyTestArg::apply($arg, &mut config);)*

                let (service, _) = LspService::new(|client| {
                    Backend::new(client, &config.backend_options)
                });

                let backend = service.inner();

                if let Some(schema_file_path) = config.schema_file_path.as_ref() {
                    let schema_uri = tombi_schema_store::SchemaUri::from_file_path(schema_file_path)
                        .expect(
                            format!(
                                "failed to convert schema path to URL: {}",
                                schema_file_path.display()
                            )
                            .as_str(),
                        );

                    backend
                        .config_manager
                        .load_config_schemas(
                            &[tombi_config::SchemaItem::Root(tombi_config::RootSchema {
                                toml_version: None,
                                path: schema_uri.to_string(),
                                include: vec!["*.toml".to_string()],
                            })],
                            None,
                        )
                        .await;
                }

                for subschema in &config.subschemas {
                    let subschema_uri = tombi_schema_store::SchemaUri::from_file_path(&subschema.path)
                        .expect(
                            format!(
                                "failed to convert subschema path to URL: {}",
                                subschema.path.display()
                            )
                            .as_str(),
                        );

                    backend
                        .config_manager
                        .load_config_schemas(
                            &[tombi_config::SchemaItem::Sub(tombi_config::SubSchema {
                                path: subschema_uri.to_string(),
                                include: vec!["*.toml".to_string()],
                                root: subschema.root.clone(),
                            })],
                            None,
                        )
                        .await;
                }

                let source_path = config
                    .source_file_path
                    .as_ref()
                    .ok_or("SourcePath must be provided for goto_declaration tests")?;

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

                let Ok(result) = handle_goto_declaration(&backend, params).await else {
                    return Err("failed to handle goto_declaration".into());
                };

                tracing::debug!("goto_declaration result: {:#?}", result);

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
                            panic!("No definition link was returned, but expected paths: {:?}", expected_paths);
                        }
                    }
                }

                Ok(())
            }
        };
    }
}
