use tombi_test_lib::{
    cargo_feature_navigation_fixture_path, dot_config_project_root_fixture_path, project_root_path,
};

fn cargo_document_link_all_enabled_config_path() -> std::path::PathBuf {
    project_root_path().join(
        "crates/tombi-lsp/tests/fixtures/extensions/cargo-document-link-all-enabled/tombi.toml",
    )
}

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
                SourcePath(project_root_path().join("Cargo.toml")),
            ) -> Ok(None);
        );

        test_document_link!(
            #[tokio::test]
            async fn cargo_workspace_package_readme_without_schema(
                r#"
                #:schema schemas/Cargo.json

                [workspace.package]
                readme = "README.md"
                "#,
                SourcePath(project_root_path().join("Cargo.toml")),
            ) -> Ok(Some(vec![
                {
                    path: project_root_path().join("schemas/Cargo.json"),
                    range: 0:9..0:27
                }
            ]));
        );

        test_document_link!(
            #[tokio::test]
            async fn cargo_workspace_package_readme_without_schema_with_subschema_fragment(
                r#"
                #:schema file://./schemas/type-test.schema.json#/definitions/TableValue

                [workspace.package]
                readme = "README.md"
                "#,
                SourcePath(project_root_path().join("Cargo.toml")),
            ) -> Ok(Some(vec![
                {
                    path: project_root_path().join("schemas/type-test.schema.json"),
                    range: 0:9..0:71
                }
            ]));
        );

        test_document_link!(
            #[tokio::test]
            async fn cargo_workspace_package_readme_without_schema_with_anchor_fragment(
                r#"
                #:schema file://./schemas/anchor-table-test.schema.json#tableType

                [workspace.package]
                readme = "README.md"
                "#,
                SourcePath(project_root_path().join("Cargo.toml")),
            ) -> Ok(Some(vec![
                {
                    path: project_root_path().join("schemas/anchor-table-test.schema.json"),
                    range: 0:9..0:65
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
                SourcePath(project_root_path().join("Cargo.toml")),
                ConfigPath(cargo_document_link_all_enabled_config_path()),
            ) -> Ok(Some(vec![
                {
                    path: project_root_path().join("crates/tombi-lsp/Cargo.toml"),
                    range: 4:0..4:9,
                    tooltip: tombi_extension_cargo::DocumentLinkToolTip::CargoToml,
                },
                {
                    path: project_root_path().join("crates/tombi-lsp/Cargo.toml"),
                    range: 4:18..4:34,
                    tooltip: tombi_extension_cargo::DocumentLinkToolTip::PathFile,
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
                SourcePath(project_root_path().join("Cargo.toml")),
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
                SourcePath(project_root_path().join("Cargo.toml")),
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
                SourcePath(project_root_path().join("Cargo.toml")),
                ConfigPath(cargo_document_link_all_enabled_config_path()),
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
                SourcePath(project_root_path().join("rust/tombi-cli/Cargo.toml")),
                ConfigPath(cargo_document_link_all_enabled_config_path()),
            ) -> Ok(Some(vec![
                {
                    path: project_root_path().join("crates/tombi-lsp/Cargo.toml"),
                    range: 4:0..4:9,
                    tooltip: tombi_extension_cargo::DocumentLinkToolTip::CargoToml,
                },
                {
                    path: project_root_path().join("crates/tombi-lsp/Cargo.toml"),
                    range: 4:18..4:40,
                    tooltip: tombi_extension_cargo::DocumentLinkToolTip::PathFile,
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
                SourcePath(project_root_path().join("subcrate/Cargo.toml")),
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
                SourcePath(project_root_path().join("subcrate/Cargo.toml")),
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
                SourcePath(project_root_path().join("subcrate/Cargo.toml")),
                ConfigPath(cargo_document_link_all_enabled_config_path()),
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
                SourcePath(project_root_path().join("subcrate/Cargo.toml")),
                ConfigPath(cargo_document_link_all_enabled_config_path()),
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
            async fn cargo_workspace_local_dependency_ignores_cargo_toml_setting(
                r#"
                [dependencies]
                member = { workspace = true }
                "#,
                SourcePath(project_root_path().join(
                    "crates/tombi-lsp/tests/fixtures/extensions/cargo-document-link-workspace-local-cargo-toml-disabled/consumer/Cargo.toml"
                )),
            ) -> Ok(Some(vec![
                {
                    path: project_root_path().join(
                        "crates/tombi-lsp/tests/fixtures/extensions/cargo-document-link-workspace-local-cargo-toml-disabled/Cargo.toml"
                    ),
                    range: 1:11..1:27,
                    tooltip: tombi_extension_cargo::DocumentLinkToolTip::WorkspaceCargoToml,
                }
            ]));
        );

        test_document_link!(
            #[tokio::test]
            async fn cargo_workspace_local_dependency_links_disabled_by_workspace_setting(
                r#"
                [dependencies]
                member = { workspace = true }
                "#,
                SourcePath(project_root_path().join(
                    "crates/tombi-lsp/tests/fixtures/extensions/cargo-document-link-workspace-disabled/consumer/Cargo.toml"
                )),
            ) -> Ok(Some(vec![
                {
                    path: project_root_path().join(
                        "crates/tombi-lsp/tests/fixtures/extensions/cargo-document-link-workspace-disabled/member/Cargo.toml"
                    ),
                    range: 1:0..1:6,
                    tooltip: tombi_extension_cargo::DocumentLinkToolTip::CargoToml,
                }
            ]));
        );

        test_document_link!(
            #[tokio::test]
            async fn cargo_workspace_renamed_local_dependency_links_to_member(
                r#"
                [dependencies]
                sev = { workspace = true }
                "#,
                SourcePath(project_root_path().join(
                    "crates/tombi-lsp/tests/fixtures/extensions/cargo-document-link-workspace-renamed/consumer/Cargo.toml"
                )),
                ConfigPath(cargo_document_link_all_enabled_config_path()),
            ) -> Ok(Some(vec![
                {
                    path: project_root_path().join(
                        "crates/tombi-lsp/tests/fixtures/extensions/cargo-document-link-workspace-renamed/provider/Cargo.toml"
                    ),
                    range: 1:0..1:3,
                    tooltip: tombi_extension_cargo::DocumentLinkToolTip::CargoToml,
                },
                {
                    path: project_root_path().join(
                        "crates/tombi-lsp/tests/fixtures/extensions/cargo-document-link-workspace-renamed/Cargo.toml"
                    ),
                    range: 1:8..1:24,
                    tooltip: tombi_extension_cargo::DocumentLinkToolTip::WorkspaceCargoToml,
                }
            ]));
        );

        test_document_link!(
            #[tokio::test]
            async fn cargo_workspace_path_dependency_has_no_links_when_path_disabled(
                r#"
                [workspace.dependencies]
                member = { path = "member" }
                "#,
                SourcePath(project_root_path().join(
                    "crates/tombi-lsp/tests/fixtures/extensions/cargo-document-link-path-disabled/Cargo.toml"
                )),
            ) -> Ok(None);
        );

        test_document_link!(
            #[tokio::test]
            async fn cargo_package_workspace(
                r#"
                [package]
                workspace = "../../"
                "#,
                SourcePath(project_root_path().join("crates/tombi-lsp/Cargo.toml")),
                ConfigPath(cargo_document_link_all_enabled_config_path()),
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
                SourcePath(project_root_path().join("Cargo.toml")),
                ConfigPath(cargo_document_link_all_enabled_config_path()),
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
                    tooltip: tombi_extension_cargo::DocumentLinkToolTip::PathFile,
                },
                {
                    path: project_root_path().join("crates/tombi-accessor/Cargo.toml"),
                    range: 8:12..8:20,
                    tooltip: tombi_extension_cargo::DocumentLinkToolTip::CargoTomlFirstMember,
                },
            ]));
        );

        test_document_link!(
            #[tokio::test]
            async fn cargo_bin_path_links_to_target(
                r#"
                [[bin]]
                name = "profile"
                path = "src/bin/profile.rs"
                "#,
                SourcePath(project_root_path().join("crates/tombi-glob/Cargo.toml")),
                ConfigPath(cargo_document_link_all_enabled_config_path()),
            ) -> Ok(Some(vec![
                {
                    path: project_root_path().join("crates/tombi-glob/src/bin/profile.rs"),
                    range: 2:8..2:26,
                    tooltip: tombi_extension_cargo::DocumentLinkToolTip::PathFile,
                }
            ]));
        );

        // Tests for platform specific dependencies (Issue #1192)
        test_document_link!(
            #[tokio::test]
            async fn cargo_target_dependencies_serde(
                r#"
                [target.'cfg(unix)'.dependencies]
                serde = "1.0"
                "#,
                SourcePath(project_root_path().join("crates/subcrate/Cargo.toml")),
            ) -> Ok(Some(vec![
                {
                    url: "https://crates.io/crates/serde",
                    range: 1:0..1:5,
                    tooltip: tombi_extension_cargo::DocumentLinkToolTip::CrateIo,
                }
            ]));
        );

        test_document_link!(
            #[tokio::test]
            async fn cargo_target_dependencies_with_workspace(
                r#"
                [target.'cfg(unix)'.dependencies]
                serde = { workspace = true }
                "#,
                SourcePath(project_root_path().join("crates/subcrate/Cargo.toml")),
                ConfigPath(cargo_document_link_all_enabled_config_path()),
            ) -> Ok(Some(vec![
                {
                    url: "https://crates.io/crates/serde",
                    range: 1:0..1:5,
                    tooltip: tombi_extension_cargo::DocumentLinkToolTip::CrateIo,
                },
                {
                    path: project_root_path().join("Cargo.toml"),
                    range: 1:10..1:26,
                    tooltip: tombi_extension_cargo::DocumentLinkToolTip::WorkspaceCargoToml,
                }
            ]));
        );

        test_document_link!(
            #[tokio::test]
            async fn cargo_target_dependencies_with_path(
                r#"
                [package]
                name = "test"

                [target.'cfg(unix)'.dependencies]
                tombi-ast = { path = "../tombi-ast" }
                "#,
                SourcePath(project_root_path().join("crates/tombi-lsp/Cargo.toml")),
                ConfigPath(cargo_document_link_all_enabled_config_path()),
            ) -> Ok(Some(vec![
                {
                    path: project_root_path().join("crates/tombi-ast/Cargo.toml"),
                    range: 4:0..4:9,
                    tooltip: tombi_extension_cargo::DocumentLinkToolTip::CargoToml,
                },
                {
                    path: project_root_path().join("crates/tombi-ast/Cargo.toml"),
                    range: 4:22..4:34,
                    tooltip: tombi_extension_cargo::DocumentLinkToolTip::PathFile,
                }
            ]));
        );

        test_document_link!(
            #[tokio::test]
            async fn cargo_feature_local_reference(
                r#"
                [package]
                name = "explicit-feature"
                version = "0.1.0"

                [dependencies]
                schemars = { version = "1.0", optional = true }

                [features]
                local = []
                bundle = ["local", "schemars", "dep:schemars"]
                "#,
                SourcePath(cargo_feature_navigation_fixture_path().join("explicit/Cargo.toml")),
                ConfigPath(cargo_document_link_all_enabled_config_path()),
            ) -> Ok(Some(vec![
                {
                    url: "https://crates.io/crates/schemars",
                    range: 5:0..5:8,
                    tooltip: tombi_extension_cargo::DocumentLinkToolTip::CrateIo,
                },
                {
                    path: cargo_feature_navigation_fixture_path().join("explicit/Cargo.toml"),
                    range: 9:11..9:16,
                    tooltip: tombi_extension_cargo::DocumentLinkToolTip::CargoToml,
                }
            ]));
        );

        test_document_link!(
            #[tokio::test]
            async fn cargo_feature_implicit_optional_reference(
                r#"
                [package]
                name = "implicit-feature"
                version = "0.1.0"

                [dependencies]
                schemars = { version = "1.0", optional = true }

                [features]
                bundle = ["schemars"]
                "#,
                SourcePath(cargo_feature_navigation_fixture_path().join("implicit/Cargo.toml")),
                ConfigPath(cargo_document_link_all_enabled_config_path()),
            ) -> Ok(Some(vec![
                {
                    url: "https://crates.io/crates/schemars",
                    range: 5:0..5:8,
                    tooltip: tombi_extension_cargo::DocumentLinkToolTip::CrateIo,
                },
                {
                    path: cargo_feature_navigation_fixture_path().join("implicit/Cargo.toml"),
                    range: 8:11..8:19,
                    tooltip: tombi_extension_cargo::DocumentLinkToolTip::CargoToml,
                }
            ]));
        );

        test_document_link!(
            #[tokio::test]
            async fn cargo_feature_dependency_reference(
                r#"
                [package]
                name = "consumer"
                version = "0.1.0"

                [dependencies]
                provider = { workspace = true, features = ["jsonschema"] }

                [features]
                local = ["provider/jsonschema"]
                "#,
                SourcePath(
                    cargo_feature_navigation_fixture_path().join("workspace/consumer/Cargo.toml")
                ),
                ConfigPath(cargo_document_link_all_enabled_config_path()),
            ) -> Ok(Some(vec![
                {
                    path: cargo_feature_navigation_fixture_path().join("workspace/provider/Cargo.toml"),
                    range: 5:0..5:8,
                    tooltip: tombi_extension_cargo::DocumentLinkToolTip::CargoToml,
                },
                {
                    path: cargo_feature_navigation_fixture_path().join("workspace/Cargo.toml"),
                    range: 5:13..5:29,
                    tooltip: tombi_extension_cargo::DocumentLinkToolTip::WorkspaceCargoToml,
                },
                {
                    path: cargo_feature_navigation_fixture_path().join("workspace/provider/Cargo.toml"),
                    range: 8:10..8:29,
                    tooltip: tombi_extension_cargo::DocumentLinkToolTip::CargoToml,
                },
                {
                    path: cargo_feature_navigation_fixture_path().join("workspace/provider/Cargo.toml"),
                    range: 5:44..5:54,
                    tooltip: tombi_extension_cargo::DocumentLinkToolTip::CargoToml,
                }
            ]));
        );

        test_document_link!(
            #[tokio::test]
            async fn cargo_feature_weak_and_renamed_references(
                r#"
                [package]
                name = "weak-consumer"
                version = "0.1.0"

                [dependencies]
                provider = { path = "../provider" }

                [features]
                weak = ["provider?/jsonschema"]
                "#,
                SourcePath(
                    cargo_feature_navigation_fixture_path().join("workspace/weak-consumer/Cargo.toml")
                ),
                ConfigPath(cargo_document_link_all_enabled_config_path()),
            ) -> Ok(Some(vec![
                {
                    path: cargo_feature_navigation_fixture_path().join("workspace/provider/Cargo.toml"),
                    range: 5:0..5:8,
                    tooltip: tombi_extension_cargo::DocumentLinkToolTip::CargoToml,
                },
                {
                    path: cargo_feature_navigation_fixture_path().join("workspace/provider/Cargo.toml"),
                    range: 5:21..5:32,
                    tooltip: tombi_extension_cargo::DocumentLinkToolTip::PathFile,
                },
                {
                    path: cargo_feature_navigation_fixture_path().join("workspace/provider/Cargo.toml"),
                    range: 8:9..8:29,
                    tooltip: tombi_extension_cargo::DocumentLinkToolTip::CargoToml,
                }
            ]));
        );

        test_document_link!(
            #[tokio::test]
            async fn cargo_path_dependency_hides_cargo_toml_links_when_cargo_toml_disabled(
                r#"
                [workspace.dependencies]
                member = { path = "member" }
                "#,
                SourcePath(project_root_path().join(
                    "crates/tombi-lsp/tests/fixtures/extensions/cargo-document-link-cargo-toml-disabled/Cargo.toml"
                )),
            ) -> Ok(Some(vec![
                {
                    path: project_root_path().join(
                        "crates/tombi-lsp/tests/fixtures/extensions/cargo-document-link-cargo-toml-disabled/member/Cargo.toml"
                    ),
                    range: 1:19..1:25,
                    tooltip: tombi_extension_cargo::DocumentLinkToolTip::PathFile,
                }
            ]));
        );

        test_document_link!(
            #[tokio::test]
            async fn cargo_path_dependency_links_disabled_by_path_setting(
                r#"
                [workspace.dependencies]
                member = { path = "member" }
                "#,
                SourcePath(project_root_path().join(
                    "crates/tombi-lsp/tests/fixtures/extensions/cargo-document-link-path-disabled/Cargo.toml"
                )),
            ) -> Ok(None);
        );

        test_document_link!(
            #[tokio::test]
            async fn cargo_path_dependency_links_ignore_crates_io_setting(
                r#"
                [workspace.dependencies]
                member = { path = "member" }
                "#,
                SourcePath(project_root_path().join(
                    "crates/tombi-lsp/tests/fixtures/extensions/cargo-document-link-crates-io-disabled/Cargo.toml"
                )),
            ) -> Ok(Some(vec![
                {
                    path: project_root_path().join(
                        "crates/tombi-lsp/tests/fixtures/extensions/cargo-document-link-crates-io-disabled/member/Cargo.toml"
                    ),
                    range: 1:0..1:6,
                    tooltip: tombi_extension_cargo::DocumentLinkToolTip::CargoToml,
                },
                {
                    path: project_root_path().join(
                        "crates/tombi-lsp/tests/fixtures/extensions/cargo-document-link-crates-io-disabled/member/Cargo.toml"
                    ),
                    range: 1:19..1:25,
                    tooltip: tombi_extension_cargo::DocumentLinkToolTip::PathFile,
                }
            ]));
        );

        test_document_link!(
            #[tokio::test]
            async fn cargo_rust_source_links_ignore_cargo_toml_setting(
                r#"
                [package]
                name = "app"
                version = "0.1.0"

                [[bin]]
                name = "tool"
                path = "src/bin/tool.rs"
                "#,
                SourcePath(project_root_path().join(
                    "crates/tombi-lsp/tests/fixtures/extensions/cargo-document-link-cargo-toml-disabled-bin/Cargo.toml"
                )),
            ) -> Ok(Some(vec![
                {
                    path: project_root_path().join(
                        "crates/tombi-lsp/tests/fixtures/extensions/cargo-document-link-cargo-toml-disabled-bin/src/bin/tool.rs"
                    ),
                    range: 6:8..6:23,
                    tooltip: tombi_extension_cargo::DocumentLinkToolTip::PathFile,
                }
            ]));
        );

        test_document_link!(
            #[tokio::test]
            async fn cargo_git_dependency_links_disabled_by_git_setting(
                r#"
                [package]
                name = "test"
                version = "0.1.0"

                [dependencies]
                serde = { git = "https://github.com/serde-rs/serde" }
                "#,
                SourcePath(project_root_path().join(
                    "crates/tombi-lsp/tests/fixtures/extensions/cargo-document-link-git-disabled/Cargo.toml"
                )),
            ) -> Ok(Some(vec![
                {
                    url: "https://crates.io/crates/serde",
                    range: 5:0..5:5,
                    tooltip: tombi_extension_cargo::DocumentLinkToolTip::CrateIo,
                }
            ]));
        );

        test_document_link!(
            #[tokio::test]
            async fn cargo_default_features_produce_no_document_links(
                r#"
                [workspace.dependencies]
                member = { path = "member" }
                "#,
                SourcePath(project_root_path().join(
                    "crates/tombi-lsp/tests/fixtures/extensions/cargo-document-link-default/Cargo.toml"
                )),
            ) -> Ok(None);
        );
    }

    mod tombi_schema {
        use super::*;

        test_document_link!(
            #[tokio::test]
            async fn tombi_schemas_path_from_dot_config_tombi_toml(
                r#"
                [[schemas]]
                path = "schemas/name.schema.json"
                "#,
                SourcePath(dot_config_project_root_fixture_path().join(".config/tombi.toml")),
            ) -> Ok(Some(vec![
                {
                    path: dot_config_project_root_fixture_path().join("schemas/name.schema.json"),
                    range: 1:8..1:32,
                    tooltip: tombi_extension_tombi::DocumentLinkToolTip::Schema,
                }
            ]));
        );

        test_document_link!(
            #[tokio::test]
            async fn tombi_schema_catalog_paths(
                r#"
                [schema]
                catalog = { path = "https://www.schemastore.org/api/json/catalog.json" }
                "#,
                SourcePath(project_root_path().join("tombi.toml")),
            ) -> Ok(Some(vec![
                {
                    url: "https://www.schemastore.org/api/json/catalog.json",
                    range: 1:20..1:69,
                    tooltip: tombi_extension_tombi::DocumentLinkToolTip::Catalog,
                }
            ]));
        );

        test_document_link!(
            #[tokio::test]
            async fn tombi_schema_catalog_path(
                r#"
                [schema]
                catalog = { paths = ["https://www.schemastore.org/api/json/catalog.json"] }
                "#,
                SourcePath(project_root_path().join("tombi.toml")),
            ) -> Ok(Some(vec![
                {
                    url: "https://www.schemastore.org/api/json/catalog.json",
                    range: 1:22..1:71,
                    tooltip: tombi_extension_tombi::DocumentLinkToolTip::Catalog,
                }
            ]));
        );

        test_document_link!(
            #[tokio::test]
            async fn tombi_schema_catalog_paths_local_path(
                r#"
                [schema]
                catalog = { paths = ["schemas/type-test.schema.json"] }
                "#,
                SourcePath(project_root_path().join("tombi.toml")),
            ) -> Ok(Some(vec![
                {
                    path: project_root_path().join("schemas/type-test.schema.json"),
                    range: 1:22..1:51,
                    tooltip: tombi_extension_tombi::DocumentLinkToolTip::Catalog,
                }
            ]));
        );

        test_document_link!(
            #[tokio::test]
            async fn tombi_schema_catalog_paths_local_path_from_dot_config_tombi_toml(
                r#"
                [schema]
                catalog = { paths = ["schemas/name.schema.json"] }
                "#,
                SourcePath(dot_config_project_root_fixture_path().join(".config/tombi.toml")),
            ) -> Ok(Some(vec![
                {
                    path: dot_config_project_root_fixture_path().join("schemas/name.schema.json"),
                    range: 1:22..1:46,
                    tooltip: tombi_extension_tombi::DocumentLinkToolTip::Catalog,
                }
            ]));
        );

        test_document_link!(
            #[tokio::test]
            async fn tombi_schema_catalog_path_disabled_by_extensions(
                r#"
                [schema]
                catalog = { paths = ["https://www.schemastore.org/api/json/catalog.json"] }
                "#,
                SourcePath(project_root_path().join(
                    "crates/tombi-lsp/tests/fixtures/extensions/tombi-disabled/tombi.toml"
                )),
            ) -> Ok(None);
        );

        test_document_link!(
            #[tokio::test]
            async fn tombi_schemas_path(
                r#"
                [[schemas]]
                path = "www.schemastore.org/tombi.json"
                "#,
                SourcePath(project_root_path().join("tombi.toml")),
            ) -> Ok(Some(vec![
                {
                    path: project_root_path().join("www.schemastore.org/tombi.json"),
                    range: 1:8..1:38,
                    tooltip: tombi_extension_tombi::DocumentLinkToolTip::Schema,
                }
            ]));
        );

        test_document_link!(
            #[tokio::test]
            async fn tombi_schemas_remote_path(
                r#"
                [[schemas]]
                path = "https://www.schemastore.org/cargo-make.json"
                "#,
                SourcePath(project_root_path().join("tombi.toml")),
            ) -> Ok(Some(vec![
                {
                    url: "https://www.schemastore.org/cargo-make.json",
                    range: 1:8..1:51,
                    tooltip: tombi_extension_tombi::DocumentLinkToolTip::Schema,
                }
            ]));
        );
    }

    mod pyproject_schema {
        use super::*;

        test_document_link!(
            #[tokio::test]
            async fn pyproject_dependencies_disabled_by_extensions(
                r#"
                [project]
                dependencies = ["anyio>=4.0"]
                "#,
                SourcePath(project_root_path().join(
                    "crates/tombi-lsp/tests/fixtures/extensions/pyproject-disabled/pyproject.toml"
                )),
            ) -> Ok(None);
        );

        test_document_link!(
            #[tokio::test]
            async fn pyproject_workspace_member_links_ignore_pypi_setting(
                r#"
                [project]
                dependencies = ["member", "anyio>=4.0"]

                [tool.uv.workspace]
                members = ["member"]

                [tool.uv.sources]
                member = { workspace = true }
                "#,
                SourcePath(project_root_path().join(
                    "crates/tombi-lsp/tests/fixtures/extensions/pyproject-document-link-pypi-disabled/pyproject.toml"
                )),
            ) -> Ok(Some(vec![
                {
                    path: project_root_path().join(
                        "crates/tombi-lsp/tests/fixtures/extensions/pyproject-document-link-pypi-disabled/member/pyproject.toml"
                    ),
                    range: 4:12..4:18,
                    tooltip: "Open pyproject.toml",
                },
                {
                    path: project_root_path().join(
                        "crates/tombi-lsp/tests/fixtures/extensions/pyproject-document-link-pypi-disabled/member/pyproject.toml"
                    ),
                    range: 7:0..7:6,
                    tooltip: "Open pyproject.toml",
                },
                {
                    path: project_root_path().join(
                        "crates/tombi-lsp/tests/fixtures/extensions/pyproject-document-link-pypi-disabled/pyproject.toml"
                    ),
                    range: 7:11..7:27,
                    tooltip: "Open Workspace pyproject.toml",
                },
                {
                    path: project_root_path().join(
                        "crates/tombi-lsp/tests/fixtures/extensions/pyproject-document-link-pypi-disabled/member/pyproject.toml"
                    ),
                    range: 1:17..1:23,
                    tooltip: "Open pyproject.toml",
                }
            ]));
        );

        test_document_link!(
            #[tokio::test]
            async fn pyproject_local_dependency_links_disabled_by_pyproject_setting(
                r#"
                [project]
                dependencies = ["member", "anyio>=4.0"]

                [tool.uv.workspace]
                members = ["member"]

                [tool.uv.sources]
                member = { workspace = true }
                "#,
                SourcePath(project_root_path().join(
                    "crates/tombi-lsp/tests/fixtures/extensions/pyproject-document-link-pyproject-disabled/pyproject.toml"
                )),
            ) -> Ok(Some(vec![
                {
                    url: "https://pypi.org/project/anyio/",
                    range: 1:27..1:37,
                    tooltip: "Open PyPI Package",
                }
            ]));
        );

        test_document_link!(
            #[tokio::test]
            async fn pyproject_tool_uv_dependency_lists(
                r#"
                [tool.uv]
                dev-dependencies = ["ruff>=0.7"]
                constraint-dependencies = ["pytest<9"]
                override-dependencies = ["werkzeug==2.3.0"]
                build-constraint-dependencies = ["setuptools==60.0.0"]
                "#,
                SourcePath(project_root_path().join("pyproject.toml")),
            ) -> Ok(Some(vec![
                {
                    url: "https://pypi.org/project/ruff/",
                    range: 1:21..1:30,
                    tooltip: "Open PyPI Package",
                },
                {
                    url: "https://pypi.org/project/pytest/",
                    range: 2:28..2:36,
                    tooltip: "Open PyPI Package",
                },
                {
                    url: "https://pypi.org/project/werkzeug/",
                    range: 3:26..3:41,
                    tooltip: "Open PyPI Package",
                },
                {
                    url: "https://pypi.org/project/setuptools/",
                    range: 4:34..4:52,
                    tooltip: "Open PyPI Package",
                }
            ]));
        );

        test_document_link!(
            #[tokio::test]
            async fn pyproject_build_system_requires(
                r#"
                [build-system]
                requires = [
                  "maturin>=1.5,<2.0",
                ]
                "#,
                SourcePath(project_root_path().join("pyproject.toml")),
            ) -> Ok(Some(vec![
                {
                    url: "https://pypi.org/project/maturin/",
                    range: 2:3..2:20,
                    tooltip: "Open PyPI Package",
                }
            ]));
        );

        test_document_link!(
            #[tokio::test]
            async fn pyproject_default_features_disable_pyproject_toml_links(
                r#"
                [project]
                dependencies = ["member", "anyio>=4.0"]

                [tool.uv.workspace]
                members = ["member"]

                [tool.uv.sources]
                member = { workspace = true }
                "#,
                SourcePath(project_root_path().join(
                    "crates/tombi-lsp/tests/fixtures/extensions/pyproject-document-link-default/pyproject.toml"
                )),
            ) -> Ok(Some(vec![
                {
                    url: "https://pypi.org/project/anyio/",
                    range: 1:27..1:37,
                    tooltip: "Open PyPI Package",
                }
            ]));
        );
    }

    mod tombi_schema_default {
        use super::*;

        test_document_link!(
            #[tokio::test]
            async fn tombi_default_features_produce_no_path_document_links(
                r#"
                [schema]
                catalog = { path = "https://www.schemastore.org/api/json/catalog.json" }
                "#,
                SourcePath(project_root_path().join(
                    "crates/tombi-lsp/tests/fixtures/extensions/tombi-document-link-default/tombi.toml"
                )),
            ) -> Ok(None);
        );
    }
}

#[macro_export]
macro_rules! test_document_link {
    (#[tokio::test] async fn $name:ident(
        $source:expr $(, $arg:expr )* $(,)?
    ) -> Ok(None);) => {
        test_document_link! {
            #[tokio::test] async fn _$name(
                $source $(, $arg)*
            ) -> Ok(None);
        }
    };
    // Pattern: with url, with tooltip
    (#[tokio::test] async fn $name:ident(
        $source:expr $(, $arg:expr )* $(,)?
    ) -> Ok(Some(vec![$($document_link:tt),* $(,)?]));) => {
        test_document_link! {
            #[tokio::test] async fn _$name(
                $source $(, $arg)*
            ) -> Ok(Some(vec![
                $(
                    _document_link!($document_link),
                )*
                ]));
        }
    };
    // Fallback: original (for DocumentLink struct literal)
    (#[tokio::test] async fn _$name:ident(
        $source:expr $(, $arg:expr )* $(,)?
    ) -> Ok($expected_links:expr);) => {
        #[tokio::test]
        async fn $name() -> Result<(), Box<dyn std::error::Error>> {
            // Use handler functions from tombi_lsp
            use itertools::Itertools;
            use tombi_lsp::handler::{handle_did_open, handle_document_link};
            use tombi_lsp::Backend;
            use tower_lsp::{
                lsp_types::{
                    DidOpenTextDocumentParams, DocumentLinkParams, PartialResultParams,
                    TextDocumentIdentifier, TextDocumentItem, Url, WorkDoneProgressParams,
                },
                LspService,
            };

            tombi_test_lib::init_log();

            #[allow(unused)]
            #[derive(Default)]
            struct TestArgs {
                source_file_path: Option<std::path::PathBuf>,
                schema_file_path: Option<std::path::PathBuf>,
                subschemas: Vec<SubSchemaPath>,
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
            struct SchemaPath(std::path::PathBuf);

            impl ApplyTestArg for SchemaPath {
                fn apply(self, args: &mut TestArgs) {
                    args.schema_file_path = Some(self.0);
                }
            }

            #[allow(unused)]
            struct SubSchemaPath {
                pub root: String,
                pub path: std::path::PathBuf,
            }

            impl ApplyTestArg for SubSchemaPath {
                fn apply(self, args: &mut TestArgs) {
                    args.subschemas.push(self);
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
            let mut schema_items = Vec::new();

            let _temp_file = tempfile::NamedTempFile::with_suffix_in(
                ".toml",
                std::env::current_dir().expect("failed to get current directory"),
            )
            .expect("failed to create temporary file for test document path");

            let source_path = match args.source_file_path.as_ref() {
                Some(path) => path,
                None => return Err("SourcePath(..) is required".into()),
            };

            if let Some(config_file_path) = args.config_file_path.as_ref() {
                let config_content = std::fs::read_to_string(config_file_path).map_err(|e| {
                    format!(
                        "failed to read config file {}: {}",
                        config_file_path.display(),
                        e
                    )
                })?;
                let config: tombi_config::Config =
                    serde_tombi::from_str_async(&config_content).await.map_err(|e| {
                        format!(
                            "failed to parse config file {}: {}",
                            config_file_path.display(),
                            e
                        )
                    })?;

                let config_schema_store = backend
                    .config_manager
                    .config_schema_store_for_file(source_path)
                    .await;

                if let Some(config_path) = config_schema_store.config_path {
                    backend
                        .config_manager
                        .update_config_with_path(config, &config_path)
                        .await
                        .map_err(|e| {
                            format!(
                                "failed to update config {}: {}",
                                config_path.display(),
                                e
                            )
                        })?;
                } else {
                    backend.config_manager.update_editor_config(config).await;
                }
            }

            if let Some(schema_file_path) = args.schema_file_path.as_ref() {
                let schema_uri = tombi_schema_store::SchemaUri::from_file_path(schema_file_path)
                    .expect(
                        format!(
                            "failed to convert schema path to URL: {}",
                            schema_file_path.display()
                        )
                        .as_str(),
                    );

                schema_items.push(tombi_config::SchemaItem::Root(tombi_config::RootSchema {
                    toml_version: None,
                    path: schema_uri.to_string(),
                    include: vec!["*.toml".into()],
                    exclude: None,
                    lint: None,
                    format: None,
                    overrides: None,
                }));
            }

            for subschema in &args.subschemas {
                let subschema_uri = tombi_schema_store::SchemaUri::from_file_path(&subschema.path)
                    .expect(
                        format!(
                            "failed to convert subschema path to URL: {}",
                            subschema.path.display()
                        )
                        .as_str(),
                    );

                schema_items.push(tombi_config::SchemaItem::Sub(tombi_config::SubSchema {
                    path: subschema_uri.to_string(),
                    include: vec!["*.toml".into()],
                    exclude: None,
                    root: subschema.root.clone(),
                    lint: None,
                    format: None,
                    overrides: None,
                }));
            }

            if !schema_items.is_empty() {
                let config_schema_store = backend
                    .config_manager
                    .config_schema_store_for_file(source_path)
                    .await;

                let mut test_config = config_schema_store.config;
                let mut existing_schemas = test_config.schemas.take().unwrap_or_default();
                existing_schemas.extend(schema_items);
                test_config.schemas = Some(existing_schemas);

                if let Some(config_path) = config_schema_store.config_path {
                    backend
                        .config_manager
                        .update_config_with_path(test_config, &config_path)
                        .await
                        .map_err(|e| {
                            format!(
                                "failed to update config {}: {}",
                                config_path.display(),
                                e
                            )
                        })?;
                } else {
                    backend.config_manager.update_editor_config(test_config).await;
                }
            }

            let toml_file_url = Url::from_file_path(source_path)
                .expect("failed to convert source file path to URL");

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

            log::debug!("document_link result: {:#?}", result);

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
                        .collect_vec()
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
