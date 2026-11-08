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
    "https://tombi-lint-test.invalid/tombi.toml",
    "tombi.toml",
    "base.toml",
    "https://tombi-lint-test.invalid/base.toml?file=tombi.toml",
]
"#;

const EMBEDDED_PATHS: &str = r#"[[schemas]]
path = "tombi://www.schemastore.org/cargo.json"
include = ["Cargo.toml"]

[[schemas]]
path = "tombi://www.schemtore.org/cargo.json"
include = ["Cargo.toml"]

[schema.catalog]
paths = ["tombi://www.schemastore.org/api/json/catalog.json", "tombi://www.schemastore.org/missing-catalog.json"]
"#;

/// Port of a local server that answers every request with 404.
fn not_found_server_port() -> u16 {
    static PORT: std::sync::LazyLock<u16> = std::sync::LazyLock::new(|| {
        use std::io::{Read, Write};

        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        std::thread::spawn(move || {
            for mut stream in listener.incoming().flatten() {
                let mut request = [0; 1024];
                let _ = stream.read(&mut request);
                let _ = stream.write_all(
                    b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                );
            }
        });
        port
    });
    *PORT
}

fn missing_remote_urls() -> (String, String, String) {
    let base = format!("http://127.0.0.1:{}", not_found_server_port());
    (
        format!("{base}/tombi.toml"),
        format!("{base}/schema.json"),
        format!("{base}/catalog.json"),
    )
}

fn missing_remote_source(with_extends: bool) -> String {
    let (extends, schema, catalog) = missing_remote_urls();
    let extends = if with_extends {
        format!("extends = [\"{extends}\"]\n\n")
    } else {
        String::new()
    };
    format!(
        r#"{extends}[[schemas]]
path = "{schema}"
include = ["*.toml"]

[schema.catalog]
paths = ["{catalog}"]
"#
    )
}

fn tombi_path() -> std::path::PathBuf {
    tombi_test_lib::project_root_path().join("tombi.toml")
}

fn dot_config_tombi_path() -> std::path::PathBuf {
    tombi_test_lib::project_root_path().join(".config/tombi.toml")
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
                            missing_schema_path,
                            missing_catalog_path,
                            missing_extends_path: tombi_config::SeverityLevel::Error,
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
        level: Level::ERROR,
        span: span_of(MISSING_PATHS, "missing-schema-for-tombi-lint-test.json")
    }, {
        code: "tombi/missing-catalog-path",
        level: Level::ERROR,
        span: span_of(MISSING_PATHS, "missing-catalog-for-tombi-lint-test.json")
    }])
}

test_lint! {
    #[test]
    fn missing_embedded_schema_and_catalog_are_reported(
        EMBEDDED_PATHS,
        SourcePath(tombi_path()),
    ) -> Diagnostics([{
        code: "tombi/missing-schema-path",
        level: Level::ERROR,
        span: span_of(EMBEDDED_PATHS, "tombi://www.schemtore.org/cargo.json")
    }, {
        code: "tombi/missing-catalog-path",
        level: Level::ERROR,
        span: span_of(EMBEDDED_PATHS, "tombi://www.schemastore.org/missing-catalog.json")
    }])
}

test_lint! {
    #[test]
    fn missing_remote_references_are_warnings(
        &missing_remote_source(true),
        SourcePath(dot_tombi_path()),
        TrustedHosts(vec!["127.0.0.1".to_string()]),
    ) -> Diagnostics([{
        code: "tombi/missing-extends-path",
        level: Level::WARNING,
        span: span_of(&missing_remote_source(true), &missing_remote_urls().0)
    }, {
        code: "tombi/missing-schema-path",
        level: Level::WARNING,
        span: span_of(&missing_remote_source(true), &missing_remote_urls().1)
    }, {
        code: "tombi/missing-catalog-path",
        level: Level::WARNING,
        span: span_of(&missing_remote_source(true), &missing_remote_urls().2)
    }])
}

