use tombi_test_lib::{
    today_local_date, today_local_date_time, today_local_time, today_offset_date_time,
};

struct Select<T>(T);
mod completion_edit {
    use super::*;

    mod tombi_schema {
        use tombi_test_lib::tombi_schema_path;

        use super::*;

        test_completion_edit! {
            #[tokio::test]
            async fn tombi_lsp_completion_dot(
                r#"
                [lsp]
                completion.█
                "#,
                Select("enabled"),
                tombi_schema_path(),
            ) -> Ok(
                r#"
                [lsp]
                completion.enabled
                "#
            );
        }

        test_completion_edit! {
            #[tokio::test]
            async fn tombi_lsp_completion_equal(
                r#"
                [lsp]
                completion=█
                "#,
                Select("enabled"),
                tombi_schema_path(),
            ) -> Ok(
                r#"
                [lsp]
                completion = { enabled$1 }$0
                "#
            );
        }
    }

    mod cargo_schema {
        use tombi_test_lib::cargo_schema_path;

        use super::*;

        test_completion_edit! {
            #[tokio::test]
            async fn cargo_package_version(
                r#"
                [package]
                version=█
                "#,
                Select("\"0.1.0\""),
                cargo_schema_path(),
            ) -> Ok(
                r#"
                [package]
                version = "0.1.0"
                "#
            );
        }

        test_completion_edit! {
            #[tokio::test]
            async fn cargo_dependencies_serde_dot_work(
                r#"
                [dependencies]
                serde.work█
                "#,
                Select("workspace"),
                cargo_schema_path(),
            ) -> Ok(
                r#"
                [dependencies]
                serde.workspace
                "#
            );
        }

        test_completion_edit! {
            #[tokio::test]
            async fn cargo_dependencies_serde_eq_work(
                r#"
                [dependencies]
                serde=work█
                "#,
                Select("workspace"),
                cargo_schema_path(),
            ) -> Ok(
                r#"
                [dependencies]
                serde = { workspace$1 }$0
                "#
            );
        }

        test_completion_edit! {
            #[tokio::test]
            async fn cargo_dependencies_serde_workspace_dot(
                r#"
                [dependencies]
                serde = { workspace.█ }
                "#,
                Select("true"),
                cargo_schema_path(),
            ) -> Ok(
                r#"
                [dependencies]
                serde = { workspace = true }
                "#
            );
        }
    }

    mod pyproject_schema {
        use tombi_test_lib::pyproject_schema_path;

        use super::*;

        test_completion_edit! {
            #[tokio::test]
            async fn pyproject_project_authors_dot(
                r#"
                [project]
                authors.█
                "#,
                Select("[]"),
                pyproject_schema_path(),
            ) -> Ok(
                r#"
                [project]
                authors = [$1]$0
                "#
            );
        }

        test_completion_edit! {
            #[tokio::test]
            async fn pyproject_project_authors_equal(
                r#"
                [project]
                authors=█
                "#,
                Select("[]"),
                pyproject_schema_path(),
            ) -> Ok(
                r#"
                [project]
                authors = [$1]$0
                "#
            );
        }

        test_completion_edit! {
            #[tokio::test]
            async fn pyproject_dependency_groups_dev_eq_array_select_single_quote(
                r#"
                [dependency-groups]
                dev=[█]
                "#,
                Select("''"),
                pyproject_schema_path(),
            ) -> Ok(
                r#"
                [dependency-groups]
                dev=['$1'$0]
                "#
            );
        }

        test_completion_edit! {
            #[tokio::test]
            async fn pyproject_dependency_groups_dev_eq_array_select_include_group(
                r#"
                [dependency-groups]
                dev=[█]
                "#,
                Select("include-group"),
                pyproject_schema_path(),
            ) -> Ok(
                r#"
                [dependency-groups]
                dev=[{ include-group$1 }$0]
                "#
            );
        }

