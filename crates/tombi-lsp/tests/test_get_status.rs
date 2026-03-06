mod get_status {
    use super::*;

    /// Test for issue #1548: `[[schemas]]` config `toml-version` not applied
    /// https://github.com/tombi-toml/tombi/issues/1548
    mod issue_1548_schema_toml_version {
        use tombi_test_lib::project_root_path;

        use super::*;
        use std::path::PathBuf;

        fn fixture_path() -> PathBuf {
            project_root_path()
                .join("crates/tombi-lsp/tests/fixtures/issue-1548-schema-toml-version")
        }

        test_get_status!(
            #[tokio::test]
            async fn schema_toml_version_is_applied(
                SourcePath(fixture_path().join("schema-toml-version/test.toml")),
                ConfigPath(fixture_path().join("schema-toml-version/tombi.toml")),
            ) -> Ok(Expected {
                toml_version: tombi_config::TomlVersion::V1_1_0,
                source: tombi_lsp::handler::TomlVersionSource::Schema,
            });
        );

        test_get_status!(
            #[tokio::test]
            async fn config_toml_version_fallback(
                SourcePath(fixture_path().join("config-toml-version/test.toml")),
                ConfigPath(fixture_path().join("config-toml-version/tombi.toml")),
            ) -> Ok(Expected {
                toml_version: tombi_config::TomlVersion::V1_1_0,
                source: tombi_lsp::handler::TomlVersionSource::Config,
            });
        );

        test_get_status!(
            #[tokio::test]
            async fn default_toml_version_when_none_specified(
                SourcePath(fixture_path().join("no-toml-version/test.toml")),
                ConfigPath(fixture_path().join("no-toml-version/tombi.toml")),
            ) -> Ok(Expected {
                toml_version: tombi_config::TomlVersion::default(),
                source: tombi_lsp::handler::TomlVersionSource::Default,
            });
        );

        test_get_status!(
            #[tokio::test]
            async fn x_tombi_toml_version_from_json_schema(
                SourcePath(fixture_path().join("x-tombi-toml-version/test.toml")),
                ConfigPath(fixture_path().join("x-tombi-toml-version/tombi.toml")),
            ) -> Ok(Expected {
                toml_version: tombi_config::TomlVersion::V1_1_0,
                source: tombi_lsp::handler::TomlVersionSource::Schema,
            });
        );
    }
}

#[macro_export]
macro_rules! test_get_status {
    (#[tokio::test] async fn $name:ident(
        $($arg:expr),+ $(,)?
    ) -> Ok(Expected { toml_version: $expected_version:expr, source: $expected_source:expr $(,)? });) => {
        #[tokio::test]
        async fn $name() -> Result<(), Box<dyn std::error::Error>> {
            use tombi_lsp::Backend;
            use tower_lsp::{
                lsp_types::{
                    Url, DidOpenTextDocumentParams,
                    TextDocumentItem, TextDocumentIdentifier,
                },
                LspService,
            };
            use tombi_lsp::handler::{handle_did_open, handle_get_status};

            tombi_test_lib::init_log();

            #[allow(unused)]
            #[derive(Default)]
            struct TestArgs {
                source_file_path: Option<std::path::PathBuf>,
                source_text: Option<String>,
                config_file_path: Option<std::path::PathBuf>,
                backend_options: tombi_lsp::backend::Options,
            }

            #[allow(unused)]
            trait ApplyTestArg {
                fn apply(self, args: &mut TestArgs);
            }

            #[allow(unused)]
            struct SourcePath(std::path::PathBuf);

            impl ApplyTestArg for SourcePath {
                fn apply(self, args: &mut TestArgs) {
                    args.source_file_path = Some(self.0);
                }
            }

            #[allow(unused)]
            struct SourceText(String);

            impl ApplyTestArg for SourceText {
                fn apply(self, args: &mut TestArgs) {
                    args.source_text = Some(self.0);
                }
            }

            #[allow(unused)]
            struct ConfigPath(std::path::PathBuf);

            impl ApplyTestArg for ConfigPath {
                fn apply(self, args: &mut TestArgs) {
                    args.config_file_path = Some(self.0);
                }
            }

            impl ApplyTestArg for tombi_lsp::backend::Options {
                fn apply(self, args: &mut TestArgs) {
                    args.backend_options = self;
                }
            }

            #[allow(unused_mut)]
            let mut args = TestArgs::default();
            $(ApplyTestArg::apply($arg, &mut args);)*

            let (service, _) = LspService::new(|client| {
                Backend::new(client, &args.backend_options)
            });

            let backend = service.inner();

            // Load config file if specified
            if let Some(config_file_path) = args.config_file_path.as_ref() {
                let config_content = std::fs::read_to_string(config_file_path)
                    .map_err(|e| format!("Failed to read config file {}: {}", config_file_path.display(), e))?;
                let tombi_config: tombi_config::Config = serde_tombi::from_str_async(&config_content)
                    .await
                    .map_err(|e| format!("Failed to parse config file {}: {}", config_file_path.display(), e))?;

                backend
                    .config_manager
                    .update_config_with_path(tombi_config, config_file_path)
                    .await
                    .map_err(|e| format!("Failed to load config file {}: {}", config_file_path.display(), e))?;
            }

            // Determine source file
            let source_path = args.source_file_path.as_ref()
                .expect("SourcePath must be provided");

            let toml_text = if let Some(source_text) = args.source_text.take() {
                source_text
            } else {
                std::fs::read_to_string(source_path)
                    .map_err(|e| format!("Failed to read source file {}: {}", source_path.display(), e))?
            };

            let toml_file_url = Url::from_file_path(source_path)
                .map_err(|_| format!("Failed to convert path to URL: {}", source_path.display()))?;

            handle_did_open(
                backend,
                DidOpenTextDocumentParams {
                    text_document: TextDocumentItem {
                        uri: toml_file_url.clone(),
                        language_id: "toml".to_string(),
                        version: 0,
                        text: toml_text,
                    },
                },
            )
            .await;

            // Get status
            let status = handle_get_status(
                backend,
                TextDocumentIdentifier {
                    uri: toml_file_url,
                },
            )
            .await
            .expect("Failed to get status");

            log::debug!("status: {:#?}", status);

            // Verify toml_version
            assert_eq!(
                status.toml_version,
                $expected_version,
                "Expected toml_version {:?}, but got {:?}",
                $expected_version,
                status.toml_version,
            );

            // Verify source
            assert_eq!(
                status.source,
                $expected_source,
                "Expected source {:?}, but got {:?}",
                $expected_source,
                status.source,
            );

            Ok(())
        }
    };
}
