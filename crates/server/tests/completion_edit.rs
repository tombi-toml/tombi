use test_lib::{cargo_schema_path, pyproject_schema_path, tombi_schema_path};

struct Select(&'static str);

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
                None,
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
            use schema_store::JsonCatalogSchema;
            use server::handler::handle_did_open;
            use server::Backend;
            use std::io::Write;
            use tower_lsp::{
                lsp_types::{
                    DidOpenTextDocumentParams, PartialResultParams, TextDocumentIdentifier,
                    TextDocumentItem, Url, WorkDoneProgressParams,
                },
                LspService,
            };

            let (service, _) = LspService::new(|client| Backend::new(client));

            let backend = service.inner();

            if let Some(schema_file_path) = $schema_file_path.as_ref() {
                let schema_url = Url::from_file_path(schema_file_path).expect(
                    format!(
                        "failed to convert schema path to URL: {}",
                        schema_file_path.display()
                    )
                    .as_str(),
                );
                backend
                    .schema_store
                    .add_catalog(JsonCatalogSchema {
                        name: "test_schema".to_string(),
                        description: "schema for testing".to_string(),
                        file_match: vec!["*.toml".to_string()],
                        url: schema_url.clone(),
                    })
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
                    return Err("failed to find completion position marker (█) in the test data".into())
                };

            toml_text.remove(index);
            if temp_file.as_file().write_all(toml_text.as_bytes()).is_err() {
                return Err("failed to write test data to the temporary file, which is used as a text document".into())
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

            let Ok(Some(completions)) = server::handler::handle_completion(
                &backend,
                tower_lsp::lsp_types::CompletionParams {
                    text_document_position: tower_lsp::lsp_types::TextDocumentPositionParams {
                        text_document: TextDocumentIdentifier { uri: toml_file_url },
                        position: (text::Position::default()
                            + text::RelativePosition::of(&toml_text[..index]))
                        .into(),
                    },
                    work_done_progress_params: WorkDoneProgressParams::default(),
                    partial_result_params: PartialResultParams {
                        partial_result_token: None,
                    },
                    context: None,
                },
            )
            .await else {
                return Err("failed to handle completion".into());
            };

            let selected = $select.0;
            let Some(completion) = completions
                .clone()
                .into_iter()
                .find(|content| content.label == selected)
                    else {
                    return Err(
                        format!(
                            "failed to find the selected completion item \"{}\" in [{}]",
                            selected,
                            completions
                                .iter()
                                .map(|content| content.label.as_str())
                                .collect::<Vec<&str>>()
                                .join(", ")
                        ).into()
                    );
                };

            let Some(completion_edit) = completion.edit else {
                return Err(
                    format!(
                        "failed to get the edit of the selected completion item: {}",
                        selected
                    ).into()
                );
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
                },
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

test_completion_edit! {
    #[tokio::test]
    async fn tombi_server_completion_dot(
        r#"
        [server]
        completion.█
        "#,
        Select("enabled"),
        tombi_schema_path(),
    ) -> Ok(
        r#"
        [server]
        completion.enabled
        "#
    );
}

test_completion_edit! {
    #[tokio::test]
    async fn tombi_server_completion_equal(
        r#"
        [server]
        completion=█
        "#,
        Select("enabled"),
        tombi_schema_path(),
    ) -> Ok(
        r#"
        [server]
        completion = { enabled$1 }
        "#
    );
}

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
        authors = [$0]
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
        authors = [$0]
        "#
    );
}
