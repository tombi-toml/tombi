use std::path::{Path, PathBuf};

use tombi_test_lib::{
    adjacent_applicators_test_schema_path, adjacent_one_of_hover_test_schema_path,
    cargo_feature_navigation_fixture_path, cargo_schema_path, lsp_consistency_test_schema_path,
    one_of_hover_discriminator_test_schema_path, pyproject_schema_path,
    ref_sibling_annotations_test_schema_path, string_format_test_schema_path, tombi_schema_path,
};

fn cargo_feature_usage_hover_description(
    project_root: &Path,
    locations: &[(PathBuf, u32)],
) -> String {
    let mut lines = vec!["Feature references in this project:".to_string()];

    for (path, line) in locations {
        let relative_path = path
            .strip_prefix(project_root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");
        let mut uri = tombi_uri::Uri::from_file_path(path).unwrap();
        uri.set_fragment(Some(&format!("L{line}")));
        lines.push(format!("- [{relative_path}:{line}]({uri})"));
    }

    lines.join("\n")
}

mod hover_keys_value {
    use super::*;

    mod tombi_schema {
        use super::*;

        test_hover_keys_value!(
            #[tokio::test]
            async fn tombi_toml_version(
                r#"
                toml-version = "█v1.0.0"
                "#,
                SchemaPath(tombi_schema_path()),
            ) -> Ok({
                "Keys": "toml-version",
                "Value": "String?"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn tombi_toml_version_without_schema(
                r#"
                toml-version = "█v1.0.0"
                "#,
            ) -> Ok({
                "Keys": "toml-version",
                "Value": "String"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn tombi_lint_rules_key_empty(
                r#"
                [lint.rules]
                key-empty = "█warn"
                "#,
                SchemaPath(tombi_schema_path()),
            ) -> Ok({
                "Keys": "lint.rules.key-empty",
                "Value": "String?"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn tombi_schema_catalog_paths(
                r#"
                [schema.catalog]
                paths = ["█https://www.schemastore.org/api/json/catalog.json"]
                "#,
                SchemaPath(tombi_schema_path()),
            ) -> Ok({
                "Keys": "schema.catalog.paths[0]",
                "Value": "String"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn tombi_schema_catalog_path_without_schema(
                r#"
                [schema.catalog]
                path = "█https://www.schemastore.org/api/json/catalog.json"
                "#,
            ) -> Ok({
                "Keys": "schema.catalog.path",
                "Value": "String"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            // NOTE: This test is correct. When you hover over the last key of the header of ArrayOfTable,
            //       the Keys in the hover content is `schema[$index]`, not `schemas`.
            //       Therefore, the Value is `Table`.
            async fn tombi_schemas(
                r#"
                [[schemas█]]
                "#,
                SchemaPath(tombi_schema_path()),
            ) -> Ok({
                "Keys": "schemas[0]",
                "Value": "Table"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn tombi_schemas_without_schema(
                r#"
                [[schemas█]]
                "#,
            ) -> Ok({
                "Keys": "schemas[0]",
                "Value": "Table"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn tombi_schemas_path(
                r#"
                [[schemas]]
                path = "█schemas/tombi.schema.json"
                "#,
                SchemaPath(tombi_schema_path()),
            ) -> Ok({
                "Keys": "schemas[0].path",
                "Value": "String"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn tombi_schemas_include(
                r#"
                [[schemas]]
                path = "schemas/tombi.schema.json"
                include█ = ["*.toml"]
                "#,
                SchemaPath(tombi_schema_path()),
            ) -> Ok({
                "Keys": "schemas[0].include",
                "Value": "Array"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn tombi_schemas_include_array_string(
                r#"
                [[schemas]]
                path = "schemas/tombi.schema.json"
                include = ["█*.toml"]
                "#,
                SchemaPath(tombi_schema_path()),
            ) -> Ok({
                "Keys": "schemas[0].include[0]",
                "Value": "String"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn tombi_comment_directive_toml_version(
                r#"
                #:tombi toml-version█ = "v1.0.0"
                "#,
                SchemaPath(tombi_schema_path()),
            ) -> Ok({
                "Keys": "toml-version",
                "Value": "String?"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn tombi_extensions_non_bare_key(
                r#"
                [extensions]
                "tombi-toml/cargo" = { lsp = { hover█ = { enabled = true } } }
                "#,
                SchemaPath(tombi_schema_path()),
            ) -> Ok({
                "Keys": r#"extensions."tombi-toml/cargo".lsp.hover"#,
                "Value": "Table?"
            });
        );
    }

    mod cargo_schema {
        use super::*;

        test_hover_keys_value!(
            #[tokio::test]
            async fn cargo_package_name(
                r#"
                [package]
                name█ = "tombi"
                "#,
                SchemaPath(cargo_schema_path()),
            ) -> Ok({
                "Keys": "package.name",
                "Value": "String" // Yes; the value is required.
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn cargo_package_name_incomplete(
                r#"
                [package]
                name = █
                "#,
                SchemaPath(cargo_schema_path()),
            ) -> Ok({
                "Keys": "package.name",
                "Value": "String"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn cargo_lints_clippy_absolute_paths_default(
                r#"
                [lints.clippy]
                absolute_paths█ = "allow"
                "#,
                SchemaPath(cargo_schema_path()),
            ) -> Ok({
                "Keys": "lints.clippy.absolute_paths",
                "Value": "(String | Table)?",
                "Title": Some("Absolute Paths"),
                "Default": "\"allow\""
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn ref_sibling_examples_are_displayed(
                r#"
                name = "█allow"
                "#,
                SchemaPath(ref_sibling_annotations_test_schema_path()),
            ) -> Ok({
                "Keys": "name",
                "Value": "String?",
                "Title": Some("Ref Sibling Annotations"),
                "Default": "\"allow\"",
                "Examples": ["\"warn\"", "\"deny\""]
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn cargo_package_readme(
                r#"
                [package]
                readme = "█README.md"
                "#,
                SchemaPath(cargo_schema_path()),
            ) -> Ok({
                "Keys": "package.readme",
                "Value": "(String | Boolean | Table)?"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn cargo_package_readme_without_schema(
                r#"
                [package]
                readme = "█README.md"
                "#,
            ) -> Ok({
                "Keys": "package.readme",
                "Value": "String"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn cargo_dependencies_key(
                r#"
                [dependencies]
                serde█ = { workspace = true }
                "#,
                SchemaPath(cargo_schema_path()),
            ) -> Ok({
                "Keys": "dependencies.serde",
                "Value": "(String | Table)?"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn cargo_dependencies_version(
                r#"
                [dependencies]
                serde = "█1.0"
                "#,
                SchemaPath(cargo_schema_path()),
            ) -> Ok({
                "Keys": "dependencies.serde",
                "Value": "(String | Table)?"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn cargo_dependencies_version_without_schema(
                r#"
                [dependencies]
                serde = "█1.0"
                "#,
            ) -> Ok({
                "Keys": "dependencies.serde",
                "Value": "String"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn cargo_dependencies_workspace(
                r#"
                [dependencies]
                serde = { workspace█ = true }
                "#,
                SchemaPath(cargo_schema_path()),
            ) -> Ok({
                "Keys": "dependencies.serde.workspace",
                "Value": "Boolean?"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn cargo_dependencies_workspace_without_schema(
                r#"
                [dependencies]
                serde = { workspace█ = true }
                "#,
            ) -> Ok({
                "Keys": "dependencies.serde.workspace",
                "Value": "Boolean"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn cargo_dependencies_features(
                r#"
                [dependencies]
                serde = { version = "^1.0.0", features█ = ["derive"] }
                "#,
                SchemaPath(cargo_schema_path()),
            ) -> Ok({
                "Keys": "dependencies.serde.features",
                "Value": "Array?"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn cargo_dependencies_features_item(
                r#"
                [dependencies]
                serde = { version = "^1.0.0", features = ["derive█"] }
                "#,
                SchemaPath(cargo_schema_path()),
            ) -> Ok({
                "Keys": "dependencies.serde.features[0]",
                "Value": "String"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn cargo_dependencies_features_item_without_schema(
                r#"
                [dependencies]
                serde = { version = "^1.0.0", features = ["derive█"] }
                "#,
            ) -> Ok({
                "Keys": "dependencies.serde.features[0]",
                "Value": "String"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn cargo_profile_release_strip_debuginfo(
                r#"
                [profile.release]
                strip = "debuginfo█"
                "#,
                SchemaPath(cargo_schema_path()),
            ) -> Ok({
                "Keys": "profile.release.strip",
                "Value": "(String ^ Boolean)?"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn cargo_profile_release_strip_true(
                r#"
                [profile.release]
                strip = true█
                "#,
                SchemaPath(cargo_schema_path()),
            ) -> Ok({
                "Keys": "profile.release.strip",
                "Value": "(String ^ Boolean)?"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn cargo_profile_release_strip_false(
                r#"
                [profile.release]
                strip = false█
                "#,
                SchemaPath(cargo_schema_path()),
            ) -> Ok({
                "Keys": "profile.release.strip",
                "Value": "(String ^ Boolean)?"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn cargo_workspace_dependency_hover_metadata(
                r#"
                [dependencies]
                tombi-extension-cargo█ = { workspace = true }
                "#,
                SourcePath(tombi_test_lib::project_root_path().join("crates/tombi-lsp/Cargo.toml")),
                SchemaPath(cargo_schema_path()),
            ) -> Ok({
                "Keys": "dependencies.tombi-extension-cargo",
                "Value": "(String | Table)?",
                "Title": Some("tombi-extension-cargo"),
                "Description": Some("Tombi extension for Cargo.toml"),
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn cargo_dependency_table_field_hover_keeps_schema_metadata(
                r#"
                [dependencies]
                tombi-extension-cargo = { version█ = "0.0.0", workspace = true }
                "#,
                SourcePath(tombi_test_lib::project_root_path().join("crates/tombi-lsp/Cargo.toml")),
                SchemaPath(cargo_schema_path()),
            ) -> Ok({
                "Keys": "dependencies.tombi-extension-cargo.version",
                "Value": "String?",
                "Title": Some("Semantic Version Requirement"),
                "Description": Some("The [version requirement](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html) of the target dependency."),
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn cargo_workspace_dependency_hover_metadata_disabled_by_extensions(
                r#"
                [dependencies]
                member█ = { workspace = true }
                "#,
                SourcePath(tombi_test_lib::project_root_path().join(
                    "crates/tombi-lsp/tests/fixtures/extensions/cargo-hover-disabled/member/Cargo.toml"
                )),
                SchemaPath(cargo_schema_path()),
            ) -> Ok({
                "Keys": "dependencies.member",
                "Value": "(String | Table)?",
                "Title": Some("Dependency"),
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn cargo_remote_dependency_hover_offline(
                r#"
                [workspace.dependencies]
                serde█ = "1.0"
                "#,
                SourcePath(tombi_test_lib::project_root_path().join("Cargo.toml")),
                tombi_lsp::backend::Options {
                    offline: Some(true),
                    no_cache: None,
                },
                SchemaPath(cargo_schema_path()),
            ) -> Ok({
                "Keys": "workspace.dependencies.serde",
                "Value": "(String | Table)?"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn cargo_feature_key_hover_lists_project_references(
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
                SchemaPath(cargo_schema_path()),
            ) -> Ok({
                "Keys": "features.jsonschema",
                "Value": "Array?",
                "Description": Some(cargo_feature_usage_hover_description(
                    &cargo_feature_navigation_fixture_path().join("workspace"),
                    &[
                        (
                            cargo_feature_navigation_fixture_path().join("workspace/Cargo.toml"),
                            11,
                        ),
                        (
                            cargo_feature_navigation_fixture_path()
                                .join("workspace/consumer/Cargo.toml"),
                            7,
                        ),
                        (
                            cargo_feature_navigation_fixture_path()
                                .join("workspace/consumer/Cargo.toml"),
                            10,
                        ),
                        (
                            cargo_feature_navigation_fixture_path()
                                .join("workspace/renamed-consumer/Cargo.toml"),
                            7,
                        ),
                        (
                            cargo_feature_navigation_fixture_path()
                                .join("workspace/renamed-consumer/Cargo.toml"),
                            10,
                        ),
                        (
                            cargo_feature_navigation_fixture_path()
                                .join("workspace/weak-consumer/Cargo.toml"),
                            10,
                        ),
                    ],
                )),
            });
        );
    }

    mod one_of_schema {
        use super::*;

        test_hover_keys_value!(
            #[tokio::test]
            async fn one_of_hover_prefers_single_valid_branch(
                r#"
                [[repos]]
                repo = "builtin"
                hooks = [
                  { id = "█hook" }
                ]
                "#,
                SchemaPath(one_of_hover_discriminator_test_schema_path()),
            ) -> Ok({
                "Keys": "repos[0].hooks[0].id",
                "Value": "String",
                "Default": "\"builtin-hook\""
            });
        );
        test_hover_keys_value!(
            #[tokio::test]
            async fn one_of_hover_does_not_leak_branch_default_when_no_branch_is_valid(
                r#"
                [[repos]]
                hooks = [
                  { id = "█hook" }
                ]
                "#,
                SchemaPath(one_of_hover_discriminator_test_schema_path()),
            ) -> Ok({
                "Keys": "repos[0]",
                "Value": "Table"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn adjacent_one_of_hover_prefers_valid_branch_property_schema(
                r#"
                [[repos]]
                repo = "builtin"
                ho█oks = []
                "#,
                SchemaPath(adjacent_one_of_hover_test_schema_path()),
            ) -> Ok({
                "Keys": "repos[0].hooks",
                "Value": "Array"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn adjacent_one_of_hover_prefers_valid_branch_nested_item_schema(
                r#"
                [[repos]]
                repo = "builtin"
                hooks = [
                  { id = "█hook" }
                ]
                "#,
                SchemaPath(adjacent_one_of_hover_test_schema_path()),
            ) -> Ok({
                "Keys": "repos[0].hooks[0].id",
                "Value": "String",
                "Default": "\"builtin-hook\""
            });
        );
    }

    mod adjacent_applicators_schema {
        use super::*;

        test_hover_keys_value!(
            #[tokio::test]
            async fn adjacent_all_of_offset_date_time_hover_merges_const(
                r#"
                offset_date_time_all = 2024-01-15T█10:30:00Z
                "#,
                SchemaPath(adjacent_applicators_test_schema_path()),
            ) -> Ok({
                "Keys": "offset_date_time_all",
                "Value": "OffsetDateTime?",
                "Enum": ["2024-01-15T10:30:00Z"]
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn adjacent_all_of_boolean_hover_merges_const(
                r#"
                boolean_all = t█rue
                "#,
                SchemaPath(adjacent_applicators_test_schema_path()),
            ) -> Ok({
                "Keys": "boolean_all",
                "Value": "Boolean?",
                "Enum": ["true"]
            });
        );
    }

    mod consistency_schema {
        use super::*;

        test_hover_keys_value!(
            #[tokio::test]
            async fn typed_extra_table_unevaluated_properties_hover(
                r#"
                [typed_extra_table]
                extra = { id = "█value" }
                "#,
                SchemaPath(lsp_consistency_test_schema_path()),
            ) -> Ok({
                "Keys": "typed_extra_table.extra.id",
                "Value": "String?"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn typed_unevaluated_tuple_hover(
                r#"
                typed_unevaluated_tuple = [1, { id = "█value" }]
                "#,
                SchemaPath(lsp_consistency_test_schema_path()),
            ) -> Ok({
                "Keys": "typed_unevaluated_tuple[1].id",
                "Value": "String?"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn typed_overflow_tuple_hover(
                r#"
                typed_overflow_tuple = [1, { id = "█value" }]
                "#,
                SchemaPath(lsp_consistency_test_schema_path()),
            ) -> Ok({
                "Keys": "typed_overflow_tuple[1].id",
                "Value": "String?"
            });
        );
    }

    mod pyproject_schema {
        use super::*;

        test_hover_keys_value!(
            #[tokio::test]
            async fn pyproject_project_readme(
                r#"
                [project]
                readme = "█1.0.0"
                "#,
                SchemaPath(pyproject_schema_path()),
            ) -> Ok({
                "Keys": "project.readme",
                "Value": "(String ^ Table)?"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn pyproject_dependency_groups(
                r#"
                [dependency-groups]
                dev = [
                    "█pytest>=8.3.3",
                ]
                "#,
                SchemaPath(pyproject_schema_path()),
            ) -> Ok({
                "Keys": "dependency-groups.dev[0]",
                "Value": "String ^ Table"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn pyproject_dependency_groups_without_schema(
                r#"
                [dependency-groups]
                dev = [
                    "█pytest>=8.3.3",
                ]
                "#,
            ) -> Ok({
                "Keys": "dependency-groups.dev[0]",
                "Value": "String"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn pyproject_workspace_dependency_hover_metadata(
                r#"
                [project]
                dependencies = [
                    "tombi-beta█",
                ]

                [tool.uv.workspace]
                members = ["python/tombi-beta"]

                [tool.uv.sources]
                tombi-beta = { workspace = true }
                "#,
                SourcePath(tombi_test_lib::project_root_path().join("pyproject.toml")),
                SchemaPath(pyproject_schema_path()),
            ) -> Ok({
                "Keys": "project.dependencies[0]",
                "Value": "String",
                "Title": Some("tombi-beta"),
                "Description": Some("Add your description here"),
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn pyproject_workspace_dependency_hover_metadata_disabled_by_extensions(
                r#"
                [project]
                dependencies = [
                    "member█",
                ]

                [tool.uv.workspace]
                members = ["member"]

                [tool.uv.sources]
                member = { workspace = true }
                "#,
                SourcePath(tombi_test_lib::project_root_path().join(
                    "crates/tombi-lsp/tests/fixtures/extensions/pyproject-hover-disabled/pyproject.toml"
                )),
                SchemaPath(pyproject_schema_path()),
            ) -> Ok({
                "Keys": "project.dependencies[0]",
                "Value": "String",
                "Title": Some("Project mandatory dependency requirements"),
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn pyproject_remote_dependency_hover_offline(
                r#"
                [project]
                dependencies = [
                    "request█s>=2.0",
                ]
                "#,
                SourcePath(tombi_test_lib::project_root_path().join("pyproject.toml")),
                tombi_lsp::backend::Options {
                    offline: Some(true),
                    no_cache: None,
                },
                SchemaPath(pyproject_schema_path()),
            ) -> Ok({
                "Keys": "project.dependencies[0]",
                "Value": "String"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn pyproject_tool_uv_constraint_dependency_hover_metadata(
                r#"
                [tool.uv]
                constraint-dependencies = [
                    "tombi-beta█",
                ]

                [tool.uv.workspace]
                members = ["python/tombi-beta"]

                [tool.uv.sources]
                tombi-beta = { workspace = true }
                "#,
                SourcePath(tombi_test_lib::project_root_path().join("pyproject.toml")),
                SchemaPath(pyproject_schema_path()),
            ) -> Ok({
                "Keys": "tool.uv.constraint-dependencies[0]",
                "Value": "String",
                "Title": Some("tombi-beta"),
                "Description": Some("Add your description here"),
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn pyproject_tool_poetry_exclude_tests(
                r#"
                [tool.poetry]
                exclude = [
                    "█tests",
                ]
                "#,
            ) -> Ok({
                "Keys": "tool.poetry.exclude[0]",
                "Value": "String"
            });
        );
    }

    mod non_schema {
        use super::*;

        test_hover_keys_value!(
            #[tokio::test]
            async fn variable_placeholder(
                r#"
                [[dependencies.${mod_id}]]
                modId="create█"
                "#,
            ) -> Ok({
                "Keys": "dependencies.${mod_id}[0].modId",
                "Value": "String"
            });
        );
    }

    mod string_format_test_schema {
        use super::*;

        test_hover_keys_value!(
            #[tokio::test]
            async fn hover_date_val_with_string_formats(
                r#"
                date_val = 2024-01-15█
                "#,
                SchemaPath(string_format_test_schema_path()),
            ) -> Ok({
                "Keys": "date_val",
                "Value": "(LocalDate | String)?"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn hover_time_local_val_with_string_formats(
                r#"
                time_local_val = 10:30:00█
                "#,
                SchemaPath(string_format_test_schema_path()),
            ) -> Ok({
                "Keys": "time_local_val",
                "Value": "(LocalTime | String)?"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn hover_date_time_local_val_with_string_formats(
                r#"
                date_time_local_val = 2024-01-15T10:30:00█
                "#,
                SchemaPath(string_format_test_schema_path()),
            ) -> Ok({
                "Keys": "date_time_local_val",
                "Value": "(LocalDateTime | String)?"
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn hover_date_time_val_with_string_formats(
                r#"
                date_time_val = 2024-01-15T10:30:00Z█
                "#,
                SchemaPath(string_format_test_schema_path()),
            ) -> Ok({
                "Keys": "date_time_val",
                "Value": "(OffsetDateTime | String)?"
            });
        );
    }

    #[macro_export]
    macro_rules! test_hover_keys_value {
        (#[tokio::test] async fn $name:ident(
            $source:expr $(, $arg:expr )* $(,)?
        ) -> Ok({
            "Keys": $keys:expr,
            "Value": $value_type:expr
            $(, "Schema": $has_schema:expr)?
            $(, "Keys Order": $keys_order:expr)?
            $(, "Title": $title:expr)?
            $(, "Description": $description:expr)?
            $(, "Enum": [$($enum_values:expr),* $(,)?])?
            $(, "Default": $default:expr)?
            $(, "Examples": [$($examples:expr),* $(,)?])?
            $(,)?
        });) => {
            #[tokio::test]
            async fn $name() -> Result<(), Box<dyn std::error::Error>> {
                use tombi_lsp::Backend;
                use std::io::Write;
                use tower_lsp::{
                    lsp_types::{
                        TextDocumentIdentifier, Url, WorkDoneProgressParams, DidOpenTextDocumentParams,
                        TextDocumentItem,
                    },
                    LspService,
                };
                use tombi_lsp::handler::handle_did_open;
                use tombi_text::IntoLsp;

                tombi_test_lib::init_log();

                #[allow(unused)]
                #[derive(Default)]
                struct TestArgs {
                    source_file_path: Option<std::path::PathBuf>,
                    schema_file_path: Option<std::path::PathBuf>,
                    inline_schema: Option<String>,
                    config_file_path: Option<std::path::PathBuf>,
                    config_text: Option<String>,
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
                struct Schema(String);

                impl ApplyTestArg for Schema {
                    fn apply(self, args: &mut TestArgs) {
                        args.inline_schema = Some(self.0);
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
                struct Config(String);

                impl ApplyTestArg for Config {
                    fn apply(self, args: &mut TestArgs) {
                        args.config_text = Some(self.0);
                    }
                }

                #[allow(unused)]
                struct ConfigPath(std::path::PathBuf);

                impl ApplyTestArg for ConfigPath {
                    fn apply(self, args: &mut TestArgs) {
                        args.config_file_path = Some(self.0);
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
                        include: vec!["*.toml".to_string()],
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
                        include: vec!["*.toml".to_string()],
                        root: subschema.root.clone(),
                        lint: None,
                        format: None,
                        overrides: None,
                    }));
                }

                let temp_dir = tempfile::tempdir()?;
                let temp_dir_path = temp_dir.path();
                let resolve_path = |path: &std::path::Path| {
                    if path.is_absolute() {
                        path.to_path_buf()
                    } else {
                        temp_dir_path.join(path)
                    }
                };
                let Ok(temp_file) = tempfile::NamedTempFile::with_suffix_in(
                    ".toml",
                    temp_dir_path,
                ) else {
                    return Err("failed to create a temporary file for the test data".into());
                };

                let mut toml_text = textwrap::dedent($source).trim().to_string();

                let Some(index) = toml_text
                    .as_str()
                    .find("█")
                        else {
                        return Err("failed to find hover position marker (█) in the test data".into())
                        };

                toml_text.remove(index);
                if temp_file.as_file().write_all(toml_text.as_bytes()).is_err() {
                    return Err("failed to write to temporary file".into());
                };

                let line_index =
                tombi_text::LineIndex::new(&toml_text, tombi_text::EncodingKind::Utf16);

                let source_path_buf = args
                    .source_file_path
                    .as_ref()
                    .map(|path| resolve_path(path))
                    .unwrap_or_else(|| temp_file.path().to_path_buf());
                let should_write_source_file = args
                    .source_file_path
                    .as_ref()
                    .is_some_and(|path| !path.is_absolute());
                if should_write_source_file {
                    if let Some(parent) = source_path_buf.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    if source_path_buf != temp_file.path() {
                        std::fs::write(&source_path_buf, &toml_text)?;
                    }
                }
                let source_path = source_path_buf.as_path();

                if let Some(schema_text) = args.inline_schema.as_ref() {
                    let schema_file_path = temp_dir_path.join("schema.json");
                    std::fs::write(&schema_file_path, textwrap::dedent(schema_text).trim())?;
                    args.schema_file_path = Some(schema_file_path);
                }

                if let Some(config_text) = args.config_text.as_ref() {
                    let config_path = args
                        .config_file_path
                        .as_ref()
                        .map(|path| resolve_path(path))
                        .unwrap_or_else(|| temp_dir_path.join("tombi.toml"));
                    let should_write_config_file = args
                        .config_file_path
                        .as_ref()
                        .is_none_or(|path| !path.is_absolute());
                    if should_write_config_file {
                        if let Some(parent) = config_path.parent() {
                            std::fs::create_dir_all(parent)?;
                        }
                        std::fs::write(&config_path, textwrap::dedent(config_text).trim())?;
                    }
                    args.config_file_path = Some(config_path);
                }

                let Ok(toml_file_url) = Url::from_file_path(source_path) else {
                    return Err("failed to convert file path to URL".into());
                };

                if let Some(config_file_path) = args.config_file_path.as_ref() {
                    let config_content = std::fs::read_to_string(config_file_path)?;
                    let tombi_config =
                        serde_tombi::config::from_str(&config_content, config_file_path)?;
                    backend
                        .config_manager
                        .update_config_with_path(tombi_config, config_file_path)
                        .await
                        .map_err(|e| {
                            format!(
                                "failed to update config {}: {}",
                                config_file_path.display(),
                                e
                            )
                        })?;
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

                let Ok(Some(tombi_lsp::HoverContent::Value(hover_content) | tombi_lsp::HoverContent::DirectiveContent(hover_content))) = tombi_lsp::handler::handle_hover(
                    &backend,
                    tower_lsp::lsp_types::HoverParams {
                        text_document_position_params: tower_lsp::lsp_types::TextDocumentPositionParams {
                            text_document: TextDocumentIdentifier {
                                uri: toml_file_url,
                            },
                            position: (tombi_text::Position::default()
                                + tombi_text::RelativePosition::of(&toml_text[..index]))
                            .into_lsp(&line_index),
                        },
                        work_done_progress_params: WorkDoneProgressParams::default(),
                    },
                )
                .await else {
                    return Err("failed to handle hover content".into());
                };

                log::debug!("hover_content: {:#?}", hover_content);

                if let Some(expected_has_schema) = None::<bool> $(.or(Some($has_schema)))? {
                    pretty_assertions::assert_eq!(
                        hover_content.schema_uri.is_some(),
                        expected_has_schema,
                        "Schema presence is not equal"
                    );
                } else if args.schema_file_path.is_some() {
                    assert!(hover_content.schema_uri.is_some(), "The hover target is not defined in the schema.");
                } else {
                    assert!(hover_content.schema_uri.is_none(), "The hover target is defined in the schema.");
                }

                pretty_assertions::assert_eq!(hover_content.accessors.to_string(), $keys, "Keys are not equal");
                pretty_assertions::assert_eq!(hover_content.value_type.to_string(), $value_type, "Value type are not equal");
                $(
                    let expected_keys_order = $keys_order.map(ToString::to_string);
                    let actual_keys_order = hover_content
                        .constraints
                        .as_ref()
                        .and_then(|constraints| constraints.keys_order.as_ref())
                        .map(|keys_order| match keys_order {
                            tombi_schema_store::XTombiTableKeysOrder::All(order) => order.to_string(),
                            tombi_schema_store::XTombiTableKeysOrder::Groups(groups) => groups
                                .iter()
                                .map(|group| format!("{}={}", group.target, group.order))
                                .collect::<Vec<_>>()
                                .join(","),
                        });
                    pretty_assertions::assert_eq!(actual_keys_order, expected_keys_order, "Keys order is not equal");
                )?
                $(
                    let expected_title = $title;
                    pretty_assertions::assert_eq!(
                        hover_content.title.as_deref(),
                        expected_title.as_deref(),
                        "Title is not equal"
                    );
                )?
                $(
                    let expected_description = $description;
                    pretty_assertions::assert_eq!(
                        hover_content.description.as_deref(),
                        expected_description.as_deref(),
                        "Description is not equal"
                    );
                )?
                $(
                    let expected_enum = vec![$($enum_values),*];
                    pretty_assertions::assert_eq!(
                        hover_content
                            .constraints
                            .as_ref()
                            .and_then(|constraints| constraints.r#enum.as_ref())
                            .map(|values| values.iter().map(ToString::to_string).collect::<Vec<_>>())
                            .unwrap_or_default(),
                        expected_enum,
                        "Enum is not equal"
                    );
                )?
                $(
                    let expected_default = $default;
                    pretty_assertions::assert_eq!(
                        hover_content
                            .constraints
                            .as_ref()
                            .and_then(|constraints| constraints.default.as_ref())
                            .map(ToString::to_string)
                            .as_deref(),
                        Some(expected_default),
                        "Default is not equal"
                    );
                )?
                $(
                    let expected_examples = vec![$($examples),*];
                    pretty_assertions::assert_eq!(
                        hover_content
                            .constraints
                            .as_ref()
                            .and_then(|constraints| constraints.examples.as_ref())
                            .map(|examples| examples.iter().map(ToString::to_string).collect::<Vec<_>>()),
                        Some(expected_examples.into_iter().map(ToString::to_string).collect()),
                        "Examples are not equal"
                    );
                )?

                Ok(())
            }
        }
    }

    mod hover_table_keys_order {
        use super::*;

        const NESTED_TABLE_SCHEMA: &str = r#"
        {
          "type": "object",
          "properties": {
            "nested": {
              "type": "object",
              "x-tombi-table-keys-order": "ascending",
              "properties": {
                "a": { "type": "integer" },
                "b": { "type": "integer" }
              }
            }
          }
        }
        "#;

        test_hover_keys_value!(
            #[tokio::test]
            async fn hides_json_schema_keys_order_when_schema_format_is_disabled(
                r#"
                [nested█]
                b = 1
                a = 2
                "#,
                Schema(NESTED_TABLE_SCHEMA.to_string()),
                Config(r#"
                [[schemas]]
                path = "schema.json"
                include = ["*.toml"]
                format.rules.table-keys-order.enabled = false
                "#.to_string()),
            ) -> Ok({
                "Keys": "nested",
                "Value": "Table?",
                "Keys Order": None::<&str>
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn prefers_schema_override_over_json_schema_keys_order(
                r#"
                [nested█]
                b = 1
                a = 2
                "#,
                Schema(NESTED_TABLE_SCHEMA.to_string()),
                Config(r#"
                [[schemas]]
                path = "schema.json"
                include = ["*.toml"]
                overrides = [
                  { targets = ["nested"], format.rules.table-keys-order = "descending" }
                ]
                "#.to_string()),
            ) -> Ok({
                "Keys": "nested",
                "Value": "Table?",
                "Keys Order": Some("descending")
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn prefers_schema_override_when_schema_format_is_disabled(
                r#"
                [nested█]
                b = 1
                a = 2
                "#,
                Schema(NESTED_TABLE_SCHEMA.to_string()),
                Config(r#"
                [[schemas]]
                path = "schema.json"
                include = ["*.toml"]
                format.rules.table-keys-order.enabled = false
                overrides = [
                  { targets = ["nested"], format.rules.table-keys-order = "descending" }
                ]
                "#.to_string()),
            ) -> Ok({
                "Keys": "nested",
                "Value": "Table?",
                "Keys Order": Some("descending")
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn prefers_comment_directive_over_schema_override_and_json_schema_keys_order(
                r#"
                # tombi: format.rules.table-keys-order = "descending"
                [nested█]
                b = 1
                a = 2
                "#,
                Schema(NESTED_TABLE_SCHEMA.to_string()),
                Config(r#"
                [[schemas]]
                path = "schema.json"
                include = ["*.toml"]
                overrides = [
                  { targets = ["nested"], format.rules.table-keys-order = "ascending" }
                ]
                "#.to_string()),
            ) -> Ok({
                "Keys": "nested",
                "Value": "Table?",
                "Keys Order": Some("descending")
            });
        );

        test_hover_keys_value!(
            #[tokio::test]
            async fn current_tombi_config_disables_json_schema_keys_order_in_hover(
                r#"
                schema.catalog.paths = ["https://www.schemastore.org/api/json/catalog.json"]

                [[schemas]]
                path = "tombi://www.schemastore.org/tombi.json"
                include = [".tombi.toml", "tombi.toml", "tombi/config.toml"]

                [[schemas█]]
                path = "tombi://www.schemastore.org/tombi.json"
                include = [".tombi.toml", "tombi.toml", "tombi/config.toml"]
                format.rules.table-keys-order.enabled = false
                "#,
                SourcePath(std::path::PathBuf::from("tombi.toml")),
            ) -> Ok({
                "Keys": "schemas[1]",
                "Value": "Table",
                "Schema": true,
                "Keys Order": None::<&str>
            });
        );
    }
}
