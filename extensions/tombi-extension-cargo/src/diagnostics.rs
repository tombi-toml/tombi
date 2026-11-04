use std::collections::HashSet;
use std::path::Path;

use tombi_config::{CargoLintRules, SeverityLevel};
use tombi_document_tree_syntax::{DocumentTree, Table, Value, dig_keys};
use tombi_text::Span;

struct PathReference<'a> {
    value: &'a str,
    span: Span,
    kind: DiagnosticKind,
    severity: SeverityLevel,
}

#[derive(Clone, Copy)]
enum DiagnosticKind {
    Readme,
    LicenseFile,
    BuildScript,
    TargetPath,
    PathDependency,
    WorkspaceMember,
    WorkspacePackageField,
    WorkspaceDependency,
    WorkspaceLints,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum WorkspaceDefinitionKind {
    Package,
    Dependencies,
    Lints,
}

impl DiagnosticKind {
    fn rule(self) -> &'static str {
        match self {
            Self::Readme => "cargo/missing-readme",
            Self::LicenseFile => "cargo/missing-license-file",
            Self::BuildScript => "cargo/missing-build-script",
            Self::TargetPath => "cargo/missing-target-path",
            Self::PathDependency => "cargo/missing-path-dependency",
            Self::WorkspaceMember => "cargo/missing-workspace-member",
            Self::WorkspacePackageField | Self::WorkspaceDependency | Self::WorkspaceLints => {
                "cargo/missing-workspace"
            }
        }
    }

    fn message(self, path: &str) -> String {
        match self {
            Self::Readme => format!("README path `{path}` does not exist"),
            Self::LicenseFile => format!("license file path `{path}` does not exist"),
            Self::BuildScript => format!("build script path `{path}` does not exist"),
            Self::TargetPath => format!("target source path `{path}` does not exist"),
            Self::PathDependency => {
                format!("path dependency `{path}` does not contain Cargo.toml")
            }
            Self::WorkspaceMember => {
                format!("workspace member `{path}` does not contain Cargo.toml")
            }
            Self::WorkspacePackageField => format!(
                "workspace package field `{path}` is not defined in the workspace Cargo.toml"
            ),
            Self::WorkspaceDependency => {
                format!("workspace dependency `{path}` is not defined in the workspace Cargo.toml")
            }
            Self::WorkspaceLints => {
                "workspace lints are not defined in the workspace Cargo.toml".to_owned()
            }
        }
    }

    fn workspace_definition(self) -> Option<WorkspaceDefinitionKind> {
        match self {
            Self::WorkspacePackageField => Some(WorkspaceDefinitionKind::Package),
            Self::WorkspaceDependency => Some(WorkspaceDefinitionKind::Dependencies),
            Self::WorkspaceLints => Some(WorkspaceDefinitionKind::Lints),
            _ => None,
        }
    }
}

