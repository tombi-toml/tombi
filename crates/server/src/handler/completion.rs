use ast::{algo::ancestors_at_position, AstNode};
use document_tree::{IntoDocumentTreeResult, TryIntoDocumentTree};
use tower_lsp::lsp_types::{
    CompletionItem, CompletionParams, CompletionResponse, TextDocumentPositionParams,
};

use crate::{
    backend,
    completion::{CompletionHint, FindCompletionItems},
};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn handle_completion(
    backend: &backend::Backend,
    CompletionParams {
        text_document_position:
            TextDocumentPositionParams {
                text_document,
                position,
            },
        ..
    }: CompletionParams,
) -> Result<Option<CompletionResponse>, tower_lsp::jsonrpc::Error> {
    tracing::info!("handle_completion");

    let config = backend.config().await;

    if !config
        .server
        .and_then(|s| s.completion)
        .unwrap_or_default()
        .value()
    {
        tracing::debug!("`server.completion` is false");
        return Ok(None);
    }

    if !config
        .schema
        .and_then(|s| s.enabled)
        .unwrap_or_default()
        .value()
    {
        tracing::debug!("`schema.enabled` is false");
        return Ok(None);
    }

    let Ok(Some(document_schema)) = &backend
        .schema_store
        .try_get_schema_from_url(&text_document.uri)
        .await
    else {
        tracing::debug!("schema not found: {}", text_document.uri);
        return Ok(None);
    };

    let toml_version = backend.toml_version().await.unwrap_or_default();
    let Some(root) = backend.get_incomplete_ast(&text_document.uri, toml_version) else {
        return Ok(None);
    };

    let items = get_completion_items(root, position.into(), document_schema, toml_version);

    Ok(Some(CompletionResponse::Array(items)))
}

fn get_completion_items(
    root: ast::Root,
    position: text::Position,
    document_schema: &schema_store::DocumentSchema,
    toml_version: config::TomlVersion,
) -> Vec<CompletionItem> {
    let mut keys: Vec<document_tree::Key> = vec![];
    let mut completion_hint = None;

    for node in ancestors_at_position(root.syntax(), position) {
        let ast_keys = if let Some(kv) = ast::KeyValue::cast(node.to_owned()) {
            kv.keys()
        } else if let Some(table) = ast::Table::cast(node.to_owned()) {
            if position < table.bracket_start().unwrap().range().start() {
                None
            } else {
                if table.contains_header(position) {
                    completion_hint = Some(CompletionHint::InTableHeader);
                }
                table.header()
            }
        } else if let Some(array_of_tables) = ast::ArrayOfTables::cast(node.to_owned()) {
            if position
                < array_of_tables
                    .double_bracket_start()
                    .unwrap()
                    .range()
                    .start()
            {
                None
            } else {
                if array_of_tables.contains_header(position) {
                    completion_hint = Some(CompletionHint::InTableHeader);
                }
                array_of_tables.header()
            }
        } else {
            continue;
        };

        let Some(ast_keys) = ast_keys else { continue };
        let mut new_keys = if ast_keys.range().contains(position) {
            let mut new_keys = Vec::with_capacity(ast_keys.keys().count());
            for key in ast_keys
                .keys()
                .take_while(|key| key.token().unwrap().range().start() <= position)
            {
                match key.try_into_document_tree(toml_version) {
                    Ok(Some(key)) => new_keys.push(key),
                    _ => return vec![],
                }
            }
            new_keys
        } else {
            let mut new_keys = Vec::with_capacity(ast_keys.keys().count());
            for key in ast_keys.keys() {
                match key.try_into_document_tree(toml_version) {
                    Ok(Some(key)) => new_keys.push(key),
                    _ => return vec![],
                }
            }
            new_keys
        };

        new_keys.extend(keys);
        keys = new_keys;
    }

    let document_tree = root.into_document_tree_result(toml_version).tree;

    let completion_items = document_tree.find_completion_items(
        &Vec::with_capacity(0),
        document_schema.value_schema(),
        toml_version,
        position,
        &keys,
        Some(&document_schema.schema_url),
        &document_schema.definitions,
        completion_hint,
    );

    completion_items
}

#[cfg(test)]
mod test {
    use itertools::Itertools;

    use crate::test::{cargo_schema_path, pyproject_schema_path, tombi_schema_path};

