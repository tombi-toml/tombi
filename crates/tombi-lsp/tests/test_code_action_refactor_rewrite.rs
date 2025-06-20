macro_rules! test_code_action_refactor_rewrite {
    (
        #[tokio::test]
        async fn $name:ident(
            $source:expr,
        ) -> Ok($expected:expr);
    ) => {
        test_code_action_refactor_rewrite! {
            #[tokio::test]
            async fn _$name(
                $source,
                "Dummy Code Action",
                Option::<std::path::PathBuf>::None,
            ) -> Ok($expected);
        }
    };

    (
        #[tokio::test]
        async fn $name:ident(
            $source:expr,
            Select($select:expr),
        ) -> Ok($expected:expr);
    ) => {
        test_code_action_refactor_rewrite! {
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
        async fn $name:ident(
            $source:expr,
            Select($select:expr),
            $toml_file_path:expr$(,)?
        ) -> Ok($expected:expr);
    ) => {
        test_code_action_refactor_rewrite! {
            #[tokio::test]
            async fn _$name(
                $source,
                $select,
                Some($toml_file_path),
            ) -> Ok($expected);
        }
    };

    (
        #[tokio::test]
        async fn $name:ident(
            $source:expr,
            $toml_file_path:expr$(,)?
        ) -> Ok($expected:expr);
    ) => {
        test_code_action_refactor_rewrite! {
            #[tokio::test]
            async fn _$name(
                $source,
                "Dummy Code Action",
                Some($toml_file_path),
            ) -> Ok($expected);
        }
    };

    (
        #[tokio::test]
        async fn _$name:ident(
            $source:expr,
            $select:expr,
            $toml_file_path:expr$(,)?
        ) -> Ok($expected:expr);
    ) => {
        #[tokio::test]
        async fn $name() -> Result<(), Box<dyn std::error::Error>> {
            use itertools::Itertools;
            use tombi_lsp::handler::handle_code_action;
            use tombi_lsp::handler::handle_did_open;
            use tombi_lsp::Backend;
            use tower_lsp::lsp_types::{CodeActionParams, TextDocumentIdentifier, Url};
            use tower_lsp::lsp_types::{DidOpenTextDocumentParams, TextDocumentItem};
            use tower_lsp::LspService;

            tombi_test_lib::init_tracing();

            let (service, _) = LspService::new(|client| {
                Backend::new(client, &tombi_lsp::backend::Options::default())
            });
            let backend = service.inner();
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
            tracing::debug!(?toml_text, "test toml text");
            tracing::debug!(?index, "test toml text index");

            let toml_file_url = $toml_file_path
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
                .into(),
                context: Default::default(),
                work_done_progress_params: Default::default(),
                partial_result_params: Default::default(),
            };

            let Ok(actions) = handle_code_action(backend, params).await else {
                return Err("failed to get code actions".into());
            };

            tracing::debug!(?actions, "code actions found");

            match (actions, $expected) {
                (Some(actions), Some(expected)) => {
                    let selected = $select;
                    let selected: &str = &selected.to_string();

                    let Some(action) = actions.into_iter().find_map(|a| match a {
                        tower_lsp::lsp_types::CodeActionOrCommand::CodeAction(ca)
                            if ca.title == selected =>
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
                                tower_lsp::lsp_types::OneOf::Left(ref e) => &e.range.start,
                                _ => return std::cmp::Ordering::Equal,
                            };
                            let b = match b {
                                tower_lsp::lsp_types::OneOf::Left(ref e) => &e.range.start,
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
                    tracing::debug!("no code actions found, as expected");
                    Ok(())
                }
                (Some(actions), None) => {
                    let selected = $select;
                    let selected: &str = &selected.to_string();

                    let None = actions.iter().find_map(|a| match a {
                        tower_lsp::lsp_types::CodeActionOrCommand::CodeAction(ca)
                            if ca.title == selected =>
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

mod refactor_rewrite {
    mod common {
        use tombi_lsp::code_action::CodeActionRefactorRewriteName;

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
                serde = { workspace = true }
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
    }
}
