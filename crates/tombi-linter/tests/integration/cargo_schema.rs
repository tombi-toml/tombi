use tombi_linter::test_lint;
use tombi_severity_level::SeverityLevel;
use tombi_test_lib::{cargo_schema_path, project_root_path, pyproject_schema_path};

test_lint! {
    #[test]
    fn test_workspace_dependencies(
        r#"
        [workspace.dependencies]
        serde.version = "^1.0.0"
        serde.features = ["derive"]
        serde.workspace = true
        "#,
        SchemaPath(cargo_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_workspace_unknown(
        r#"
        [workspace]
        aaa = 1
        "#,
        SchemaPath(cargo_schema_path()),
    ) -> Err([tombi_validator::DiagnosticKind::TableStrictAdditionalKeys {
        accessors: tombi_schema_store::SchemaAccessors::from(vec![
            tombi_schema_store::SchemaAccessor::Key("workspace".to_string()),
        ]),
        schema_uri: tombi_schema_store::SchemaUri::from_file_path(cargo_schema_path()).unwrap(),
        key: "aaa".to_string(),
    }])
}

test_lint! {
    #[test]
    fn test_unknown_keys(
        r#"
        [aaa]
        bbb = 1
        "#,
        SchemaPath(cargo_schema_path()),
    ) -> Err([tombi_validator::DiagnosticKind::KeyNotAllowed { key: "aaa".to_string() }])
}

test_lint! {
    #[test]
    fn test_package_name_wrong_type(
        r#"
        [package]
        name = 1
        "#,
        SchemaPath(cargo_schema_path()),
    ) -> Err([tombi_validator::DiagnosticKind::TypeMismatch {
        expected: tombi_schema_store::ValueType::String,
        actual: tombi_document_tree::ValueType::Integer,
    }])
}

test_lint! {
    #[test]
    fn test_package_name_wrong_type_with_comment_directive_disabled_eq_true(
        r#"
        [package]
        name = 1 # tombi: lint.rules.type-mismatch.disabled = true
        "#,
        SchemaPath(cargo_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_package_name_wrong_type_with_wrong_comment_directive_disabled_eq_true(
        r#"
        [package]
        name = 1 # tombi: lint.rules.type-mism.disabled = true
        "#,
        SchemaPath(cargo_schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::KeyNotAllowed { key: "type-mism".to_string() },
        tombi_validator::DiagnosticKind::TypeMismatch {
            expected: tombi_schema_store::ValueType::String,
            actual: tombi_document_tree::ValueType::Integer,
        }
    ])
}

fn deprecated_schema_config(deprecated_level: Option<SeverityLevel>) -> tombi_config::Config {
    let schema_path = project_root_path()
        .join("schemas")
        .join("deprecated-test.schema.json");
    let schema_uri = tombi_schema_store::SchemaUri::from_file_path(schema_path).unwrap();

    let mut config = tombi_config::Config::default();
    config.schemas = Some(vec![tombi_config::SchemaItem::Root(
        tombi_config::RootSchema {
            toml_version: None,
            path: schema_uri.to_string(),
            include: vec!["*.toml".into()],
            exclude: None,
            lint: deprecated_level.map(|deprecated_level| tombi_config::SchemaLintOptions {
                rules: Some(tombi_config::SchemaLintRules {
                    deprecated: Some(tombi_severity_level::SeverityLevelDefaultWarn::from(
                        deprecated_level,
                    )),
                }),
            }),
            format: None,
            overrides: None,
        },
    )]);
    config
}

fn deprecated_sub_schema_config(deprecated_level: Option<SeverityLevel>) -> tombi_config::Config {
    let schema_path = project_root_path()
        .join("schemas")
        .join("deprecated-test.schema.json");
    let schema_uri = tombi_schema_store::SchemaUri::from_file_path(schema_path).unwrap();

    let mut config = tombi_config::Config::default();
    config.schemas = Some(vec![tombi_config::SchemaItem::Sub(
        tombi_config::SubSchema {
            root: "tool.example".to_string(),
            path: schema_uri.to_string(),
            include: vec!["*.toml".into()],
            exclude: None,
            lint: deprecated_level.map(|deprecated_level| tombi_config::SchemaLintOptions {
                rules: Some(tombi_config::SchemaLintRules {
                    deprecated: Some(tombi_severity_level::SeverityLevelDefaultWarn::from(
                        deprecated_level,
                    )),
                }),
            }),
            format: None,
            overrides: None,
        },
    )]);
    config
}

fn deprecated_override_config(deprecated_level: SeverityLevel) -> tombi_config::Config {
    let schema_path = project_root_path()
        .join("schemas")
        .join("deprecated-test.schema.json");
    let schema_uri = tombi_schema_store::SchemaUri::from_file_path(schema_path).unwrap();

    let mut config = tombi_config::Config::default();
    config.schemas = Some(vec![tombi_config::SchemaItem::Root(
        tombi_config::RootSchema {
            toml_version: None,
            path: schema_uri.to_string(),
            include: vec!["*.toml".into()],
            exclude: None,
            lint: None,
            format: None,
            overrides: Some(vec![tombi_config::SchemaOverrideItem {
                targets: vec!["value".into()],
                format: None,
                lint: Some(tombi_config::SchemaOverrideLintOptions {
                    rules: Some(tombi_config::SchemaOverrideLintRules {
                        deprecated: Some(deprecated_level.into()),
                    }),
                }),
            }]),
        },
    )]);
    config
}

fn deprecated_root_lint_for_subschema_config(
    deprecated_level: SeverityLevel,
) -> tombi_config::Config {
    let schema_path = project_root_path()
        .join("schemas")
        .join("deprecated-test.schema.json");
    let schema_uri = tombi_schema_store::SchemaUri::from_file_path(schema_path).unwrap();

    let mut config = tombi_config::Config::default();
    config.schemas = Some(vec![
        tombi_config::SchemaItem::Root(tombi_config::RootSchema {
            toml_version: None,
            path: pyproject_schema_path().to_string_lossy().into_owned(),
            include: vec!["pyproject.toml".into()],
            exclude: None,
            lint: Some(tombi_config::SchemaLintOptions {
                rules: Some(tombi_config::SchemaLintRules {
                    deprecated: Some(deprecated_level.into()),
                }),
            }),
            format: None,
            overrides: None,
        }),
        tombi_config::SchemaItem::Sub(tombi_config::SubSchema {
            root: "tool.example".to_string(),
            path: schema_uri.to_string(),
            include: vec!["pyproject.toml".into()],
            exclude: None,
            lint: None,
            format: None,
            overrides: None,
        }),
    ]);
    config
}

fn deprecated_exact_index_override_config() -> tombi_config::Config {
    let schema_path = project_root_path()
        .join("schemas")
        .join("exact-index-override-test.schema.json");
    let schema_uri = tombi_schema_store::SchemaUri::from_file_path(schema_path).unwrap();

    let mut config = tombi_config::Config::default();
    config.schemas = Some(vec![tombi_config::SchemaItem::Root(
        tombi_config::RootSchema {
            toml_version: None,
            path: schema_uri.to_string(),
            include: vec!["*.toml".into()],
            exclude: None,
            lint: None,
            format: None,
            overrides: Some(vec![tombi_config::SchemaOverrideItem {
                targets: vec!["tuple[1].value".into()],
                format: None,
                lint: Some(tombi_config::SchemaOverrideLintOptions {
                    rules: Some(tombi_config::SchemaOverrideLintRules {
                        deprecated: Some(SeverityLevel::Off.into()),
                    }),
                }),
            }]),
        },
    )]);
    config
}

fn deprecated_exact_index_precedence_config() -> tombi_config::Config {
    let schema_path = project_root_path()
        .join("schemas")
        .join("exact-index-override-test.schema.json");
    let schema_uri = tombi_schema_store::SchemaUri::from_file_path(schema_path).unwrap();

    let mut config = tombi_config::Config::default();
    config.schemas = Some(vec![tombi_config::SchemaItem::Root(
        tombi_config::RootSchema {
            toml_version: None,
            path: schema_uri.to_string(),
            include: vec!["*.toml".into()],
            exclude: None,
            lint: None,
            format: None,
            overrides: Some(vec![
                tombi_config::SchemaOverrideItem {
                    targets: vec!["tuple[*].value".into()],
                    format: None,
                    lint: Some(tombi_config::SchemaOverrideLintOptions {
                        rules: Some(tombi_config::SchemaOverrideLintRules {
                            deprecated: Some(SeverityLevel::Off.into()),
                        }),
                    }),
                },
                tombi_config::SchemaOverrideItem {
                    targets: vec!["tuple[1].value".into()],
                    format: None,
                    lint: Some(tombi_config::SchemaOverrideLintOptions {
                        rules: Some(tombi_config::SchemaOverrideLintRules {
                            deprecated: Some(SeverityLevel::Error.into()),
                        }),
                    }),
                },
            ]),
        },
    )]);
    config
}

test_lint! {
    #[test]
    fn test_deprecated_schema_lint_level_default_config(
        "value = 1\n",
        Config(deprecated_schema_config(None)),
    ) -> Diagnostics([{
        code: "deprecated",
        level: tombi_diagnostic::Level::WARNING,
    }])
}

test_lint! {
    #[test]
    fn test_deprecated_schema_lint_level_off(
        "value = 1\n",
        Config(deprecated_schema_config(Some(SeverityLevel::Off))),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_deprecated_schema_lint_level_warn(
        "value = 1\n",
        Config(deprecated_schema_config(Some(SeverityLevel::Warn))),
    ) -> Diagnostics([{
        code: "deprecated",
        level: tombi_diagnostic::Level::WARNING,
    }])
}

test_lint! {
    #[test]
    fn test_deprecated_schema_lint_level_error(
        "value = 1\n",
        Config(deprecated_schema_config(Some(SeverityLevel::Error))),
    ) -> Diagnostics([{
        code: "deprecated",
        level: tombi_diagnostic::Level::ERROR,
    }])
}

test_lint! {
    #[test]
    fn test_deprecated_sub_schema_lint_level_error(
        "[tool.example]\nvalue = 1\n",
        Config(deprecated_sub_schema_config(Some(SeverityLevel::Error))),
    ) -> Diagnostics([{
        code: "deprecated",
        level: tombi_diagnostic::Level::ERROR,
    }])
}

test_lint! {
    #[test]
    fn test_deprecated_schema_override_off(
        "value = 1\n",
        Config(deprecated_override_config(SeverityLevel::Off)),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_deprecated_schema_override_error(
        "value = 1\n",
        Config(deprecated_override_config(SeverityLevel::Error)),
    ) -> Diagnostics([{
        code: "deprecated",
        level: tombi_diagnostic::Level::ERROR,
    }])
}

test_lint! {
    #[test]
    fn test_root_schema_lint_rules_propagate_to_subschema(
        "[tool.example]\nvalue = 1\n",
        Config(deprecated_root_lint_for_subschema_config(SeverityLevel::Error)),
        SourcePath(project_root_path().join("pyproject.toml")),
    ) -> Diagnostics([{
        code: "deprecated",
        level: tombi_diagnostic::Level::ERROR,
    }])
}

test_lint! {
    #[test]
    fn test_deprecated_schema_override_matches_exact_tuple_index(
        "tuple = [{ value = 1 }, { value = 2 }]\n",
        Config(deprecated_exact_index_override_config()),
    ) -> Diagnostics([{
        code: "deprecated",
        level: tombi_diagnostic::Level::WARNING,
    }])
}

test_lint! {
    #[test]
    fn test_deprecated_schema_override_message_keeps_exact_index(
        "tuple = [{ value = 1 }, { value = 2 }]\n",
        Config(deprecated_exact_index_override_config()),
    ) -> Err([
        tombi_validator::DiagnosticKind::DeprecatedValue(
            tombi_schema_store::SchemaAccessors::from(vec![
                tombi_schema_store::SchemaAccessor::Key("tuple".to_string()),
                tombi_schema_store::SchemaAccessor::Index(0),
                tombi_schema_store::SchemaAccessor::Key("value".to_string()),
            ]),
            "1".to_string(),
        ),
    ])
}

test_lint! {
    #[test]
    fn test_deprecated_exact_index_override_beats_wildcard_override(
        "tuple = [{ value = 1 }, { value = 2 }]\n",
        Config(deprecated_exact_index_precedence_config()),
    ) -> Diagnostics([{
        code: "deprecated",
        level: tombi_diagnostic::Level::ERROR,
    }])
}
