use tombi_config::{JSON_SCHEMASTORE_CATALOG_URL, TOMBI_SCHEMASTORE_CATALOG_URL};
use tombi_test_lib::{
    TestCacheHome, adjacent_applicators_test_schema_path,
    adjacent_one_of_additional_properties_test_schema_path, adjacent_one_of_hover_test_schema_path,
    dot_config_project_root_fixture_path, exact_index_string_test_schema_path,
    issue_1895_rustfmt_like_schema_path, lsp_consistency_test_schema_path, project_root_path,
    string_format_test_schema_path, today_local_date, today_local_date_time, today_local_time,
    today_offset_date_time,
};

mod completion_labels {
    use super::*;

    mod tombi_schema {
        use tombi_test_lib::tombi_schema_path;

        use super::*;

        test_completion_labels! {
            #[tokio::test]
            async fn tombi_empty(
                "█",
                SchemaPath(tombi_schema_path()),
            ) -> Ok([
                "extensions",
                "files",
                "format",
                "lint",
                "lsp",
                "overrides",
                "schema",
                "schemas",
                "toml-version",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn tombi_comment(
                "# █",
                SchemaPath(tombi_schema_path()),
            ) -> Ok([]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn schema_comment_directive(
                "#:█",
                SchemaPath(tombi_schema_path()),
            ) -> Ok(["schema", "tombi"]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn tombi_comment_space_schema_directive(
                "# :█",
                SchemaPath(tombi_schema_path()),
            ) -> Ok(["schema", "tombi"]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn schema_comment_directive_and_comment(
                r#"
                #:schema https://www.schemastore.org/tombi.json
                # █
                "#,
                SchemaPath(tombi_schema_path()),
            ) -> Ok([]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn tombi_comment_directive_toml_version(
                r#"
                #:tombi toml-version█
                "#,
                SchemaPath(tombi_schema_path()),
            ) -> Ok([".", "="]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn space_tombi_comment_directive_toml_version(
                r#"
                    #:tombi   toml-version█
                key = "value"
                "#,
                SchemaPath(tombi_schema_path()),
            ) -> Ok([".", "="]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn tombi_comment_directive_and_colon(
                r#"
                #:schema https://www.schemastore.org/tombi.json
                #:█
                "#,
                SchemaPath(tombi_schema_path()),
            ) -> Ok(["tombi"]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn tombi_toml_version_comment(
                r#"toml-version = "v1.0.0"  # █"#,
                SchemaPath(tombi_schema_path()),
            ) -> Ok([]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn tombi_toml_version_directive_comment(
                r#"toml-version = "v1.0.0"  #:█"#,
                SchemaPath(tombi_schema_path()),
            ) -> Ok([]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn tombi_used_toml_version(
                r#"
                toml-version = "v1.0.0"
                █
                "#,
                SchemaPath(tombi_schema_path()),
            ) -> Ok([
                "extensions",
                "files",
                "format",
                "lint",
                "lsp",
                "overrides",
                "schema",
                "schemas",
                // "toml-version",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn tombi_used_toml_version_with_schema_directive(
                r#"
                #:schema tombi://www.schemastore.org/tombi.json

                toml-version = "v1.0.0"
                █
                "#,
                SchemaPath(tombi_schema_path()),
            ) -> Ok([
                "extensions",
                "files",
                "format",
                "lint",
                "lsp",
                "overrides",
                "schema",
                "schemas",
                // "toml-version",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn tombi_used_toml_version_and_other_table(
                r#"
                toml-version = "v1.0.0"
                █

                [lsp]
                "#,
                SchemaPath(tombi_schema_path()),
            ) -> Ok([
                "extensions",
                "files",
                "format",
                "lint",
                "overrides",
                "schema",
                "schemas",
                // "toml-version",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn tombi_used_toml_version_and_space(
                r#"
                toml-version = "v1.0.0" █
                "#,
                SchemaPath(tombi_schema_path()),
            ) -> Ok([]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn tombi_lsp_completion_enabled_true_and_space(
                r#"
                [lsp]
                completion.enabled = true █
                "#,
                SchemaPath(tombi_schema_path()),
            ) -> Ok([]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn tombi_extensions_table(
                r#"
                [extensions]
                █
                "#,
                SchemaPath(tombi_schema_path()),
            ) -> Ok([
                "tombi-toml/cargo",
                "tombi-toml/pyproject",
                "tombi-toml/tombi",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn schema_override_used_table_key_with_remaining_nested_key(
                r#"
                [[schemas]]
                path = "tombi://www.schemastore.org/tombi.json"
                include = [".tombi.toml", "tombi.toml", "tombi/config.toml"]
                overrides = [
                  {
                    targets = [""],
                    format.rules.table-keys-order = "ascending",
                    █
                  }
                ]
                "#,
                SchemaPath(tombi_schema_path()),
            ) -> Ok([
                "format",
                "lint",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn tombi_lint_rules_key_empty_equal_warn_and_space(
                r#"
                [lint.rules]
                key-empty = "warn" █
                "#,
                SchemaPath(tombi_schema_path()),
            ) -> Ok([]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn tombi_empty_bracket(
                "[█]",
                SchemaPath(tombi_schema_path()),
            ) -> Ok([
                "extensions",
                "files",
                "format",
                "lint",
                "lsp",
                "overrides",
                "schema",
                "schemas",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn tombi_empty_bracket2(
                r#"
                toml-version = "v1.0.0"

                [█]
                "#,
                SchemaPath(tombi_schema_path()),
            ) -> Ok([
                "extensions",
                "files",
                "format",
                "lint",
                "lsp",
                "overrides",
                "schema",
                "schemas",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn tombi_empty_bracket3(
                r#"
                toml-version = "v1.0.0"

                [█]

                [format]
                "#,
                SchemaPath(tombi_schema_path()),
            ) -> Ok([
                "extensions",
                "files",
                "format",
                "lint",
                "lsp",
                "overrides",
                "schema",
                "schemas",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn tombi_empty_bracket4(
                r#"
                toml-version = "v1.0.0"

                [█]

                [lsp]
                "#,
                SchemaPath(tombi_schema_path()),
            ) -> Ok([
                "extensions",
                "files",
                "format",
                "lint",
                "lsp",
                "overrides",
                "schema",
                "schemas",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn tombi_empty_double_bracket(
                "[[█]]",
                SchemaPath(tombi_schema_path()),
            ) -> Ok([
                "extensions",
                "files",
                "format",
                "lint",
                "lsp",
                "overrides",
                "schema",
                "schemas",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn tombi_lint_rules_key_empty_equal(
                r#"
                [lint.rules]
                key-empty = █
                "#,
                SchemaPath(tombi_schema_path()),
            ) -> Ok([
                "\"warn\"",
                "\"error\"",
                "\"off\"",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn tombi_lint_rules_key_empty_equal_empty_string(
                r#"
                [lint.rules]
                key-empty = "█"
                "#,
                SchemaPath(tombi_schema_path()),
            ) -> Ok([
                "\"warn\"",
                "\"error\"",
                "\"off\"",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn tombi_schema(
                r#"
                [schema.█]
                "#,
                SchemaPath(tombi_schema_path()),
            ) -> Ok([
                "catalog",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn tombi_schema_after_bracket(
                "[schema]█",
                SchemaPath(tombi_schema_path()),
            ) -> Ok([]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn tombi_schema_catalog_dot_on_header(
                "[schema.catalog.█]",
                SchemaPath(tombi_schema_path()),
            ) -> Ok([]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn tombi_schema_catalog(
                r#"
                [schema]
                catalog█
                "#,
                SchemaPath(tombi_schema_path()),
            ) -> Ok([
                ".",
                "=",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn tombi_lsp_completion_dot(
                r#"
                [lsp]
                completion.█
                "#,
                SchemaPath(tombi_schema_path()),
            ) -> Ok([
                "enabled",
                "{}"
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn tombi_lsp_completion_equal(
                r#"
                [lsp]
                completion=█
                "#,
                SchemaPath(tombi_schema_path()),
            ) -> Ok([
                "enabled",
                "{}"
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn tombi_schema_catalog_path(
                r#"
                [schema.catalog]
                paths =[█]
                "#,
                SchemaPath(tombi_schema_path()),
            ) -> Ok([
                "\"https://www.schemastore.org/api/json/catalog.json\"",
                "\"tombi://www.schemastore.org/api/json/catalog.json\"",
                "\"\"",
                "''",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn tombi_schema_catalog_path2(
                r#"
                [schema.catalog]
                paths = [█]
                "#,
                SchemaPath(tombi_schema_path()),
            ) -> Ok([
                "\"https://www.schemastore.org/api/json/catalog.json\"",
                "\"tombi://www.schemastore.org/api/json/catalog.json\"",
                "\"\"",
                "''",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn tombi_schema_catalog_path_inline(
                r#"
                schema.catalog.paths =█
                "#,
                SchemaPath(tombi_schema_path()),
            ) -> Ok([
                format!("[\"{TOMBI_SCHEMASTORE_CATALOG_URL}\", \"{JSON_SCHEMASTORE_CATALOG_URL}\"]"),
                "[]",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn tombi_lsp2(
                r#"
                [lsp]
                █
                completion.enabled = true
                "#,
                SchemaPath(tombi_schema_path()),
            ) -> Ok([
                "code-action",
                "diagnostic",
                "document-link",
                "formatting",
                "goto-declaration",
                "goto-definition",
                "goto-type-definition",
                "hover",
                "references",
                "workspace-diagnostic",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn tombi_lsp3(
                r#"
                [lsp]
                formatting.enabled = true
                █
                completion.enabled = true
                "#,
                SchemaPath(tombi_schema_path()),
            ) -> Ok([
                "code-action",
                "diagnostic",
                "document-link",
                "goto-declaration",
                "goto-definition",
                "goto-type-definition",
                "hover",
                "references",
                "workspace-diagnostic",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn tombi_lsp4(
                r#"
                [lsp]
                code-action.enabled = true
                formatting.enabled = true

                [lsp.█]
                "#,
                SchemaPath(tombi_schema_path()),
            ) -> Ok([
                "completion",
                "diagnostic",
                "document-link",
                "goto-declaration",
                "goto-definition",
                "goto-type-definition",
                "hover",
                "references",
                "workspace-diagnostic",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn tombi_lsp_completion(
                r#"
                [lsp]
                completion.enabled = █
                "#,
                SchemaPath(tombi_schema_path()),
            ) -> Ok([
                "true",
                "false",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn tombi_extension_tombi_completion_variant_after_path_enabled(
                r#"
                [extensions]
                "tombi-toml/tombi" = {
                  lsp = {
                    completion = {
                      █
                      path.enabled = true,
                    },
                  },
                }
                "#,
                SchemaPath(tombi_schema_path()),
            ) -> Ok([]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn tombi_format_rules_array_bracket_space_width(
                r#"
                [format.rules]
                array-bracket-space-width = █
                "#,
                SchemaPath(tombi_schema_path()),
            ) -> Ok([
                "0",
                "42",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn tombi_lsp_comp(
                r#"
                [lsp]
                comp█
                "#,
                SchemaPath(tombi_schema_path()),
            ) -> Ok([
                "completion",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn tombi_lsp_comp2(
                r#"
                [lsp.comp█]
                "#,
                SchemaPath(tombi_schema_path()),
            ) -> Ok([
                "completion",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn tombi_lsp_comp3(
                r#"
                [lsp]
                comp█

                [schema]
                "#,
                SchemaPath(tombi_schema_path()),
            ) -> Ok([
                "completion",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn tombi_schemars(
                r#"
                [[schemas]]
                █
                "#,
                SchemaPath(tombi_schema_path()),
            ) -> Ok([
                "include",
                "path",
                "root",
                "exclude",
                "format",
                "lint",
                "overrides",
                "toml-version",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn tombi_schemars_overrides_rules(
                r#"
                [[schemas]]
                overrides = [{ targets = [""], format.rules.█ }]
                "#,
                SchemaPath(tombi_schema_path()),
            ) -> Ok([
                "array-values-order",
                "table-keys-order",
                "{}",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn tombi_schemars_path(
                r#"
                [[schemas]]
                path.█
                "#,
                SchemaPath(tombi_schema_path()),
            ) -> Ok([
                "\"https://www.schemastore.org/cargo.json\"",
                "\"https://www.schemastore.org/pyproject.json\"",
                "\"tombi://www.schemastore.org/cargo.json\"",
                "\"tombi://www.schemastore.org/pyproject.json\"",
                "\"tombi://www.schemastore.org/tombi.json\"",
                "\"\"",
                "''",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn tombi_schemars_path_file_completion_from_dot_config_project_root(
                r#"
                [[schemas]]
                path = "sch█"
                "#,
                SourcePath(dot_config_project_root_fixture_path().join(".config/tombi.toml")),
                SchemaPath(tombi_schema_path()),
            ) -> Ok([
                "schemas/",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn tombi_schemas_include_file_completion(
                r#"
                [[schemas]]
                include = ["sch█"]
                "#,
                SourcePath(project_root_path().join("tombi.toml")),
                SchemaPath(tombi_schema_path()),
            ) -> Ok([
                "schemas/",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn tombi_schemas_exclude_file_completion(
                r#"
                [[schemas]]
                include = ["schemas/*"]
                exclude = ["sch█"]
                "#,
                SourcePath(project_root_path().join("tombi.toml")),
                SchemaPath(tombi_schema_path()),
            ) -> Ok([
                "schemas/",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn tombi_schemas_include_file_completion_from_dot_config_project_root(
                r#"
                [[schemas]]
                include = ["█"]
                "#,
                SourcePath(dot_config_project_root_fixture_path().join(".config/tombi.toml")),
                SchemaPath(tombi_schema_path()),
            ) -> Ok([
                "product.toml",
                "schemas/",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn tombi_schemas_exclude_file_completion_from_dot_config_project_root(
                r#"
                [[schemas]]
                include = ["schemas/*"]
                exclude = ["█"]
                "#,
                SourcePath(dot_config_project_root_fixture_path().join(".config/tombi.toml")),
                SchemaPath(tombi_schema_path()),
            ) -> Ok([
                "product.toml",
                "schemas/",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn tombi_schema_catalog_paths_file_completion(
                r#"
                [schema.catalog]
                paths = ["sch█"]
                "#,
                SourcePath(project_root_path().join("tombi.toml")),
                SchemaPath(tombi_schema_path()),
            ) -> Ok([
                "schemas/",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn tombi_schema_catalog_paths_file_completion_from_dot_config_project_root(
                r#"
                [schema.catalog]
                paths = ["sch█"]
                "#,
                SourcePath(dot_config_project_root_fixture_path().join(".config/tombi.toml")),
                SchemaPath(tombi_schema_path()),
            ) -> Ok([
                "schemas/",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn tombi_files_include_file_completion(
                r#"
                [files]
                include = ["sch█"]
                "#,
                SourcePath(project_root_path().join("tombi.toml")),
                SchemaPath(tombi_schema_path()),
            ) -> Ok([
                "schemas/",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn tombi_files_exclude_file_completion(
                r#"
                [files]
                exclude = ["sch█"]
                "#,
                SourcePath(project_root_path().join("tombi.toml")),
                SchemaPath(tombi_schema_path()),
            ) -> Ok([
                "schemas/",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn tombi_files_include_file_completion_from_dot_config_project_root(
                r#"
                [files]
                include = ["█"]
                "#,
                SourcePath(dot_config_project_root_fixture_path().join(".config/tombi.toml")),
                SchemaPath(tombi_schema_path()),
            ) -> Ok([
                "product.toml",
                "schemas/",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn tombi_files_exclude_file_completion_from_dot_config_project_root(
                r#"
                [files]
                exclude = ["█"]
                "#,
                SourcePath(dot_config_project_root_fixture_path().join(".config/tombi.toml")),
                SchemaPath(tombi_schema_path()),
            ) -> Ok([
                "product.toml",
                "schemas/",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn tombi_toml_version_v1_0_0_comment_directive(
                r#"
                toml-version = "v1.0.0" # tombi:█
                "#,
                SchemaPath(tombi_schema_path()),
            ) -> Ok([
                "format",
                "lint",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn schema_directive_file_path_empty(
                r#"
                #:schema █
                "#,
                SourcePath(project_root_path().join("www.schemastore.org/dummy.toml")),
                SchemaPath(tombi_schema_path()),
            ) -> Ok([
                "api/",
                "cargo.json",
                "pyproject.json",
                "tombi.json",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn schema_directive_file_path_www_schemastore_org(
                r#"
                #:schema ./www.schemastore.org/█
                "#,
                SourcePath(project_root_path().join("Cargo.toml")),
                SchemaPath(tombi_schema_path()),
            ) -> Ok([
                "./www.schemastore.org/api/",
                "./www.schemastore.org/cargo.json",
                "./www.schemastore.org/pyproject.json",
                "./www.schemastore.org/tombi.json",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn schema_directive_file_path_partial_match(
                r#"
                #:schema schemas/type█
                "#,
                SourcePath(project_root_path().join("Cargo.toml")),
                SchemaPath(tombi_schema_path()),
            ) -> Ok([
                "schemas/type-test.schema.json",
            ]);
        }
    }

    mod adjacent_one_of_schema {
        use super::*;

        test_completion_labels! {
            #[tokio::test]
            async fn adjacent_one_of_builtin_hook_empty_inline_table_keys_completion(
                r#"
                [[repos]]
                repo = "builtin"
                hooks = [ {█} ]
                "#,
                SchemaPath(adjacent_one_of_hover_test_schema_path()),
            ) -> Ok(["id"]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn adjacent_one_of_builtin_hook_empty_inline_table_keys_completion_with_space(
                r#"
                [[repos]]
                repo = "builtin"
                hooks = [ { █} ]
                "#,
                SchemaPath(adjacent_one_of_hover_test_schema_path()),
            ) -> Ok(["id"]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn adjacent_one_of_builtin_hook_incomplete_inline_table_keys_completion(
                r#"
                [[repos]]
                repo = "builtin"
                hooks = [
                  { █
                ]
                "#,
                SchemaPath(adjacent_one_of_hover_test_schema_path()),
            ) -> Ok(["id"]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn adjacent_one_of_builtin_hook_incomplete_inline_table_keys_completion_with_prek_context(
                r#"
                fail_fast = false

                [[repos]]
                repo = "builtin"
                hooks = [
                  { █
                ]
                "#,
                SchemaPath(adjacent_one_of_hover_test_schema_path()),
            ) -> Ok(["id"]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn adjacent_one_of_additional_properties_builtin_hook_incomplete_inline_table_keys_completion(
                r#"
                fail_fast = false

                [[repos]]
                repo = "builtin"
                hooks = [
                  { █
                ]
                "#,
                SchemaPath(adjacent_one_of_additional_properties_test_schema_path()),
            ) -> Ok(["id"]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn adjacent_one_of_builtin_hook_id_value_completion(
                r#"
                [[repos]]
                repo = "builtin"
                hooks = [
                  { id = █ }
                ]
                "#,
                SchemaPath(adjacent_one_of_hover_test_schema_path()),
            ) -> Ok(["\"builtin-hook\"", "\"\"", "''"]);
        }
    }

    mod adjacent_applicators_schema {
        use super::*;

        test_completion_labels! {
            #[tokio::test]
            async fn adjacent_all_of_offset_date_time_value_completion(
                r#"
                offset_date_time_all = █
                "#,
                SchemaPath(adjacent_applicators_test_schema_path()),
            ) -> Ok(["2024-01-15T10:30:00Z"]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn adjacent_all_of_boolean_value_completion(
                r#"
                boolean_all = █
                "#,
                SchemaPath(adjacent_applicators_test_schema_path()),
            ) -> Ok(["true"]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn adjacent_all_of_table_merges_properties_and_all_of_keys(
                r#"
                [object_all]
                █
                "#,
                SchemaPath(adjacent_applicators_test_schema_path()),
            ) -> Ok(["bar", "baz", "foo"]);
        }
    }

    mod consistency_schema {
        use super::*;

        test_completion_labels! {
            #[tokio::test]
            async fn typed_extra_table_unevaluated_properties_inline_table_keys_completion(
                r#"
                [typed_extra_table]
                extra = { █ }
                "#,
                SchemaPath(lsp_consistency_test_schema_path()),
            ) -> Ok(["id"]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn typed_unevaluated_tuple_inline_table_keys_completion(
                r#"
                typed_unevaluated_tuple = [1, { █ }]
                "#,
                SchemaPath(lsp_consistency_test_schema_path()),
            ) -> Ok(["id"]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn typed_overflow_tuple_inline_table_keys_completion(
                r#"
                typed_overflow_tuple = [1, { █ }]
                "#,
                SchemaPath(lsp_consistency_test_schema_path()),
            ) -> Ok(["id"]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn issue_1895_root_table_completion_includes_annotation_only_property(
                r#"
                █
                "#,
                SchemaPath(issue_1895_rustfmt_like_schema_path()),
            ) -> Ok(["ignore", "max_width"]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn issue_1895_annotation_only_property_completion_keeps_description(
                r#"
                █
                "#,
                SchemaPath(issue_1895_rustfmt_like_schema_path()),
            ) -> Ok([
                {
                    "label": "ignore",
                    "documentation": "Annotation-only property schema that should still allow the key.",
                },
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn exact_index_string_subschema_does_not_apply_to_first_item_completion(
                r#"
                items = [█, "scoped"]
                "#,
                SubSchema {
                    root: "items[1]",
                    path: exact_index_string_test_schema_path(),
                },
            ) -> Ok(AnyValue);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn exact_index_string_subschema_applies_to_second_item_completion(
                r#"
                items = ["zero", █]
                "#,
                SubSchema {
                    root: "items[1]",
                    path: exact_index_string_test_schema_path(),
                },
            ) -> Ok([
                "\"\"",
                "''",
            ]);
        }
    }

    mod pyproject_schema {
        use tombi_test_lib::pyproject_schema_path;

        use super::*;

        test_completion_labels! {
            #[tokio::test]
            async fn pyproject_empty(
                "█",
                SchemaPath(pyproject_schema_path()),
            ) -> Ok([
                "build-system",
                "dependency-groups",
                "project",
                "tool",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn pyproject_project(
                r#"
                [project]
                █
                "#,
                SchemaPath(pyproject_schema_path()),
            ) -> Ok([
                "name",
                "authors",
                "classifiers",
                "dependencies",
                "description",
                "dynamic",
                "entry-points",
                "gui-scripts",
                "import-names",
                "import-namespaces",
                "keywords",
                "license",
                "license-files",
                "maintainers",
                "optional-dependencies",
                "readme",
                "requires-python",
                "scripts",
                "urls",
                "version",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn pyproject_project_readme_file_completion(
                r#"
                [project]
                readme = "py█"
                "#,
                SourcePath(project_root_path().join(
                    "crates/tombi-lsp/tests/fixtures/pyproject_workspace/pyproject.toml"
                )),
                SchemaPath(pyproject_schema_path()),
            ) -> Ok([
                "pyproject.toml",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn pyproject_project_readme_file_object_completion(
                r#"
                [project]
                readme = { file = "py█" }
                "#,
                SourcePath(project_root_path().join(
                    "crates/tombi-lsp/tests/fixtures/pyproject_workspace/pyproject.toml"
                )),
                SchemaPath(pyproject_schema_path()),
            ) -> Ok([
                "pyproject.toml",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn pyproject_project_license_file_completion(
                r#"
                [project]
                license = { file = "py█" }
                "#,
                SourcePath(project_root_path().join(
                    "crates/tombi-lsp/tests/fixtures/pyproject_workspace/pyproject.toml"
                )),
                SchemaPath(pyproject_schema_path()),
            ) -> Ok([
                "pyproject.toml",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn pyproject_project_license_files_completion(
                r#"
                [project]
                license-files = ["py█"]
                "#,
                SourcePath(project_root_path().join(
                    "crates/tombi-lsp/tests/fixtures/pyproject_workspace/pyproject.toml"
                )),
                SchemaPath(pyproject_schema_path()),
            ) -> Ok([
                "pyproject.toml",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn pyproject_project_dynamic_array(
                r#"
                [project]
                dynamic = [█]
                "#,
                SchemaPath(pyproject_schema_path()),
            ) -> Ok([
                "\"authors\"",
                "\"classifiers\"",
                "\"dependencies\"",
                "\"description\"",
                "\"entry-points\"",
                "\"gui-scripts\"",
                "\"import-names\"",
                "\"import-namespaces\"",
                "\"keywords\"",
                "\"license\"",
                "\"license-files\"",
                "\"maintainers\"",
                "\"optional-dependencies\"",
                "\"readme\"",
                "\"requires-python\"",
                "\"scripts\"",
                "\"urls\"",
                "\"version\"",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn pyproject_project_dynamic_array_in_values_with_last_comma(
                // Check `unique_items = true` case.
                r#"
                [project]
                dynamic = [
                  "authors",
                  "classifiers",
                  █
                ]
                "#,
                SchemaPath(pyproject_schema_path()),
            ) -> Ok([
                "\"dependencies\"",
                "\"description\"",
                "\"entry-points\"",
                "\"gui-scripts\"",
                "\"import-names\"",
                "\"import-namespaces\"",
                "\"keywords\"",
                "\"license\"",
                "\"license-files\"",
                "\"maintainers\"",
                "\"optional-dependencies\"",
                "\"readme\"",
                "\"requires-python\"",
                "\"scripts\"",
                "\"urls\"",
                "\"version\"",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn pyproject_build_system(
                r#"
                [build-system]
                requires = ["maturin>=1.5,<2.0"]
                build-backend = "maturin"
                █
                "#,
                SchemaPath(pyproject_schema_path()),
            ) -> Ok([
                "backend-path",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn pyproject_build_system_backend_path_file_completion(
                r#"
                [build-system]
                backend-path = ["mem█"]
                "#,
                SourcePath(project_root_path().join(
                    "crates/tombi-lsp/tests/fixtures/pyproject_workspace/pyproject.toml"
                )),
                SchemaPath(pyproject_schema_path()),
            ) -> Ok([
                "members/",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn pyproject_dependency_groups_last(
                r#"
                [dependency-groups]
                dev = [
                    "pytest>=8.3.3",
                    "ruff>=0.7.4",
                    █
                ]
                "#,
                SchemaPath(pyproject_schema_path()),
            ) -> Ok([
                "include-group",
                "\"\"",
                "''",
                "{}",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn pyproject_dependency_groups_dev_eq_array_last(
                r#"
                [dependency-groups]
                dev = [
                    "pytest>=8.3.3",
                    "ruff>=0.7.4",
                ]█
                "#,
                SchemaPath(pyproject_schema_path()),
            ) -> Ok([]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn pyproject_tool(
                r#"
                [tool.█]
                "#,
                SchemaPath(pyproject_schema_path()),
            ) -> Ok([
                "black",
                "cibuildwheel",
                "dfc",
                "docstring-format-checker",
                "fastapi",
                "hatch",
                "maturin",
                "mypy",
                "pdm",
                "pixi",
                "poe",
                "poetry",
                "pyright",
                "pytest",
                "repo-review",
                "ruff",
                "scheduled",
                "scikit-build",
                "setuptools",
                "setuptools_scm",
                "taskipy",
                "tombi",
                "tox",
                "ty",
                "uv",
                "$tool_name",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn pyproject_tool_third_party_field(
                r#"
                [tool.third_party]
                field█
                "#,
                SchemaPath(pyproject_schema_path()),
            ) -> Ok([
                ".",
                "=",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn pyproject_tool_third_party_field_equal(
                r#"
                [tool.third_party]
                field=█
                "#,
                SchemaPath(pyproject_schema_path()),
            ) -> Ok(AnyValue);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn pyproject_tool_third_party_field_equal_array(
                r#"
                [tool.third_party]
                field = [█]
                "#,
                SchemaPath(pyproject_schema_path()),
            ) -> Ok(AnyValue);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn pyproject_project_leading_comments_directive_newline_name_eq_tombi(
                r#"
                # tombi: lint.rules█
                [project]
                name = "tombi"
                "#,
                SchemaPath(pyproject_schema_path()),
            ) -> Ok([
                ".",
                "="
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn pyproject_project_comment_directive_newline_name_eq_tombi(
                r#"
                [project]
                # tombi: lint.rules█

                name = "tombi"
                "#,
                SchemaPath(pyproject_schema_path()),
            ) -> Ok([
                ".",
                "="
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn pyproject_project_trailing_comment_directive_newline_name_eq_tombi(
                r#"
                [project]  # tombi: lint.rules█

                name = "tombi"
                "#,
                SchemaPath(pyproject_schema_path()),
            ) -> Ok([
                ".",
                "="
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn pyproject_project_comment_directive_name_eq_tombi(
                r#"
                [project]
                # tombi: lint.rules█
                name = "tombi"
                "#,
                SchemaPath(pyproject_schema_path()),
            ) -> Ok([
                ".",
                "="
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn pyproject_project_name_eq_tombi_comment_directive(
                r#"
                [project]
                name = "tombi" # tombi: lint█
                "#,
                SchemaPath(pyproject_schema_path()),
            ) -> Ok([
                ".",
                "="
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn pyproject_project_description_comment_directive(
                r#"
                [project]
                description = "🦅 TOML Toolkit 🦅" # tombi: lint█
                "#,
                SchemaPath(pyproject_schema_path()),
            ) -> Ok([
                ".",
                "="
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn pyproject_project_dependencies_eq_array_comment_directive(
                r#"
                [project]
                name = "tombi"
                dependencies = [
                    # tombi: lint█
                ]
                "#,
                SchemaPath(pyproject_schema_path()),
            ) -> Ok([
                ".",
                "="
            ]);
        }
    }

    mod root_ref_schema {
        use tombi_test_lib::root_ref_test_schema_path;

        use super::*;

        test_completion_labels! {
            #[tokio::test]
            async fn root_ref_schema_empty(
                "█",
                SchemaPath(root_ref_test_schema_path()),
            ) -> Ok([
                "compatibility_date",
                "name",
                "workers_dev",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn root_ref_schema_completion_documentation(
                "█",
                SchemaPath(root_ref_test_schema_path()),
            ) -> Ok([
                {
                    "label": "compatibility_date",
                    "documentation": "Date that selects the Workers runtime version.",
                },
            ]);
        }
    }

    mod cargo_schema {
        use tombi_test_lib::cargo_schema_path;

        use super::*;

        test_completion_labels! {
            #[tokio::test]
            async fn cargo_empty(
                "█",
                SchemaPath(cargo_schema_path()),
            ) -> Ok([
                "badges",
                "bench",
                "bin",
                "build-dependencies",
                "cargo-features",
                "dependencies",
                "dev-dependencies",
                "example",
                "features",
                "lib",
                "lints",
                "package",
                "patch",
                "profile",
                "target",
                "test",
                "workspace",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn cargo_dependencies_and_next_section(
                r#"
                [dependencies]

                [█]
                "#,
                SchemaPath(cargo_schema_path()),
            ) -> Ok([
                "badges",
                "bench",
                "bin",
                "build-dependencies",
                "dependencies",
                "dev-dependencies",
                "example",
                "features",
                "lib",
                "lints",
                "package",
                "patch",
                "profile",
                "target",
                "test",
                "workspace",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn cargo_dependencies(
                r#"
                [dependencies]
                █
                "#,
                SchemaPath(cargo_schema_path()),
            ) -> Ok([
                "$crate_name",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn cargo_dependencies_workspace_inheritance_candidate(
                r#"
                [dependencies]
                s█
                "#,
                SourcePath(project_root_path().join(
                    "crates/tombi-lsp/tests/fixtures/issue-1621-cargo-workspace-completion/member/Cargo.toml"
                )),
                SchemaPath(cargo_schema_path()),
            ) -> Ok([
                "serde",
                ".",
                "=",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn cargo_dependencies_workspace_inheritance_candidate_on_empty_line(
                r#"
                [dependencies]
                █
                "#,
                SourcePath(project_root_path().join(
                    "crates/tombi-lsp/tests/fixtures/issue-1621-cargo-workspace-completion/member/Cargo.toml"
                )),
                SchemaPath(cargo_schema_path()),
            ) -> Ok([
                "serde",
                "$crate_name",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn cargo_dependencies_workspace_inheritance_candidate_disabled_by_extensions(
                r#"
                [dependencies]
                s█
                "#,
                SourcePath(project_root_path().join(
                    "crates/tombi-lsp/tests/fixtures/extensions/cargo-disabled/member/Cargo.toml"
                )),
                SchemaPath(cargo_schema_path()),
            ) -> Ok([
                ".",
                "=",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn cargo_dev_dependencies_workspace_inheritance_candidate(
                r#"
                [dev-dependencies]
                s█
                "#,
                SourcePath(project_root_path().join(
                    "crates/tombi-lsp/tests/fixtures/issue-1621-cargo-workspace-completion/member/Cargo.toml"
                )),
                SchemaPath(cargo_schema_path()),
            ) -> Ok([
                "serde",
                ".",
                "=",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn cargo_build_dependencies_workspace_inheritance_candidate(
                r#"
                [build-dependencies]
                s█
                "#,
                SourcePath(project_root_path().join(
                    "crates/tombi-lsp/tests/fixtures/issue-1621-cargo-workspace-completion/member/Cargo.toml"
                )),
                SchemaPath(cargo_schema_path()),
            ) -> Ok([
                "serde",
                ".",
                "=",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn cargo_package_build_path_file_completion(
                r#"
                [package]
                build = "bui█"
                "#,
                SourcePath(project_root_path().join("rust/tombi-cli/Cargo.toml")),
                SchemaPath(cargo_schema_path()),
            ) -> Ok([
                "build.rs",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn cargo_dependencies_path_completion_local_prefix(
                r#"
                [dependencies]
                local-path-crate = { path = "local-█" }
                "#,
                SourcePath(project_root_path().join(
                    "crates/tombi-lsp/tests/fixtures/cargo/path-dependency-with-features/Cargo.toml"
                )),
                SchemaPath(cargo_schema_path()),
            ) -> Ok([
                "local-path-crate/",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn cargo_dependencies_inline_table_last(
                r#"
                [dependencies]
                serde = { workspace = true }█
                "#,
                SchemaPath(cargo_schema_path()),
            ) -> Ok([]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn cargo_workspace_members_path_completion_local_prefix(
                r#"
                [workspace]
                members = ["local-█"]
                "#,
                SourcePath(project_root_path().join(
                    "crates/tombi-lsp/tests/fixtures/cargo/path-dependency-with-features/Cargo.toml"
                )),
                SchemaPath(cargo_schema_path()),
            ) -> Ok([
                "local-path-crate/",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn cargo_dependencies_serde_bra_work_key(
                r#"
                [dependencies]
                serde = { work█ }
                "#,
                SchemaPath(cargo_schema_path()),
            ) -> Ok([
                "workspace = true",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn cargo_dependencies_serde_workspace(
                r#"
                [dependencies]
                serde.workspace█
                "#,
                SchemaPath(cargo_schema_path()),
            ) -> Ok([
                ".",
                "=",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn cargo_dependencies_serde_workspace_dot(
                r#"
                [dependencies]
                serde = { workspace.█ }
                "#,
                SchemaPath(cargo_schema_path()),
            ) -> Ok([
                "true",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn cargo_dependencies_serde_workspace_duplicated(
                r#"
                [dependencies]
                serde.workspace = true
                serde.work█
                "#,
                SchemaPath(cargo_schema_path()),
            ) -> Ok([]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn cargo_workspace_dependencies_tombi_date_time_features(
                r#"
                [workspace.dependencies]
                tombi-date-time = { features█, path = "crates/tombi-date-time" }
                "#,
                SourcePath(project_root_path().join("Cargo.toml")),
                SchemaPath(cargo_schema_path())
            ) -> Ok([
                ".",
                "=",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn cargo_workspace_dependencies_tombi_date_time_features_eq(
                r#"
                [workspace.dependencies]
                tombi-date-time = { features=█, path = "crates/tombi-date-time" }
                "#,
                SourcePath(project_root_path().join("Cargo.toml")),
                SchemaPath(cargo_schema_path()),
            ) -> Ok([
                "[]",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn cargo_workspace_dependencies_tombi_date_time_features_eq_array_with_workspace(
                r#"
                [workspace.dependencies]
                tombi-date-time = { features=[█], path = "crates/tombi-date-time" }
                "#,
                SourcePath(project_root_path().join("Cargo.toml")),
                SchemaPath(cargo_schema_path()),
            ) -> Ok([
                "\"default\"",
                "\"chrono\"",
                "\"serde\"",
                "\"\"",
                "''",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn cargo_dependencies_num_chrono_duration_equal(
                r#"
                [dependencies]
                num-chrono-duration=█
                "#,
                SourcePath(project_root_path().join("Cargo.toml")),
                SchemaPath(cargo_schema_path()),
            ) -> Ok([
                "\"0.1.0\"",
                "\"*\"",
                "branch",
                "default-features",
                "features",
                "git",
                "optional = true",
                "package",
                "path",
                "registry",
                "rev",
                "tag",
                "version",
                "workspace = true",
                "\"\"",
                "''",
                "{}",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn cargo_dependencies_num_chrono_duration_dot(
                r#"
                [dependencies]
                num-chrono-duration.█
                "#,
                SourcePath(project_root_path().join("Cargo.toml")),
                SchemaPath(cargo_schema_path()),
            ) -> Ok([
                "\"0.1.0\"",
                "\"*\"",
                "branch",
                "default-features",
                "features",
                "git",
                "optional = true",
                "package",
                "path",
                "registry",
                "rev",
                "tag",
                "version",
                "workspace = true",
                "\"\"",
                "''",
                "{}",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn cargo_dependencies_num_chrono_duration_equal_version(
                r#"
                [dependencies]
                num-chrono-duration = { version█ }
                "#,
                SourcePath(project_root_path().join("Cargo.toml")),
                SchemaPath(cargo_schema_path()),
            ) -> Ok([
                "\"0.1.0\"",
                ".",
                "=",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn cargo_dependencies_num_chrono_duration_equal_version_dot(
                r#"
                [dependencies]
                num-chrono-duration = { version.█ }
                "#,
                SourcePath(project_root_path().join("Cargo.toml")),
                SchemaPath(cargo_schema_path()),
            ) -> Ok([
                "\"0.1.0\"",
                "\"*\"",
                "\"\"",
                "''",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn cargo_dependencies_num_chrono_duration_equal_string(
                r#"
                [dependencies]
                num-chrono-duration = "█"
                "#,
                SourcePath(project_root_path().join("Cargo.toml")),
                SchemaPath(cargo_schema_path()),
            ) -> Ok([
                "\"0.1.0\"",
                "\"*\"",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn cargo_dependencies_num_chrono_duration_equal_string_with_comment(
                r#"
                [dependencies]
                num-chrono-duration = "█"  \# comment
                "#,
                SourcePath(project_root_path().join("Cargo.toml")),
                SchemaPath(cargo_schema_path()),
            ) -> Ok([
                "\"0.1.0\"",
                "\"*\"",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn cargo_dependencies_num_chrono_duration_equal_version_equal(
                r#"
                [dependencies]
                num-chrono-duration = { version=█ }
                "#,
                SourcePath(project_root_path().join("Cargo.toml")),
                SchemaPath(cargo_schema_path()),
            ) -> Ok([
                "\"0.1.0\"",
                "\"*\"",
                "\"\"",
                "''",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn cargo_dependencies_num_chrono_duration_equal_version_eq_string(
                r#"
                [dependencies]
                num-chrono-duration = { version= "█" }
                "#,
                SourcePath(project_root_path().join("Cargo.toml")),
                SchemaPath(cargo_schema_path()),
            ) -> Ok([
                "\"0.1.0\"",
                "\"*\"",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn cargo_dependencies_semver_versions_are_sorted_descending(
                r#"
                [dependencies]
                tombi-semver-order-test = "█"
                "#,
                SourcePath(project_root_path().join("Cargo.toml")),
                SchemaPath(cargo_schema_path()),
                tombi_lsp::backend::Options {
                    offline: Some(true),
                    no_cache: None,
                },
                PreselectedLabels(&["\"0.3.0-pre.2\""]),
                CachedResponse::new(
                    "https://crates.io/api/v1/crates/tombi-semver-order-test/versions",
                    r#"{"versions":[
                        {"num":"0.2.7","features":{}},
                        {"num":"0.2.6","features":{}},
                        {"num":"0.2.0-pre.1","features":{}},
                        {"num":"0.3.0-pre.2","features":{}},
                        {"num":"0.1.12","features":{}}
                    ]}"#,
                ),
            ) -> Ok([
                "\"0.3.0-pre.2\"",
                "\"0.2.7\"",
                "\"0.2.6\"",
                "\"0.2.0-pre.1\"",
                "\"0.1.12\"",
                "\"*\"",
            ]);
        }

        test_completion_labels! {
         #[tokio::test]
            async fn cargo_dependencies_tombi_date_time_features_with_workspace_eq_true_comma(
                r#"
                [dependencies]
                tombi-date-time = { workspace = true, █ }
                "#,
                SourcePath(project_root_path().join("crates/subcrate/Cargo.toml")),
                SchemaPath(cargo_schema_path()),
            ) -> Ok([
                "branch",
                "default-features",
                "features",
                "git",
                "optional = true",
                "package",
                "path",
                "registry",
                "rev",
                "tag",
                "version",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn cargo_dependencies_tombi_date_time_features_with_workspace(
                r#"
                [dependencies]
                tombi-date-time = { workspace = true, features█ }
                "#,
                SourcePath(project_root_path().join("crates/subcrate/Cargo.toml")),
                SchemaPath(cargo_schema_path()),
            ) -> Ok([
                ".",
                "=",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn cargo_dependencies_tombi_date_time_features_eq_with_workspace(
                r#"
                [dependencies]
                tombi-date-time = { workspace = true, features=█ }
                "#,
                SourcePath(project_root_path().join("crates/subcrate/Cargo.toml")),
                SchemaPath(cargo_schema_path()),
            ) -> Ok([
                "[]",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn cargo_dependencies_tombi_date_time_features_eq_array_with_workspace(
                r#"
                [dependencies]
                tombi-date-time = { workspace = true, features=[█] }
                "#,
                SourcePath(project_root_path().join("crates/subcrate/Cargo.toml")),
                SchemaPath(cargo_schema_path()),
            ) -> Ok([
                "\"default\"",
                "\"chrono\"",
                "\"serde\"",
                "\"\"",
                "''",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn cargo_dependencies_tombi_date_time_features_eq_array_with_path(
                r#"
                [dependencies]
                tombi-date-time = { path = "../tombi-date-time", features=[█] }
                "#,
                SourcePath(project_root_path().join("crates/tombi-document/Cargo.toml")),
                SchemaPath(cargo_schema_path()),
            ) -> Ok([
                "\"default\"",
                "\"chrono\"",
                "\"serde\"",
                "\"\"",
                "''",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn cargo_dependencies_local_path_features(
                r#"
                [dependencies]
                local-path-crate = { path = "local-path-crate", features = [█] }
                "#,
                SourcePath(project_root_path().join(
                    "crates/tombi-lsp/tests/fixtures/cargo/path-dependency-with-features/Cargo.toml"
                )),
                SchemaPath(cargo_schema_path()),
            ) -> Ok([
                "\"default\"",
                "\"extras\"",
                "\"flag\"",
                "\"\"",
                "''",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn cargo_dependencies_local_path_no_features(
                r#"
                [dependencies]
                local-path-no-features = { path = "local-path-no-features", features = [█] }
                "#,
                SourcePath(project_root_path().join(
                    "crates/tombi-lsp/tests/fixtures/cargo/path-dependency-no-features/Cargo.toml"
                )),
                SchemaPath(cargo_schema_path()),
            ) -> Ok([
                "\"\"",
                "''",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn cargo_dependencies_patch(
                r#"
                [patch]
                █
                "#,
                SchemaPath(cargo_schema_path()),
            ) -> Ok([
                "crates-io",
                "$source_url_or_registry_name"
            ]);
        }

        // Tests for platform specific dependencies (Issue #1192)
        test_completion_labels! {
            #[tokio::test]
            async fn cargo_target_dependencies(
                r#"
                [target.'cfg(unix)'.dependencies]
                █
                "#,
                SchemaPath(cargo_schema_path()),
            ) -> Ok([
                "$crate_name",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn cargo_target_dependencies_keys(
                r#"
                [target.'cfg(unix)'.dependencies]
                serde = { █ }
                "#,
                SchemaPath(cargo_schema_path()),
            ) -> Ok([
                "branch",
                "default-features",
                "features",
                "git",
                "optional = true",
                "package",
                "path",
                "registry",
                "rev",
                "tag",
                "version",
                "workspace = true",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn cargo_target_dependencies_tombi_date_time_features_eq_array_with_path(
                r#"
                [target.'cfg(unix)'.dependencies]
                tombi-date-time = { features=[█], path = "crates/tombi-date-time" }
                "#,
                SourcePath(project_root_path().join("Cargo.toml")),
                SchemaPath(cargo_schema_path()),
            ) -> Ok([
                "\"default\"",
                "\"chrono\"",
                "\"serde\"",
                "\"\"",
                "''",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn cargo_target_dev_dependencies(
                r#"
                [target.'cfg(target_os = "linux")'.dev-dependencies]
                █
                "#,
                SchemaPath(cargo_schema_path()),
            ) -> Ok([
                "$crate_name",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn cargo_package_license(
                r#"
                [package]
                license = █
                "#,
                SchemaPath(cargo_schema_path()),
            ) -> Ok([
                "\"Apache-2.0\"",
                "\"BSD-2-Clause\"",
                "\"GPL-2.0-or-later WITH Bison-exception-2.2\"",
                "\"LGPL-2.1-only\"",
                "\"MIT\"",
                "workspace = true",
                "\"\"",
                "''",
                "{}",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn cargo_target_build_dependencies(
                r#"
                [target.'cfg(windows)'.build-dependencies]
                █
                "#,
                SchemaPath(cargo_schema_path()),
            ) -> Ok([
                "$crate_name",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn cargo_target_dependencies_workspace_inheritance_candidate(
                r#"
                [target.'cfg(unix)'.dependencies]
                s█
                "#,
                SourcePath(project_root_path().join(
                    "crates/tombi-lsp/tests/fixtures/issue-1621-cargo-workspace-completion/member/Cargo.toml"
                )),
                SchemaPath(cargo_schema_path()),
            ) -> Ok([
                "serde",
                ".",
                "=",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn cargo_target_build_dependencies_workspace_inheritance_candidate(
                r#"
                [target.'cfg(unix)'.build-dependencies]
                s█
                "#,
                SourcePath(project_root_path().join(
                    "crates/tombi-lsp/tests/fixtures/issue-1621-cargo-workspace-completion/member/Cargo.toml"
                )),
                SchemaPath(cargo_schema_path()),
            ) -> Ok([
                "serde",
                ".",
                "=",
            ]);
        }
    }

    mod untagged_union {
        use tombi_test_lib::untagged_union_schema_path;

        use super::*;

        test_completion_labels! {
            #[tokio::test]
            async fn untagged_union(
                "█",
                SchemaPath(untagged_union_schema_path()),
            ) -> Ok([
                "favorite_color",
                "number_of_pets",
            ]);
        }
    }

    mod one_of_regression {
        use super::*;

        test_completion_labels! {
            #[tokio::test]
            async fn one_of_variant_after_path_enabled(
                r#"
                completion = {
                  █
                  path.enabled = true,
                }
                "#,
                SchemaPath(project_root_path().join(
                    "crates/tombi-lsp/tests/fixtures/feature-toggle-one-of-test.schema.json"
                )),
            ) -> Ok([]);
        }
    }

    mod without_schema {
        use super::*;

        test_completion_labels! {
            #[tokio::test]
            async fn empty(
                "█"
            ) -> Ok(["$key"]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn key(
                "key█"
            ) -> Ok([".", "="]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn key_dot(
                "key.█"
            ) -> Ok(AnyValue);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn key_equal(
                "key=█"
            ) -> Ok(AnyValue);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn keys_dot(
                "key1.key2.█"
            ) -> Ok(AnyValue);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn keys_equal(
                "key1.key2=█"
            ) -> Ok(AnyValue);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn keys_equal_array(
                "key1= [█]"
            ) -> Ok(AnyValue);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn aaa_equal_inline_table_bbb(
                "aaa = { bbb█ }"
            ) -> Ok([
                ".",
                "=",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn aaa_equal_inline_table_last(
                "aaa = { bbb = 1 }█"
            ) -> Ok([]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn aaa_equal_array_bbb(
                "aaa = [bbb█]"
            ) -> Ok(["$key"]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn aaa_equal_array_inline_table_bbb(
                "aaa = [{ bbb█ }]"
            ) -> Ok([
                ".",
                "=",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn aaa_equal_array_inline_table_ccc_equal_inline_table_bbb(
                "aaa = [{ ccc = { bbb█ } }]"
            ) -> Ok([
                ".",
                "=",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn aaa_equal_array_inline_table_ccc_dot_ddd_equal_inline_table_bbb(
                "aaa = [{ ccc.ddd = { bbb█ } }]"
            ) -> Ok([
                ".",
                "=",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn aaa_equal_array_1_comma_bbb(
                "aaa = [1, bbb.█]"
            ) -> Ok(AnyValue);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn aaa_bbb_double_bracket_ccc(
                r#"
                [[aaa.bbb]]
                ccc█
                "#
            ) -> Ok([
                ".",
                "=",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn aaa_bbb_double_bracket_ccc_equal(
                r#"
                [[aaa.bbb]]
                ccc=█
                "#
            ) -> Ok(AnyValue);
        }
    }

    mod type_test_schema {
        use tombi_test_lib::type_test_schema_path;

        use super::*;

        test_completion_labels! {
            #[tokio::test]
            async fn type_test_schema(
                "█",
                SchemaPath(type_test_schema_path()),
            ) -> Ok([
                "array",
                "boolean",
                "float",
                "integer",
                "literal",
                "local-date",
                "local-date-time",
                "local-time",
                "offset-date-time",
                "string",
                "table",
                "table-allows-empty-key",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn type_test_schema_invalid_key_comment_directive(
                r#"
                # tombi: lint.█
                "" = 1
                "#,
                SchemaPath(type_test_schema_path()),
            ) -> Ok([
                "rules",
                "{}",
            ]);
        }
    }

    mod with_subschema {
        use tombi_test_lib::{project_root_path, pyproject_schema_path, type_test_schema_path};

        use super::*;

        test_completion_labels! {
            #[tokio::test]
            async fn pyproject_tool_type_test(
                r#"
                [tool.type_test]
                █
                "#,
                SchemaPath(pyproject_schema_path()),
                SubSchema {
                    root: "tool.type_test",
                    path: type_test_schema_path(),
                },
            ) -> Ok([
                "array",
                "boolean",
                "float",
                "integer",
                "literal",
                "local-date",
                "local-date-time",
                "local-time",
                "offset-date-time",
                "string",
                "table",
                "table-allows-empty-key",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn aaa_bbb_type_test(
                r#"
                [aaa.bbb]
                █
                "#,
                SubSchema {
                    root: "aaa.bbb",
                    path: type_test_schema_path(),
                },
            ) -> Ok([
                "array",
                "boolean",
                "float",
                "integer",
                "literal",
                "local-date",
                "local-date-time",
                "local-time",
                "offset-date-time",
                "string",
                "table",
                "table-allows-empty-key",
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn aaa_bbb_singleton_any_of_subschema(
                r#"
                [aaa.bbb]
                fl█
                "#,
                SubSchema {
                    root: "aaa.bbb",
                    path: project_root_path().join("schemas/subschema-singleton-label-test.schema.json"),
                },
            ) -> Ok([
                "flag = true",
            ]);
        }
    }

    mod string_format_test_schema {
        use super::*;

        test_completion_labels! {
            #[tokio::test]
            async fn completion_date_val_with_string_formats(
                r#"
                date_val = █
                "#,
                SchemaPath(string_format_test_schema_path()),
            ) -> Ok([
                "\"\"",
                "''",
                today_local_date(),
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn completion_time_local_val_with_string_formats(
                r#"
                time_local_val = █
                "#,
                SchemaPath(string_format_test_schema_path()),
            ) -> Ok([
                "\"\"",
                "''",
                today_local_time(),
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn completion_date_time_local_val_with_string_formats(
                r#"
                date_time_local_val = █
                "#,
                SchemaPath(string_format_test_schema_path()),
            ) -> Ok([
                "\"\"",
                "''",
                today_local_date_time(),
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn completion_date_time_val_with_string_formats(
                r#"
                date_time_val = █
                "#,
                SchemaPath(string_format_test_schema_path()),
            ) -> Ok([
                "\"\"",
                "''",
                today_offset_date_time(),
            ]);
        }

        test_completion_labels! {
            #[tokio::test]
            async fn completion_ipv4_addr_no_string_type_hint(
                r#"
                ipv4_addr = █
                "#,
                SchemaPath(string_format_test_schema_path()),
            ) -> Ok([
                "\"\"",
                "''",
            ]);
        }
    }

    #[macro_export]
    macro_rules! test_completion_labels {
        (
            #[tokio::test]
            async fn $name:ident($source:expr $(, $arg:expr )* $(,)?) -> Ok(AnyValue);
        ) => {
            test_completion_labels! {
                #[tokio::test]
                async fn $name($source $(, $arg)*) -> Ok([
                    "\"\"",
                    "''",
                    today_local_time(),
                    today_local_date(),
                    today_local_date_time(),
                    today_offset_date_time(),
                    "3.14",
                    "42",
                    "[]",
                    "{}",
                    "$key",
                    "true",
                    "false",
                ]);
            }
        };

        (
            #[tokio::test]
            async fn $name:ident($source:expr $(, $arg:expr )* $(,)?) -> Ok([
                $({
                    "label": $label:expr,
                    "documentation": $documentation:expr $(,)?
                }),* $(,)?
            ]);
        ) => {
            test_completion_labels! {
                @run_documented
                #[tokio::test]
                async fn $name($source $(, $arg)*) -> docs [$(
                    ($label, $documentation)
                ),*];
            }
        };

        (
            #[tokio::test]
            async fn $name:ident($source:expr $(, $arg:expr )* $(,)?) -> Ok([$($label:expr),*$(,)?]);
        ) => {
            test_completion_labels! {
                @run
                #[tokio::test]
                async fn $name($source $(, $arg)*) -> labels [$($label),*] docs [];
            }
        };

        (
            @run
            #[tokio::test]
            async fn $name:ident($source:expr $(, $arg:expr )* $(,)?) -> labels [$($label:expr),*$(,)?] docs [$(
                ($doc_label:expr, $documentation:expr)
            ),*$(,)?];
        ) => {
            test_completion_labels! {
                @run_documented
                #[tokio::test]
                async fn $name($source $(, $arg)*) -> labels [$($label),*] docs [$(($doc_label, $documentation)),*];
            }
        };

        (
            @run_documented
            #[tokio::test]
            async fn $name:ident($source:expr $(, $arg:expr )* $(,)?) -> docs [$(
                ($doc_label:expr, $documentation:expr)
            ),*$(,)?];
        ) => {
            test_completion_labels! {
                @run_documented
                #[tokio::test]
                async fn $name($source $(, $arg)*) -> labels [] skip_label_assertion true docs [$(($doc_label, $documentation)),*];
            }
        };

        (
            @run_documented
            #[tokio::test]
            async fn $name:ident($source:expr $(, $arg:expr )* $(,)?) -> labels [$($label:expr),*$(,)?] docs [$(
                ($doc_label:expr, $documentation:expr)
            ),*$(,)?];
        ) => {
            test_completion_labels! {
                @run_documented
                #[tokio::test]
                async fn $name($source $(, $arg)*) -> labels [$($label),*] skip_label_assertion false docs [$(($doc_label, $documentation)),*];
            }
        };

        (
            @run_documented
            #[tokio::test]
            async fn $name:ident($source:expr $(, $arg:expr )* $(,)?) -> labels [$($label:expr),*$(,)?] skip_label_assertion $skip_label_assertion:tt docs [$(
                ($doc_label:expr, $documentation:expr)
            ),*$(,)?];
        ) => {
            #[tokio::test]
            async fn $name() -> Result<(), Box<dyn std::error::Error>> {
                use itertools::Itertools;
                use tombi_lsp::Backend;
                use std::io::Write;
                use tower_lsp::{
                    lsp_types::{
                        CompletionItem, CompletionParams, DidOpenTextDocumentParams,
                        PartialResultParams, TextDocumentIdentifier, TextDocumentItem,
                        TextDocumentPositionParams, Url, WorkDoneProgressParams,
                    },
                    LspService,
                };
                use tombi_lsp::handler::handle_did_open;
                use tombi_text::IntoLsp;

                tombi_test_lib::init_log();

                #[allow(unused)]
                #[derive(Default)]
                pub struct TestArgs {
                    source_file_path: Option<std::path::PathBuf>,
                    schema_file_path: Option<std::path::PathBuf>,
                    subschemas: Vec<SubSchema>,
                    cached_responses: Vec<CachedResponse>,
                    preselected_labels: Option<Vec<String>>,
                    backend_options: tombi_lsp::backend::Options,
                }

                #[allow(unused)]
                pub trait ApplyTestArg {
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
                struct SubSchema {
                    pub root: &'static str,
                    pub path: std::path::PathBuf,
                }

                impl ApplyTestArg for SubSchema {
                    fn apply(self, args: &mut TestArgs) {
                        args.subschemas.push(self);
                    }
                }

                #[allow(unused)]
                struct CachedResponse {
                    url: &'static str,
                    body: &'static str,
                }

                #[allow(unused)]
                impl CachedResponse {
                    fn new(url: &'static str, body: &'static str) -> Self {
                        Self { url, body }
                    }
                }

                impl ApplyTestArg for CachedResponse {
                    fn apply(self, args: &mut TestArgs) {
                        args.cached_responses.push(self);
                    }
                }

                #[allow(unused)]
                struct PreselectedLabels(&'static [&'static str]);

                impl ApplyTestArg for PreselectedLabels {
                    fn apply(self, args: &mut TestArgs) {
                        args.preselected_labels =
                            Some(self.0.iter().map(ToString::to_string).collect());
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

                let _cache_home = if args.cached_responses.is_empty() {
                    None
                } else {
                    Some(TestCacheHome::new())
                };

                let (service, _) =
                    LspService::new(|client| Backend::new(client, &args.backend_options));
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
                        root: subschema.root.to_string(),
                        lint: None,
                        format: None,
                        overrides: None,
                    }));
                }

                for cached_response in &args.cached_responses {
                    let uri = cached_response
                        .url
                        .parse::<tombi_uri::Uri>()
                        .map_err(|err| format!("failed to parse cache URL {}: {err}", cached_response.url))?;
                    let cache_path = tombi_cache::get_cache_file_path(&uri)
                        .await
                        .ok_or_else(|| {
                            format!(
                                "failed to derive cache path for {}",
                                cached_response.url
                            )
                        })?;
                    if let Some(parent) = cache_path.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    std::fs::write(&cache_path, cached_response.body)?;
                }

                let Ok(temp_file) = tempfile::NamedTempFile::with_suffix_in(
                    ".toml",
                    std::env::current_dir().expect("failed to get current directory"),
                ) else {
                    return Err("failed to create a temporary file for the test data".into());
                };

                let mut toml_text = textwrap::dedent($source).trim().to_string();

                let Some(index) = toml_text.as_str().find("█") else {
                    return Err(
                        "failed to find completion position marker (█) in the test data".into()
                    );
                };

                toml_text.remove(index);
                if temp_file.as_file().write_all(toml_text.as_bytes()).is_err() {
                    return Err(
                        "failed to write test data to the temporary file, which is used as a text document"
                            .into(),
                    );
                };
                let line_index =
                    tombi_text::LineIndex::new(&toml_text, tombi_text::EncodingKind::Utf16);

                let source_path = args.source_file_path.as_deref().unwrap_or(temp_file.path());
                let toml_file_url = Url::from_file_path(source_path)
                    .map_err(|_| "failed to convert file path to URL")?;

                if !schema_items.is_empty() {
                    let config_schema_store = backend
                        .config_manager
                        .config_schema_store_for_file(&source_path)
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

                let Ok(Some(completions)) = tombi_lsp::handler::handle_completion(
                    &backend,
                    CompletionParams {
                        text_document_position: TextDocumentPositionParams {
                            text_document: TextDocumentIdentifier { uri: toml_file_url },
                            position: (tombi_text::Position::default()
                                + tombi_text::RelativePosition::of(&toml_text[..index]))
                            .into_lsp(&line_index),
                        },
                        work_done_progress_params: WorkDoneProgressParams::default(),
                        partial_result_params: PartialResultParams {
                            partial_result_token: None,
                        },
                        context: None,
                    },
                )
                .await
                else {
                    return Err("failed to handle completion".into());
                };

                let completion_items = completions
                    .into_iter()
                    .map(|content| IntoLsp::<CompletionItem>::into_lsp(content, &line_index))
                    .sorted_by(|a, b| {
                        a.sort_text
                            .as_ref()
                            .unwrap_or(&a.label)
                            .cmp(&b.sort_text.as_ref().unwrap_or(&b.label))
                    })
                    .collect_vec();

                let labels = completion_items
                    .iter()
                    .map(|item| item.label.clone())
                    .collect_vec();

                let expected_labels = vec![$($label.to_string()),*] as Vec<String>;

                if !$skip_label_assertion {
                    pretty_assertions::assert_eq!(labels, expected_labels);
                }

                if let Some(expected_preselected_labels) = args.preselected_labels {
                    let preselected_labels = completion_items
                        .iter()
                        .filter(|item| item.preselect == Some(true))
                        .map(|item| item.label.clone())
                        .collect_vec();

                    pretty_assertions::assert_eq!(
                        preselected_labels,
                        expected_preselected_labels
                    );
                }

                let expected_documentation_pairs: Vec<(String, String)> =
                    vec![$(($doc_label.to_string(), $documentation.to_string())),*];

                for (expected_label, expected_documentation) in expected_documentation_pairs {
                    let item = completion_items
                        .iter()
                        .find(|item| item.label == expected_label)
                        .ok_or_else(|| {
                            format!(
                                "failed to find completion item {expected_label:?}: {labels:?}"
                            )
                        })?;

                    let documentation = match &item.documentation {
                        Some(tower_lsp::lsp_types::Documentation::String(value)) => {
                            Some(value.clone())
                        }
                        Some(tower_lsp::lsp_types::Documentation::MarkupContent(content)) => {
                            Some(content.value.clone())
                        }
                        None => None,
                    };

                    assert!(
                        documentation
                            .as_deref()
                            .is_some_and(|documentation| documentation.contains(&expected_documentation)),
                        "completion documentation for {expected_label:?} did not include expected text {expected_documentation:?}: {documentation:?}"
                    );
                }

                Ok(())
            }
        };
    }
}
