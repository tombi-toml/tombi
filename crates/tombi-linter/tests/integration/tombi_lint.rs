use tombi_diagnostic::Level;
use tombi_linter::test_lint;

const MISSING_PATHS: &str = r#"[[schemas]]
path = "missing-schema-for-tombi-lint-test.json"
include = ["*.toml"]

[schema.catalog]
paths = ["missing-catalog-for-tombi-lint-test.json"]
"#;

const INVALID_EXTENDS: &str = r#"extends = [
    "missing-extends-for-tombi-lint-test/tombi.toml",
    "ftp://example.org/tombi.toml",
    "https://example.org/tombi.toml",
    "tombi.toml",
]
"#;

fn tombi_path() -> std::path::PathBuf {
    tombi_test_lib::project_root_path().join("tombi.toml")
}

fn dot_tombi_path() -> std::path::PathBuf {
    tombi_test_lib::project_root_path().join(".tombi.toml")
}

fn user_config_path() -> std::path::PathBuf {
    tombi_test_lib::project_root_path().join(".config/tombi/config.toml")
}

fn unrelated_config_path() -> std::path::PathBuf {
    tombi_test_lib::project_root_path().join("services/example/config.toml")
}

fn span_of(source: &str, value: &str) -> tombi_text::Span {
    let value = format!("\"{value}\"");
    let start = source.find(&value).unwrap();
    tombi_text::Span::new(
        tombi_text::Offset::new(start.try_into().unwrap()),
        tombi_text::Offset::new((start + value.len()).try_into().unwrap()),
    )
}

fn config(
    missing_schema_path: tombi_config::SeverityLevel,
    missing_catalog_path: tombi_config::SeverityLevel,
) -> tombi_config::Config {
    let mut config = tombi_config::Config::default();
    config.extensions = Some(tombi_config::Extensions {
        cargo: None,
        nagi_sql: None,
        pyproject: None,
        tombi: Some(tombi_config::TombiExtensionFeatures::Features(
            tombi_config::TombiExtensionFeatureTree {
                lint: Some(tombi_config::TombiLintFeatures::Features(
                    tombi_config::TombiLintFeatureTree {
                        rules: Some(tombi_config::TombiLintRules {
                            invalid_extends: tombi_config::SeverityLevel::Error,
                            missing_schema_path,
                            missing_catalog_path,
                        }),
                    },
                )),
                lsp: None,
            },
        )),
    });
    config
}

test_lint! {
    #[test]
    fn missing_schema_and_catalog_paths_are_reported(
        MISSING_PATHS,
        SourcePath(tombi_path()),
    ) -> Diagnostics([{
        code: "tombi/missing-schema-path",
        level: Level::WARNING,
        span: span_of(MISSING_PATHS, "missing-schema-for-tombi-lint-test.json")
    }, {
        code: "tombi/missing-catalog-path",
        level: Level::WARNING,
        span: span_of(MISSING_PATHS, "missing-catalog-for-tombi-lint-test.json")
    }])
}

test_lint! {
    #[test]
    fn user_config_tombi_paths_are_linted(
        MISSING_PATHS,
        SourcePath(user_config_path()),
    ) -> Diagnostics([{
        code: "tombi/missing-schema-path",
        level: Level::WARNING,
        span: span_of(MISSING_PATHS, "missing-schema-for-tombi-lint-test.json")
    }, {
        code: "tombi/missing-catalog-path",
        level: Level::WARNING,
        span: span_of(MISSING_PATHS, "missing-catalog-for-tombi-lint-test.json")
    }])
}

test_lint! {
    #[test]
    fn unrelated_config_toml_is_not_linted_as_tombi_config(
        MISSING_PATHS,
        SourcePath(unrelated_config_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn dot_tombi_config_reports_missing_paths(
        MISSING_PATHS,
        SourcePath(dot_tombi_path()),
    ) -> Diagnostics([{
        code: "tombi/missing-schema-path",
        level: Level::WARNING,
        span: span_of(MISSING_PATHS, "missing-schema-for-tombi-lint-test.json")
    }, {
        code: "tombi/missing-catalog-path",
        level: Level::WARNING,
        span: span_of(MISSING_PATHS, "missing-catalog-for-tombi-lint-test.json")
    }])
}

test_lint! {
    #[test]
    fn path_diagnostic_severity_can_be_configured(
        MISSING_PATHS,
        SourcePath(tombi_path()),
        Config(config(tombi_config::SeverityLevel::Warn, tombi_config::SeverityLevel::Off)),
    ) -> Diagnostics([{
        code: "tombi/missing-schema-path",
        level: Level::WARNING,
        span: span_of(MISSING_PATHS, "missing-schema-for-tombi-lint-test.json")
    }])
}

test_lint! {
    #[test]
    fn catalog_path_diagnostic_can_be_downgraded_to_warning(
        MISSING_PATHS,
        SourcePath(tombi_path()),
        Config(config(tombi_config::SeverityLevel::Off, tombi_config::SeverityLevel::Warn)),
    ) -> Diagnostics([{
        code: "tombi/missing-catalog-path",
        level: Level::WARNING,
        span: span_of(MISSING_PATHS, "missing-catalog-for-tombi-lint-test.json")
    }])
}

test_lint! {
    #[test]
    fn tombi_extension_can_be_disabled(
        MISSING_PATHS,
        SourcePath(tombi_path()),
        Config({
            let mut config = tombi_config::Config::default();
            config.extensions = Some(tombi_config::Extensions {
                cargo: None,
                nagi_sql: None,
                pyproject: None,
                tombi: Some(tombi_config::TombiExtensionFeatures::Enabled(
                    tombi_config::EnabledOnly { enabled: false.into() },
                )),
            });
            config
        }),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn existing_and_remote_paths_are_ignored(
        r#"[[schemas]]
path = "Cargo.toml"
include = ["*.toml"]

[[schemas]]
path = "https://example.com/schema.json"
include = ["*.toml"]

[schema.catalog]
paths = ["Cargo.toml", "https://example.com/catalog.json"]
"#,
        SourcePath(tombi_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn invalid_extends_are_reported(
        INVALID_EXTENDS,
        SourcePath(dot_tombi_path()),
    ) -> Diagnostics([{
        code: "tombi/invalid-extends",
        level: Level::ERROR,
        span: span_of(INVALID_EXTENDS, "missing-extends-for-tombi-lint-test/tombi.toml")
    }, {
        code: "tombi/invalid-extends",
        level: Level::ERROR,
        span: span_of(INVALID_EXTENDS, "ftp://example.org/tombi.toml")
    }])
}
