mod get_status {
    use super::*;

    /// Test for issue #1548: `[[schemas]]` config `toml-version` not applied
    /// https://github.com/tombi-toml/tombi/issues/1548
    mod issue_1548_schema_toml_version {
        use tombi_test_lib::project_root_path;

        use super::*;
        use std::{path::PathBuf, str::FromStr};

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

        test_get_status!(
            #[tokio::test]
            async fn selected_schema_keeps_priority_after_rebuild_and_cache_refresh(
                SourcePath(fixture_path().join("schema-toml-version/test.toml")),
                ConfigPath(fixture_path().join("schema-toml-version/tombi.toml")),
                UseTestCacheHome,
            ) -> Ok(Expected {
                toml_version: tombi_config::TomlVersion::V1_1_0,
                source: tombi_lsp::handler::TomlVersionSource::Schema,
                schema: tombi_schema_store::SchemaUri::from_str("tombi://www.schemastore.org/tombi.json")?,
            });
            AfterOpen(|backend, source_path, document_uri| {
                use std::str::FromStr;
                use tombi_schema_store::{AssociateSchemaOptions, SchemaUri};

                let selected_schema = SchemaUri::from_str("tombi://www.schemastore.org/tombi.json")?;
                let file_match = [source_path.to_string_lossy().into_owned()];
                for schema_uri in [
                    SchemaUri::from_str("tombi://www.schemastore.org/cargo.json")?,
                    selected_schema.clone(),
                ] {
                    backend
                        .config_manager
                        .associate_schema(
                            &schema_uri,
                            &file_match,
                            &AssociateSchemaOptions {
                                force: true,
                                ..Default::default()
                            },
                        )
                        .await;
                }

                let config_path = source_path.parent().unwrap().join("tombi.toml");
                let config = serde_tombi::config::try_from_path(&config_path)?.unwrap();
                backend
                    .config_manager
                    .update_config_with_path(config, &config_path)
                    .await?;
                let status = tombi_lsp::handler::handle_get_status(
                    backend,
                    tower_lsp_server::gen_lsp_types::TextDocumentIdentifier {
                        uri: document_uri.clone(),
                    },
                )
                .await?;
                assert_eq!(status.schema.map(|schema| schema.uri), Some(selected_schema.clone()));

                assert!(backend.config_manager.refresh_cache().await?);
            });
        );
    }

    mod associate_schema_updates_status {
        use std::str::FromStr;

        use tombi_schema_store::{AssociateSchemaOptions, SchemaUri};
        use tower_lsp_server::{
            LspService,
            gen_lsp_types::{
                DidOpenTextDocumentParams, TextDocumentIdentifier, TextDocumentItem, Uri,
            },
        };

        #[tokio::test]
        async fn returns_associated_schema_after_selection()
        -> Result<(), Box<dyn std::error::Error>> {
            use tombi_lsp::{
                Backend,
                handler::{handle_did_open, handle_get_status},
            };

            tombi_test_lib::init_log();

            let temp_dir = tempfile::tempdir()?;
            let source_path = temp_dir.path().join("selected-schema.toml");
            std::fs::write(&source_path, "answer = 42\n")?;

            let (service, _) = LspService::new(|client| Backend::new(client, &Default::default()));
            let backend = service.inner();

            let uri = Uri::from_file_path(&source_path)
                .map_err(|_| format!("Failed to convert path to URL: {}", source_path.display()))?;

            handle_did_open(
                backend,
                DidOpenTextDocumentParams {
                    text_document: TextDocumentItem {
                        uri: uri.clone(),
                        language_id: "toml".to_string().into(),
                        version: 0,
                        text: "answer = 42\n".to_string(),
                    },
                },
            )
            .await;

            let schema_uri = SchemaUri::from_str("tombi://www.schemastore.org/tombi.json")?;

            backend
                .config_manager
                .associate_schema(
                    &schema_uri,
                    &[source_path.to_string_lossy().into_owned()],
                    &AssociateSchemaOptions {
                        title: Some("Tombi".to_string()),
                        description: None,
                        toml_version: None,
                        force: true,
                    },
                )
                .await;

            let status =
                handle_get_status(backend, TextDocumentIdentifier { uri: uri.clone() }).await?;

            assert_eq!(
                status.schema.as_ref().map(|schema| schema.uri.clone()),
                Some(schema_uri)
            );

            Ok(())
        }
    }

