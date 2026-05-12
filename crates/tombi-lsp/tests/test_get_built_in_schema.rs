mod get_built_in_schema {
    use tombi_lsp::{Backend, handler::GetBuiltInSchemaParams};
    use tower_lsp::LspService;

    #[tokio::test]
    async fn returns_embedded_schema_content_for_valid_tombi_uri()
    -> Result<(), Box<dyn std::error::Error>> {
        tombi_test_lib::init_log();

        let (service, _) = LspService::new(|client| Backend::new(client, &Default::default()));
        let backend = service.inner();

        let response = backend
            .get_built_in_schema(GetBuiltInSchemaParams {
                uri: "tombi://www.schemastore.org/tombi.json".to_string(),
            })
            .await?;

        let content = response.expect("expected embedded schema content");
        assert!(content.contains("\"$schema\""));

        Ok(())
    }

    #[tokio::test]
    async fn returns_none_for_unknown_tombi_uri() -> Result<(), Box<dyn std::error::Error>> {
        tombi_test_lib::init_log();

        let (service, _) = LspService::new(|client| Backend::new(client, &Default::default()));
        let backend = service.inner();

        let response = backend
            .get_built_in_schema(GetBuiltInSchemaParams {
                uri: "tombi://www.schemastore.org/unknown.json".to_string(),
            })
            .await?;

        assert!(response.is_none());

        Ok(())
    }
}