test_lint! {
    #[test]
    fn missing_remote_references_can_be_turned_off(
        &missing_remote_source(false),
        SourcePath(tombi_path()),
        TrustedHosts(vec!["127.0.0.1".to_string()]),
        Config(config(tombi_config::SeverityLevel::Off, tombi_config::SeverityLevel::Off)),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn user_config_tombi_paths_are_linted(
        MISSING_PATHS,
        SourcePath(user_config_path()),
    ) -> Diagnostics([{
        code: "tombi/missing-schema-path",
        level: Level::ERROR,
        span: span_of(MISSING_PATHS, "missing-schema-for-tombi-lint-test.json")
    }, {
        code: "tombi/missing-catalog-path",
        level: Level::ERROR,
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
        level: Level::ERROR,
        span: span_of(MISSING_PATHS, "missing-schema-for-tombi-lint-test.json")
    }, {
        code: "tombi/missing-catalog-path",
        level: Level::ERROR,
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
path = "https://tombi-lint-test.invalid/schema.json"
include = ["*.toml"]

[schema.catalog]
paths = ["Cargo.toml", "https://tombi-lint-test.invalid/catalog.json"]
"#,
        SourcePath(tombi_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn extends_of_dot_config_tombi_toml_is_relative_to_the_project_root(
        r#"extends = ["tombi.toml"]
"#,
        SourcePath(dot_config_tombi_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn invalid_extends_are_reported(
        INVALID_EXTENDS,
        SourcePath(dot_tombi_path()),
    ) -> Diagnostics([{
        code: "tombi/missing-extends-path",
        level: Level::ERROR,
        span: span_of(INVALID_EXTENDS, "missing-extends-for-tombi-lint-test/tombi.toml")
    }, {
        code: "config/invalid-extends",
        level: Level::ERROR,
        span: span_of(INVALID_EXTENDS, "ftp://example.org/tombi.toml")
    }, {
        code: "config/invalid-extends",
        level: Level::ERROR,
        span: span_of(INVALID_EXTENDS, "https://tombi-lint-test.invalid/tombi.toml")
    }, {
        code: "config/invalid-extends",
        level: Level::ERROR,
        span: span_of(INVALID_EXTENDS, "base.toml")
    }, {
        code: "config/invalid-extends",
        level: Level::ERROR,
        span: span_of(INVALID_EXTENDS, "https://tombi-lint-test.invalid/base.toml?file=tombi.toml")
    }])
}

test_lint! {
    #[test]
    fn self_extends_is_reported(
        r#"extends = ["tombi.toml"]
"#,
        SourcePath(tombi_path()),
    ) -> Diagnostics([{
        code: "config/invalid-extends",
        level: Level::ERROR,
        span: span_of(r#"extends = ["tombi.toml"]
"#, "tombi.toml")
    }])
}

test_lint! {
    #[test]
    fn extends_must_be_an_array_of_strings(
        r#"extends = "tombi.toml"
"#,
        SourcePath(tombi_path()),
    ) -> Diagnostics([{
        code: "config/invalid-extends",
        level: Level::ERROR,
        span: span_of(r#"extends = "tombi.toml"
"#, "tombi.toml")
    }])
}

test_lint! {
    #[test]
    fn extends_entries_must_be_strings(
        r#"extends = [1]
"#,
        SourcePath(tombi_path()),
    ) -> Diagnostics([{
        code: "config/invalid-extends",
        level: Level::ERROR,
        span: tombi_text::Span::new(tombi_text::Offset::new(11), tombi_text::Offset::new(12))
    }])
}

test_lint! {
    #[test]
    fn self_extends_is_reported_even_if_the_file_does_not_exist(
        r#"extends = ["tombi.toml"]
"#,
        SourcePath(tombi_test_lib::project_root_path().join("self-extends-for-tombi-lint-test/tombi.toml")),
        Config({
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
                                    missing_extends_path: tombi_config::SeverityLevel::Off,
                                    ..Default::default()
                                }),
                            },
                        )),
                        lsp: None,
                    },
                )),
            });
            config
        }),
    ) -> Diagnostics([{
        code: "config/invalid-extends",
        level: Level::ERROR,
        span: span_of(r#"extends = ["tombi.toml"]
"#, "tombi.toml")
    }])
}

test_lint! {
    #[test]
    fn missing_extends_path_can_be_turned_off(
        r#"extends = ["missing-extends-for-tombi-lint-test/tombi.toml"]
"#,
        SourcePath(tombi_path()),
        Config({
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
                                    missing_extends_path: tombi_config::SeverityLevel::Off,
                                    ..Default::default()
                                }),
                            },
                        )),
                        lsp: None,
                    },
                )),
            });
            config
        }),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn invalid_extends_are_reported_even_if_tombi_extension_is_disabled(
        INVALID_EXTENDS,
        SourcePath(dot_tombi_path()),
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
    ) -> Diagnostics([{
        code: "config/invalid-extends",
        level: Level::ERROR,
        span: span_of(INVALID_EXTENDS, "ftp://example.org/tombi.toml")
    }, {
        code: "config/invalid-extends",
        level: Level::ERROR,
        span: span_of(INVALID_EXTENDS, "https://tombi-lint-test.invalid/tombi.toml")
    }, {
        code: "config/invalid-extends",
        level: Level::ERROR,
        span: span_of(INVALID_EXTENDS, "base.toml")
    }, {
        code: "config/invalid-extends",
        level: Level::ERROR,
        span: span_of(INVALID_EXTENDS, "https://tombi-lint-test.invalid/base.toml?file=tombi.toml")
    }])
}
