use std::path::{Path, PathBuf};

use tombi_test_lib::{
    cargo_feature_navigation_fixture_path, cargo_schema_path, pyproject_schema_path,
    string_format_test_schema_path, tombi_schema_path,
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
                        (cargo_feature_navigation_fixture_path().join("workspace/Cargo.toml"), 5),
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
            $(, "Title": $title:expr)?
            $(, "Description": $description:expr)?
            $(, "Default": $default:expr)?
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
                        include: vec!["*.toml".to_string()],
                    lint: None,
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
                    }));
                }

                let current_dir = std::env::current_dir().expect("failed to get current directory");
                let temp_dir = if let Some(source_path) = args.source_file_path.as_ref() {
                    source_path.parent().ok_or("failed to get parent directory")?
                } else {
                    current_dir.as_path()
                };
                let Ok(temp_file) = tempfile::NamedTempFile::with_suffix_in(
                    ".toml",
                    temp_dir,
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

                let source_path = args.source_file_path.as_deref().unwrap_or(temp_file.path());

                let Ok(toml_file_url) = Url::from_file_path(source_path) else {
                    return Err("failed to convert file path to URL".into());
                };

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

                if args.schema_file_path.is_some() {
                    assert!(hover_content.schema_uri.is_some(), "The hover target is not defined in the schema.");
                } else {
                    assert!(hover_content.schema_uri.is_none(), "The hover target is defined in the schema.");
                }

                pretty_assertions::assert_eq!(hover_content.accessors.to_string(), $keys, "Keys are not equal");
                pretty_assertions::assert_eq!(hover_content.value_type.to_string(), $value_type, "Value type are not equal");
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

                Ok(())
            }
        }
    }
}
