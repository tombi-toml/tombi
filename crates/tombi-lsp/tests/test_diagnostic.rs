mod diagnostic {
    use super::*;

    /// Test for issue #1495: Local schema and subdirectories
    /// https://github.com/tombi-toml/tombi/issues/1495
    mod issue_1495_subdirectory_glob {
        use tombi_test_lib::project_root_path;

        use super::*;
        use std::path::PathBuf;

        fn fixture_path() -> PathBuf {
            project_root_path().join("crates/tombi-lsp/tests/fixtures/issue-1495-subdirectory-glob")
        }

        test_diagnostic_file!(
            #[tokio::test]
            async fn product_toml_boolean_error(
                "name = false",
                SourcePath(fixture_path().join("product.toml")),
                ConfigPath(fixture_path().join("tombi.toml")),
            ) -> Ok([
                Diagnostic {
                    message: "Expected a value of type String, but found Boolean",
                    range: ((0, 7), (0, 12)),
                }
            ]);
        );

        test_diagnostic_file!(
            #[tokio::test]
            async fn subdir_subproduct_toml_boolean_error(
                "name = false",
                SourcePath(fixture_path().join("subdir").join("subproduct.toml")),
                ConfigPath(fixture_path().join("tombi.toml")),
            ) -> Ok([
                Diagnostic {
                    message: "Expected a value of type String, but found Boolean",
                    range: ((0, 7), (0, 12)),
                }
            ]);
        );
    }
}

