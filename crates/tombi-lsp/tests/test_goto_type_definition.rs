mod goto_type_definition_tests {
    use super::*;

    mod tombi_schema {
        use super::*;
        use tombi_test_lib::tombi_schema_path;

        test_goto_type_definition!(
            #[tokio::test]
            async fn tombi_toml_version(
                r#"
                toml-version = "█v1.0.0"
                "#,
                tombi_schema_path(),
            ) -> Ok(tombi_schema_path());
        );

        test_goto_type_definition!(
            #[tokio::test]
            async fn tombi_schema_catalog_path(
                r#"
                [schema.catalog]
                path = "█https://www.schemastore.org/api/json/catalog.json"
                "#,
                tombi_schema_path(),
            ) -> Ok(tombi_schema_path());
        );

        test_goto_type_definition!(
            #[tokio::test]
            async fn tombi_schemas(
                r#"
                [[schemas█]]
                "#,
                tombi_schema_path(),
            ) -> Ok(tombi_schema_path());
        );
    }

    mod cargo_schema {
        use super::*;
        use tombi_test_lib::cargo_schema_path;

        test_goto_type_definition!(
            #[tokio::test]
            async fn cargo_package_name(
                r#"
                [package]
                name█ = "tombi"
                "#,
                cargo_schema_path(),
            ) -> Ok(cargo_schema_path());
        );

        test_goto_type_definition!(
            #[tokio::test]
            async fn cargo_package_readme(
                r#"
                [package]
                readme = "█README.md"
                "#,
                cargo_schema_path(),
            ) -> Ok(cargo_schema_path());
        );

        test_goto_type_definition!(
            #[tokio::test]
            async fn cargo_dependencies_key(
                r#"
                [dependencies]
                serde█ = { workspace = true }
                "#,
                cargo_schema_path(),
            ) -> Ok(cargo_schema_path());
        );

        test_goto_type_definition!(
            #[tokio::test]
            async fn cargo_profile_release_strip_debuginfo(
                r#"
                [profile.release]
                strip = "debuginfo█"
                "#,
                cargo_schema_path(),
            ) -> Ok(cargo_schema_path());
        );
    }

    mod pyproject_schema {
        use super::*;

        use tombi_test_lib::pyproject_schema_path;

        test_goto_type_definition!(
            #[tokio::test]
            async fn pyproject_project_readme(
                r#"
                [project]
                readme = "█1.0.0"
                "#,
                pyproject_schema_path(),
            ) -> Ok(pyproject_schema_path());
        );

        test_goto_type_definition!(
            #[tokio::test]
            async fn pyproject_dependency_groups(
                r#"
                [dependency-groups]
                dev = [
                    "█pytest>=8.3.3",
                ]
                "#,
                pyproject_schema_path(),
            ) -> Ok(pyproject_schema_path());
        );

        test_goto_type_definition!(
            #[tokio::test]
            async fn pyproject_tool_taskipy_tasks_format(
                r#"
                [tool.taskipy.tasks]
                format█ = "ruff"
                "#,
                pyproject_schema_path(),
            ) -> Ok("https://www.schemastore.org/partial-taskipy.json");
        );

        test_goto_type_definition!(
            #[tokio::test]
            async fn pyproject_tombi_document_directive_toml_version(
                r#"
                #:tombi toml-version█ = "v1.0.0"
                [project]
                name = "tombi"
                "#,
                pyproject_schema_path(),
            ) -> Ok("tombi://json.tombi.dev/tombi-document-directive.json");
        );
    }

    mod type_test_schema {
        use super::*;

        use tombi_test_lib::type_test_schema_path;

        test_goto_type_definition!(
            #[tokio::test]
            async fn type_test_tombi_document_directive(
                r#"
                #:tombi schema.strict█ = true

                [table]
                integer = 42
                "#,
                type_test_schema_path(),
            ) -> Ok("tombi://json.tombi.dev/tombi-document-directive.json");
        );

        test_goto_type_definition!(
            #[tokio::test]
            async fn type_test_tombi_document_directive_in_integer_scope(
                r#"
                #:tombi schema.strict█ = true
                integer = 42
                "#,
                type_test_schema_path(),
            ) -> Ok("tombi://json.tombi.dev/tombi-document-directive.json");
        );

        test_goto_type_definition!(
            #[tokio::test]
            async fn type_test_tombi_document_directive_in_table_scope(
                r#"
                #:tombi schema.strict█ = true

                [table]
                integer = 42
                "#,
                type_test_schema_path(),
            ) -> Ok("tombi://json.tombi.dev/tombi-document-directive.json");
        );

        test_goto_type_definition!(
            #[tokio::test]
            async fn type_test_tombi_root_table_directive(
                r#"
                # tombi: lint.rules.const-value.disabled█ = true

                key = "value"
                "#,
                type_test_schema_path(),
            ) -> Ok("tombi://json.tombi.dev/tombi-root-table-directive.json");
        );

        test_goto_type_definition!(
            #[tokio::test]
            async fn type_test_tombi_root_table_directive_at_end(
                r#"
                key = "value"

                # tombi: lint.rules.const-value.disabled█ = true
                "#,
                type_test_schema_path(),
            ) -> Ok("tombi://json.tombi.dev/tombi-root-table-directive.json");
        );

        test_goto_type_definition!(
            #[tokio::test]
            async fn type_test_tombi_key_string_directive(
                r#"
                # tombi: lint.rules.key-empty█ = "off"
                string = "string"
                "#,
                type_test_schema_path(),
            ) -> Ok("tombi://json.tombi.dev/tombi-key-string-directive.json");
        );

        test_goto_type_definition!(
            #[tokio::test]
            async fn type_test_tombi_comment_directive_array_newline_string(
                r#"
                # tombi: lint.rules.array-min-items█ = "off"
                array = [

                  "string"
                ]
                "#,
                type_test_schema_path(),
            ) -> Ok("tombi://json.tombi.dev/tombi-key-array-directive.json");
        );

        test_goto_type_definition!(
            #[tokio::test]
            async fn type_test_tombi_key_array_comment_directive_newline_string(
                r#"
                array = [
                  # tombi: lint.rules.array-min-items█ = "off"

                  "string"
                ]
                "#,
                type_test_schema_path(),
            ) -> Ok("tombi://json.tombi.dev/tombi-array-directive.json");
        );

        test_goto_type_definition!(
            #[tokio::test]
            async fn type_test_tombi_key_array_comment_directive_string(
                r#"
                array = [
                  # tombi: lint.rules.string-min-length█ = "off"
                  "string"
                ]
                "#,
                type_test_schema_path(),
            ) -> Ok("tombi://json.tombi.dev/tombi-string-directive.json");
        );

        test_goto_type_definition!(
            #[tokio::test]
            async fn type_test_tombi_key_array_string_directive(
                r#"
                array = [
                  "string" # tombi: lint.rules.string-min-length█ = "off"
                ]
                "#,
                type_test_schema_path(),
            ) -> Ok("tombi://json.tombi.dev/tombi-string-directive.json");
        );

        test_goto_type_definition!(
            #[tokio::test]
            async fn type_test_tombi_key_array_string_comma_directive(
                r#"
                array = [
                  "string", # tombi: lint.rules.string-min-length█ = "off"
                ]
                "#,
                type_test_schema_path(),
            ) -> Ok("tombi://json.tombi.dev/tombi-string-directive.json");
        );

        test_goto_type_definition!(
            #[tokio::test]
            async fn type_test_tombi_key_array_string_newline_comma_directive(
                r#"
                array = [
                  "string"
                  , # tombi: lint.rules.string-min-length█ = "off"
                ]
                "#,
                type_test_schema_path(),
            ) -> Ok("tombi://json.tombi.dev/tombi-string-directive.json");
        );

        test_goto_type_definition!(
            #[tokio::test]
            async fn type_test_tombi_key_array_string_comma_newline_bracket_directive(
                r#"
                array = [
                  "string",
                  # tombi: lint.rules.array-min-items█ = "off"
                ]
                "#,
                type_test_schema_path(),
            ) -> Ok("tombi://json.tombi.dev/tombi-array-directive.json");
        );

        test_goto_type_definition!(
            #[tokio::test]
            async fn type_test_tombi_key_array_string_newline_comma_bracket_directive(
                r#"
                array = [
                  "string"
                  ,
                ] # tombi: lint.rules.array-min-items█ = "off"
                "#,
                type_test_schema_path(),
            ) -> Ok("tombi://json.tombi.dev/tombi-key-array-directive.json");
        );

        test_goto_type_definition!(
            #[tokio::test]
            async fn type_test_tombi_key_inline_table_directive(
                r#"
                inline-table = { key = "value", } # tombi: lint.rules.table-min-properties█ = "off"
                "#,
                type_test_schema_path(),
            ) -> Ok("tombi://json.tombi.dev/tombi-key-inline-table-directive.json");
        );

        test_goto_type_definition!(
            #[tokio::test]
            async fn type_test_tombi_key_table_directive(
                r#"
                # tombi: lint.rules.const-value.disabled█ = true
                [table]
                "#,
                type_test_schema_path(),
            ) -> Ok("tombi://json.tombi.dev/tombi-key-table-directive.json");
        );

        test_goto_type_definition!(
            #[tokio::test]
            async fn type_test_tombi_table_directive(
                r#"
                [table]
                # tombi: lint.rules.const-value.disabled█ = true
                "#,
                type_test_schema_path(),
            ) -> Ok("tombi://json.tombi.dev/tombi-table-directive.json");
        );

        test_goto_type_definition!(
            #[tokio::test]
            async fn type_test_tombi_key_array_of_table_directive(
                r#"
                # tombi: lint.rules.const-value.disabled█ = true
                [[array]]
                "#,
                type_test_schema_path(),
            ) -> Ok("tombi://json.tombi.dev/tombi-key-array-of-table-directive.json");
        );

        test_goto_type_definition!(
            #[tokio::test]
            async fn type_test_tombi_table_key_array_of_table_directive(
                r#"
                [[array]] # tombi: lint.rules.const-value.disabled█ = true
                "#,
                type_test_schema_path(),
            ) -> Ok("tombi://json.tombi.dev/tombi-key-array-of-table-directive.json");
        );

        test_goto_type_definition!(
            #[tokio::test]
            async fn type_test_tombi_array_of_table_directive(
                r#"
                [[array]]
                # tombi: lint.rules.const-value.disabled█ = true
                "#,
                type_test_schema_path(),
            ) -> Ok("tombi://json.tombi.dev/tombi-table-directive.json");
        );

        test_goto_type_definition!(
            #[tokio::test]
            async fn key_eq_value_with_comment_directive(
                r#"
                key = "value"  # tombi: lint.rules.string-pattern.disabled█ = true
                "#,
                type_test_schema_path(),
            ) -> Ok("tombi://json.tombi.dev/tombi-key-string-directive.json");
        );

        test_goto_type_definition!(
            #[tokio::test]
            async fn key1_key2_eq_value_with_comment_directive(
                r#"
                key1.key2 = "value"  # tombi: lint.rules.string-pattern.disabled█ = true
                "#,
                type_test_schema_path(),
            ) -> Ok("tombi://json.tombi.dev/tombi-key-string-directive.json");
        );
    }

    #[macro_export]
    macro_rules! test_goto_type_definition {
        (#[tokio::test] async fn $name:ident(
            $source:expr,
            $schema_file_path:expr$(,)?
        ) -> Ok(_)$(;)?) => {
            test_goto_type_definition!(
                #[tokio::test]
                async fn $name(
                    $source,
                    $schema_file_path,
                ) -> Ok($schema_file_path);
            );
        };

        (#[tokio::test] async fn $name:ident(
            $source:expr,
            $schema_file_path:expr$(,)?
        ) -> Ok($expected_schema_path:expr)$(;)?) => {
            test_goto_type_definition!(
                #[tokio::test]
                async fn _$name(
                    $source,
                    Some($schema_file_path),
                ) -> Ok($expected_schema_path);
            );
        };

        (#[tokio::test] async fn _$name:ident(
            $source:expr,
            $schema_file_path:expr,
        ) -> Ok($expected_schema_path:expr);) => {
            #[tokio::test]
            async fn $name() -> Result<(), Box<dyn std::error::Error>> {
                use std::io::Write;
                use itertools::Itertools;
                use tombi_lsp::handler::{handle_did_open, handle_goto_type_definition};
                use tombi_lsp::Backend;
                use tower_lsp::{
                    lsp_types::{
                        DidOpenTextDocumentParams, PartialResultParams, TextDocumentIdentifier,
                        TextDocumentItem, TextDocumentPositionParams, Url, WorkDoneProgressParams,
                    },
                    LspService,
                };
                use tombi_text::IntoLsp;

                tombi_test_lib::init_tracing();

                let (service, _) = LspService::new(|client| {
                    Backend::new(client, &tombi_lsp::backend::Options::default())
                });

                let backend = service.inner();

                // Load schema file
                if let Some(schema_file_path) = $schema_file_path {
                    let schema_file_url =
                        tombi_schema_store::SchemaUri::from_file_path(&schema_file_path).expect(
                            format!(
                                "failed to convert schema path to URL: {}",
                                schema_file_path.display()
                            )
                            .as_str(),
                        );
                    backend
                        .config_manager
                        .load_config_schemas(
                            &[tombi_config::Schema::Root(tombi_config::RootSchema {
                                toml_version: None,
                                path: schema_file_url.to_string(),
                                include: vec!["*.toml".to_string()],
                            })],
                            None,
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

                let Some(index) = toml_text.as_str().find("█") else {
                    return Err("failed to find position marker (█) in the test data".into());
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

                let params = tower_lsp::lsp_types::request::GotoTypeDefinitionParams {
                    text_document_position_params: TextDocumentPositionParams {
                        text_document: TextDocumentIdentifier { uri: toml_file_url },
                        position: (tombi_text::Position::default()
                            + tombi_text::RelativePosition::of(&toml_text[..index]))
                        .into_lsp(&line_index),
                    },
                    work_done_progress_params: WorkDoneProgressParams::default(),
                    partial_result_params: PartialResultParams::default(),
                };

                let Ok(result) = handle_goto_type_definition(&backend, params).await else {
                    return Err("failed to handle goto_type_definition".into());
                };

                tracing::debug!("goto_type_definition result: {:#?}", result);

                let expected_path = $expected_schema_path.to_owned();

                trait IntoPathString {
                    fn into_path_string(self) -> String;
                }

                impl IntoPathString for String {
                    fn into_path_string(self) -> String {
                        self
                    }
                }

                impl IntoPathString for std::path::PathBuf {
                    fn into_path_string(self) -> String {
                        self.to_string_lossy().to_string()
                    }
                }

                match result {
                    Some(definition_links) => {
                        let definition_urls = definition_links.into_iter().map(|mut link| {
                                match link.uri.scheme() {
                                    "file" => link.uri.to_file_path().unwrap().into_path_string(),
                                    "tombi" | "http" | "https" => {
                                        link.uri.set_fragment(None);
                                        link.uri.to_string()
                                    },
                                    _ => panic!("unexpected schema: {}", link.uri.scheme()),
                                }
                            }).collect_vec();

                        tracing::debug!("definition_urls: {:#?}", definition_urls);

                        pretty_assertions::assert_eq!(
                            definition_urls,
                            vec![expected_path.into_path_string()],
                        );},
                    None => {
                        panic!("No type definition link was returned, but expected path: {:?}", expected_path);
                    }
                }

                Ok(())
            }
        };
    }
}
