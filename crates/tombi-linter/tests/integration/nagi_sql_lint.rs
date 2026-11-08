use tombi_diagnostic::Level;
use tombi_linter::test_lint;

const MISSING_MEMBER: &str = r#"[workspace]
members = ["missing"]
"#;

const EXISTING_MEMBERS: &str = r#"[workspace]
members = ["members/plain", "members/dot", "members/config"]
"#;

const GLOB_MEMBER: &str = r#"[workspace]
members = ["members/*"]
"#;

fn workspace_path() -> std::path::PathBuf {
    tombi_test_lib::project_root_path()
        .join("crates/tombi-linter/tests/fixtures/nagi-sql/workspace/nagi.toml")
}

fn dot_config_workspace_path() -> std::path::PathBuf {
    tombi_test_lib::project_root_path()
        .join("crates/tombi-linter/tests/fixtures/nagi-sql/dot-config/.config/nagi.toml")
}

fn glob_meta_workspace_path() -> std::path::PathBuf {
    tombi_test_lib::project_root_path()
        .join("crates/tombi-linter/tests/fixtures/nagi-sql/project[1]/nagi.toml")
}

fn span_of(source: &str, value: &str) -> tombi_text::Span {
    let value = format!("\"{value}\"");
    let start = source.find(&value).unwrap();
    tombi_text::Span::new(
        tombi_text::Offset::new(start.try_into().unwrap()),
        tombi_text::Offset::new((start + value.len()).try_into().unwrap()),
    )
}

fn lint_config(rule: tombi_config::SeverityLevel) -> tombi_config::Config {
    let mut config = tombi_config::Config::default();
    config.extensions = Some(tombi_config::Extensions {
        cargo: None,
        nagi_sql: Some(tombi_config::NagiSqlExtensionFeatures::Features(
            tombi_config::NagiSqlExtensionFeatureTree {
                lint: Some(tombi_config::NagiSqlLintFeatures::Features(
                    tombi_config::NagiSqlLintFeatureTree {
                        rules: Some(tombi_config::NagiSqlLintRules {
                            missing_member_config: rule,
                        }),
                    },
                )),
                lsp: None,
            },
        )),
        pyproject: None,
        tombi: None,
    });
    config
}

test_lint! {
    #[test]
    fn missing_workspace_member_config_is_reported(
        MISSING_MEMBER,
        SourcePath(workspace_path()),
    ) -> Diagnostics([{
        code: "nagi-sql/missing-member-config",
        level: Level::ERROR,
        span: span_of(MISSING_MEMBER, "missing")
    }])
}

test_lint! {
    #[test]
    fn supported_member_config_names_are_accepted(
        EXISTING_MEMBERS,
        SourcePath(workspace_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn dot_config_nagi_file_uses_project_root_for_members(
        MISSING_MEMBER,
        SourcePath(dot_config_workspace_path()),
    ) -> Diagnostics([{
        code: "nagi-sql/missing-member-config",
        level: Level::ERROR,
        span: span_of(MISSING_MEMBER, "missing")
    }])
}

test_lint! {
    #[test]
    fn workspace_root_glob_metacharacters_are_escaped(
        GLOB_MEMBER,
        SourcePath(glob_meta_workspace_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn missing_member_severity_can_be_error(
        MISSING_MEMBER,
        SourcePath(workspace_path()),
        Config(lint_config(tombi_config::SeverityLevel::Error)),
    ) -> Diagnostics([{
        code: "nagi-sql/missing-member-config",
        level: Level::ERROR,
        span: span_of(MISSING_MEMBER, "missing")
    }])
}

test_lint! {
    #[test]
    fn missing_member_rule_can_be_disabled(
        MISSING_MEMBER,
        SourcePath(workspace_path()),
        Config(lint_config(tombi_config::SeverityLevel::Off)),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn nagi_sql_extension_can_be_disabled(
        MISSING_MEMBER,
        SourcePath(workspace_path()),
        Config({
            let mut config = tombi_config::Config::default();
            config.extensions = Some(tombi_config::Extensions {
                cargo: None,
                nagi_sql: Some(tombi_config::NagiSqlExtensionFeatures::Enabled(
                    tombi_config::EnabledOnly { enabled: false.into() },
                )),
                pyproject: None,
                tombi: None,
            });
            config
        }),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn nagi_sql_lint_feature_can_be_disabled(
        MISSING_MEMBER,
        SourcePath(workspace_path()),
        Config({
            let mut config = lint_config(tombi_config::SeverityLevel::Warn);
            config.extensions.as_mut().unwrap().nagi_sql = Some(
                tombi_config::NagiSqlExtensionFeatures::Features(
                    tombi_config::NagiSqlExtensionFeatureTree {
                        lint: Some(tombi_config::NagiSqlLintFeatures::Enabled(
                            tombi_config::EnabledOnly { enabled: false.into() },
                        )),
                        lsp: None,
                    },
                ),
            );
            config
        }),
    ) -> Ok(_)
}
