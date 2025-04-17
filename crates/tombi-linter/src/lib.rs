mod error;
mod lint;
mod linter;
mod rule;
mod warning;

pub use error::{Error, ErrorKind};
use lint::Lint;
pub use linter::Linter;
use rule::Rule;
pub use tombi_config::LintOptions;
use tombi_diagnostic::Diagnostic;
pub use warning::{Warning, WarningKind};

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

            test_lib::init_tracing();

            // Initialize schema store
            let schema_store = schema_store::SchemaStore::new();

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
            let source_path = test_lib::project_root().join("test.toml");
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

            test_lib::init_tracing();

            // Initialize schema store
            let schema_store = schema_store::SchemaStore::new();

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
            let source_path = test_lib::project_root().join("test.toml");
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
        use test_lib::cargo_schema_path;

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

    mod non_schema {
        use schema_store::SchemaUrl;

        use super::*;

        test_lint! {
            #[test]
            fn test_warning_empty(
                r#"
                "" = 1
                "#,
            ) -> Err([
                crate::WarningKind::KeyEmpty
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
                #:schema ./tombi.schema.json
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
                schema_store::Error::SchemaFetchFailed{
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
                schema_store::Error::SchemaFileNotFound{
                    schema_path: test_lib::project_root().join("does-not-exist.schema.json"),
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
                schema_store::Error::SchemaFileNotFound{
                    schema_path: test_lib::project_root().join("does-not-exist.schema.json"),
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
                schema_store::Error::SchemaFileNotFound{
                    schema_path: test_lib::project_root().join("../does-not-exist.schema.json"),
                }
            ]);
        }
    }
}