    mod update_config_rebuilds_schema_store {
        use std::str::FromStr;

        use tombi_schema_store::{AssociateSchemaOptions, SchemaUri};
        use tower_lsp_server::LspService;

        #[tokio::test]
        async fn applies_strict_and_keeps_associated_schema()
        -> Result<(), Box<dyn std::error::Error>> {
            use tombi_lsp::Backend;

            tombi_test_lib::init_log();

            let temp_dir = tempfile::tempdir()?;
            std::fs::write(
                temp_dir.path().join("tombi.toml"),
                "toml-version = \"v1.0.0\"\n",
            )?;
            let source_path = temp_dir.path().join("source.toml");
            std::fs::write(&source_path, "answer = 42\n")?;

            let (service, _) = LspService::new(|client| Backend::new(client, &Default::default()));
            let config_manager = &service.inner().config_manager;

            let config_schema_store = config_manager
                .config_schema_store_for_file(&source_path)
                .await;
            let config_path = config_schema_store
                .config_path
                .clone()
                .ok_or("tombi.toml should be discovered")?;
            assert_eq!(config_schema_store.schema_store.strict(), None);

            let schema_uri = SchemaUri::from_str("tombi://www.schemastore.org/tombi.json")?;
            config_manager
                .associate_schema(
                    &schema_uri,
                    &["*.toml".to_string()],
                    &AssociateSchemaOptions {
                        title: None,
                        description: None,
                        toml_version: None,
                        force: true,
                    },
                )
                .await;

            let mut config = config_schema_store.config;
            config.schema = Some(tombi_config::SchemaOverviewOptions {
                strict: Some(false.into()),
                ..Default::default()
            });
            config_manager
                .update_config_with_path(config, &config_path)
                .await?;

            let schema_store = config_manager
                .config_schema_store_for_file(&source_path)
                .await
                .schema_store;
            assert_eq!(
                schema_store.strict().map(|strict| strict.value()),
                Some(false)
            );
            assert!(
                schema_store
                    .list_schemas()
                    .await
                    .iter()
                    .any(|schema| schema.schema_uri == schema_uri)
            );

            Ok(())
        }
    }
}

#[macro_export]
macro_rules! test_get_status {
    (#[tokio::test] async fn $name:ident(
        $($arg:expr),+ $(,)?
    ) -> Ok(Expected { toml_version: $expected_version:expr, source: $expected_source:expr $(, schema: $expected_schema:expr)? $(,)? });
    $(AfterOpen(|$backend:ident, $source_path:ident, $document_uri:ident| $after_open:block);)?) => {
        #[tokio::test]
        async fn $name() -> Result<(), Box<dyn std::error::Error>> {
            use tombi_lsp::Backend;
            use tower_lsp_server::{
                gen_lsp_types::{
                    Uri, DidOpenTextDocumentParams,
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
                use_test_cache_home: bool,
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

            struct UseTestCacheHome;

            impl ApplyTestArg for UseTestCacheHome {
                fn apply(self, args: &mut TestArgs) {
                    args.use_test_cache_home = true;
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

            let _cache_home = args.use_test_cache_home.then(tombi_test_lib::TestCacheHome::new);

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

            let toml_file_url = Uri::from_file_path(source_path)
                .map_err(|_| format!("Failed to convert path to URL: {}", source_path.display()))?;

            handle_did_open(
                backend,
                DidOpenTextDocumentParams {
                    text_document: TextDocumentItem {
                        uri: toml_file_url.clone(),
                        language_id: "toml".to_string().into(),
                        version: 0,
                        text: toml_text,
                    },
                },
            )
            .await;

            $(
                let $backend = backend;
                let $source_path = source_path;
                let $document_uri = &toml_file_url;
                $after_open
            )?

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

            $(
                assert_eq!(
                    status.schema.map(|schema| schema.uri),
                    Some($expected_schema),
                );
            )?

            Ok(())
        }
    };
}
