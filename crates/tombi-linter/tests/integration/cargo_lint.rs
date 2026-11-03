use tombi_diagnostic::Level;
use tombi_linter::test_lint;

fn cargo_path() -> std::path::PathBuf {
    tombi_test_lib::project_root_path().join("Cargo.toml")
}

fn member_cargo_path() -> std::path::PathBuf {
    tombi_test_lib::project_root_path().join("crates/tombi-linter/Cargo.toml")
}

fn workspace_without_lints_cargo_path() -> std::path::PathBuf {
    tombi_test_lib::project_root_path().join(
        "crates/tombi-lsp/tests/fixtures/cargo/feature-navigation/workspace/consumer/Cargo.toml",
    )
}

fn span_of(source: &str, value: &str) -> tombi_text::Span {
    let quoted = format!("\"{value}\"");
    let start = source.find(&quoted).unwrap();
    tombi_text::Span::new(
        tombi_text::Offset::new(start.try_into().unwrap()),
        tombi_text::Offset::new((start + quoted.len()).try_into().unwrap()),
    )
}

fn span_of_token(source: &str, value: &str) -> tombi_text::Span {
    let start = source.find(value).unwrap();
    tombi_text::Span::new(
        tombi_text::Offset::new(start.try_into().unwrap()),
        tombi_text::Offset::new((start + value.len()).try_into().unwrap()),
    )
}

const MISSING_PATHS: &str = r#"[package]
name = "cargo-lint-fixture"
version = "0.1.0"
edition = "2024"
readme = "missing-README.md"
license-file = "missing-LICENSE"
build = "missing-build.rs"

[lib]
path = "missing-lib.rs"

[[bin]]
name = "fixture"
path = "missing-main.rs"

[dependencies]
local = { path = "missing-local" }
"#;

test_lint! {
    #[test]
    fn missing_cargo_paths_are_reported_by_rule(
        MISSING_PATHS,
        SourcePath(cargo_path()),
    ) -> Diagnostics([{
        code: "cargo/missing-readme",
        level: Level::ERROR,
        span: span_of(MISSING_PATHS, "missing-README.md")
    }, {
        code: "cargo/missing-license-file",
        level: Level::ERROR,
        span: span_of(MISSING_PATHS, "missing-LICENSE")
    }, {
        code: "cargo/missing-build-script",
        level: Level::ERROR,
        span: span_of(MISSING_PATHS, "missing-build.rs")
    }, {
        code: "cargo/missing-target-path",
        level: Level::ERROR,
        span: span_of(MISSING_PATHS, "missing-lib.rs")
    }, {
        code: "cargo/missing-target-path",
        level: Level::ERROR,
        span: span_of(MISSING_PATHS, "missing-main.rs")
    }, {
        code: "cargo/missing-path-dependency",
        level: Level::ERROR,
        span: span_of(MISSING_PATHS, "missing-local")
    }])
}

test_lint! {
    #[test]
    fn workspace_dependency_without_workspace_manifest_is_reported(
        r#"[package]
name = "cargo-lint-fixture"
version = "0.1.0"
workspace = "missing-workspace"

[dependencies]
shared = { workspace = true }
"#,
        SourcePath(cargo_path()),
    ) -> Diagnostics([{
        code: "cargo/missing-workspace",
        level: Level::ERROR,
        span: span_of_token(r#"[package]
name = "cargo-lint-fixture"
version = "0.1.0"
workspace = "missing-workspace"

[dependencies]
shared = { workspace = true }
"#, "true")
    }])
}

test_lint! {
    #[test]
    fn workspace_dependency_without_definition_is_reported(
        r#"[package]
name = "cargo-lint-fixture"
version = "0.1.0"
workspace = "../.."

[dependencies]
missing-cargo-lint-workspace-dependency = { workspace = true }
"#,
        SourcePath(member_cargo_path()),
    ) -> Diagnostics([{
        code: "cargo/missing-workspace",
        level: Level::ERROR,
        span: span_of_token(r#"[package]
name = "cargo-lint-fixture"
version = "0.1.0"
workspace = "../.."

[dependencies]
missing-cargo-lint-workspace-dependency = { workspace = true }
"#, "true")
    }])
}

test_lint! {
    #[test]
    fn missing_workspace_package_field_is_reported(
        r#"[package]
name = "cargo-lint-fixture"
description.workspace = true
workspace = "../.."
"#,
        SourcePath(member_cargo_path()),
    ) -> Diagnostics([{
        code: "cargo/missing-workspace",
        level: Level::ERROR,
        span: span_of_token(r#"[package]
name = "cargo-lint-fixture"
description.workspace = true
workspace = "../.."
"#, "true")
    }])
}

test_lint! {
    #[test]
    fn inherited_workspace_package_fields_are_accepted(
        r#"[package]
name = "cargo-lint-fixture"
version.workspace = true
edition.workspace = true
workspace = "../.."
"#,
        SourcePath(member_cargo_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn missing_workspace_lints_are_reported(
        r#"[package]
name = "cargo-lint-fixture"
version = "0.1.0"

[lints]
workspace = true
"#,
        SourcePath(workspace_without_lints_cargo_path()),
    ) -> Diagnostics([{
        code: "cargo/missing-workspace",
        level: Level::ERROR,
        span: span_of_token(r#"[package]
name = "cargo-lint-fixture"
version = "0.1.0"

[lints]
workspace = true
"#, "true")
    }])
}

test_lint! {
    #[test]
    fn inherited_workspace_lints_are_accepted(
        r#"[package]
name = "cargo-lint-fixture"
version = "0.1.0"

[lints]
workspace = true
"#,
        SourcePath(member_cargo_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn workspace_dependency_with_definition_is_accepted(
        r#"[package]
name = "cargo-lint-fixture"
version = "0.1.0"
workspace = "../.."

[dependencies]
serde = { workspace = true }
"#,
        SourcePath(member_cargo_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn existing_paths_and_inferred_paths_are_accepted(
        r#"[package]
name = "cargo-lint-fixture"
version = "0.1.0"
edition = "2024"
readme = "README.md"

[lib]
"#,
        SourcePath(cargo_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn missing_path_dependency_severity_can_be_downgraded(
        r#"[dependencies]
local = { path = "missing-local" }
"#,
        SourcePath(cargo_path()),
        Config({
            let mut config = tombi_config::Config::default();
            config.extensions = Some(tombi_config::Extensions {
                cargo: Some(tombi_config::CargoExtensionFeatures::Features(
                    tombi_config::CargoExtensionFeatureTree {
                        lint: Some(tombi_config::CargoLintFeatures::Features(
                            tombi_config::CargoLintFeatureTree {
                                rules: Some(tombi_config::CargoLintRules {
                                    missing_path_dependency: tombi_config::SeverityLevel::Warn,
                                    ..Default::default()
                                }),
                            },
                        )),
                        lsp: None,
                    },
                )),
                nagi_sql: None,
                pyproject: None,
                tombi: None,
            });
            config
        }),
    ) -> Diagnostics([{
        code: "cargo/missing-path-dependency",
        level: Level::WARNING,
        span: span_of(r#"[dependencies]
local = { path = "missing-local" }
"#, "missing-local")
    }])
}
