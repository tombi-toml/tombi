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
                tombi_schema_path(),
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
            async fn tombi_schema_catalog_path(
                r#"
                [schema.catalog]
                path = "█https://json.schemastore.org/api/json/catalog.json"
                "#,
                tombi_schema_path()
            ) -> Ok({
                "Keys": "schema.catalog.path",
                "Value": "(String | Array)?"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn tombi_schema_catalog_path_without_schema(
                r#"
                [schema.catalog]
                path = "█https://json.schemastore.org/api/json/catalog.json"
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
                tombi_schema_path(),
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
                tombi_schema_path(),
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
                tombi_schema_path(),
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
                tombi_schema_path(),
            ) -> Ok({
                "Keys": "schemas[0].include[0]",
                "Value": "String"
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
                cargo_schema_path(),
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
                cargo_schema_path(),
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
                cargo_schema_path(),
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
                cargo_schema_path(),
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
                cargo_schema_path(),
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
                cargo_schema_path(),
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
                cargo_schema_path(),
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
                cargo_schema_path(),
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
                cargo_schema_path(),
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
                cargo_schema_path(),
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
                cargo_schema_path(),
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
                pyproject_schema_path(),
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
                pyproject_schema_path(),
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

    #[macro_export]
    macro_rules! test_hover_keys_value {
        (#[tokio::test] async fn $name:ident(
            $source:expr,
            $schema_file_path:expr$(,)?
        ) -> Ok({
            "Keys": $keys:expr,
            "Value": $value_type:expr$(,)?
        });) => {
            test_hover_keys_value!(#[tokio::test] async fn __$name($source, Some($schema_file_path)) -> Ok({
                "Keys": $keys,
                "Value": $value_type
            }););
        };
        (#[tokio::test] async fn $name:ident(
            $source:expr,
        ) -> Ok({
            "Keys": $keys:expr,
            "Value": $value_type:expr$(,)?
        });) => {
            test_hover_keys_value!(#[tokio::test] async fn __$name($source, Option::<std::path::PathBuf>::None ) -> Ok({
                "Keys": $keys,
                "Value": $value_type
            }););
        };
        (#[tokio::test] async fn __$name:ident(
            $source:expr,
            $schema_file_path:expr$(,)?
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

                tombi_test_lib::init_tracing();

                let (service, _) = LspService::new(|client| Backend::new(client, &tombi_lsp::backend::Options::default()));

                let backend = service.inner();

                if let Some(schema_file_path) = &$schema_file_path {
                    let schema_file_url = tombi_schema_store::SchemaUrl::from_file_path(schema_file_path).expect(
                        format!(
                            "failed to convert schema path to URL: {}",
                            schema_file_path.display()
                        )
                        .as_str(),
                    );
                    backend
                        .schema_store
                        .load_schemas(
                            &[
                                tombi_config::Schema::Root(
                                    tombi_config::RootSchema {
                                        toml_version: None,
                                        path: schema_file_url.to_string(),
                                        include: vec!["*.toml".to_string()],
                                    }
                                )
                            ],
                            None
                        )
                        .await;
                }

                let Ok(temp_file) = tempfile::NamedTempFile::with_suffix_in(
                        ".toml",
                        std::env::current_dir().expect("failed to get current directory"),
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

                let Ok(Some(hover_content)) = tombi_lsp::handler::handle_hover(
                    &backend,
                    tower_lsp::lsp_types::HoverParams {
                        text_document_position_params: tower_lsp::lsp_types::TextDocumentPositionParams {
                            text_document: TextDocumentIdentifier {
                                uri: toml_file_url,
                            },
                            position: (tombi_text::Position::default()
                                + tombi_text::RelativePosition::of(&toml_text[..index]))
                            .into(),
                        },
                        work_done_progress_params: WorkDoneProgressParams::default(),
                    },
                )
                .await else {
                    return Err("failed to handle hover".into());
                };

                tracing::debug!("hover_content: {:#?}", hover_content);

                if $schema_file_path.is_some() {
                    assert!(hover_content.schema_url.is_some(), "The hover target is not defined in the schema.");
                } else {
                    assert!(hover_content.schema_url.is_none(), "The hover target is defined in the schema.");
                }

                pretty_assertions::assert_eq!(hover_content.accessors.to_string(), $keys, "Keys are not equal");
                pretty_assertions::assert_eq!(hover_content.value_type.to_string(), $value_type, "Value type are not equal");

                Ok(())
            }
        }
    }
}