        test_completion_edit! {
            #[tokio::test]
            async fn pyproject_tool_mytool_key_select_dot(
                r#"
                [tool.mytool]
                key█
                "#,
                Select("."),
                pyproject_schema_path(),
            ) -> Ok(
                r#"
                [tool.mytool]
                key.
                "#
            );
        }

        test_completion_edit! {
            #[tokio::test]
            async fn pyproject_tool_mytool_key_select_equal(
                r#"
                [tool.mytool]
                key█
                "#,
                Select("="),
                pyproject_schema_path(),
            ) -> Ok(
                r#"
                [tool.mytool]
                key=
                "#
            );
        }
    }

    mod without_schema {
        use super::*;

        test_completion_edit! {
            #[tokio::test]
            async fn key_dot_select_true(
                "key.█",
                Select("true"),
            ) -> Ok(
                "key = true"
            );
        }

        test_completion_edit! {
            #[tokio::test]
            async fn key_dot_select_false(
                "key.█",
                Select("false"),
            ) -> Ok(
                "key = false"
            );
        }

        test_completion_edit! {
            #[tokio::test]
            async fn key_dot_select_integer(
                "key.█",
                Select("42"),
            ) -> Ok(
                "key = ${0:42}"
            );
        }

        test_completion_edit! {
            #[tokio::test]
            async fn key_dot_select_float(
                "key.█",
                Select("3.14"),
            ) -> Ok(
                "key = ${0:3.14}"
            );
        }

        test_completion_edit! {
            #[tokio::test]
            async fn key_dot_select_basic_string(
                "key.█",
                Select("\"\""),
            ) -> Ok(
                "key = \"$1\"$0"
            );
        }

        test_completion_edit! {
            #[tokio::test]
            async fn key_dot_select_today_offset_date_time(
                "key.█",
                Select(today_offset_date_time()),
            ) -> Ok(
                &format!("key = ${{0:{}}}", today_offset_date_time())
            );
        }

        test_completion_edit! {
            #[tokio::test]
            async fn key_dot_select_today_local_date_time(
                "key.█",
                Select(today_local_date_time()),
            ) -> Ok(
                &format!("key = ${{0:{}}}", today_local_date_time())
            );
        }

        test_completion_edit! {
            #[tokio::test]
            async fn key_dot_select_today_local_date(
                "key.█",
                Select(today_local_date()),
            ) -> Ok(
                &format!("key = ${{0:{}}}", today_local_date())
            );
        }

        test_completion_edit! {
            #[tokio::test]
            async fn key_dot_select_today_local_time(
                "key.█",
                Select(today_local_time()),
            ) -> Ok(
                &format!("key = ${{0:{}}}", today_local_time())
            );
        }

        test_completion_edit! {
            #[tokio::test]
            async fn key_dot_select_array(
                "key.█",
                Select("[]"),
            ) -> Ok(
                "key = [$1]$0"
            );
        }

        test_completion_edit! {
            #[tokio::test]
            async fn key_equal_select_true(
                "key=█",
                Select("true"),
            ) -> Ok(
                "key = true"
            );
        }

        test_completion_edit! {
            #[tokio::test]
            async fn key_equal_select_false(
                "key=█",
                Select("false"),
            ) -> Ok(
                "key = false"
            );
        }

        test_completion_edit! {
            #[tokio::test]
            async fn key_equal_select_integer(
                "key=█",
                Select("42"),
            ) -> Ok(
                "key = ${0:42}"
            );
        }

        test_completion_edit! {
            #[tokio::test]
            async fn key_equal_select_float(
                "key=█",
                Select("3.14"),
            ) -> Ok(
                "key = ${0:3.14}"
            );
        }

        test_completion_edit! {
            #[tokio::test]
            async fn key_equal_select_basic_string(
                "key=█",
                Select("\"\""),
            ) -> Ok(
                "key = \"$1\"$0"
            );
        }