// Unified test macro
#[macro_export]
macro_rules! test_diagnostic {
    // Pattern 1: First argument is a string literal - wrap it with SourceText
    (#[tokio::test] async fn $name:ident(
        $source:literal $(, $arg:expr )* $(,)?
    ) -> Ok($expected:expr);) => {
        test_diagnostic! {
            #[tokio::test] async fn $name(
                SourceText(textwrap::dedent($source).trim().to_string()) $(, $arg)*
            ) -> Ok($expected);
        }
    };

    // Pattern 2: All arguments are options
    (#[tokio::test] async fn $name:ident(
        $($arg:expr),+ $(,)?
    ) -> Ok($expected:expr);) => {
        #[tokio::test]
        async fn $name() -> Result<(), Box<dyn std::error::Error>> {
        use tombi_lsp::Backend;
        use tower_lsp::{
            lsp_types::{
                Url, DidOpenTextDocumentParams,
                TextDocumentItem, DocumentDiagnosticParams,
                TextDocumentIdentifier, WorkDoneProgressParams,
            },
            LspService,
        };
        use tombi_lsp::handler::{handle_did_open, handle_diagnostic};

        tombi_test_lib::init_log();

        #[allow(unused)]
        #[derive(Default)]
        struct TestArgs {
            source_file_path: Option<std::path::PathBuf>,
            source_text: Option<String>,
            schema_file_path: Option<std::path::PathBuf>,
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
        struct SchemaPath(std::path::PathBuf);

        impl ApplyTestArg for SchemaPath {
            fn apply(self, args: &mut TestArgs) {
                args.schema_file_path = Some(self.0);
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
        } else if let Some(schema_file_path) = config.schema_file_path.as_ref() {
            // Fallback to schema path if no config file
            let schema_uri = tombi_schema_store::SchemaUri::from_file_path(schema_file_path)
                .expect(
                    format!(
                        "failed to convert schema path to URL: {}",
                        schema_file_path.display()
                    )
                    .as_str(),
                );

            backend
                .config_manager
                .update_editor_config({
                    let mut editor_config = tombi_config::Config::default();
                    editor_config.schemas = Some(vec![
                        tombi_config::SchemaItem::Root(tombi_config::RootSchema {
                            toml_version: None,
                            path: schema_uri.to_string(),
                            include: vec!["*.toml".to_string()],
                        }),
                    ]);
                    editor_config
                })
                .await;
        }

        // Determine source based on config
        let (toml_text, toml_file_url) = if let Some(source_text) = config.source_text {
            // Use inline text via SourceText
            use std::io::Write;

            let toml_text = source_text;

            let current_dir = std::env::current_dir().expect("failed to get current directory");
            // If ConfigPath is specified, use its parent directory for temp file
            let temp_dir = if let Some(config_path) = config.config_file_path.as_ref() {
                config_path.parent().ok_or("failed to get parent directory")?
            } else {
                current_dir.as_path()
            };
            let Ok(temp_file) = tempfile::NamedTempFile::with_suffix_in(
                ".toml",
                temp_dir,
            ) else {
                return Err("failed to create a temporary file for the test data".into());
            };

            if temp_file.as_file().write_all(toml_text.as_bytes()).is_err() {
                return Err("failed to write to temporary file".into());
            };

            let Ok(toml_file_url) = Url::from_file_path(temp_file.path()) else {
                return Err("failed to convert temporary file path to URL".into());
            };

            (toml_text, toml_file_url)
        } else {
            // Use file path via SourcePath
            let source_path = config.source_file_path.as_ref()
                .expect("SourcePath or SourceText must be provided");
            let toml_text = std::fs::read_to_string(source_path)
                .map_err(|e| format!("Failed to read source file {}: {}", source_path.display(), e))?;
            let toml_file_url = Url::from_file_path(source_path)
                .map_err(|_| format!("Failed to convert path to URL: {}", source_path.display()))?;
            (toml_text, toml_file_url)
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

        // Get diagnostics
        let diagnostic_result = handle_diagnostic(
            backend,
            DocumentDiagnosticParams {
                text_document: TextDocumentIdentifier {
                    uri: toml_file_url.clone(),
                },
                work_done_progress_params: WorkDoneProgressParams::default(),
                partial_result_params: Default::default(),
                identifier: None,
                previous_result_id: None,
            },
        )
        .await
        .expect("Failed to get diagnostics");

        let diagnostics = match diagnostic_result {
            tower_lsp::lsp_types::DocumentDiagnosticReportResult::Report(report) => {
                match report {
                    tower_lsp::lsp_types::DocumentDiagnosticReport::Full(full) => {
                        full.full_document_diagnostic_report.items
                    }
                    tower_lsp::lsp_types::DocumentDiagnosticReport::Unchanged(_) => {
                        Vec::new()
                    }
                }
            }
            tower_lsp::lsp_types::DocumentDiagnosticReportResult::Partial(_) => {
                panic!("Unexpected partial diagnostic result")
            }
        };

        log::debug!("diagnostics: {:#?}", diagnostics);

        // Expected diagnostics
        #[allow(unused)]
        struct Diagnostic {
            message: &'static str,
            range: ((u32, u32), (u32, u32)),
        }

        let expected: &[Diagnostic] = &$expected;

        // Verify number of diagnostics
        assert_eq!(
            diagnostics.len(),
            expected.len(),
            "Expected {} diagnostic(s), but got {}.\nDiagnostics: {:#?}",
            expected.len(),
            diagnostics.len(),
            diagnostics
        );

        // Verify each diagnostic
        for (i, (actual, expected)) in diagnostics.iter().zip(expected.iter()).enumerate() {
            assert_eq!(
                actual.message,
                expected.message,
                "Diagnostic #{}: Expected message '{}', but got '{}'",
                i,
                expected.message,
                actual.message
            );

            let expected_start = (expected.range.0.0, expected.range.0.1);
            let actual_start = (actual.range.start.line, actual.range.start.character);
            assert_eq!(
                actual_start,
                expected_start,
                "Diagnostic #{}: Expected range start {:?}, but got {:?}",
                i,
                expected_start,
                actual_start
            );

            let expected_end = (expected.range.1.0, expected.range.1.1);
            let actual_end = (actual.range.end.line, actual.range.end.character);
            assert_eq!(
                actual_end,
                expected_end,
                "Diagnostic #{}: Expected range end {:?}, but got {:?}",
                i,
                expected_end,
                actual_end
            );
        }

        Ok(())
        }
    };
}

// File-based test macro with special handling for SourceText + SourcePath
#[macro_export]
macro_rules! test_diagnostic_file {
    // Pattern 1: First argument is a string literal - wrap it with SourceText
    (#[tokio::test] async fn $name:ident(
        $source:literal $(, $arg:expr )* $(,)?
    ) -> Ok($expected:expr);) => {
        test_diagnostic_file! {
            #[tokio::test] async fn $name(
                SourceText($source.to_string()) $(, $arg)*
            ) -> Ok($expected);
        }
    };

    // Pattern 2: Main implementation
    (#[tokio::test] async fn $name:ident(
        $($arg:expr),+ $(,)?
    ) -> Ok($expected:expr);) => {
        #[tokio::test]
        async fn $name() -> Result<(), Box<dyn std::error::Error>> {
        use tombi_lsp::Backend;
        use tower_lsp::{
            lsp_types::{
                Url, DidOpenTextDocumentParams,
                TextDocumentItem, DocumentDiagnosticParams,
                TextDocumentIdentifier, WorkDoneProgressParams,
            },
            LspService,
        };
        use tombi_lsp::handler::{handle_did_open, handle_diagnostic};

        tombi_test_lib::init_log();

        #[allow(unused)]
        #[derive(Default)]
        struct TestArgs {
            source_file_path: Option<std::path::PathBuf>,
            source_text: Option<String>,
            schema_file_path: Option<std::path::PathBuf>,
            config_file_path: Option<std::path::PathBuf>,
            backend_options: tombi_lsp::backend::Options,
        }

        #[allow(unused)]
        trait ApplyTestArg {
            fn apply(self, config: &mut TestArgs);
        }

        #[allow(unused)]
        struct SourcePath(std::path::PathBuf);

        impl ApplyTestArg for SourcePath {
            fn apply(self, config: &mut TestArgs) {
                config.source_file_path = Some(self.0);
            }
        }

        #[allow(unused)]
        struct SourceText(String);

        impl ApplyTestArg for SourceText {
            fn apply(self, config: &mut TestArgs) {
                config.source_text = Some(self.0);
            }
        }

        #[allow(unused)]
        struct SchemaPath(std::path::PathBuf);

        impl ApplyTestArg for SchemaPath {
            fn apply(self, config: &mut TestArgs) {
                config.schema_file_path = Some(self.0);
            }
        }

        #[allow(unused)]
        struct ConfigPath(std::path::PathBuf);

        impl ApplyTestArg for ConfigPath {
            fn apply(self, config: &mut TestArgs) {
                config.config_file_path = Some(self.0);
            }
        }

        impl ApplyTestArg for tombi_lsp::backend::Options {
            fn apply(self, config: &mut TestArgs) {
                config.backend_options = self;
            }
        }

        #[allow(unused_mut)]
        let mut config = TestArgs::default();
        $(ApplyTestArg::apply($arg, &mut config);)*

        // SourcePath must be provided for file-based tests
        let source_path = config.source_file_path.as_ref()
            .expect("SourcePath must be provided for test_diagnostic_file");

        let (service, _) = LspService::new(|client| {
            Backend::new(client, &config.backend_options)
        });

        let backend = service.inner();

        // Load config file if specified
        if let Some(config_file_path) = config.config_file_path.as_ref() {
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
        } else if let Some(schema_file_path) = config.schema_file_path.as_ref() {
            // Fallback to schema path if no config file
            let schema_uri = tombi_schema_store::SchemaUri::from_file_path(schema_file_path)
                .expect(
                    format!(
                        "failed to convert schema path to URL: {}",
                        schema_file_path.display()
                    )
                    .as_str(),
                );

            backend
                .config_manager
                .update_editor_config({
                    let mut editor_config = tombi_config::Config::default();
                    editor_config.schemas = Some(vec![
                        tombi_config::SchemaItem::Root(tombi_config::RootSchema {
                            toml_version: None,
                            path: schema_uri.to_string(),
                            include: vec!["*.toml".to_string()],
                        }),
                    ]);
                    editor_config
                })
                .await;
        }

        // Use SourceText when provided, otherwise read from SourcePath.
        let toml_text = if let Some(source_text) = config.source_text.take() {
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
                    text: toml_text.clone(),
                },
            },
        )
        .await;

        // Get diagnostics
        let diagnostic_result = handle_diagnostic(
            backend,
            DocumentDiagnosticParams {
                text_document: TextDocumentIdentifier {
                    uri: toml_file_url.clone(),
                },
                work_done_progress_params: WorkDoneProgressParams::default(),
                partial_result_params: Default::default(),
                identifier: None,
                previous_result_id: None,
            },
        )
        .await
        .expect("Failed to get diagnostics");

        let diagnostics = match diagnostic_result {
            tower_lsp::lsp_types::DocumentDiagnosticReportResult::Report(report) => {
                match report {
                    tower_lsp::lsp_types::DocumentDiagnosticReport::Full(full) => {
                        full.full_document_diagnostic_report.items
                    }
                    tower_lsp::lsp_types::DocumentDiagnosticReport::Unchanged(_) => {
                        Vec::new()
                    }
                }
            }
            tower_lsp::lsp_types::DocumentDiagnosticReportResult::Partial(_) => {
                panic!("Unexpected partial diagnostic result")
            }
        };

        log::debug!("diagnostics: {:#?}", diagnostics);

        // Expected diagnostics
        #[allow(unused)]
        struct Diagnostic {
            message: &'static str,
            range: ((u32, u32), (u32, u32)),
        }

        let expected: &[Diagnostic] = &$expected;

        // Verify number of diagnostics
        assert_eq!(
            diagnostics.len(),
            expected.len(),
            "Expected {} diagnostic(s), but got {}.\nDiagnostics: {:#?}",
            expected.len(),
            diagnostics.len(),
            diagnostics
        );

        // Verify each diagnostic
        for (i, (actual, expected)) in diagnostics.iter().zip(expected.iter()).enumerate() {
            assert_eq!(
                actual.message,
                expected.message,
                "Diagnostic #{}: Expected message '{}', but got '{}'",
                i,
                expected.message,
                actual.message
            );

            let expected_start = (expected.range.0.0, expected.range.0.1);
            let actual_start = (actual.range.start.line, actual.range.start.character);
            assert_eq!(
                actual_start,
                expected_start,
                "Diagnostic #{}: Expected range start {:?}, but got {:?}",
                i,
                expected_start,
                actual_start
            );

            let expected_end = (expected.range.1.0, expected.range.1.1);
            let actual_end = (actual.range.end.line, actual.range.end.character);
            assert_eq!(
                actual_end,
                expected_end,
                "Diagnostic #{}: Expected range end {:?}, but got {:?}",
                i,
                expected_end,
                actual_end
            );
        }

        Ok(())
        }
    };
}
