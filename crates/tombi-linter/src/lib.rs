mod diagnostic;
mod error;
mod lint;
mod linter;
mod rule;

pub use diagnostic::{Diagnostic, DiagnosticKind};
pub use error::{Error, ErrorKind};
use lint::Lint;
pub use linter::Linter;
use rule::Rule;
pub use tombi_config::LintOptions;

#[cfg(test)]
#[macro_export]
macro_rules! test_lint {
    (
        #[test]
        fn $name:ident(
            $source:expr$(,)?
        ) -> Ok(_);
    ) => {
        test_lint! {
            #[test]
            fn _$name(
                $source,
                Option::<std::path::PathBuf>::None,
                TomlVersion::default(),
            ) -> Ok(_);
        }
    };

    (
        #[test]
        fn $name:ident(
            $source:expr,
            TomlVersion($toml_version:expr)$(,)?
        ) -> Ok(_);
    ) => {
        test_lint! {
            #[test]
            fn _$name(
                $source,
                Option::<std::path::PathBuf>::None,
                $toml_version,
            ) -> Ok(_);
        }
    };

    (
        #[test]
        fn $name:ident(
            $source:expr,
            $schema_path:expr$(,)?
        ) -> Ok(_);
    ) => {
        test_lint! {
            #[test]
            fn _$name(
                $source,
                Some($schema_path),
                TomlVersion::default(),
            ) -> Ok(_);
        }
    };

    (
        #[test]
        fn _$name:ident(
            $source:expr,
            $schema_path:expr,
            $toml_version:expr,
        ) -> Ok(_);
    ) => {
        #[tokio::test]
        async fn $name() {
            use tombi_config::TomlVersion;

            tombi_test_lib::init_tracing();

            // Initialize schema store
            let schema_store = tombi_schema_store::SchemaStore::new();

            if let Some(schema_path) = $schema_path {
                // Load schemas
                schema_store
                    .load_config_schemas(
                        &[tombi_config::Schema::Root(tombi_config::RootSchema {
                            toml_version: None,
                            path: schema_path.to_string_lossy().to_string(),
                            include: vec!["*.toml".to_string()],
                        })],
                        None,
                    )
                    .await;
            }

            // Initialize linter with schema if provided
            let source_path = tombi_test_lib::project_root_path().join("test.toml");
            let options = $crate::LintOptions::default();
            let linter = $crate::Linter::new(
                $toml_version,
                &options,
                Some(itertools::Either::Right(source_path.as_path())),
                &schema_store,
            );

            match linter.lint($source).await {
                Ok(_) => {}
                Err(errors) => {
                    pretty_assertions::assert_eq!(
                        Vec::<tombi_diagnostic::Diagnostic>::new(),
                        errors,
                        "Expected success but got errors."
                    );
                }
            }
        }
    };

    (
        #[test]
        fn $name:ident(
            $source:expr,
            $schema_path:expr$(,)?
        ) -> Err([$( $error:expr ),*$(,)?]);
    ) => {
        test_lint! {
            #[test]
            fn _$name(
                $source,
                Some($schema_path),
                TomlVersion::default(),
            ) -> Err([$($error.to_string()),*]);
        }
    };

    (
        #[test]
        fn $name:ident(
            $source:expr,
        ) -> Err([$( $error:expr ),*$(,)?]);
    ) => {
        test_lint! {
            #[test]
            fn _$name(
                $source,
                Option::<std::path::PathBuf>::None,
                TomlVersion::default(),
            ) -> Err([$($error.to_string()),*]);
        }
    };

    (
        #[test]
        fn $name:ident(
            $source:expr,
            TomlVersion($toml_version:expr),
        ) -> Err([$( $error:expr ),*$(,)?]);
    ) => {
        test_lint! {
            #[test]
            fn _$name(
                $source,
                Option::<std::path::PathBuf>::None,
                $toml_version,
            ) -> Err([$($error.to_string()),*]);
        }
    };

    (
        #[test]
        fn _$name:ident(
            $source:expr,
            $schema_path:expr,
            $toml_version:expr,
        ) -> Err([$( $error:expr ),*$(,)?]);
    ) => {
        #[tokio::test]
        async fn $name() {
            use tombi_config::TomlVersion;
            use itertools::Itertools;

            tombi_test_lib::init_tracing();

            // Initialize schema store
            let schema_store = tombi_schema_store::SchemaStore::new();

            if let Some(schema_path) = $schema_path {
                // Load schemas
                schema_store
                    .load_config_schemas(
                        &[tombi_config::Schema::Root(tombi_config::RootSchema {
                            toml_version: None,
                            path: schema_path.to_string_lossy().to_string(),
                            include: vec!["*.toml".to_string()],
                        })],
                        None,
                    )
                    .await;
            }

            // Initialize linter with schema if provided
            let source_path = tombi_test_lib::project_root_path().join("test.toml");
            let options = $crate::LintOptions::default();
            let linter = $crate::Linter::new(
                $toml_version,
                &options,
                Some(itertools::Either::Right(source_path.as_path())),
                &schema_store,
            );

            let result = linter.lint($source).await;
            match result {
                Ok(_) => {
                    panic!("Expected errors but got success");
                }
                Err(errors) => {
                    pretty_assertions::assert_eq!(
                        errors
                            .into_iter()
                            .map(|error| error.message().to_string())
                            .collect_vec(),
                        [$($error.to_string()),*].into_iter().collect::<Vec<String>>()
                    );
                }
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    mod cargo_schema {
        use super::*;
        use tombi_test_lib::cargo_schema_path;

        test_lint! {
            #[test]
            fn test_workspace_dependencies(
                r#"
                [workspace.dependencies]
                serde.version = "^1.0.0"
                serde.features = ["derive"]
                serde.workspace = true
                "#,
                cargo_schema_path(),
            ) -> Ok(_);
        }

        test_lint! {
            #[test]
            fn test_workspace_unknown(
                r#"
                [workspace]
                aaa = 1
                "#,
                cargo_schema_path(),
            ) -> Err([tombi_validator::DiagnosticKind::StrictAdditionalKeys {
                accessors: tombi_schema_store::SchemaAccessors::from(vec![
                    tombi_schema_store::SchemaAccessor::Key("workspace".to_string()),
                ]),
                schema_uri: tombi_schema_store::SchemaUri::from_file_path(cargo_schema_path()).unwrap(),
                key: "aaa".to_string(),
            }]);
        }

        test_lint! {
            #[test]
            fn test_unonkwn_keys(
                r#"
                [aaa]
                bbb = 1
                "#,
                cargo_schema_path(),
            ) -> Err([tombi_validator::DiagnosticKind::KeyNotAllowed { key: "aaa".to_string() }]);
        }

        test_lint! {
            #[test]
            fn test_package_name_wrong_type(
                r#"
                [package]
                name = 1
                "#,
                cargo_schema_path(),
            ) -> Err([tombi_validator::DiagnosticKind::TypeMismatch {
                expected: tombi_schema_store::ValueType::String,
                actual: tombi_document_tree::ValueType::Integer,
            }]);
        }

        test_lint! {
            #[test]
            fn test_package_name_wrong_type_with_comment_directive_disabled_eq_true(
                r#"
                [package]
                name = 1 # tombi: lint.rules.type-mismatch.disabled = true
                "#,
                cargo_schema_path(),
            ) -> Ok(_);
        }

        test_lint! {
            #[test]
            fn test_package_name_wrong_type_with_wrong_comment_directive_disabled_eq_true(
                r#"
                [package]
                name = 1 # tombi: lint.rules.type-mism.disabled = true
                "#,
                cargo_schema_path(),
            ) -> Err([
                tombi_validator::DiagnosticKind::KeyNotAllowed { key: "type-mism".to_string() },
                tombi_validator::DiagnosticKind::TypeMismatch {
                    expected: tombi_schema_store::ValueType::String,
                    actual: tombi_document_tree::ValueType::Integer,
                }
            ]);
        }
    }

    mod tombi_schema {
        use super::*;
        use tombi_test_lib::tombi_schema_path;

        test_lint! {
            #[test]
            fn test_tombi_config_in_this_repository(
                include_str!("../../../tombi.toml"),
                tombi_schema_path(),
            ) -> Ok(_);
        }

        test_lint! {
            #[test]
            fn test_tombi_schema_format_array_bracket_space_width_eq_0(
                r#"
                [format]
                array-bracket-space-width = 0
                "#,
                tombi_schema_path(),
            ) -> Err([
                tombi_validator::DiagnosticKind::Deprecated(
                    tombi_schema_store::SchemaAccessors::from(
                        vec![
                            tombi_schema_store::SchemaAccessor::Key("format".to_string()),
                            tombi_schema_store::SchemaAccessor::Key("array-bracket-space-width".to_string()),
                        ]
                    ),
                )
            ]);
        }

        test_lint! {
            #[test]
            fn test_tombi_schema_invalid_root(
                r#"
                [[schemas]]
                path = "schemas/partial-taskipy.schema.json"
                include = ["pyproject.toml"]
                root-keys = "tool.taskipy"
                "#,
                tombi_schema_path(),
            ) -> Err([
                tombi_validator::DiagnosticKind::DeprecatedValue(
                    tombi_schema_store::SchemaAccessors::from(
                        vec![
                            tombi_schema_store::SchemaAccessor::Key("schemas".to_string()),
                            tombi_schema_store::SchemaAccessor::Index,
                            tombi_schema_store::SchemaAccessor::Key("root-keys".to_string()),
                        ]
                    ),
                    "\"tool.taskipy\"".to_string(),
                )
            ]);
        }

        test_lint! {
            #[test]
            fn test_tombi_schema_lint_rules_with_unknown_key(
                r#"
                [[schemas]]
                root = "tool.taskipy"
                path = "schemas/partial-taskipy.schema.json"
                include = ["pyproject.toml"]
                unknown = true
                "#,
                tombi_schema_path(),
            ) -> Err([
                tombi_validator::DiagnosticKind::KeyNotAllowed {
                    key: "unknown".to_string(),
                },
            ]);
        }

        test_lint! {
            #[test]
            fn test_tombi_schema_lint_rules_key_empty_undefined(
                r#"
                [lint.rules]
                key-empty = "undefined"
                "#,
                tombi_schema_path(),
            ) -> Err([
                tombi_validator::DiagnosticKind::Enumerate {
                    expected: vec!["\"off\"".to_string(), "\"warn\"".to_string(), "\"error\"".to_string()],
                    actual: "\"undefined\"".to_string()
                },
            ]);
        }
    }

    mod untagged_union_schema {
        use super::*;
        use tombi_test_lib::untagged_union_schema_path;

        test_lint! {
            #[test]
            fn test_untagged_union_schema(
                r#"
                #:schema schemas/untagged-union.schema.json

                favorite_color = "blue"
                "#,
                untagged_union_schema_path(),
            ) -> Ok(_);
        }
    }

    mod other_schema {
        test_lint! {
            // Ref: https://github.com/tombi-toml/tombi/issues/517
            #[test]
            fn test_mise_toml(
                r#"
                #:schema https://mise.jdx.dev/schema/mise.json

                [env]
                PROJECT_SLUG = '{{ config_root | basename | slugify }}'

                _.python.venv.path = '{% if env.UV_PROJECT_ENVIRONMENT %}{{ env.UV_PROJECT_ENVIRONMENT }}{% else %}.venv{% endif %}'
                _.python.venv.create = true

                # Flask/Poster dev ONLY settings
                FLASK_DEBUG=1
                "#,
            ) -> Ok(_);
        }
    }

    mod non_schema {
        use std::str::FromStr;
        use tombi_schema_store::SchemaUri;

        use super::*;

        test_lint! {
            #[test]
            // Ref: https://github.com/tombi-toml/tombi/issues/1031
            fn test_error_report_case1(
                r#"
                [job]
                name = "foo"
                prod.cpu = 10
                prod.autoscale = { min = 10, max = 20 }
                "#,
            ) -> Ok(_);
        }

        test_lint! {
            #[test]
            fn test_warning_empty(
                r#"
                "" = 1
                "#,
            ) -> Err([
                crate::DiagnosticKind::KeyEmpty
            ]);
        }

        test_lint! {
            #[test]
            fn test_empty_document_with_dangling_value_comment_directive(
                r#"
                # tombi: format.rules.table-keys-order = "descending"
                "#,
            ) -> Ok(_);
        }

        test_lint! {
            #[test]
            fn test_key_value_with_dangling_value_comment_directive(
                r#"
                # tombi: format.rules.table-keys-order = "descending"

                key = "value"
                "#,
            ) -> Ok(_);
        }

        test_lint! {
            #[test]
            fn test_table_with_dangling_value_comment_directive(
                r#"
                # tombi: format.rules.table-keys-order = "descending"

                [aaa]
                "#,
            ) -> Ok(_);
        }

        test_lint! {
            #[test]
            fn test_table_warning_empty(
                r#"
                [aaa]
                "" = 1
                "#,
            ) -> Err([
                crate::DiagnosticKind::KeyEmpty
            ]);
        }

        test_lint! {
            #[test]
            fn test_array_of_table_warning_empty(
                r#"
                [[aaa]]
                "" = 1
                "#,
            ) -> Err([
                crate::DiagnosticKind::KeyEmpty
            ]);
        }

        test_lint! {
            #[test]
            fn test_inline_table_warning_empty(
                r#"
                key = { "" = 1 }
                "#,
            ) -> Err([
                crate::DiagnosticKind::KeyEmpty
            ]);
        }

        test_lint! {
            #[test]
            fn test_nested_inline_table_warning_empty(
                r#"
                key = { key2 = { "" = 1 } }
                "#,
            ) -> Err([
                crate::DiagnosticKind::KeyEmpty
            ]);
        }

        test_lint! {
            #[test]
            fn test_table_inline_table_warning_empty(
                r#"
                [array]
                key = { "" = 1 }
                "#,
            ) -> Err([
                crate::DiagnosticKind::KeyEmpty
            ]);
        }

        test_lint! {
            #[test]
            fn test_array_of_table_inline_table_warning_empty(
                r#"
                [[array]]
                key = { "" = 1 }
                "#,
            ) -> Err([
                crate::DiagnosticKind::KeyEmpty
            ]);
        }

        test_lint! {
            #[test]
            fn test_dotted_keys_out_of_order(
                r#"
                apple.type = "fruit"
                orange.type = "fruit"

                apple.skin = "thin"
                orange.skin = "thick"

                apple.color = "red"
                orange.color = "orange"
                "#,
            ) -> Err([
                crate::DiagnosticKind::DottedKeysOutOfOrder,
                crate::DiagnosticKind::DottedKeysOutOfOrder,
                crate::DiagnosticKind::DottedKeysOutOfOrder,
                crate::DiagnosticKind::DottedKeysOutOfOrder,
                crate::DiagnosticKind::DottedKeysOutOfOrder,
                crate::DiagnosticKind::DottedKeysOutOfOrder
            ]);
        }

        test_lint! {
            #[test]
            fn test_dotted_keys_out_of_order_with_comment_directive_table_keys_order_disabled_eq_true(
                r#"
                # tombi: format.rules.table-keys-order.disabled = true

                apple.type = "fruit"
                orange.type = "fruit"

                apple.skin = "thin"
                orange.skin = "thick"

                apple.color = "red"
                orange.color = "orange"
                "#,
            ) -> Ok(_);
        }

        test_lint! {
            #[test]
            fn test_schema_uri(
                r#"
                #:schema https://www.schemastore.org/tombi.json
                "#,
            ) -> Ok(_);
        }

        test_lint! {
            #[test]
            fn test_schema_file(
                r#"
                #:schema ./json.schemastore.org/tombi.json
                "#,
            ) -> Ok(_);
        }

        test_lint! {
            #[test]
            fn test_file_schema_does_not_exist_url(
                r#"
                #:schema https://does-not-exist.co.jp
                "#,
            ) -> Err([
                tombi_schema_store::Error::SchemaFetchFailed{
                    schema_uri: SchemaUri::from_str("https://does-not-exist.co.jp").unwrap(),
                    reason: "error sending request for url (https://does-not-exist.co.jp/)".to_string(),
                }
            ]);
        }

        test_lint! {
            #[test]
            fn test_file_schema_does_not_exist_file(
                r#"
                #:schema does-not-exist.schema.json
                "#,
            ) -> Err([
                tombi_schema_store::Error::SchemaFileNotFound{
                    schema_path: tombi_test_lib::project_root_path().join("does-not-exist.schema.json"),
                }
            ]);
        }

        test_lint! {
            #[test]
            fn test_file_schema_relative_does_not_exist_file(
                r#"
                #:schema ./does-not-exist.schema.json
                "#,
            ) -> Err([
                tombi_schema_store::Error::SchemaFileNotFound{
                    schema_path: tombi_test_lib::project_root_path().join("does-not-exist.schema.json"),
                }
            ]);
        }

        test_lint! {
            #[test]
            fn test_file_schema_parent_does_not_exist_file(
                r#"
                #:schema ../does-not-exist.schema.json
                "#,
            ) -> Err([
                tombi_schema_store::Error::SchemaFileNotFound{
                    schema_path: tombi_test_lib::project_root_path().join("../does-not-exist.schema.json"),
                }
            ]);
        }

        test_lint! {
            #[test]
            fn test_tombi_document_comment_directive_lint_not_exist_eq_true(
                r#"
                #:tombi lint.not-exist = true
                "#,
            ) -> Err([
                tombi_validator::DiagnosticKind::KeyNotAllowed { key: "not-exist".to_string() }
            ]);
        }

        test_lint! {
            #[test]
            fn test_tombi_document_comment_directive_lint_disable_eq_true(
                r#"
                #:tombi lint.disable = true
                "#,
            ) -> Err([
                tombi_validator::DiagnosticKind::DeprecatedValue(
                    tombi_schema_store::SchemaAccessors::from(
                        vec![
                        tombi_schema_store::SchemaAccessor::Key("lint".to_string()),
                        tombi_schema_store::SchemaAccessor::Key("disable".to_string()),
                    ]),
                    "true".to_string(),
                )
            ]);
        }
    }
}
