use tombi_test_lib::{
    cargo_feature_navigation_fixture_path, dot_config_project_root_fixture_path, project_root_path,
};

mod goto_definition_tests {
    use super::*;

    mod document_schema {
        use super::*;

        test_goto_definition!(
            #[tokio::test]
            async fn relative_schema_path(
                r#"
                #:schema ./www.schemastore.org/tombi.json█

                toml-version = "v1.0.0"
                "#,
                SourcePath(project_root_path().join("tombi.toml")),
            ) -> Ok([project_root_path().join("www.schemastore.org/tombi.json")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn ignores_https_schema_uri(
                r#"
                #:schema https://www.schemastore.org/tombi.json█

                toml-version = "v1.0.0"
                "#,
                SourcePath(project_root_path().join("tombi.toml")),
            ) -> Ok(["https://www.schemastore.org/tombi.json"]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn ignores_missing_relative_path(
                r#"
                #:schema schemas/does-not-exist.json█

                toml-version = "v1.0.0"
                "#,
                SourcePath(project_root_path().join("tombi.toml")),
            ) -> Ok([]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn relative_schema_path_with_subschema_fragment(
                r#"
                #:schema file://./schemas/type-test.schema.json#/definitions/TableValue█

                boolean = true
                integer = 1
                "#,
                SourcePath(project_root_path().join("tombi.toml")),
            ) -> Ok([{
                let mut uri = tombi_uri::Uri::from_file_path(
                    project_root_path().join("schemas/type-test.schema.json")
                ).unwrap();
                uri.set_fragment(Some("/definitions/TableValue"));
                uri
            }]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn relative_schema_path_with_anchor_fragment(
                r#"
                #:schema file://./schemas/anchor-table-test.schema.json#tableType█

                boolean = true
                "#,
                SourcePath(project_root_path().join("tombi.toml")),
            ) -> Ok([{
                let mut uri = tombi_uri::Uri::from_file_path(
                    project_root_path().join("schemas/anchor-table-test.schema.json")
                ).unwrap();
                uri.set_fragment(Some("tableType"));
                uri
            }]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn parent_relative_schema_path_with_subschema_fragment(
                r#"
                #:schema file://../../schemas/type-test.schema.json#/definitions/TableValue█

                boolean = true
                integer = 1
                "#,
                SourcePath(project_root_path().join("crates/tombi-lsp/Cargo.toml")),
            ) -> Ok([{
                let mut uri = tombi_uri::Uri::from_file_path(
                    project_root_path().join("schemas/type-test.schema.json")
                ).unwrap();
                uri.set_fragment(Some("/definitions/TableValue"));
                uri
            }]);
        );
    }

    mod cargo_schema {
        use super::*;

        test_goto_definition!(
            #[tokio::test]
            async fn dependencies_serde_workspace(
                r#"
                [dependencies]
                serde = { workspace█ = true }
                "#,
                SourcePath(project_root_path().join("crates/test-crate/Cargo.toml")),
            ) -> Ok([project_root_path().join("Cargo.toml")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn dependencies_serde_with_workspace(
                r#"
                [dependencies]
                serde█ = { workspace = true }
                "#,
                SourcePath(project_root_path().join("crates/test-crate/Cargo.toml")),
            ) -> Ok([project_root_path().join("Cargo.toml")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn dependencies_tombi_ast_workspace(
                r#"
                [dependencies]
                tombi-ast = { workspace█ = true }
                "#,
                SourcePath(project_root_path().join("crates/test-crate/Cargo.toml")),
            ) -> Ok([project_root_path().join("Cargo.toml")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn dependencies_tombi_ast(
                r#"
                [dependencies]
                tombi-ast█ = { workspace = true }
                "#,
                SourcePath(project_root_path().join("crates/test-crate/Cargo.toml")),
            ) -> Ok([project_root_path().join("crates/tombi-ast/Cargo.toml")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn dev_dependencies_rstest_workspace(
                r#"
                [dev-dependencies]
                rstest = { workspace█ = true }
                "#,
                SourcePath(project_root_path().join("crates/test-crate/Cargo.toml")),
            ) -> Ok([project_root_path().join("Cargo.toml")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn build_dependencies_rstest_workspace(
                r#"
                [build-dependencies]
                serde = { workspace█ = true }
                "#,
                SourcePath(project_root_path().join("crates/test-crate/Cargo.toml")),
            ) -> Ok([project_root_path().join("Cargo.toml")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn dev_dependencies_tombi_ast_with_workspace(
                r#"
                [dev-dependencies]
                tombi-ast█ = { workspace = true }
                "#,
                SourcePath(project_root_path().join("crates/test-crate/Cargo.toml")),
            ) -> Ok([project_root_path().join("crates/tombi-ast/Cargo.toml")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn dev_dependencies_tombi_ast_workspace(
                r#"
                [dev-dependencies]
                tombi-ast = { workspace█ = true }
                "#,
                SourcePath(project_root_path().join("crates/test-crate/Cargo.toml")),
            ) -> Ok([project_root_path().join("Cargo.toml")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn build_dependencies_tombi_ast_workspace(
                r#"
                [build-dependencies]
                tombi-ast = { workspace█ = true }
                "#,
                SourcePath(project_root_path().join("crates/test-crate/Cargo.toml")),
            ) -> Ok([project_root_path().join("Cargo.toml")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn package_version_workspace(
                r#"
                [package]
                version.workspace█ = true
                "#,
                SourcePath(project_root_path().join("crates/tombi-future/Cargo.toml")),
            ) -> Ok([project_root_path().join("Cargo.toml")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn package_authors_workspace(
                r#"
                [package]
                authors.workspace█ = true
                "#,
                SourcePath(project_root_path().join("crates/tombi-future/Cargo.toml")),
            ) -> Ok([project_root_path().join("Cargo.toml")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn package_edition_workspace(
                r#"
                [package]
                edition.workspace█ = true
                "#,
                SourcePath(project_root_path().join("crates/tombi-future/Cargo.toml")),
            ) -> Ok([project_root_path().join("Cargo.toml")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn package_repository_workspace(
                r#"
                [package]
                repository.workspace█ = true
                "#,
                SourcePath(project_root_path().join("crates/tombi-future/Cargo.toml")),
            ) -> Ok([project_root_path().join("Cargo.toml")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn package_license_workspace(
                r#"
                [package]
                license.workspace█ = true
                "#,
                SourcePath(project_root_path().join("crates/tombi-future/Cargo.toml")),
            ) -> Ok([project_root_path().join("Cargo.toml")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn workspace_dependencies_serde(
                r#"
                [workspace.dependencies]
                serde█ = { version = "1.0.0" }
                "#,
                SourcePath(project_root_path().join("Cargo.toml")),
            ) -> Ok([project_root_path().join("Cargo.toml")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn workspace_dependencies_tombi_ast(
                r#"
                [workspace.dependencies]
                tombi-ast = { path█ = "crates/tombi-ast" }
                "#,
                SourcePath(project_root_path().join("Cargo.toml")),
            ) -> Ok([project_root_path().join("crates/tombi-ast/Cargo.toml")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn workspace_dependencies_tombi_ast_editor(
                r#"
                [workspace]
                resolver = "2"
                members = ["crates/*"]

                [workspace.dependencies]
                tombi-ast-editor█ = { path = "crates/tombi-ast-editor" }
                "#,
                SourcePath(project_root_path().join("Cargo.toml")),
            ) -> Ok([project_root_path().join("crates/tombi-ast-editor/Cargo.toml")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn workspace_dependencies_semver(
                r#"
                [workspace]
                resolver = "2"
                members = ["crates/*"]

                [workspace.dependencies]
                semver█ = { version = "1.0.23" }
                "#,
                SourcePath(project_root_path().join("Cargo.toml")),
            ) -> Ok([project_root_path().join("Cargo.toml")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn workspace_members_xtask(
                r#"
                [workspace]
                members = [
                    "xtask█"
                ]
                "#,
                SourcePath(project_root_path().join("Cargo.toml")),
            ) -> Ok([project_root_path().join("xtask/Cargo.toml")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn workspace_members_crate_tombi_ast(
                r#"
                [workspace]
                members = [
                    "crates/tombi-ast█"
                ]
                "#,
                SourcePath(project_root_path().join("Cargo.toml")),
            ) -> Ok([project_root_path().join("crates/tombi-ast/Cargo.toml")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn workspace_members_extension_tombi_extension_cargo(
                r#"
                [workspace]
                members = [
                    "extensions/*█"
                ]
                "#,
                SourcePath(project_root_path().join("Cargo.toml")),
            ) -> Ok([
                project_root_path().join("extensions/tombi-extension-cargo/Cargo.toml"),
                project_root_path().join("extensions/tombi-extension-pyproject/Cargo.toml"),
                project_root_path().join("extensions/tombi-extension-tombi/Cargo.toml"),
            ]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn bin_path_resolves_existing_file(
                r#"
                [[bin]]
                name = "profile"
                path = "src/bin/profile.rs█"
                "#,
                SourcePath(project_root_path().join("crates/tombi-glob/Cargo.toml")),
            ) -> Ok([project_root_path().join("crates/tombi-glob/src/bin/profile.rs")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn bin_path_missing_file_returns_none(
                r#"
                [[bin]]
                name = "missing"
                path = "src/bin/missing.rs█"
                "#,
                SourcePath(project_root_path().join("crates/tombi-glob/Cargo.toml")),
            ) -> Ok([]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn bin_path_multiple_entries_follow_active_table(
                r#"
                [[bin]]
                name = "primary"
                path = "src/bin/profile.rs"

                [[bin]]
                name = "secondary"
                path = "src/bin/profile.rs█"
                "#,
                SourcePath(project_root_path().join("crates/tombi-glob/Cargo.toml")),
            ) -> Ok([project_root_path().join("crates/tombi-glob/src/bin/profile.rs")]);
        );

        // Tests for platform specific dependencies (Issue #1192)
        test_goto_definition!(
            #[tokio::test]
            async fn target_dependencies_serde_workspace(
                r#"
                [target.'cfg(unix)'.dependencies]
                serde = { workspace█ = true }
                "#,
                SourcePath(project_root_path().join("crates/tombi-lsp/Cargo.toml")),
            ) -> Ok([project_root_path().join("Cargo.toml")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn target_dependencies_tombi_ast_workspace(
                r#"
                [target.'cfg(unix)'.dependencies]
                tombi-ast = { workspace█ = true }
                "#,
                SourcePath(project_root_path().join("crates/tombi-lsp/Cargo.toml")),
            ) -> Ok([project_root_path().join("Cargo.toml")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn target_dependencies_path(
                r#"
                [target.'cfg(unix)'.dependencies]
                tombi-ast = { path█ = "crates/tombi-ast" }
                "#,
                SourcePath(project_root_path().join("Cargo.toml")),
            ) -> Ok([project_root_path().join("crates/tombi-ast/Cargo.toml")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn target_dev_dependencies_workspace(
                r#"
                [target.'cfg(target_os = "linux")'.dev-dependencies]
                serde = { workspace█ = true }
                "#,
                SourcePath(project_root_path().join("crates/tombi-lsp/Cargo.toml")),
            ) -> Ok([project_root_path().join("Cargo.toml")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn target_build_dependencies_workspace(
                r#"
                [target.'cfg(windows)'.build-dependencies]
                serde = { workspace█ = true }
                "#,
                SourcePath(project_root_path().join("crates/tombi-lsp/Cargo.toml")),
            ) -> Ok([project_root_path().join("Cargo.toml")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn feature_local_feature_reference(
                r#"
                [package]
                name = "explicit-feature"
                version = "0.1.0"

                [dependencies]
                schemars = { version = "1.0", optional = true }

                [features]
                local = []
                bundle = ["local█", "schemars", "dep:schemars"]
                "#,
                SourcePath(cargo_feature_navigation_fixture_path().join("explicit/Cargo.toml")),
            ) -> Ok([
                cargo_feature_navigation_fixture_path().join("explicit/Cargo.toml")
            ]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn feature_default_entry_reference_with_workspace_optional_dependency(
                r#"
                [package]
                name = "nagi-config"
                version = "0.1.0"
                edition = "2024"

                [dependencies]
                nagi_uri = { workspace = true, optional = true }

                [features]
                default = ["postgres█", "serde"]
                postgres = []
                serde = ["dep:nagi_uri"]
                "#,
                SourcePath(cargo_feature_navigation_fixture_path().join("explicit/Cargo.toml")),
            ) -> Ok([
                cargo_feature_navigation_fixture_path().join("explicit/Cargo.toml")
            ]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn feature_key_collects_same_file_and_workspace_usages(
                r#"
                [package]
                name = "provider"
                version = "0.1.0"
                edition = "2024"

                [features]
                jsonschema█ = []
                "#,
                SourcePath(
                    cargo_feature_navigation_fixture_path().join("workspace/provider/Cargo.toml")
                ),
            ) -> Ok([
                cargo_feature_navigation_fixture_path().join("workspace/provider/Cargo.toml")
            ]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn package_name_lists_workspace_and_member_dependency_usages(
                r#"
                [package]
                name = "provider█"
                version = "0.1.0"
                edition = "2024"

                [features]
                jsonschema = []
                "#,
                SourcePath(
                    cargo_feature_navigation_fixture_path().join("workspace/provider/Cargo.toml")
                ),
            ) -> Ok([
                cargo_feature_navigation_fixture_path().join("workspace/provider/Cargo.toml")
            ]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn feature_implicit_optional_dependency_reference(
                r#"
                [package]
                name = "implicit-feature"
                version = "0.1.0"

                [dependencies]
                schemars = { version = "1.0", optional = true }

                [features]
                bundle = ["schemars█"]
                "#,
                SourcePath(cargo_feature_navigation_fixture_path().join("implicit/Cargo.toml")),
            ) -> Ok([
                cargo_feature_navigation_fixture_path().join("implicit/Cargo.toml")
            ]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn feature_implicit_optional_dependency_disabled_by_dep_syntax(
                r#"
                [package]
                name = "explicit-feature"
                version = "0.1.0"

                [dependencies]
                schemars = { version = "1.0", optional = true }

                [features]
                local = []
                bundle = ["local", "schemars█", "dep:schemars"]
                "#,
                SourcePath(cargo_feature_navigation_fixture_path().join("explicit/Cargo.toml")),
            ) -> Ok([]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn feature_dep_optional_dependency_reference(
                r#"
                [package]
                name = "explicit-feature"
                version = "0.1.0"

                [dependencies]
                schemars = { version = "1.0", optional = true }

                [features]
                local = []
                bundle = ["local", "schemars", "dep:schemars█"]
                "#,
                SourcePath(cargo_feature_navigation_fixture_path().join("explicit/Cargo.toml")),
            ) -> Ok([
                cargo_feature_navigation_fixture_path().join("explicit/Cargo.toml")
            ]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn optional_dependency_definition_collects_same_manifest_feature_usages(
                r#"
                [package]
                name = "explicit-feature"
                version = "0.1.0"

                [dependencies]
                schemars = { version = "1.0", optional█ = true }

                [features]
                local = []
                bundle = ["local", "schemars", "dep:schemars"]
                "#,
                SourcePath(cargo_feature_navigation_fixture_path().join("explicit/Cargo.toml")),
            ) -> Ok([
                cargo_feature_navigation_fixture_path().join("explicit/Cargo.toml")
            ]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn optional_dependency_definition_collects_dep_syntax_usages_for_workspace_dependency(
                r#"
                [package]
                name = "nagi-config"
                version = "0.1.0"
                edition = "2024"

                [dependencies]
                nagi_uri = { workspace = true, optional█ = true }

                [features]
                serde = ["dep:nagi_uri"]
                "#,
                SourcePath(cargo_feature_navigation_fixture_path().join("explicit/Cargo.toml")),
            ) -> Ok([
                cargo_feature_navigation_fixture_path().join("explicit/Cargo.toml")
            ]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn feature_dependency_feature_reference(
                r#"
                [package]
                name = "consumer"
                version = "0.1.0"

                [dependencies]
                provider = { workspace = true, features = ["jsonschema"] }

                [features]
                local = ["provider/jsonschema█"]
                "#,
                SourcePath(
                    cargo_feature_navigation_fixture_path().join("workspace/consumer/Cargo.toml")
                ),
            ) -> Ok([
                cargo_feature_navigation_fixture_path().join("workspace/provider/Cargo.toml")
            ]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn feature_weak_dependency_feature_reference(
                r#"
                [package]
                name = "weak-consumer"
                version = "0.1.0"

                [dependencies]
                provider = { path = "../provider" }

                [features]
                weak = ["provider?/jsonschema█"]
                "#,
                SourcePath(
                    cargo_feature_navigation_fixture_path().join("workspace/weak-consumer/Cargo.toml")
                ),
            ) -> Ok([
                cargo_feature_navigation_fixture_path().join("workspace/provider/Cargo.toml")
            ]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn feature_renamed_dependency_feature_reference(
                r#"
                [package]
                name = "renamed-consumer"
                version = "0.1.0"

                [dependencies]
                sev = { package = "provider", path = "../provider", features = ["jsonschema"] }

                [features]
                rename = ["sev/jsonschema█", "provider/jsonschema"]
                "#,
                SourcePath(
                    cargo_feature_navigation_fixture_path()
                        .join("workspace/renamed-consumer/Cargo.toml")
                ),
            ) -> Ok([
                cargo_feature_navigation_fixture_path().join("workspace/provider/Cargo.toml")
            ]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn feature_wrong_package_name_for_renamed_dependency_returns_none(
                r#"
                [package]
                name = "renamed-consumer"
                version = "0.1.0"

                [dependencies]
                sev = { package = "provider", path = "../provider", features = ["jsonschema"] }

                [features]
                rename = ["sev/jsonschema", "provider/jsonschema█"]
                "#,
                SourcePath(
                    cargo_feature_navigation_fixture_path()
                        .join("workspace/renamed-consumer/Cargo.toml")
                ),
            ) -> Ok([]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn dependency_features_reference_local_feature(
                r#"
                [package]
                name = "consumer"
                version = "0.1.0"

                [dependencies]
                provider = { workspace = true, features = ["jsonschema█"] }

                [features]
                local = ["provider/jsonschema"]
                "#,
                SourcePath(
                    cargo_feature_navigation_fixture_path().join("workspace/consumer/Cargo.toml")
                ),
            ) -> Ok([
                cargo_feature_navigation_fixture_path().join("workspace/provider/Cargo.toml")
            ]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn dependency_features_registry_dependency_returns_none(
                r#"
                [package]
                name = "registry-consumer"
                version = "0.1.0"

                [dependencies]
                serde = { version = "1.0", features = ["derive█"] }
                "#,
                SourcePath(
                    cargo_feature_navigation_fixture_path()
                        .join("workspace/registry-consumer/Cargo.toml")
                ),
            ) -> Ok([]);
        );
    }

    mod pyproject_schema {
        use super::*;

        test_goto_definition!(
            #[tokio::test]
            async fn dependency_groups_group_name_lists_include_group_usages(
                r#"
                [dependency-groups]
                dev = [{ include-group = "ci" }]
                qa = [{ include-group = "ci" }]
                ci█ = ["ruff"]
                "#,
                SourcePath(project_root_path().join("pyproject.toml")),
            ) -> Ok([project_root_path().join("pyproject.toml")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn dependency_groups_include_group_value_jumps_to_group_name(
                r#"
                [dependency-groups]
                dev = [{ include-group = "ci█" }]
                ci = ["ruff"]
                "#,
                SourcePath(project_root_path().join("pyproject.toml")),
            ) -> Ok([project_root_path().join("pyproject.toml")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn tool_pyproject_sources_package_with_workspace(
                r#"
                [tool.uv.sources]
                tombi-beta█ = { workspace = true }
                "#,
                SourcePath(project_root_path().join("python/tombi-beta/pyproject.toml")),
            ) -> Ok([project_root_path().join("python/tombi-beta/pyproject.toml")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn tool_pyproject_sources_package_workspace(
                r#"
                [tool.uv.sources]
                tombi-beta = { workspace█ = true }
                "#,
                SourcePath(project_root_path().join("python/tombi-beta/pyproject.toml")),
            ) -> Ok([project_root_path().join("pyproject.toml")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn tool_pyproject_sources_path_dependency(
                r#"
                [tool.uv.sources]
                app1 = { path = "members/app1█" }
                "#,
                SourcePath(project_root_path().join("crates/tombi-lsp/tests/fixtures/pyproject_workspace/pyproject.toml")),
            ) -> Ok([project_root_path().join("crates/tombi-lsp/tests/fixtures/pyproject_workspace/members/app1/pyproject.toml")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn tool_pyproject_workspace_members(
                r#"
                [tool.uv.workspace]
                members█ = ["python/tombi-beta"]
                "#,
                SourcePath(project_root_path().join("pyproject.toml")),
            ) -> Ok([project_root_path().join("python/tombi-beta/pyproject.toml")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn tool_pyproject_workspace_members_python_tombi_beta(
                r#"
                [tool.uv.workspace]
                members = ["python/tombi-beta█"]
                "#,
                SourcePath(project_root_path().join("pyproject.toml")),
            ) -> Ok([project_root_path().join("python/tombi-beta/pyproject.toml")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn project_readme_relative_file(
                r#"
                [project]
                readme = "python/tombi/README.md█"
                "#,
                SourcePath(project_root_path().join("pyproject.toml")),
            ) -> Ok([project_root_path().join("python/tombi/README.md")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn project_readme_file_object_relative_file(
                r#"
                [project]
                readme = { file = "python/tombi/README.md█" }
                "#,
                SourcePath(project_root_path().join("pyproject.toml")),
            ) -> Ok([project_root_path().join("python/tombi/README.md")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn project_license_file_relative_file(
                r#"
                [project]
                license = { file = "python/tombi/README.md█" }
                "#,
                SourcePath(project_root_path().join("pyproject.toml")),
            ) -> Ok([project_root_path().join("python/tombi/README.md")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn project_license_files_relative_file(
                r#"
                [project]
                license-files = ["python/tombi/README.md█"]
                "#,
                SourcePath(project_root_path().join("pyproject.toml")),
            ) -> Ok([project_root_path().join("python/tombi/README.md")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn build_system_backend_path_relative_directory(
                r#"
                [build-system]
                backend-path = ["python█"]
                "#,
                SourcePath(project_root_path().join("pyproject.toml")),
            ) -> Ok([project_root_path().join("python")]);
        );
    }

    mod pyproject_workspace_dependencies {
        use super::*;

        fn pyproject_workspace_fixtures_path() -> std::path::PathBuf {
            project_root_path().join("crates/tombi-lsp/tests/fixtures/pyproject_workspace")
        }

        test_goto_definition!(
            #[tokio::test]
            async fn project_dependencies_inherit_workspace_version(
                r#"
                [project]
                name = "app1"
                version = "0.1.0"
                dependencies = [
                    "pydantic█"
                ]
                "#,
                SourcePath(pyproject_workspace_fixtures_path().join("members/app1/pyproject.toml")),
            ) -> Ok([pyproject_workspace_fixtures_path().join("pyproject.toml")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn project_name_lists_workspace_and_member_dependency_usages(
                r#"
                [project]
                name = "app2█"
                version = "0.1.0"
                dependencies = [
                  "pydantic",
                ]
                "#,
                SourcePath(pyproject_workspace_fixtures_path().join("members/app2/pyproject.toml")),
            ) -> Ok([
                pyproject_workspace_fixtures_path().join("members/app2/pyproject.toml")
            ]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn member_pypi_dependency_not_managed_by_workspace_stays_on_itself(
                r#"
                [project]
                name = "app2"
                version = "0.1.0"
                dependencies = [
                  "httpx█",
                ]
                "#,
                SourcePath(pyproject_workspace_fixtures_path().join("members/app2/pyproject.toml")),
            ) -> Ok([
                pyproject_workspace_fixtures_path().join("members/app2/pyproject.toml")
            ]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn member_workspace_dependency_with_version_jumps_to_member_definition(
                r#"
                [project]
                name = "app2"
                version = "0.1.0"
                dependencies = [
                  "app3>=0.1.0█",
                ]
                "#,
                SourcePath(pyproject_workspace_fixtures_path().join("members/app2/pyproject.toml")),
            ) -> Ok([
                pyproject_workspace_fixtures_path().join("members/app3/pyproject.toml")
            ]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn workspace_dependencies_list_member_usages(
                r#"
                [project]
                name = "workspace"
                version = "0.1.0"
                dependencies = ["pydantic█"]

                [tool.uv.workspace]
                members = [
                    "members/app1",
                    "members/app2",
                    "members/app3",
                ]
                "#,
                SourcePath(pyproject_workspace_fixtures_path().join("pyproject.toml")),
            ) -> Ok([pyproject_workspace_fixtures_path().join("pyproject.toml")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn workspace_dependency_with_sources_jumps_to_definition_only(
                r#"
                [project]
                name = "workspace"
                version = "0.1.0"
                dependencies = ["app2█"]

                [tool.uv.sources]
                app2 = { workspace = true }

                [tool.uv.workspace]
                members = [
                    "members/app1",
                    "members/app2",
                    "members/app3",
                ]
                "#,
                SourcePath(pyproject_workspace_fixtures_path().join("pyproject.toml")),
            ) -> Ok([pyproject_workspace_fixtures_path().join("members/app2/pyproject.toml")]);
        );
    }

    mod tombi_schema {
        use super::*;

        test_goto_definition!(
            #[tokio::test]
            async fn schema_path_from_dot_config_tombi_toml(
                r#"
                [[schemas]]
                path = "█schemas/name.schema.json"
                "#,
                SourcePath(dot_config_project_root_fixture_path().join(".config/tombi.toml")),
            ) -> Ok([dot_config_project_root_fixture_path().join("schemas/name.schema.json")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn schema_catalog_paths_local_path(
                r#"
                [schema]
                catalog = { paths = ["█schemas/type-test.schema.json"] }
                "#,
                SourcePath(project_root_path().join("tombi.toml")),
            ) -> Ok([project_root_path().join("schemas/type-test.schema.json")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn schema_catalog_paths_local_path_from_dot_config_tombi_toml(
                r#"
                [schema]
                catalog = { paths = ["█schemas/name.schema.json"] }
                "#,
                SourcePath(dot_config_project_root_fixture_path().join(".config/tombi.toml")),
            ) -> Ok([dot_config_project_root_fixture_path().join("schemas/name.schema.json")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn schema_catalog_path(
                r#"
                [[schemas]]
                path = "█www.schemastore.org/tombi.json"
                "#,
                SourcePath(project_root_path().join("tombi.toml")),
            ) -> Ok([project_root_path().join("www.schemastore.org/tombi.json")]);
        );

        test_goto_definition!(
            #[tokio::test]
            async fn schema_path_disabled_by_extensions(
                r#"
                [[schemas]]
                path = "█www.schemastore.org/tombi.json"
                "#,
                SourcePath(project_root_path().join(
                    "crates/tombi-lsp/tests/fixtures/extensions/tombi-disabled/tombi.toml"
                )),
            ) -> Ok([]);
        );
    }

    #[macro_export]
    macro_rules! test_goto_definition {
        (#[tokio::test] async fn $name:ident(
            $source:expr $(, $arg:expr )* $(,)?
        ) -> Ok([$($expected_file_path:expr),*$(,)?]);) => {
            #[tokio::test]
            async fn $name() -> Result<(), Box<dyn std::error::Error>> {
                use std::str::FromStr;

                use itertools::Itertools;
                use tombi_lsp::handler::{handle_did_open, handle_goto_definition};
                use tombi_lsp::Backend;
                use tower_lsp::{
                    lsp_types::{
                        DidOpenTextDocumentParams, GotoDefinitionParams,
                        PartialResultParams, TextDocumentIdentifier, TextDocumentItem,
                        TextDocumentPositionParams, Url, WorkDoneProgressParams,
                    },
                    LspService,
                };
                use tombi_text::IntoLsp;

                tombi_test_lib::init_log();

                #[allow(unused)]
                #[derive(Default)]
                struct TestArgs {
                    source_file_path: Option<std::path::PathBuf>,
                    schema_file_path: Option<std::path::PathBuf>,
                    subschemas: Vec<SubSchemaPath>,
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

                let source_path = args
                    .source_file_path
                    .as_ref()
                    .ok_or("SourcePath must be provided for goto_definition tests")?;

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

                let mut toml_text = textwrap::dedent($source).trim().to_string();
                let Some(index) = toml_text.as_str().find("█") else {
                    return Err("failed to find position marker (█) in the test data".into());
                };
                toml_text.remove(index);
                let line_index =
                tombi_text::LineIndex::new(&toml_text, tombi_text::EncodingKind::Utf16);

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

                let params = GotoDefinitionParams {
                    text_document_position_params: TextDocumentPositionParams {
                        text_document: TextDocumentIdentifier { uri: toml_file_url },
                        position: (tombi_text::Position::default()
                            + tombi_text::RelativePosition::of(&toml_text[..index]))
                        .into_lsp(&line_index),
                    },
                    work_done_progress_params: WorkDoneProgressParams::default(),
                    partial_result_params: PartialResultParams::default(),
                };

                let Ok(result) = handle_goto_definition(&backend, params).await else {
                    return Err("failed to handle goto_definition".into());
                };

                log::debug!("goto_definition result: {:#?}", result);

                trait ToUri {
                    fn to_uri(&self) -> tombi_uri::Uri;
                }

                impl ToUri for tombi_uri::Uri {
                    fn to_uri(&self) -> tombi_uri::Uri {
                        self.clone()
                    }
                }

                impl ToUri for std::path::PathBuf {
                    fn to_uri(&self) -> tombi_uri::Uri {
                        tombi_uri::Uri::from_file_path(self).unwrap()
                    }
                }

                impl ToUri for &str {
                    fn to_uri(&self) -> tombi_uri::Uri {
                        tombi_uri::Uri::from_str(self).unwrap()
                    }
                }

                let expected_paths: Vec<tombi_uri::Uri> = vec![$($expected_file_path.to_uri()),*];

                match result {
                    Some(definition_links) => {
                        pretty_assertions::assert_eq!(
                            definition_links.into_iter().map(|link| link.uri.to_uri()).collect_vec(),
                            expected_paths,
                        );
                    },
                    None => {
                        if !expected_paths.is_empty() {
                            panic!("No definition link was returned, but expected paths: {:?}", expected_paths);
                        }
                    }
                }

                Ok(())
            }
        };
    }
}