        test_completion_edit! {
            #[tokio::test]
            async fn key_equal_select_today_offset_date_time(
                "key=█",
                Select(today_offset_date_time()),
            ) -> Ok(
                &format!("key = ${{0:{}}}", today_offset_date_time())
            );
        }

        test_completion_edit! {
            #[tokio::test]
            async fn key_equal_select_today_local_date_time(
                "key=█",
                Select(today_local_date_time()),
            ) -> Ok(
                &format!("key = ${{0:{}}}", today_local_date_time())
            );
        }

        test_completion_edit! {
            #[tokio::test]
            async fn key_equal_select_today_local_date(
                "key=█",
                Select(today_local_date()),
            ) -> Ok(
                &format!("key = ${{0:{}}}", today_local_date())
            );
        }

        test_completion_edit! {
            #[tokio::test]
            async fn key_equal_select_today_local_time(
                "key=█",
                Select(today_local_time()),
            ) -> Ok(
                &format!("key = ${{0:{}}}", today_local_time())
            );
        }

        test_completion_edit! {
            #[tokio::test]
            async fn key_equal_select_array(
                "key=█",
                Select("[]"),
            ) -> Ok(
                "key = [$1]$0"
            );
        }

        test_completion_edit! {
            #[tokio::test]
            async fn key_dot_abc(
                "key.abc█",
                Select("$key"),
            ) -> Ok(
                "key.abc"
            );
        }

        test_completion_edit! {
            #[tokio::test]
            async fn key_equal_abc(
                "key=abc█",
                Select("$key"),
            ) -> Ok(
                "key = { abc$1 }$0"
            );
        }

        test_completion_edit! {
            #[tokio::test]
            async fn key_equal_array(
                "key=[█]",
                Select("$key"),
            ) -> Ok(
                "key=[${0:key}]"
            );
        }

        test_completion_edit! {
            #[tokio::test]
            async fn key_equal_array_abc(
                "key=[abc█]",
                Select("$key"),
            ) -> Ok(
                "key=[{ abc$1 }$0]"
            );
        }

        test_completion_edit! {
            #[tokio::test]
            async fn key_equal_array_bra_abc_dot_def_ket(
                "key=[{ abc.def█ }]",
                Select("$key"),
            ) -> Ok(
                "key=[{ abc.def }]"
            );
        }
    }

