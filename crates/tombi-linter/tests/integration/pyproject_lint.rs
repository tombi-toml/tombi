use tombi_diagnostic::Level;
use tombi_linter::test_lint;

const INVALID_DEPENDENCY: &str = r#"[project]
name = "example"
version = "1.0"
dependencies = ["not a valid requirement !!!"]
"#;

fn pyproject_path() -> std::path::PathBuf {
    tombi_test_lib::project_root_path().join("pyproject.toml")
}

fn invalid_dependency_span() -> tombi_text::Span {
    let start = INVALID_DEPENDENCY
        .find("\"not a valid requirement !!!\"")
        .unwrap();
    tombi_text::Span::new(
        tombi_text::Offset::new(start.try_into().unwrap()),
        tombi_text::Offset::new(
            (start + "\"not a valid requirement !!!\"".len())
                .try_into()
                .unwrap(),
        ),
    )
}

test_lint! {
    #[test]
    fn valid_pyproject_metadata_is_accepted(
        r#"[project]
name = "example"
version = "1.0"
dependencies = ["requests>=2"]
"#,
        SourcePath(pyproject_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn invalid_dependency_specifier_uses_default_error_level(
        INVALID_DEPENDENCY,
        SourcePath(pyproject_path()),
    ) -> Diagnostics([{
        code: "pyproject/invalid-dependency-specifier",
        level: Level::ERROR,
        span: invalid_dependency_span()
    }])
}

test_lint! {
    #[test]
    fn invalid_dependency_specifier_can_be_disabled(
        INVALID_DEPENDENCY,
        SourcePath(pyproject_path()),
        Config({
            let mut config = tombi_config::Config::default();
            config.extensions = Some(tombi_config::Extensions {
                cargo: None,
                nagi_sql: None,
                pyproject: Some(tombi_config::PyprojectExtensionFeatures::Features(
                    tombi_config::PyprojectExtensionFeatureTree {
                        lint: Some(tombi_config::PyprojectLintFeatures::Features(
                            tombi_config::PyprojectLintFeatureTree {
                                rules: Some(tombi_config::PyprojectLintRules {
                                    invalid_dependency_specifier: tombi_config::SeverityLevel::Off,
                                    ..Default::default()
                                }),
                            },
                        )),
                        lsp: None,
                    },
                )),
                tombi: None,
            });
            config
        }),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn invalid_dependency_specifier_can_be_downgraded_to_warning(
        INVALID_DEPENDENCY,
        SourcePath(pyproject_path()),
        Config({
            let mut config = tombi_config::Config::default();
            config.extensions = Some(tombi_config::Extensions {
                cargo: None,
                nagi_sql: None,
                pyproject: Some(tombi_config::PyprojectExtensionFeatures::Features(
                    tombi_config::PyprojectExtensionFeatureTree {
                        lint: Some(tombi_config::PyprojectLintFeatures::Features(
                            tombi_config::PyprojectLintFeatureTree {
                                rules: Some(tombi_config::PyprojectLintRules {
                                    invalid_dependency_specifier: tombi_config::SeverityLevel::Warn,
                                    ..Default::default()
                                }),
                            },
                        )),
                        lsp: None,
                    },
                )),
                tombi: None,
            });
            config
        }),
    ) -> Diagnostics([{
        code: "pyproject/invalid-dependency-specifier",
        level: Level::WARNING,
        span: invalid_dependency_span()
    }])
}

test_lint! {
    #[test]
    fn pyproject_extension_can_be_disabled(
        INVALID_DEPENDENCY,
        SourcePath(pyproject_path()),
        Config({
            let mut config = tombi_config::Config::default();
            config.extensions = Some(tombi_config::Extensions {
                cargo: None,
                nagi_sql: None,
                pyproject: Some(tombi_config::PyprojectExtensionFeatures::Enabled(
                    tombi_config::EnabledOnly { enabled: false.into() },
                )),
                tombi: None,
            });
            config
        }),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn invalid_dependency_diagnostic_covers_the_requirement(
        INVALID_DEPENDENCY,
        SourcePath(pyproject_path()),
    ) -> Diagnostics([{
        code: "pyproject/invalid-dependency-specifier",
        level: Level::ERROR,
        span: invalid_dependency_span()
    }])
}
