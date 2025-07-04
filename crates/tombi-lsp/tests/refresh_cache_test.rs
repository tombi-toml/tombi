#[cfg(test)]
mod tests {
    use tower_lsp::lsp_types::TextDocumentIdentifier;
    use tower_lsp::lsp_types::Url;

    #[tokio::test]
    async fn test_refresh_cache_command() {
        // Create a mock backend
        let (client, _) = tower_lsp::Client::test();
        let backend = tombi_lsp::Backend::new(
            client,
            &tombi_lsp::backend::Options::default(),
        );

        // Create a test document URI
        let test_uri = Url::parse("file:///test.toml").unwrap();
        let params = TextDocumentIdentifier {
            uri: test_uri,
        };

        // Call refresh_cache
        let result = backend.refresh_cache(params).await;

        // The result should be Ok(false) since the document doesn't exist
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);
    }
}