pub fn validate(
    document_tree: &DocumentTree<'_>,
    manifest_path: Option<&Path>,
    toml_version: tombi_config::TomlVersion,
    rules: CargoLintRules,
) -> Vec<tombi_diagnostic::Diagnostic> {
    if rules == CargoLintRules::off() {
        return Vec::new();
    }

    let Some(manifest_path) = manifest_path else {
        return Vec::new();
    };
    let Some(manifest_dir) = manifest_path.parent() else {
        return Vec::new();
    };

    // Disabled rules must not trigger filesystem checks or workspace manifest discovery.
    let references = collect_path_references(document_tree, rules)
        .into_iter()
        .filter(|reference| reference.severity != SeverityLevel::Off)
        .collect::<Vec<_>>();

    let workspace_references = references
        .iter()
        .filter(|reference| reference.kind.workspace_definition().is_some())
        .collect::<Vec<_>>();
    let missing_workspace_definitions = if workspace_references.is_empty() {
        HashSet::new()
    } else if document_tree.contains_key("workspace") {
        missing_workspace_definitions(document_tree, &workspace_references)
    } else {
        crate::find_workspace_cargo_toml(
            manifest_path,
            crate::get_workspace_cargo_toml_path(document_tree),
            toml_version,
            |_, workspace_document_tree, _| {
                missing_workspace_definitions(workspace_document_tree, &workspace_references)
            },
        )
        .unwrap_or_else(|| {
            workspace_references
                .iter()
                .filter_map(|reference| {
                    reference
                        .kind
                        .workspace_definition()
                        .map(|kind| (kind, reference.value.to_owned()))
                })
                .collect()
        })
    };

    let mut diagnostics = references
        .into_iter()
        .filter_map(|reference| {
            let exists = if matches!(reference.kind, DiagnosticKind::PathDependency) {
                let dependency_directory = Path::new(reference.value);
                let dependency_directory = if dependency_directory.is_absolute() {
                    dependency_directory.to_path_buf()
                } else {
                    manifest_dir.join(dependency_directory)
                };
                tombi_fs::is_file(&dependency_directory.join("Cargo.toml"))
            } else if let Some(kind) = reference.kind.workspace_definition() {
                !missing_workspace_definitions.contains(&(kind, reference.value.to_owned()))
            } else {
                tombi_fs::is_file(&manifest_dir.join(reference.value))
            };
            if exists {
                return None;
            }
            let rule = reference.kind.rule();
            let message = reference.kind.message(reference.value);
            Some(match reference.severity {
                SeverityLevel::Error => {
                    tombi_diagnostic::Diagnostic::new_error(message, rule, reference.span)
                }
                SeverityLevel::Warn => {
                    tombi_diagnostic::Diagnostic::new_warning(message, rule, reference.span)
                }
                SeverityLevel::Off => return None,
            })
        })
        .collect::<Vec<_>>();

    validate_workspace_members(
        document_tree,
        manifest_dir,
        rules.missing_workspace_member,
        &mut diagnostics,
    );

    diagnostics
}

pub fn references_path(
    document_tree: &DocumentTree<'_>,
    manifest_path: &Path,
    changed_path: &Path,
) -> bool {
    let Some(manifest_dir) = manifest_path.parent() else {
        return false;
    };
    let changed_path = tombi_fs::normalize(changed_path);
    let references = collect_path_references(document_tree, CargoLintRules::default());

    if !document_tree.contains_key("workspace")
        && references
            .iter()
            .any(|reference| reference.kind.workspace_definition().is_some())
        && changed_path
            .file_name()
            .is_some_and(|name| name == "Cargo.toml")
    {
        if let Some(workspace_path) = crate::get_workspace_cargo_toml_path(document_tree) {
            let workspace_path = Path::new(workspace_path);
            let workspace_path = if workspace_path.is_absolute() {
                workspace_path.to_path_buf()
            } else {
                manifest_dir.join(workspace_path)
            };
            let workspace_manifest = if workspace_path
                .file_name()
                .is_some_and(|name| name == "Cargo.toml")
            {
                workspace_path
            } else {
                workspace_path.join("Cargo.toml")
            };
            if tombi_fs::normalize(&workspace_manifest) == changed_path {
                return true;
            }
        } else if changed_path
            .parent()
            .is_some_and(|changed_dir| manifest_dir.starts_with(changed_dir))
        {
            return true;
        }
    }

    if changed_path
        .file_name()
        .is_some_and(|name| name == "Cargo.toml")
        && workspace_member_references_path(document_tree, manifest_dir, &changed_path)
    {
        return true;
    }

    references.iter().any(|reference| {
        let path = if matches!(reference.kind, DiagnosticKind::PathDependency) {
            Path::new(reference.value).join("Cargo.toml")
        } else if reference.kind.workspace_definition().is_some() {
            return false;
        } else {
            Path::new(reference.value).to_path_buf()
        };
        let path = if path.is_absolute() {
            path
        } else {
            manifest_dir.join(path)
        };
        tombi_fs::normalize(&path) == changed_path
    })
}