    use super::*;

    #[macro_export]
    macro_rules! test_completion_labels {
        (
            #[tokio::test]
            async fn $name:ident(
                $schema_file_path:expr,
                $source:expr
            ) -> Ok([$($label:expr),*$(,)?]);
        ) => {
            #[tokio::test]
            async fn $name() {
                use backend::Backend;
                use schema_store::JsonCatalogSchema;
                use std::io::Write;
                use tower_lsp::{
                    lsp_types::{
                        DidOpenTextDocumentParams, PartialResultParams, TextDocumentIdentifier,
                        TextDocumentItem, Url, WorkDoneProgressParams,
                    },
                    LspService,
                };
                use crate::handler::handle_did_open;

                let (service, _) = LspService::new(|client| Backend::new(client));

                let backend = service.inner();

                let schema_url = Url::from_file_path($schema_file_path).expect(
                    format!(
                        "failed to convert schema path to URL: {}",
                        tombi_schema_path().display()
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

                let temp_file = tempfile::NamedTempFile::with_suffix_in(
                    ".toml",
                    std::env::current_dir().expect("failed to get current directory"),
                )
                .expect("failed to create temporary file");

                let mut toml_data = textwrap::dedent($source).trim().to_string();

                let index = toml_data
                    .as_str()
                    .find("█")
                    .expect("failed to find completion position marker (█) in the test data");

                toml_data.remove(index);
                temp_file.as_file().write_all(toml_data.as_bytes()).expect(
                    "failed to write test data to the temporary file, which is used as a text document",
                );

                let toml_file_url = Url::from_file_path(temp_file.path())
                    .expect("failed to convert temporary file path to URL");

                handle_did_open(
                    backend,
                    DidOpenTextDocumentParams {
                        text_document: TextDocumentItem {
                            uri: toml_file_url.clone(),
                            language_id: "toml".to_string(),
                            version: 0,
                            text: toml_data.clone(),
                        },
                    },
                )
                .await;

                let completions = handle_completion(
                    &backend,
                    CompletionParams {
                        text_document_position: TextDocumentPositionParams {
                            text_document: TextDocumentIdentifier { uri: toml_file_url },
                            position: (text::Position::default()
                                + text::RelativePosition::of(&toml_data[..index]))
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
                .expect("failed to handle completion")
                .expect("failed to get completion items");

                match completions {
                    CompletionResponse::Array(items) => {
                        let labels = items
                            .into_iter()
                            .sorted_by(|a, b|
                                a.sort_text.as_ref().unwrap_or(&a.label).cmp(&b.sort_text.as_ref().unwrap_or(&b.label))
                            )
                            .map(|item| item.label)
                            .collect::<Vec<_>>();

                        pretty_assertions::assert_eq!(
                            labels,
                            vec![$($label.to_string()),*]
                        );
                    }
                    _ => panic!("expected completion items"),
                }
            }
        };
    }

    test_completion_labels! {
        #[tokio::test]
        async fn tombi_empty(
            tombi_schema_path(),
            "█"
        ) -> Ok([
            "format",
            "lint",
            "schema",
            "schemas",
            "server",
            "toml-version",
        ]);
    }

    test_completion_labels! {
        #[tokio::test]
        async fn tombi_used_toml_version(
            tombi_schema_path(),
            r#"
            toml-version = "v1.0.0"
            █
            "#
        ) -> Ok([
            "format",
            "lint",
            "schema",
            "schemas",
            "server",
            // "toml-version",
        ]);
    }

    test_completion_labels! {
        #[tokio::test]
        async fn pyproject_empty(
            pyproject_schema_path(),
            "█"
        ) -> Ok([
            "build-system",
            "dependency-groups",
            "project",
            "tool",
        ]);
    }

    test_completion_labels! {
        #[tokio::test]
        async fn cargo_empty(
            cargo_schema_path(),
            "█"
        ) -> Ok([
            "badges",
            "bench",
            "bin",
            "build-dependencies",
            "build_dependencies",
            "cargo-features",
            "dependencies",
            "dev-dependencies",
            "dev_dependencies",
            "example",
            "features",
            "lib",
            "lints",
            "package",
            "patch",
            "profile",
            "project",
            "replace",
            "target",
            "test",
            "workspace",
        ]);
    }
}
