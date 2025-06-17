use tombi_test_lib::project_root_path;

mod document_link_tests {
    use super::*;

    mod cargo_schema {
        use super::*;

        test_document_link!(
            #[tokio::test]
            async fn cargo_package_readme(
                r#"
                [package]
                readme = "README.md"
                "#,
                project_root_path().join("Cargo.toml"),
            ) -> Ok(Some(vec![]));
        );

        test_document_link!(
            #[tokio::test]
            async fn cargo_workspace_package_readme_without_schema(
                r#"
                #:schema schemas/Cargo.json

                [workspace.package]
                readme = "README.md"
                "#,
                project_root_path().join("Cargo.toml"),
            ) -> Ok(Some(vec![
                {
                    path: project_root_path().join("schemas/Cargo.json"),
                    range: 0:9..0:27
                }
            ]));
        );

        test_document_link!(
            #[tokio::test]
            async fn cargo_workspace_dependencies_tombi_lsp(
                r#"
                [workspace.package]
                readme = "README.md"

                [workspace.dependencies]
                tombi-lsp.path = "crates/tombi-lsp"
                "#,
                project_root_path().join("Cargo.toml"),
            ) -> Ok(Some(vec![
                {
                    path: project_root_path().join("crates/tombi-lsp/Cargo.toml"),
                    range: 4:0..4:9,
                    tooltip: tombi_extension_cargo::DocumentLinkToolTip::CargoToml,
                },
                {
                    path: project_root_path().join("crates/tombi-lsp/Cargo.toml"),
                    range: 4:18..4:34,
                    tooltip: tombi_extension_cargo::DocumentLinkToolTip::CargoToml,
                }
            ]));
        );

        test_document_link!(
            #[tokio::test]
            async fn cargo_workspace_dependencies_serde(
                r#"
                [workspace.package]
                readme = "README.md"

                [workspace.dependencies]
                serde = "1.0"
                "#,
                project_root_path().join("Cargo.toml"),
            ) -> Ok(Some(vec![
                {
                    url: "https://crates.io/crates/serde",
                    range: 4:0..4:5,
                    tooltip: tombi_extension_cargo::DocumentLinkToolTip::CrateIo,
                }
            ]));
        );

        test_document_link!(
            #[tokio::test]
            async fn cargo_workspace_dependencies_serde_toml(
                r#"
                [workspace.package]
                readme = "README.md"

                [workspace.dependencies]
                serde_toml = { version = "0.1", package = "toml" }
                "#,
                project_root_path().join("Cargo.toml"),
            ) -> Ok(Some(vec![
                {
                    url: "https://crates.io/crates/toml",
                    range: 4:0..4:10,
                    tooltip: tombi_extension_cargo::DocumentLinkToolTip::CrateIo,
                }
            ]));
        );

        test_document_link!(
            #[tokio::test]
            async fn cargo_workspace_dependencies_serde_git(
                r#"
                [workspace.package]
                readme = "README.md"

                [workspace.dependencies]
                serde = { git = "https://github.com/serde-rs/serde" }
                "#,
                project_root_path().join("Cargo.toml"),
            ) -> Ok(Some(vec![
                {
                    url: "https://github.com/serde-rs/serde",
                    range: 4:0..4:5,
                    tooltip: tombi_extension_cargo::DocumentLinkToolTip::GitRepository,
                },
                {
                    url: "https://github.com/serde-rs/serde",
                    range: 4:17..4:50,
                    tooltip: tombi_extension_cargo::DocumentLinkToolTip::GitRepository,
                }
            ]));
        );

        test_document_link!(
            #[tokio::test]
            async fn cargo_dependencies_tombi_lsp(
                r#"
                [package]
                readme = "README.md"

                [dependencies]
                tombi-lsp.path = "../../crates/tombi-lsp"
                "#,
                project_root_path().join("rust/tombi-cli/Cargo.toml"),
            ) -> Ok(Some(vec![
                {
                    path: project_root_path().join("crates/tombi-lsp/Cargo.toml"),
                    range: 4:0..4:9,
                    tooltip: tombi_extension_cargo::DocumentLinkToolTip::CargoToml,
                },
                {
                    path: project_root_path().join("crates/tombi-lsp/Cargo.toml"),
                    range: 4:18..4:40,
                    tooltip: tombi_extension_cargo::DocumentLinkToolTip::CargoToml,
                }
            ]));
        );

        test_document_link!(
            #[tokio::test]
            async fn cargo_dependencies_serde(
                r#"
                [package]
                readme = "README.md"

                [dependencies]
                serde = "1.0"
                "#,
                project_root_path().join("subcrate/Cargo.toml"),
            ) -> Ok(Some(vec![
                {
                    url: "https://crates.io/crates/serde",
                    range: 4:0..4:5,
                    tooltip: tombi_extension_cargo::DocumentLinkToolTip::CrateIo,
                }
            ]));
        );