/// Returns whether a changed `Cargo.toml` is referenced by a `workspace.members` or
/// `workspace.default-members` entry, so the workspace manifest can be re-diagnosed.
fn workspace_member_references_path(
    document_tree: &DocumentTree<'_>,
    manifest_dir: &Path,
    changed_path: &Path,
) -> bool {
    let manifest_dir_normalized = tombi_fs::normalize(manifest_dir);
    for members_key in ["members", "default-members"] {
        let Some((_, Value::Array(members))) = dig_keys(document_tree, &["workspace", members_key])
        else {
            continue;
        };
        for member in members.values() {
            let Value::String(member) = member else {
                continue;
            };
            let value = member.value();
            if is_glob_pattern(value) {
                // Match the user pattern against the changed directory relative to the workspace,
                // so glob metacharacters in the workspace path itself are not reinterpreted.
                if let Ok(pattern) = glob::Pattern::new(value)
                    && let Some(relative) = changed_path
                        .parent()
                        .and_then(|parent| parent.strip_prefix(&manifest_dir_normalized).ok())
                    && pattern.matches_path(relative)
                {
                    return true;
                }
            } else if tombi_fs::normalize(&member_dir(manifest_dir, value).join("Cargo.toml"))
                == changed_path
            {
                return true;
            }
        }
    }
    false
}

fn collect_path_references<'a>(
    document_tree: &'a DocumentTree<'a>,
    rules: CargoLintRules,
) -> Vec<PathReference<'a>> {
    let mut references = Vec::new();
    collect_file_field(
        document_tree,
        &["package", "readme"],
        DiagnosticKind::Readme,
        Some("README.md"),
        rules.missing_readme,
        &mut references,
    );
    collect_workspace_package_fields(document_tree, rules.missing_workspace, &mut references);
    collect_workspace_lints(document_tree, rules.missing_workspace, &mut references);
    collect_file_field(
        document_tree,
        &["workspace", "package", "readme"],
        DiagnosticKind::Readme,
        Some("README.md"),
        rules.missing_readme,
        &mut references,
    );
    collect_file_field(
        document_tree,
        &["package", "license-file"],
        DiagnosticKind::LicenseFile,
        None,
        rules.missing_license_file,
        &mut references,
    );
    collect_file_field(
        document_tree,
        &["workspace", "package", "license-file"],
        DiagnosticKind::LicenseFile,
        None,
        rules.missing_license_file,
        &mut references,
    );
    collect_file_field(
        document_tree,
        &["package", "build"],
        DiagnosticKind::BuildScript,
        Some("build.rs"),
        rules.missing_build_script,
        &mut references,
    );

    if rules.missing_target_path != SeverityLevel::Off {
        for name in ["lib", "bin", "example", "test", "bench"] {
            if let Some(Value::Array(tables)) = document_tree.get(name) {
                for table in tables.values() {
                    if let Value::Table(table) = table {
                        collect_table_file_field(
                            table,
                            "path",
                            DiagnosticKind::TargetPath,
                            rules.missing_target_path,
                            &mut references,
                        );
                    }
                }
            } else if let Some(Value::Table(table)) = document_tree.get(name) {
                collect_table_file_field(
                    table,
                    "path",
                    DiagnosticKind::TargetPath,
                    rules.missing_target_path,
                    &mut references,
                );
            }
        }
    }

    if rules.missing_path_dependency != SeverityLevel::Off
        || rules.missing_workspace != SeverityLevel::Off
    {
        for name in ["dependencies", "dev-dependencies", "build-dependencies"] {
            if let Some((_, Value::Table(table))) = dig_keys(document_tree, &[name]) {
                collect_dependency_paths(
                    table,
                    rules.missing_path_dependency,
                    rules.missing_workspace,
                    &mut references,
                );
            }
        }
        if let Some((_, Value::Table(table))) =
            dig_keys(document_tree, &["workspace", "dependencies"])
        {
            collect_dependency_paths(
                table,
                rules.missing_path_dependency,
                rules.missing_workspace,
                &mut references,
            );
        }
        if let Some((_, Value::Table(registries))) = dig_keys(document_tree, &["patch"]) {
            for packages in registries.values() {
                if let Value::Table(packages) = packages {
                    collect_dependency_paths(
                        packages,
                        rules.missing_path_dependency,
                        rules.missing_workspace,
                        &mut references,
                    );
                }
            }
        }
        if let Some((_, Value::Table(replacements))) = dig_keys(document_tree, &["replace"]) {
            collect_dependency_paths(
                replacements,
                rules.missing_path_dependency,
                rules.missing_workspace,
                &mut references,
            );
        }
        if let Some((_, Value::Table(targets))) = dig_keys(document_tree, &["target"]) {
            for target in targets.values() {
                let Value::Table(target) = target else {
                    continue;
                };
                for name in ["dependencies", "dev-dependencies", "build-dependencies"] {
                    if let Some(Value::Table(table)) = target.get(name) {
                        collect_dependency_paths(
                            table,
                            rules.missing_path_dependency,
                            rules.missing_workspace,
                            &mut references,
                        );
                    }
                }
            }
        }
    }
    references
}

