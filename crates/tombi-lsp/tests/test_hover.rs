use tombi_test_lib::{cargo_schema_path, pyproject_schema_path, tombi_schema_path};
mod hover_keys_value {
    use super::*;

    mod tombi_schema {
        use super::*;

        test_hover_keys_value!(
            #[tokio::test]
            async fn tombi_toml_version(
                r#"
                toml-version = "█v1.0.0"
                "#,
                SchemaPath(tombi_schema_path()),
            ) -> Ok({
                "Keys": "toml-version",
                "Value": "String?"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn tombi_toml_version_without_schema(
                r#"
                toml-version = "█v1.0.0"
                "#,
            ) -> Ok({
                "Keys": "toml-version",
                "Value": "String"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn tombi_lint_rules_key_empty(
                r#"
                [lint.rules]
                key-empty = "█warn"
                "#,
                SchemaPath(tombi_schema_path()),
            ) -> Ok({
                "Keys": "lint.rules.key-empty",
                "Value": "String?"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn tombi_schema_catalog_paths(
                r#"
                [schema.catalog]
                paths = ["█https://www.schemastore.org/api/json/catalog.json"]
                "#,
                SchemaPath(tombi_schema_path()),
            ) -> Ok({
                "Keys": "schema.catalog.paths[0]",
                "Value": "String"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn tombi_schema_catalog_path_without_schema(
                r#"
                [schema.catalog]
                path = "█https://www.schemastore.org/api/json/catalog.json"
                "#,
            ) -> Ok({
                "Keys": "schema.catalog.path",
                "Value": "String"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            // NOTE: This test is correct. When you hover over the last key of the header of ArrayOfTable,
            //       the Keys in the hover content is `schema[$index]`, not `schemas`.
            //       Therefore, the Value is `Table`.
            async fn tombi_schemas(
                r#"
                [[schemas█]]
                "#,
                SchemaPath(tombi_schema_path()),
            ) -> Ok({
                "Keys": "schemas[0]",
                "Value": "Table"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn tombi_schemas_without_schema(
                r#"
                [[schemas█]]
                "#,
            ) -> Ok({
                "Keys": "schemas[0]",
                "Value": "Table"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn tombi_schemas_path(
                r#"
                [[schemas]]
                path = "█schemas/tombi.schema.json"
                "#,
                SchemaPath(tombi_schema_path()),
            ) -> Ok({
                "Keys": "schemas[0].path",
                "Value": "String"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn tombi_schemas_include(
                r#"
                [[schemas]]
                path = "schemas/tombi.schema.json"
                include█ = ["*.toml"]
                "#,
                SchemaPath(tombi_schema_path()),
            ) -> Ok({
                "Keys": "schemas[0].include",
                "Value": "Array"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn tombi_schemas_include_array_string(
                r#"
                [[schemas]]
                path = "schemas/tombi.schema.json"
                include = ["█*.toml"]
                "#,
                SchemaPath(tombi_schema_path()),
            ) -> Ok({
                "Keys": "schemas[0].include[0]",
                "Value": "String"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn tombi_comment_directive_toml_version(
                r#"
                #:tombi toml-version█ = "v1.0.0"
                "#,
                SchemaPath(tombi_schema_path()),
            ) -> Ok({
                "Keys": "toml-version",
                "Value": "String?"
            });
        );
    }

    mod cargo_schema {
        use super::*;

        test_hover_keys_value!(
            #[tokio::test]
            async fn cargo_package_name(
                r#"
                [package]
                name█ = "tombi"
                "#,
                SchemaPath(cargo_schema_path()),
            ) -> Ok({
                "Keys": "package.name",
                "Value": "String" // Yes; the value is required.
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn cargo_package_name_incomplete(
                r#"
                [package]
                name = █
                "#,
                SchemaPath(cargo_schema_path()),
            ) -> Ok({
                "Keys": "package.name",
                "Value": "String"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn cargo_package_readme(
                r#"
                [package]
                readme = "█README.md"
                "#,
                SchemaPath(cargo_schema_path()),
            ) -> Ok({
                "Keys": "package.readme",
                "Value": "(String | Boolean | Table)?"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn cargo_package_readme_without_schema(
                r#"
                [package]
                readme = "█README.md"
                "#,
            ) -> Ok({
                "Keys": "package.readme",
                "Value": "String"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn cargo_dependencies_key(
                r#"
                [dependencies]
                serde█ = { workspace = true }
                "#,
                SchemaPath(cargo_schema_path()),
            ) -> Ok({
                "Keys": "dependencies.serde",
                "Value": "(String | Table)?"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn cargo_dependencies_version(
                r#"
                [dependencies]
                serde = "█1.0"
                "#,
                SchemaPath(cargo_schema_path()),
            ) -> Ok({
                "Keys": "dependencies.serde",
                "Value": "(String | Table)?"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn cargo_dependencies_version_without_schema(
                r#"
                [dependencies]
                serde = "█1.0"
                "#,
            ) -> Ok({
                "Keys": "dependencies.serde",
                "Value": "String"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn cargo_dependencies_workspace(
                r#"
                [dependencies]
                serde = { workspace█ = true }
                "#,
                SchemaPath(cargo_schema_path()),
            ) -> Ok({
                "Keys": "dependencies.serde.workspace",
                "Value": "Boolean?"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn cargo_dependencies_workspace_without_schema(
                r#"
                [dependencies]
                serde = { workspace█ = true }
                "#,
            ) -> Ok({
                "Keys": "dependencies.serde.workspace",
                "Value": "Boolean"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn cargo_dependencies_features(
                r#"
                [dependencies]
                serde = { version = "^1.0.0", features█ = ["derive"] }
                "#,
                SchemaPath(cargo_schema_path()),
            ) -> Ok({
                "Keys": "dependencies.serde.features",
                "Value": "Array?"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn cargo_dependencies_features_item(
                r#"
                [dependencies]
                serde = { version = "^1.0.0", features = ["derive█"] }
                "#,
                SchemaPath(cargo_schema_path()),
            ) -> Ok({
                "Keys": "dependencies.serde.features[0]",
                "Value": "String"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn cargo_dependencies_features_item_without_schema(
                r#"
                [dependencies]
                serde = { version = "^1.0.0", features = ["derive█"] }
                "#,
            ) -> Ok({
                "Keys": "dependencies.serde.features[0]",
                "Value": "String"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn cargo_profile_release_strip_debuginfo(
                r#"
                [profile.release]
                strip = "debuginfo█"
                "#,
                SchemaPath(cargo_schema_path()),
            ) -> Ok({
                "Keys": "profile.release.strip",
                "Value": "(String ^ Boolean)?"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn cargo_profile_release_strip_true(
                r#"
                [profile.release]
                strip = true█
                "#,
                SchemaPath(cargo_schema_path()),
            ) -> Ok({
                "Keys": "profile.release.strip",
                "Value": "(String ^ Boolean)?"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn cargo_profile_release_strip_false(
                r#"
                [profile.release]
                strip = false█
                "#,
                SchemaPath(cargo_schema_path()),
            ) -> Ok({
                "Keys": "profile.release.strip",
                "Value": "(String ^ Boolean)?"
            });
        );
    }

    mod pyproject_schema {
        use super::*;

        test_hover_keys_value!(
            #[tokio::test]
            async fn pyproject_project_readme(
                r#"
                [project]
                readme = "█1.0.0"
                "#,
                SchemaPath(pyproject_schema_path()),
            ) -> Ok({
                "Keys": "project.readme",
                "Value": "(String ^ Table)?"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn pyproject_dependency_groups(
                r#"
                [dependency-groups]
                dev = [
                    "█pytest>=8.3.3",
                ]
                "#,
                SchemaPath(pyproject_schema_path()),
            ) -> Ok({
                "Keys": "dependency-groups.dev[0]",
                "Value": "String ^ Table"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn pyproject_dependency_groups_without_schema(
                r#"
                [dependency-groups]
                dev = [
                    "█pytest>=8.3.3",
                ]
                "#,
            ) -> Ok({
                "Keys": "dependency-groups.dev[0]",
                "Value": "String"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn pyproject_tool_poetry_exclude_tests(
                r#"
                [tool.poetry]
                exclude = [
                    "█tests",
                ]
                "#,
            ) -> Ok({
                "Keys": "tool.poetry.exclude[0]",
                "Value": "String"
            });
        );
    }

    mod non_schema {
        use super::*;

        test_hover_keys_value!(
            #[tokio::test]
            async fn variable_placeholder(
                r#"
                [[dependencies.${mod_id}]]
                modId="create█"
                "#,
            ) -> Ok({
                "Keys": "dependencies.${mod_id}[0].modId",
                "Value": "String"
            });
        );
    }

    #[macro_export]
    macro_rules! test_hover_keys_value {
        (#[tokio::test] async fn $name:ident(
            $source:expr $(, $arg:expr )* $(,)?
        ) -> Ok({
            "Keys": $keys:expr,
            "Value": $value_type:expr$(,)?
        });) => {
            #[tokio::test]
            async fn $name() -> Result<(), Box<dyn std::error::Error>> {
                use tombi_lsp::Backend;
                use std::io::Write;
                use tower_lsp::{
                    lsp_types::{
                        TextDocumentIdentifier, Url, WorkDoneProgressParams, DidOpenTextDocumentParams,
                        TextDocumentItem,
                    },
                    LspService,
                };
                use tombi_lsp::handler::handle_did_open;
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

                let current_dir = std::env::current_dir().expect("failed to get current directory");
                let temp_dir = if let Some(source_path) = config.source_file_path.as_ref() {
                    source_path.parent().ok_or("failed to get parent directory")?
                } else {
                    current_dir.as_path()
                };
                let Ok(temp_file) = tempfile::NamedTempFile::with_suffix_in(
                    ".toml",
                    temp_dir,
                ) else {
                    return Err("failed to create a temporary file for the test data".into());
                };

                let mut toml_text = textwrap::dedent($source).trim().to_string();

                let Some(index) = toml_text
                    .as_str()
                    .find("█")
                        else {
                        return Err("failed to find hover position marker (█) in the test data".into())
                        };

                toml_text.remove(index);
                if temp_file.as_file().write_all(toml_text.as_bytes()).is_err() {
                    return Err("failed to write to temporary file".into());
                };

                let line_index =
                tombi_text::LineIndex::new(&toml_text, tombi_text::EncodingKind::Utf16);

                let Ok(toml_file_url) = Url::from_file_path(temp_file.path()) else {
                    return Err("failed to convert temporary file path to URL".into());
                };

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

                let Ok(Some(tombi_lsp::HoverContent::Value(hover_content) | tombi_lsp::HoverContent::DirectiveContent(hover_content))) = tombi_lsp::handler::handle_hover(
                    &backend,
                    tower_lsp::lsp_types::HoverParams {
                        text_document_position_params: tower_lsp::lsp_types::TextDocumentPositionParams {
                            text_document: TextDocumentIdentifier {
                                uri: toml_file_url,
                            },
                            position: (tombi_text::Position::default()
                                + tombi_text::RelativePosition::of(&toml_text[..index]))
                            .into_lsp(&line_index),
                        },
                        work_done_progress_params: WorkDoneProgressParams::default(),
                    },
                )
                .await else {
                    return Err("failed to handle hover content".into());
                };

                tracing::debug!("hover_content: {:#?}", hover_content);

                if config.schema_file_path.is_some() {
                    assert!(hover_content.schema_uri.is_some(), "The hover target is not defined in the schema.");
                } else {
                    assert!(hover_content.schema_uri.is_none(), "The hover target is defined in the schema.");
                }

                pretty_assertions::assert_eq!(hover_content.accessors.to_string(), $keys, "Keys are not equal");
                pretty_assertions::assert_eq!(hover_content.value_type.to_string(), $value_type, "Value type are not equal");

                Ok(())
            }
        }
    }
}
