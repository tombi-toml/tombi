use tombi_test_lib::project_root_path;

macro_rules! test_get_schemas {
    ($(#[$attr:meta])* async fn $name:ident($toml_text:expr, $source_path:expr $(, $arg:expr)* $(,)?) -> Ok($expected:expr);) => {
        $(#[$attr])*
        async fn $name() {
            tombi_test_lib::init_tracing();

            let source_path: std::path::PathBuf = $source_path;

            let (service, _) = tower_lsp::LspService::new(|client| {
                tombi_lsp::Backend::new(client, &tombi_lsp::backend::Options::default())
            });

            let backend = service.inner();

            $(
                backend
                    .config_manager
                    .load_config_schemas(&[$arg], None)
                    .await;
            )*

            let toml_file_url = tower_lsp::lsp_types::Url::from_file_path(&source_path)
                .expect("failed to convert source file path to URL");

            let toml_text = textwrap::dedent($toml_text).trim().to_string();

            tombi_lsp::handler::handle_did_open(
                backend,
                tower_lsp::lsp_types::DidOpenTextDocumentParams {
                    text_document: tower_lsp::lsp_types::TextDocumentItem {
                        uri: toml_file_url.clone(),
                        language_id: "toml".to_string(),
                        version: 0,
                        text: toml_text,
                    },
                },
            )
            .await;

            let response = tombi_lsp::handler::handle_get_schemas(
                backend,
                tower_lsp::lsp_types::TextDocumentIdentifier { uri: toml_file_url },
            )
            .await
            .unwrap();

            let mut actual: Vec<std::path::PathBuf> = response
                .schemas
                .into_iter()
                .filter_map(|schema| schema.uri.to_file_path().ok())
                .collect();
            actual.sort();

            let mut expected: Vec<std::path::PathBuf> = $expected;
            expected.sort();

            pretty_assertions::assert_eq!(actual, expected);
        }
    };
}

mod get_schemas_tests {
    use super::*;

    test_get_schemas!(
        #[tokio::test]
        async fn root_schema_config_applies_to_document(
            r#"
            a = 1
            "#,
            project_root_path().join("tmp/unknown.toml"),
            tombi_config::SchemaItem::Root(tombi_config::RootSchema {
                toml_version: None,
                path: tombi_schema_store::SchemaUri::from_file_path(
                    project_root_path().join("schemas/type-test.schema.json"),
                )
                .unwrap()
                .to_string(),
                include: vec!["*.toml".to_string()],
                ..Default::default()
            }),
        ) -> Ok(vec![project_root_path().join("schemas/type-test.schema.json")]);
    );
}