fn missing_workspace_definitions(
    document_tree: &DocumentTree<'_>,
    references: &[&PathReference<'_>],
) -> HashSet<(WorkspaceDefinitionKind, String)> {
    references
        .iter()
        .filter_map(|reference| {
            let kind = reference.kind.workspace_definition()?;
            let exists = match kind {
                WorkspaceDefinitionKind::Package => {
                    dig_keys(document_tree, &["workspace", "package"])
                        .and_then(|(_, value)| match value {
                            Value::Table(table) => Some(table.contains_key(reference.value)),
                            _ => None,
                        })
                        .unwrap_or(false)
                }
                WorkspaceDefinitionKind::Dependencies => {
                    dig_keys(document_tree, &["workspace", "dependencies"])
                        .and_then(|(_, value)| match value {
                            Value::Table(table) => Some(table.contains_key(reference.value)),
                            _ => None,
                        })
                        .unwrap_or(false)
                }
                WorkspaceDefinitionKind::Lints => {
                    dig_keys(document_tree, &["workspace", "lints"]).is_some()
                }
            };
            (!exists).then(|| (kind, reference.value.to_owned()))
        })
        .collect()
}

fn collect_workspace_package_fields<'a>(
    document_tree: &'a DocumentTree<'a>,
    severity: SeverityLevel,
    out: &mut Vec<PathReference<'a>>,
) {
    if severity == SeverityLevel::Off {
        return;
    }

    const FIELDS: &[&str] = &[
        "authors",
        "categories",
        "description",
        "documentation",
        "edition",
        "exclude",
        "homepage",
        "include",
        "keywords",
        "license",
        "license-file",
        "publish",
        "readme",
        "repository",
        "rust-version",
        "version",
    ];

    let Some((_, Value::Table(package))) = dig_keys(document_tree, &["package"]) else {
        return;
    };

    for field in FIELDS {
        let Some(Value::Table(inherited)) = package.get(field) else {
            continue;
        };
        if let Some(Value::Boolean(workspace)) = inherited.get("workspace")
            && workspace.value()
        {
            out.push(PathReference {
                value: field,
                span: workspace.span(),
                kind: DiagnosticKind::WorkspacePackageField,
                severity,
            });
        }
    }
}

