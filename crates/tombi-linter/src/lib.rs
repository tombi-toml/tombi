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

#[macro_export]
macro_rules! test_lint {
    {
        #[test]
        fn $name:ident($source:expr $(, $arg:expr )* $(,)? ) -> Ok(_)
    } => {
        #[tokio::test]
        async fn $name() {
            use tombi_config::TomlVersion;

            tombi_test_lib::init_log();

            /// Test-time configuration overridden via `test_lint!` arguments.
            #[allow(unused)]
            #[derive(Default)]
            pub struct TestArgs {
                pub toml_version: TomlVersion,
                pub options: $crate::LintOptions,
                pub config: Option<tombi_config::Config>,
                pub schema_path: Option<std::path::PathBuf>,
                pub source_path: Option<std::path::PathBuf>,
            }

            #[allow(unused)]
            pub trait ApplyTestArg {
                fn apply(self, args: &mut TestArgs);
            }

            impl ApplyTestArg for TomlVersion {
                fn apply(self, args: &mut TestArgs) {
                    args.toml_version = self;
                }
            }

            impl ApplyTestArg for $crate::LintOptions {
                fn apply(self, args: &mut TestArgs) {
                    args.options = self;
                }
            }

            /// Set full config for the test case.
            #[allow(unused)]
            pub struct Config(pub tombi_config::Config);

            impl ApplyTestArg for Config {
                fn apply(self, args: &mut TestArgs) {
                    args.config = Some(self.0);
                }
            }

            /// Set schema path for the test case.
            #[allow(unused)]
            pub struct SchemaPath(pub std::path::PathBuf);

            impl ApplyTestArg for SchemaPath {
                fn apply(self, args: &mut TestArgs) {
                    args.schema_path = Some(self.0);
                }
            }

            /// Set source path for the test case.
            #[allow(unused)]
            pub struct SourcePath(pub std::path::PathBuf);

            impl ApplyTestArg for SourcePath {
                fn apply(self, args: &mut TestArgs) {
                    args.source_path = Some(self.0);
                }
            }

            #[allow(unused_mut)]
            let mut args = TestArgs::default();
            $(
                ApplyTestArg::apply($arg, &mut args);
            )*

            // Initialize schema store
            let schema_store = tombi_schema_store::SchemaStore::new();

            if let Some(config) = &args.config {
                schema_store.load_config(config, None).await.unwrap();
            } else if let Some(schema_path) = &args.schema_path {
                let schema_uri = tombi_schema_store::SchemaUri::from_file_path(schema_path.as_path())
                    .expect("failed to convert test schema path to schema uri");
                schema_store
                    .associate_schema(
                        schema_uri,
                        vec!["*.toml".to_string()],
                        &tombi_schema_store::AssociateSchemaOptions::default(),
                    )
                    .await;
            }

            // Initialize linter
            let source_path = args.source_path.unwrap_or_else(|| tombi_test_lib::project_root_path().join("test.toml"));
            let linter = $crate::Linter::new(
                args.toml_version,
                &args.options,
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

    {
        #[test]
        fn $name:ident($source:expr $(, $arg:expr )* $(,)? ) -> Err([$( $error:expr ),* $(,)?])
    } => {
        #[tokio::test]
        async fn $name() {
            use tombi_config::TomlVersion;
            use itertools::Itertools;

            tombi_test_lib::init_log();

            /// Test-time configuration overridden via `test_lint!` arguments.
            #[allow(unused)]
            #[derive(Default)]
            pub struct TestArgs {
                pub toml_version: TomlVersion,
                pub options: $crate::LintOptions,
                pub config: Option<tombi_config::Config>,
                pub schema_path: Option<std::path::PathBuf>,
                pub source_path: Option<std::path::PathBuf>,
            }

            #[allow(unused)]
            pub trait ApplyTestArg {
                fn apply(self, args: &mut TestArgs);
            }

            impl ApplyTestArg for TomlVersion {
                fn apply(self, args: &mut TestArgs) {
                    args.toml_version = self;
                }
            }

            impl ApplyTestArg for $crate::LintOptions {
                fn apply(self, args: &mut TestArgs) {
                    args.options = self;
                }
            }

            /// Set full config for the test case.
            #[allow(unused)]
            pub struct Config(pub tombi_config::Config);

            impl ApplyTestArg for Config {
                fn apply(self, args: &mut TestArgs) {
                    args.config = Some(self.0);
                }
            }

            /// Set schema path for the test case.
            #[allow(unused)]
            pub struct SchemaPath(pub std::path::PathBuf);

            impl ApplyTestArg for SchemaPath {
                fn apply(self, args: &mut TestArgs) {
                    args.schema_path = Some(self.0);
                }
            }

            /// Set source path for the test case.
            #[allow(unused)]
            pub struct SourcePath(pub std::path::PathBuf);

            impl ApplyTestArg for SourcePath {
                fn apply(self, args: &mut TestArgs) {
                    args.source_path = Some(self.0);
                }
            }

            #[allow(unused_mut)]
            let mut args = TestArgs::default();
            $(
                ApplyTestArg::apply($arg, &mut args);
            )*

            // Initialize schema store
            let schema_store = tombi_schema_store::SchemaStore::new();

            if let Some(config) = &args.config {
                schema_store.load_config(config, None).await.unwrap();
            } else if let Some(schema_path) = args.schema_path {
                let schema_uri = tombi_schema_store::SchemaUri::from_file_path(schema_path)
                    .expect("failed to convert test schema path to schema uri");
                schema_store
                    .associate_schema(
                        schema_uri,
                        vec!["*.toml".to_string()],
                        &tombi_schema_store::AssociateSchemaOptions::default(),
                    )
                    .await;
            }

            // Initialize linter
            let source_path = args.source_path.unwrap_or_else(|| tombi_test_lib::project_root_path().join("test.toml"));
            let linter = $crate::Linter::new(
                args.toml_version,
                &args.options,
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

    {
        #[test]
        fn $name:ident($source:expr $(, $arg:expr )* $(,)? ) -> Diagnostics([$({ code: $code:expr, level: $level:expr $(,)? }),* $(,)?])
    } => {
        #[tokio::test]
        async fn $name() {
            use tombi_config::TomlVersion;
            use itertools::Itertools;

            tombi_test_lib::init_log();

            /// Test-time configuration overridden via `test_lint!` arguments.
            #[allow(unused)]
            #[derive(Default)]
            pub struct TestArgs {
                pub toml_version: TomlVersion,
                pub options: $crate::LintOptions,
                pub config: Option<tombi_config::Config>,
                pub schema_path: Option<std::path::PathBuf>,
                pub source_path: Option<std::path::PathBuf>,
            }

            #[allow(unused)]
            pub trait ApplyTestArg {
                fn apply(self, args: &mut TestArgs);
            }

            impl ApplyTestArg for TomlVersion {
                fn apply(self, args: &mut TestArgs) {
                    args.toml_version = self;
                }
            }

            impl ApplyTestArg for $crate::LintOptions {
                fn apply(self, args: &mut TestArgs) {
                    args.options = self;
                }
            }

            /// Set full config for the test case.
            #[allow(unused)]
            pub struct Config(pub tombi_config::Config);

            impl ApplyTestArg for Config {
                fn apply(self, args: &mut TestArgs) {
                    args.config = Some(self.0);
                }
            }

            /// Set schema path for the test case.
            #[allow(unused)]
            pub struct SchemaPath(pub std::path::PathBuf);

            impl ApplyTestArg for SchemaPath {
                fn apply(self, args: &mut TestArgs) {
                    args.schema_path = Some(self.0);
                }
            }

            /// Set source path for the test case.
            #[allow(unused)]
            pub struct SourcePath(pub std::path::PathBuf);

            impl ApplyTestArg for SourcePath {
                fn apply(self, args: &mut TestArgs) {
                    args.source_path = Some(self.0);
                }
            }

            #[allow(unused_mut)]
            let mut args = TestArgs::default();
            $(
                ApplyTestArg::apply($arg, &mut args);
            )*

            // Initialize schema store
            let schema_store = tombi_schema_store::SchemaStore::new();

            if let Some(config) = &args.config {
                schema_store.load_config(config, None).await.unwrap();
            } else if let Some(schema_path) = args.schema_path {
                let schema_uri = tombi_schema_store::SchemaUri::from_file_path(schema_path)
                    .expect("failed to convert test schema path to schema uri");
                schema_store
                    .associate_schema(
                        schema_uri,
                        vec!["*.toml".to_string()],
                        &tombi_schema_store::AssociateSchemaOptions::default(),
                    )
                    .await;
            }

            // Initialize linter
            let source_path = args.source_path.unwrap_or_else(|| tombi_test_lib::project_root_path().join("test.toml"));
            let linter = $crate::Linter::new(
                args.toml_version,
                &args.options,
                Some(itertools::Either::Right(source_path.as_path())),
                &schema_store,
            );

            let result = linter.lint($source).await;
            match result {
                Ok(_) => {
                    panic!("Expected diagnostics but got success");
                }
                Err(errors) => {
                    pretty_assertions::assert_eq!(
                        errors
                            .into_iter()
                            .map(|error| (error.code().to_string(), error.level()))
                            .collect_vec(),
                        [$(($code.to_string(), $level)),*].into_iter().collect::<Vec<_>>()
                    );
                }
            }
        }
    };
}
