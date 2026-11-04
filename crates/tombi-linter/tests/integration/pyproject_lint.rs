use tombi_diagnostic::Level;
use tombi_linter::test_lint;

const INVALID_DEPENDENCY: &str = r#"[project]
name = "example"
version = "1.0"
dependencies = ["not a valid requirement !!!"]
"#;

const UNDEFINED_INCLUDE_GROUP: &str = r#"[dependency-groups]
dev = [{ include-group = "missing" }]
"#;

fn pyproject_path() -> std::path::PathBuf {
    tombi_test_lib::project_root_path().join("pyproject.toml")
}

fn span_of_member(source: &str, value: &str) -> tombi_text::Span {
    let quoted = format!("\"{value}\"");
    let start = source.find(&quoted).unwrap();
    tombi_text::Span::new(
        tombi_text::Offset::new(start.try_into().unwrap()),
        tombi_text::Offset::new((start + quoted.len()).try_into().unwrap()),
    )
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

fn undefined_include_group_span() -> tombi_text::Span {
    let start = UNDEFINED_INCLUDE_GROUP.find("\"missing\"").unwrap();
    tombi_text::Span::new(
        tombi_text::Offset::new(start.try_into().unwrap()),
        tombi_text::Offset::new((start + "\"missing\"".len()).try_into().unwrap()),
    )
}

fn long_acyclic_dependency_group_chain() -> String {
    use std::fmt::Write;

    let mut source = String::from("[dependency-groups]\n");
    for index in 0..10_000 {
        if index == 9_999 {
            writeln!(source, "group-{index} = []").unwrap();
        } else {
            writeln!(
                source,
                "group-{index} = [{{ include-group = \"group-{}\" }}]",
                index + 1
            )
            .unwrap();
        }
    }
    source
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
    fn include_group_can_reference_a_defined_group(
        r#"[dependency-groups]
dev = [{ include-group = "Lint_Tools" }]
"lint-tools" = ["ruff"]
"#,
        SourcePath(pyproject_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn dependency_groups_with_equivalent_normalized_names_are_errors(
        r#"[dependency-groups]
qa_tools = ["ruff"]
"QA.Tools" = ["pytest"]
"#,
        SourcePath(pyproject_path()),
    ) -> Diagnostics([{
        code: "pyproject/duplicate-dependency-group",
        level: Level::ERROR,
        span: {
            let start = r#"[dependency-groups]
qa_tools = ["ruff"]
"QA.Tools" = ["pytest"]
"#.find("\"QA.Tools\"").unwrap();
            tombi_text::Span::new(
                tombi_text::Offset::new(start.try_into().unwrap()),
                tombi_text::Offset::new((start + "\"QA.Tools\"".len()).try_into().unwrap()),
            )
        }
    }])
}

test_lint! {
    #[test]
    fn extra_names_with_equivalent_normalized_names_are_errors(
        r#"[project.optional-dependencies]
qa_tools = ["ruff"]
"QA.Tools" = ["pytest"]
"#,
        SourcePath(pyproject_path()),
    ) -> Diagnostics([{
        code: "pyproject/duplicate-extra-name",
        level: Level::ERROR,
        span: {
            let start = r#"[project.optional-dependencies]
qa_tools = ["ruff"]
"QA.Tools" = ["pytest"]
"#.find("\"QA.Tools\"").unwrap();
            tombi_text::Span::new(
                tombi_text::Offset::new(start.try_into().unwrap()),
                tombi_text::Offset::new((start + "\"QA.Tools\"".len()).try_into().unwrap()),
            )
        }
    }])
}

test_lint! {
    #[test]
    fn cyclic_dependency_group_includes_are_errors(
        r#"[dependency-groups]
a = [{ include-group = "b" }]
b = [{ include-group = "a" }]
"#,
        SourcePath(pyproject_path()),
    ) -> Diagnostics([{
        code: "pyproject/cyclic-include-group",
        level: Level::ERROR,
        span: {
            let start = r#"[dependency-groups]
a = [{ include-group = "b" }]
b = [{ include-group = "a" }]
"#.find("\"a\"").unwrap();
            tombi_text::Span::new(
                tombi_text::Offset::new(start.try_into().unwrap()),
                tombi_text::Offset::new((start + "\"a\"".len()).try_into().unwrap()),
            )
        }
    }])
}

test_lint! {
    #[test]
    fn long_acyclic_dependency_group_chain_does_not_overflow_the_stack(
        &long_acyclic_dependency_group_chain(),
        SourcePath(pyproject_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn undefined_include_group_uses_default_error_level(
        UNDEFINED_INCLUDE_GROUP,
        SourcePath(pyproject_path()),
    ) -> Diagnostics([{
        code: "pyproject/undefined-include-group",
        level: Level::ERROR,
        span: undefined_include_group_span()
    }])
}

test_lint! {
    #[test]
    fn undefined_include_group_can_be_disabled(
        UNDEFINED_INCLUDE_GROUP,
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
                                    undefined_include_group: tombi_config::SeverityLevel::Off,
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

test_lint! {
    #[test]
    fn missing_workspace_member_is_reported(
        r#"[tool.uv.workspace]
members = ["python/tombi", "missing-member-dir"]
"#,
        SourcePath(pyproject_path()),
    ) -> Diagnostics([{
        code: "pyproject/missing-workspace-member",
        level: Level::ERROR,
        span: span_of_member(r#"[tool.uv.workspace]
members = ["python/tombi", "missing-member-dir"]
"#, "missing-member-dir")
    }])
}

test_lint! {
    #[test]
    fn glob_workspace_members_are_not_checked(
        r#"[tool.uv.workspace]
members = ["missing-glob-*"]
"#,
        SourcePath(pyproject_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn existing_workspace_member_is_accepted(
        r#"[tool.uv.workspace]
members = ["python/tombi"]
"#,
        SourcePath(pyproject_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn missing_workspace_member_can_be_disabled(
        r#"[tool.uv.workspace]
members = ["missing-member-dir"]
"#,
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
                                    missing_workspace_member: tombi_config::SeverityLevel::Off,
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