fn collect_workspace_lints<'a>(
    document_tree: &'a DocumentTree<'a>,
    severity: SeverityLevel,
    out: &mut Vec<PathReference<'a>>,
) {
    if severity == SeverityLevel::Off {
        return;
    }

    if let Some(Value::Table(lints)) = document_tree.get("lints")
        && let Some(Value::Boolean(workspace)) = lints.get("workspace")
        && workspace.value()
    {
        out.push(PathReference {
            value: "lints",
            span: workspace.span(),
            kind: DiagnosticKind::WorkspaceLints,
            severity,
        });
    }
}

fn validate_workspace_members(
    document_tree: &DocumentTree<'_>,
    manifest_dir: &Path,
    severity: SeverityLevel,
    out: &mut Vec<tombi_diagnostic::Diagnostic>,
) {
    if severity == SeverityLevel::Off {
        return;
    }

    let exclude = collect_member_exclude_matchers(document_tree, manifest_dir);

    for members_key in ["members", "default-members"] {
        let Some((_, Value::Array(members))) = dig_keys(document_tree, &["workspace", members_key])
        else {
            continue;
        };
        for member in members.values() {
            let Value::String(member) = member else {
                continue;
            };
            validate_member_pattern(
                member.value(),
                member.span(),
                manifest_dir,
                &exclude,
                severity,
                out,
            );
        }
    }
}

fn validate_member_pattern(
    value: &str,
    span: Span,
    manifest_dir: &Path,
    exclude: &[ExcludeMatcher],
    severity: SeverityLevel,
    out: &mut Vec<tombi_diagnostic::Diagnostic>,
) {
    if is_glob_pattern(value) {
        let matched_paths = match tombi_fs::glob(&member_glob_pattern(manifest_dir, value)) {
            Ok(paths) => paths,
            Err(_) => return,
        };
        if matched_paths.is_empty() {
            // Cargo falls back to treating the glob as a literal member path only when it matches
            // nothing, so only report it when that literal fallback also lacks a Cargo.toml.
            if !tombi_fs::is_file(&member_dir(manifest_dir, value).join("Cargo.toml")) {
                push_member_diagnostic(value, span, severity, out);
            }
            return;
        }
        // Cargo ignores glob matches that are not directories, but every matched directory
        // must contain a Cargo.toml unless it is excluded.
        for dir in matched_paths
            .into_iter()
            .filter(|path| tombi_fs::is_dir(path))
        {
            if exclude.iter().any(|matcher| matcher.matches(&dir)) {
                continue;
            }
            if !tombi_fs::is_file(&dir.join("Cargo.toml")) {
                let display = dir
                    .strip_prefix(manifest_dir)
                    .unwrap_or(&dir)
                    .to_string_lossy()
                    .into_owned();
                push_member_diagnostic(&display, span, severity, out);
            }
        }
    } else if !tombi_fs::is_file(&member_dir(manifest_dir, value).join("Cargo.toml")) {
        push_member_diagnostic(value, span, severity, out);
    }
}

fn push_member_diagnostic(
    member: &str,
    span: Span,
    severity: SeverityLevel,
    out: &mut Vec<tombi_diagnostic::Diagnostic>,
) {
    let rule = DiagnosticKind::WorkspaceMember.rule();
    let message = DiagnosticKind::WorkspaceMember.message(member);
    match severity {
        SeverityLevel::Error => {
            out.push(tombi_diagnostic::Diagnostic::new_error(message, rule, span))
        }
        SeverityLevel::Warn => out.push(tombi_diagnostic::Diagnostic::new_warning(
            message, rule, span,
        )),
        SeverityLevel::Off => {}
    }
}

struct ExcludeMatcher {
    literal: std::path::PathBuf,
    glob: Option<glob::Pattern>,
}

impl ExcludeMatcher {
    fn matches(&self, dir: &Path) -> bool {
        if dir == self.literal || dir.starts_with(&self.literal) {
            return true;
        }
        self.glob
            .as_ref()
            .is_some_and(|pattern| pattern.matches_path(dir))
    }
}