        test_document_link!(
            #[tokio::test]
            async fn cargo_dependencies_serde_toml(
                r#"
                [package]
                readme = "README.md"

                [dependencies]
                serde_toml = { version = "0.1", package = "toml" }
                "#,
                project_root_path().join("subcrate/Cargo.toml"),
            ) -> Ok(Some(vec![
                {
                    url: "https://crates.io/crates/toml",
                    range: 4:0..4:10,
                    tooltip: tombi_extension_cargo::DocumentLinkToolTip::CrateIo,
                }
            ]));
        );

        test_document_link!(
            #[tokio::test]
            async fn cargo_dependencies_serde_git(
                r#"
                [package]
                readme = "README.md"

                [dependencies]
                serde = { git = "https://github.com/serde-rs/serde" }
                "#,
                project_root_path().join("subcrate/Cargo.toml"),
            ) -> Ok(Some(vec![
                {
                    url: "https://github.com/serde-rs/serde",
                    range: 4:0..4:5,
                    tooltip: tombi_extension_cargo::DocumentLinkToolTip::GitRepository,
                },
                {
                    url: "https://github.com/serde-rs/serde",
                    range: 4:17..4:50,
                    tooltip: tombi_extension_cargo::DocumentLinkToolTip::GitRepository,
                }
            ]));
        );

        test_document_link!(
            #[tokio::test]
            async fn cargo_dependencies_tombi_lsp_workspace_true(
                r#"
                [package]
                readme = "README.md"

                [dependencies]
                tombi-lsp = { workspace = true, default-features = [] }
                "#,
                project_root_path().join("subcrate/Cargo.toml"),
            ) -> Ok(Some(vec![
                {
                    path: project_root_path().join("crates/tombi-lsp/Cargo.toml"),
                    range: 4:0..4:9,
                    tooltip: tombi_extension_cargo::DocumentLinkToolTip::CargoToml,
                },
                {
                    path: project_root_path().join("Cargo.toml"),
                    range: 4:14..4:30,
                    tooltip: tombi_extension_cargo::DocumentLinkToolTip::WorkspaceCargoToml,
                }
            ]));
        );

        test_document_link!(
            #[tokio::test]
            async fn cargo_package_workspace(
                r#"
                [package]
                workspace = "../../"
                "#,
                project_root_path().join("crates/tombi-lsp/Cargo.toml"),
            ) -> Ok(Some(vec![
                {
                    path: project_root_path().join("Cargo.toml"),
                    range: 1:13..1:19,
                    tooltip: tombi_extension_cargo::DocumentLinkToolTip::WorkspaceCargoToml,
                }
            ]));
        );

        test_document_link!(
            #[tokio::test]
            async fn cargo_root_package(
                r#"
                [package]
                name = "root-package"
                version = "0.1.0"

                [dependencies]
                tombi-ast = { workspace = true }

                [workspace]
                members = ["crates/*"]

                [workspace.dependencies]
                serde = "1.0"
                tombi-ast = { path = "crates/tombi-ast" }
                "#,
                project_root_path().join("Cargo.toml"),
            ) -> Ok(Some(vec![
                {
                    url: "https://crates.io/crates/serde",
                    range: 11:0..11:5,
                    tooltip: tombi_extension_cargo::DocumentLinkToolTip::CrateIo,
                },
                {
                    path: project_root_path().join("crates/tombi-ast/Cargo.toml"),
                    range: 12:0..12:9,
                    tooltip: tombi_extension_cargo::DocumentLinkToolTip::CargoToml,
                },
                {
                    path: project_root_path().join("crates/tombi-ast/Cargo.toml"),
                    range: 12:22..12:38,
                    tooltip: tombi_extension_cargo::DocumentLinkToolTip::CargoToml,
                },
                {
                    path: project_root_path().join("crates/tombi-ast/Cargo.toml"),
                    range: 8:12..8:20,
                    tooltip: tombi_extension_cargo::DocumentLinkToolTip::CargoTomlFirstMember,
                },
            ]));
        );
    }

    mod tombi_schema {
        use super::*;

        test_document_link!(
            #[tokio::test]
            async fn tombi_schema_catalog_paths(
                r#"
                [schema]
                catalog = { path = "https://json.schemastore.org/api/json/catalog.json" }
                "#,
                project_root_path().join("tombi.toml"),
            ) -> Ok(Some(vec![
                {
                    url: "https://json.schemastore.org/api/json/catalog.json",
                    range: 1:20..1:70,
                    tooltip: tombi_extension_tombi::DocumentLinkToolTip::Catalog,
                }
            ]));
        );

        test_document_link!(
            #[tokio::test]
            async fn tombi_schema_catalog_path(
                r#"
                [schema]
                catalog = { paths = ["https://json.schemastore.org/api/json/catalog.json"] }
                "#,
                project_root_path().join("tombi.toml"),
            ) -> Ok(Some(vec![
                {
                    url: "https://json.schemastore.org/api/json/catalog.json",
                    range: 1:22..1:72,
                    tooltip: tombi_extension_tombi::DocumentLinkToolTip::Catalog,
                }
            ]));
        );