    #[macro_export]
    macro_rules! test_completion_edit {
        (
            #[tokio::test]
            async fn $name:ident(
                $source:expr,
                $select:expr,
                $schema_file_path:expr$(,)?
            ) -> Ok($expected:expr);
        ) => {
            test_completion_edit! {
                #[tokio::test]
                async fn _$name(
                    $source,
                    $select,
                    Some($schema_file_path),
                ) -> Ok($expected);
            }
        };

        (
            #[tokio::test]
            async fn $name:ident(
                $source:expr,
                $select:expr$(,)?
            ) -> Ok($expected:expr);
        ) => {
            test_completion_edit! {
                #[tokio::test]
                async fn _$name(
                    $source,
                    $select,
                    Option::<std::path::PathBuf>::None,
                ) -> Ok($expected);
            }
        };

        (
            #[tokio::test]
            async fn _$name:ident(
                $source:expr,
                $select:expr,
                $schema_file_path:expr$(,)?
            ) -> Ok($expected:expr);
        ) => {
            #[tokio::test]
            async fn $name() -> Result<(), Box<dyn std::error::Error>> {
                use tombi_lsp::handler::handle_did_open;
                use tombi_lsp::Backend;
                use std::io::Write;
                use tower_lsp::{
                    lsp_types::{
                        CompletionParams, DidOpenTextDocumentParams, PartialResultParams,
                        TextDocumentIdentifier, TextDocumentItem, TextDocumentPositionParams,
                        Url, WorkDoneProgressParams,
                    },
                    LspService,
                };

                tombi_test_lib::init_tracing();

                let (service, _) = LspService::new(|client| Backend::new(client, &tombi_lsp::backend::Options::default()));
                let backend = service.inner();

                if let Some(schema_file_path) = $schema_file_path.as_ref() {
                    let schema_url = tombi_schema_store::SchemaUrl::from_file_path(schema_file_path)
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
                            &[
                                tombi_config::Schema::Root(tombi_config::RootSchema {
                                    toml_version: None,
                                    path: schema_url.to_string(),
                                    include: vec!["*.toml".to_string()],
                                }),
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

                let Some(index) = toml_text.as_str().find("█") else {
                    return Err(
                        "failed to find completion position marker (█) in the test data".into()
                    );
                };

                toml_text.remove(index);
                if temp_file.as_file().write_all(toml_text.as_bytes()).is_err() {
                    return Err(
                        "failed to write test data to the temporary file, which is used as a text document"
                            .into(),
                    );
                }

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

                let Ok(Some(completion_contents)) = tombi_lsp::handler::handle_completion(
                    &backend,
                    CompletionParams {
                        text_document_position: TextDocumentPositionParams {
                            text_document: TextDocumentIdentifier {
                                uri: toml_file_url,
                            },
                            position: (tombi_text::Position::default()
                                + tombi_text::RelativePosition::of(&toml_text[..index]))
                            .into(),
                        },
                        work_done_progress_params: WorkDoneProgressParams::default(),
                        partial_result_params: PartialResultParams {
                            partial_result_token: None,
                        },
                        context: None,
                    },
                )
                .await
                else {
                    return Err("failed to handle completion".into());
                };

                let selected = $select.0;
                let selected: &str = selected.as_ref();

                let Some(completion_content) = completion_contents
                    .clone()
                    .into_iter()
                    .find(|content| content.label == selected)
                else {
                    return Err(
                        format!(
                            "failed to find the selected completion item \"{}\" in [{}]",
                            selected,
                            completion_contents
                                .iter()
                                .map(|content| content.label.as_str())
                                .collect::<Vec<&str>>()
                                .join(", ")
                        )
                        .into(),
                    );
                };

                let Some(completion_edit) = completion_content.edit else {
                    return Err(format!(
                        "failed to get the edit of the selected completion item {}",
                        selected
                    )
                    .into());
                };

                let mut new_text = "".to_string();
                match completion_edit.text_edit {
                    tower_lsp::lsp_types::CompletionTextEdit::Edit(edit) => {
                        for (index, line) in toml_text.split('\n').enumerate() {
                            if index != 0 {
                                new_text.push('\n');
                            }
                            if edit.range.start.line as usize == index {
                                new_text.push_str(&line[..edit.range.start.character as usize]);
                                new_text.push_str(&edit.new_text);
                                new_text.push_str(&line[edit.range.end.character as usize..]);
                            } else {
                                new_text.push_str(line);
                            }
                        }
                    }
                    _ => {
                        return Err("failed to get the text edit of the selected completion item".into());
                    }
                }

                if let Some(text_edits) = completion_edit.additional_text_edits {
                    for text_edit in text_edits {
                        let mut additional_new_text = "".to_string();
                        for (index, line) in new_text.split('\n').enumerate() {
                            if index != 0 {
                                additional_new_text.push('\n');
                            }
                            if text_edit.range.start.line as usize == index {
                                additional_new_text
                                    .push_str(&line[..text_edit.range.start.character as usize]);
                                additional_new_text.push_str(&text_edit.new_text);
                                additional_new_text
                                    .push_str(&line[text_edit.range.end.character as usize..]);
                            } else {
                                additional_new_text.push_str(line);
                            }
                        }
                        new_text = additional_new_text;
                    }
                }

                pretty_assertions::assert_eq!(new_text, textwrap::dedent($expected).trim());

                Ok(())
            }
        };
    }
}