fn collect_member_exclude_matchers(
    document_tree: &DocumentTree<'_>,
    manifest_dir: &Path,
) -> Vec<ExcludeMatcher> {
    let Some((_, Value::Array(exclude))) = dig_keys(document_tree, &["workspace", "exclude"])
    else {
        return Vec::new();
    };
    exclude
        .values()
        .iter()
        .filter_map(|value| match value {
            Value::String(value) => {
                let literal = tombi_fs::normalize(&member_dir(manifest_dir, value.value()));
                // Escape the workspace directory so glob metacharacters in the path itself are not
                // reinterpreted, keeping the user exclude pattern intact.
                let glob =
                    glob::Pattern::new(&member_glob_pattern(manifest_dir, value.value())).ok();
                Some(ExcludeMatcher { literal, glob })
            }
            _ => None,
        })
        .collect()
}

fn member_dir(manifest_dir: &Path, value: &str) -> std::path::PathBuf {
    let path = Path::new(value);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        manifest_dir.join(path)
    }
}

/// Builds the glob pattern for a relative member, escaping the workspace directory so glob
/// metacharacters in the workspace path are not reinterpreted while keeping the user pattern intact.
fn member_glob_pattern(manifest_dir: &Path, value: &str) -> String {
    if Path::new(value).is_absolute() {
        return value.to_owned();
    }
    let mut pattern = glob::Pattern::escape(&manifest_dir.to_string_lossy());
    pattern.push(std::path::MAIN_SEPARATOR);
    pattern.push_str(value);
    pattern
}

fn is_glob_pattern(value: &str) -> bool {
    value.contains(['*', '?', '['])
}

fn collect_file_field<'a>(
    tree: &'a DocumentTree<'a>,
    keys: &[&str],
    kind: DiagnosticKind,
    default: Option<&'static str>,
    severity: SeverityLevel,
    out: &mut Vec<PathReference<'a>>,
) {
    if severity == SeverityLevel::Off {
        return;
    }

    if let Some((_, value)) = dig_keys(tree, keys) {
        match value {
            Value::String(value) => out.push(PathReference {
                value: value.value(),
                span: value.span(),
                kind,
                severity,
            }),
            Value::Boolean(value)
                if value.value()
                    && let Some(default) = default =>
            {
                out.push(PathReference {
                    value: default,
                    span: value.span(),
                    kind,
                    severity,
                })
            }
            _ => {}
        }
    }
}

fn collect_table_file_field<'a>(
    table: &'a Table<'a>,
    key: &str,
    kind: DiagnosticKind,
    severity: SeverityLevel,
    out: &mut Vec<PathReference<'a>>,
) {
    if severity == SeverityLevel::Off {
        return;
    }

    if let Some(Value::String(value)) = table.get(key) {
        out.push(PathReference {
            value: value.value(),
            span: value.span(),
            kind,
            severity,
        });
    }
}

fn collect_dependency_paths<'a>(
    table: &'a Table<'a>,
    path_dependency_severity: SeverityLevel,
    missing_workspace_severity: SeverityLevel,
    out: &mut Vec<PathReference<'a>>,
) {
    if path_dependency_severity == SeverityLevel::Off
        && missing_workspace_severity == SeverityLevel::Off
    {
        return;
    }

    for (key, value) in table.key_values() {
        if path_dependency_severity != SeverityLevel::Off
            && let Value::Table(options) = value
            && let Some(Value::String(path)) = options.get("path")
        {
            out.push(PathReference {
                value: path.value(),
                span: path.span(),
                kind: DiagnosticKind::PathDependency,
                severity: path_dependency_severity,
            });
        }
        if missing_workspace_severity != SeverityLevel::Off
            && let Value::Table(options) = value
            && let Some(Value::Boolean(workspace)) = options.get("workspace")
            && workspace.value()
        {
            out.push(PathReference {
                value: key.value(),
                span: workspace.span(),
                kind: DiagnosticKind::WorkspaceDependency,
                severity: missing_workspace_severity,
            });
        }
    }
}