        test_document_link!(
            #[tokio::test]
            async fn tombi_schemas_path(
                r#"
                [[schemas]]
                path = "schemas/tombi.schema.json"
                "#,
                project_root_path().join("tombi.toml"),
            ) -> Ok(Some(vec![
                {
                    path: project_root_path().join("schemas/tombi.schema.json"),
                    range: 1:8..1:33,
                    tooltip: tombi_extension_tombi::DocumentLinkToolTip::Schema,
                }
            ]));
        );

        test_document_link!(
            #[tokio::test]
            async fn tombi_schemas_remote_path(
                r#"
                [[schemas]]
                path = "https://json.schemastore.org/cargo-make.json"
                "#,
                project_root_path().join("tombi.toml"),
            ) -> Ok(Some(vec![
                {
                    url: "https://json.schemastore.org/cargo-make.json",
                    range: 1:8..1:52,
                    tooltip: tombi_extension_tombi::DocumentLinkToolTip::Schema,
                }
            ]));
        );
    }
}

#[macro_export]
macro_rules! test_document_link {
    // Pattern: with url, with tooltip
    (#[tokio::test] async fn $name:ident(
        $source:expr,
        $file_path:expr,
    ) -> Ok(Some(vec![$($document_link:tt),* $(,)?]));) => {
        test_document_link! {
            #[tokio::test] async fn _$name(
                $source,
                $file_path,
            ) -> Ok(Some(vec![
                $(
                    _document_link!($document_link),
                )*
                ]));
        }
    };
    // Fallback: original (for DocumentLink struct literal)
    (#[tokio::test] async fn _$name:ident(
        $source:expr,
        $file_path:expr,
    ) -> Ok($expected_links:expr);) => {
        #[tokio::test]
        async fn $name() -> Result<(), Box<dyn std::error::Error>> {
            // Use handler functions from tombi_lsp
            use tombi_lsp::handler::{handle_did_open, handle_document_link};
            use tombi_lsp::Backend;
            use tower_lsp::{
                lsp_types::{
                    DidOpenTextDocumentParams, DocumentLinkParams, PartialResultParams,
                    TextDocumentIdentifier, TextDocumentItem, Url, WorkDoneProgressParams,
                },
                LspService,
            };

            tombi_test_lib::init_tracing();

            let (service, _) = LspService::new(|client| {
                Backend::new(client, &tombi_lsp::backend::Options::default())
            });

            let backend = service.inner();

            let toml_file_url =
                Url::from_file_path($file_path).expect("failed to convert file path to URL");

            let toml_text = textwrap::dedent($source).trim().to_string();

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

            let params = DocumentLinkParams {
                text_document: TextDocumentIdentifier { uri: toml_file_url },
                work_done_progress_params: WorkDoneProgressParams::default(),
                partial_result_params: PartialResultParams::default(),
            };

            let result = handle_document_link(&backend, params).await;

            tracing::debug!("document_link result: {:#?}", result);

            let result = result.map(|result| {
                result.map(|document_links| {
                    document_links
                        .into_iter()
                        .map(|mut document_link| {
                            document_link.target.as_mut().map(|target| {
                                target.set_fragment(None);
                                target
                            });
                            document_link
                        })
                        .collect::<Vec<_>>()
                })
            });

            pretty_assertions::assert_eq!(result, Ok($expected_links));

            Ok(())
        }
    };
}

#[macro_export]
macro_rules! _document_link {
    ({
        path: $path:expr,
        range: $start_line:literal : $start_char:literal .. $end_line:literal : $end_char:literal $(,)?
    }) => {
        _document_link! ({
            path: $path,
            range: $start_line:$start_char..$end_line:$end_char,
            tooltip: "Open JSON Schema",
        })
    };
    ({
        path: $path:expr,
        range: $start_line:literal : $start_char:literal .. $end_line:literal : $end_char:literal,
        tooltip: $tooltip:expr $(,)?
    }) => {
        tower_lsp::lsp_types::DocumentLink {
            range: tower_lsp::lsp_types::Range {
                start: tower_lsp::lsp_types::Position {
                    line: $start_line,
                    character: $start_char,
                },
                end: tower_lsp::lsp_types::Position {
                    line: $end_line,
                    character: $end_char,
                },
            },
            target: Url::from_file_path($path).ok(),
            tooltip: Some($tooltip.to_string()),
            data: None,
        }
    };
    ({
        url: $url:expr,
        range: $start_line:literal : $start_char:literal .. $end_line:literal : $end_char:literal,
        tooltip: $tooltip:expr $(,)?
    }) => {
        tower_lsp::lsp_types::DocumentLink {
            range: tower_lsp::lsp_types::Range {
                start: tower_lsp::lsp_types::Position {
                    line: $start_line,
                    character: $start_char,
                },
                end: tower_lsp::lsp_types::Position {
                    line: $end_line,
                    character: $end_char,
                },
            },
            target: Url::parse($url).ok(),
            tooltip: Some($tooltip.to_string()),
            data: None,
        }
    };
}
