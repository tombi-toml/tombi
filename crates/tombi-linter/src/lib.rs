mod error;
mod lint;
mod linter;
mod rule;
mod severity;

pub use error::{Error, ErrorKind};
use lint::Lint;
pub use linter::Linter;
use rule::Rule;
pub use severity::{Severity, SeverityKind};
pub use tombi_config::LintOptions;
use tombi_diagnostic::Diagnostic;

#[cfg(test)]
#[macro_export]
macro_rules! test_lint {
    (
        #[test]
        fn $name:ident(
            $source:expr,
        ) -> Ok(_);
    ) => {
        test_lint! {
            #[test]
            fn _$name($source, Option::<std::path::PathBuf>::None) -> Ok(_);
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
            fn _$name($source, Some($schema_path)) -> Ok(_);
        }
    };

    (
        #[test]
        fn _$name:ident(
            $source:expr,
            $schema_path:expr$(,)?
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
                    .load_schemas(
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
                TomlVersion::default(),
                &options,
                Some(itertools::Either::Right(source_path.as_path())),
                &schema_store,
            );

            let result = linter.lint($source).await;

            match result {
                Ok(_) => {}
                Err(errors) => {
                    panic!("Expected success but got errors: {:?}", errors);
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
            fn _$name($source, Some($schema_path)) -> Err([$($error.to_string()),*]);
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
            fn _$name($source, Option::<std::path::PathBuf>::None) -> Err([$($error.to_string()),*]);
        }
    };

    (
        #[test]
        fn _$name:ident(
            $source:expr,
            $schema_path:expr$(,)?
        ) -> Err([$( $error:expr ),*$(,)?]);
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
                    .load_schemas(
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
                TomlVersion::default(),
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
                            .collect::<Vec<_>>(),
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

    mod cargo {
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
            ) -> Err([tombi_validator::WarningKind::StrictAdditionalProperties {
                accessors: tombi_schema_store::SchemaAccessors::new(vec![
                    tombi_schema_store::SchemaAccessor::Key("workspace".to_string()),
                ]),
                schema_url: tombi_schema_store::SchemaUrl::from_file_path(cargo_schema_path()).unwrap(),
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
            ) -> Err([tombi_validator::ErrorKind::KeyNotAllowed { key: "aaa".to_string() }]);
        }
    }

    mod tombi_schema {
        use super::*;
        use tombi_test_lib::tombi_schema_path;

        test_lint! {
            #[test]
            fn test_tombi_schema(
                include_str!("../../../tombi.toml"),
                tombi_schema_path(),
            ) -> Ok(_);
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
                tombi_validator::WarningKind::Deprecated(tombi_schema_store::SchemaAccessors::new(vec![
                    tombi_schema_store::SchemaAccessor::Key("schemas".to_string()),
                    tombi_schema_store::SchemaAccessor::Index,
                    tombi_schema_store::SchemaAccessor::Key("root-keys".to_string()),
                ])),
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
                tombi_validator::ErrorKind::Const {
                    expected: "\"off\"".to_string(),
                    actual: "\"undefined\"".to_string()
                },
                tombi_validator::ErrorKind::Const {
                    expected: "\"warn\"".to_string(),
                    actual: "\"undefined\"".to_string()
                },
                tombi_validator::ErrorKind::Const {
                    expected: "\"error\"".to_string(),
                    actual: "\"undefined\"".to_string()
                }
            ]);
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
        use tombi_schema_store::SchemaUrl;

        use super::*;

        test_lint! {
            #[test]
            fn test_warning_empty(
                r#"
                "" = 1
                "#,
            ) -> Err([
                crate::SeverityKind::KeyEmpty
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
                crate::SeverityKind::DottedKeysOutOfOrder,
                crate::SeverityKind::DottedKeysOutOfOrder,
                crate::SeverityKind::DottedKeysOutOfOrder,
                crate::SeverityKind::DottedKeysOutOfOrder,
                crate::SeverityKind::DottedKeysOutOfOrder,
                crate::SeverityKind::DottedKeysOutOfOrder
            ]);
        }

        test_lint! {
            #[test]
            fn test_schema_url(
                r#"
                #:schema https://json.schemastore.org/tombi.json
                "#,
            ) -> Ok(_);
        }

        test_lint! {
            #[test]
            fn test_schema_file(
                r#"
                #:schema ./schemas/tombi.schema.json
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
                    schema_url: SchemaUrl::parse("https://does-not-exist.co.jp").unwrap(),
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
    }
}
