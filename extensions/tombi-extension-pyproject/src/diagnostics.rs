use std::{
    collections::{HashMap, HashSet},
    path::Path,
    sync::LazyLock,
};

use tombi_document_tree_syntax::{DocumentTree, Value, dig_keys};

use crate::normalize_pep503_name;

/// Validate the pyproject metadata rules that cannot be expressed by JSON Schema.
pub fn validate(
    document_tree: &DocumentTree<'_>,
    manifest_path: Option<&Path>,
    options: &tombi_config::PyprojectLintFeatureTree,
) -> Vec<tombi_diagnostic::Diagnostic> {
    let rules = options.rules.unwrap_or_default();
    if rules == tombi_config::PyprojectLintRules::off() {
        return Vec::new();
    }

    let mut diagnostics = Vec::new();
    if rules.invalid_dependency_specifier != tombi_config::SeverityLevel::Off
        || rules.duplicate_dependency_group != tombi_config::SeverityLevel::Off
        || rules.undefined_include_group != tombi_config::SeverityLevel::Off
        || rules.cyclic_include_group != tombi_config::SeverityLevel::Off
    {
        validate_dependency_specifiers(document_tree, &rules, &mut diagnostics);
    }
    validate_workspace_members(document_tree, manifest_path, &rules, &mut diagnostics);
    validate_extra_names(document_tree, &rules, &mut diagnostics);

    if rules.invalid_import_name == tombi_config::SeverityLevel::Off
        && rules.duplicate_import_name == tombi_config::SeverityLevel::Off
        && rules.ambiguous_import_name == tombi_config::SeverityLevel::Off
        && rules.missing_import_name_parent == tombi_config::SeverityLevel::Off
        && rules.unknown_classifier == tombi_config::SeverityLevel::Off
        && rules.invalid_email_display_name == tombi_config::SeverityLevel::Off
        && rules.invalid_entry_point_target == tombi_config::SeverityLevel::Off
    {
        return diagnostics;
    }

    let Some((_, Value::Table(project))) = dig_keys(document_tree, &["project"]) else {
        return diagnostics;
    };

    let check_import_names = rules.invalid_import_name != tombi_config::SeverityLevel::Off
        || rules.duplicate_import_name != tombi_config::SeverityLevel::Off
        || rules.ambiguous_import_name != tombi_config::SeverityLevel::Off
        || rules.missing_import_name_parent != tombi_config::SeverityLevel::Off;
    let (import_names, import_namespaces) = if check_import_names {
        (
            read_import_names(project.get("import-names")),
            read_import_names(project.get("import-namespaces")),
        )
    } else {
        (Vec::new(), Vec::new())
    };

    if rules.unknown_classifier != tombi_config::SeverityLevel::Off
        && let Some(Value::Array(classifiers)) = project.get("classifiers")
    {
        for classifier in classifiers.iter().filter_map(|value| match value {
            Value::String(value) => Some(value),
            _ => None,
        }) {
            if !classifier.value().starts_with("Private ::")
                && !TROVE_CLASSIFIERS.contains(classifier.value())
            {
                push_diagnostic(
                    DiagnosticKind::UnknownTroveClassifier {
                        classifier: classifier.value().to_owned(),
                    },
                    rules.unknown_classifier,
                    classifier.span(),
                    &mut diagnostics,
                );
            }
        }
    }

    if rules.invalid_email_display_name != tombi_config::SeverityLevel::Off {
        for field in ["authors", "maintainers"] {
            if let Some(Value::Array(people)) = project.get(field) {
                for person in people.iter().filter_map(|value| match value {
                    Value::Table(person) => Some(person),
                    _ => None,
                }) {
                    let Some(Value::String(name)) = person.get("name") else {
                        continue;
                    };
                    if name.value().contains(',') || !is_rfc822_display_name(name.value()) {
                        push_diagnostic(
                            DiagnosticKind::InvalidEmailDisplayName,
                            rules.invalid_email_display_name,
                            name.span(),
                            &mut diagnostics,
                        );
                    }
                }
            }
        }
    }

    if rules.invalid_entry_point_target != tombi_config::SeverityLevel::Off {
        for field in ["scripts", "gui-scripts"] {
            if let Some(value) = project.get(field) {
                validate_entry_point_table(value, &rules, &mut diagnostics);
            }
        }
        if let Some(Value::Table(groups)) = project.get("entry-points") {
            for entries in groups.values() {
                validate_entry_point_table(entries, &rules, &mut diagnostics);
            }
        }
    }

    if rules.invalid_import_name != tombi_config::SeverityLevel::Off {
        for entry in import_names.iter().chain(import_namespaces.iter()) {
            if let Some(identifier) = entry
                .name
                .split('.')
                .find(|part| is_python_keyword(part) || has_invalid_unicode_identifier_syntax(part))
            {
                push_diagnostic(
                    DiagnosticKind::InvalidImportName {
                        identifier: identifier.to_owned(),
                    },
                    rules.invalid_import_name,
                    entry.span,
                    &mut diagnostics,
                );
            }
        }
    }

    check_duplicate_import_names(&import_names, rules.duplicate_import_name, &mut diagnostics);
    check_duplicate_import_names(
        &import_namespaces,
        rules.duplicate_import_name,
        &mut diagnostics,
    );

    if rules.ambiguous_import_name != tombi_config::SeverityLevel::Off {
        let exclusive = import_names
            .iter()
            .map(|entry| entry.name)
            .collect::<HashSet<_>>();
        for entry in &import_namespaces {
            if exclusive.contains(entry.name) {
                push_diagnostic(
                    DiagnosticKind::AmbiguousImportName,
                    rules.ambiguous_import_name,
                    entry.span,
                    &mut diagnostics,
                );
            }
        }
    }

    if rules.missing_import_name_parent != tombi_config::SeverityLevel::Off {
        let declared = import_names
            .iter()
            .chain(import_namespaces.iter())
            .map(|entry| entry.name)
            .collect::<HashSet<_>>();
        for entry in import_names.iter().chain(import_namespaces.iter()) {
            let mut parts = entry.name.split('.');
            let mut parent = parts.next().unwrap_or_default().to_owned();
            for part in parts {
                if !declared.contains(parent.as_str()) {
                    push_diagnostic(
                        DiagnosticKind::MissingImportNameParent {
                            parent: parent.clone(),
                        },
                        rules.missing_import_name_parent,
                        entry.span,
                        &mut diagnostics,
                    );
                }
                parent.push('.');
                parent.push_str(part);
            }
        }
    }

    diagnostics
}

fn validate_extra_names(
    document_tree: &DocumentTree<'_>,
    rules: &tombi_config::PyprojectLintRules,
    diagnostics: &mut Vec<tombi_diagnostic::Diagnostic>,
) {
    if rules.duplicate_dependency_group == tombi_config::SeverityLevel::Off {
        return;
    }
    let Some((_, Value::Table(extras))) =
        dig_keys(document_tree, &["project", "optional-dependencies"])
    else {
        return;
    };

    let mut normalized_names = HashSet::new();
    for name in extras.keys() {
        if !normalized_names.insert(normalize_pep503_name(name.value())) {
            push_diagnostic(
                DiagnosticKind::DuplicateExtraName {
                    name: name.value().to_owned(),
                },
                rules.duplicate_dependency_group,
                name.span(),
                diagnostics,
            );
        }
    }
}

fn validate_dependency_specifiers(
    document_tree: &DocumentTree<'_>,
    rules: &tombi_config::PyprojectLintRules,
    diagnostics: &mut Vec<tombi_diagnostic::Diagnostic>,
) {
    if rules.invalid_dependency_specifier != tombi_config::SeverityLevel::Off {
        for path in [
            &["build-system", "requires"][..],
            &["project", "dependencies"][..],
        ] {
            if let Some((_, value)) = dig_keys(document_tree, path) {
                validate_dependency_array(value, rules, diagnostics);
            }
        }

        if let Some((_, Value::Table(uv))) = dig_keys(document_tree, &["tool", "uv"]) {
            for key in crate::UV_DEPENDENCY_KEYS {
                if let Some(value) = uv.get(key) {
                    validate_dependency_array(value, rules, diagnostics);
                }
            }
        }

        for path in [
            &["project", "optional-dependencies"][..],
            &["dependency-groups"][..],
        ] {
            let Some((_, Value::Table(groups))) = dig_keys(document_tree, path) else {
                continue;
            };
            for value in groups.values() {
                validate_dependency_array(value, rules, diagnostics);
            }
        }
    }

    if rules.duplicate_dependency_group != tombi_config::SeverityLevel::Off
        || rules.undefined_include_group != tombi_config::SeverityLevel::Off
        || rules.cyclic_include_group != tombi_config::SeverityLevel::Off
    {
        validate_include_groups(document_tree, rules, diagnostics);
    }
}

fn validate_include_groups(
    document_tree: &DocumentTree<'_>,
    rules: &tombi_config::PyprojectLintRules,
    diagnostics: &mut Vec<tombi_diagnostic::Diagnostic>,
) {
    if rules.duplicate_dependency_group == tombi_config::SeverityLevel::Off
        && rules.undefined_include_group == tombi_config::SeverityLevel::Off
        && rules.cyclic_include_group == tombi_config::SeverityLevel::Off
    {
        return;
    }
    let Some((_, Value::Table(groups))) = dig_keys(document_tree, &["dependency-groups"]) else {
        return;
    };

    let mut normalized_groups = HashSet::new();
    for key in groups.keys() {
        let normalized_name = normalize_pep503_name(key.value());
        if !normalized_groups.insert(normalized_name) {
            if rules.duplicate_dependency_group == tombi_config::SeverityLevel::Off {
                continue;
            }
            push_diagnostic(
                DiagnosticKind::DuplicateDependencyGroup {
                    group: key.value().to_owned(),
                },
                rules.duplicate_dependency_group,
                key.span(),
                diagnostics,
            );
        }
    }

    let mut includes = HashMap::<String, Vec<(String, tombi_text::Span)>>::new();
    if rules.undefined_include_group != tombi_config::SeverityLevel::Off
        || rules.cyclic_include_group != tombi_config::SeverityLevel::Off
    {
        for (key, group) in groups.key_values() {
            let Value::Array(dependencies) = group else {
                continue;
            };
            for dependency in dependencies.values() {
                let Value::Table(include_group) = dependency else {
                    continue;
                };
                let Some(Value::String(name)) = include_group.get("include-group") else {
                    continue;
                };
                let included_group = normalize_pep503_name(name.value());
                if !normalized_groups.contains(&included_group) {
                    if rules.undefined_include_group != tombi_config::SeverityLevel::Off {
                        push_diagnostic(
                            DiagnosticKind::UndefinedIncludeGroup {
                                group: name.value().to_owned(),
                            },
                            rules.undefined_include_group,
                            name.span(),
                            diagnostics,
                        );
                    }
                } else if rules.cyclic_include_group != tombi_config::SeverityLevel::Off {
                    includes
                        .entry(normalize_pep503_name(key.value()))
                        .or_default()
                        .push((included_group, name.span()));
                }
            }
        }
    }

    if rules.cyclic_include_group != tombi_config::SeverityLevel::Off {
        let mut visited = HashSet::new();
        let mut visiting = HashSet::new();
        let mut group_names = normalized_groups.iter().collect::<Vec<_>>();
        group_names.sort();
        for group in group_names {
            report_include_cycles(
                group,
                &includes,
                &mut visited,
                &mut visiting,
                rules.cyclic_include_group,
                diagnostics,
            );
        }
    }
}

fn report_include_cycles(
    group: &str,
    includes: &HashMap<String, Vec<(String, tombi_text::Span)>>,
    visited: &mut HashSet<String>,
    visiting: &mut HashSet<String>,
    severity: tombi_config::SeverityLevel,
    diagnostics: &mut Vec<tombi_diagnostic::Diagnostic>,
) {
    if visited.contains(group) || !visiting.insert(group.to_owned()) {
        return;
    }

    let mut stack = vec![(group.to_owned(), 0)];
    while let Some((current_group, next_include_index)) = stack.last() {
        let current_group = current_group.clone();
        let next_include_index = *next_include_index;
        let Some(included_groups) = includes.get(&current_group) else {
            let completed_group = stack.pop().unwrap().0;
            visiting.remove(&completed_group);
            visited.insert(completed_group);
            continue;
        };

        let Some((included_group, span)) = included_groups.get(next_include_index) else {
            let completed_group = stack.pop().unwrap().0;
            visiting.remove(&completed_group);
            visited.insert(completed_group);
            continue;
        };

        stack.last_mut().unwrap().1 += 1;
        if visiting.contains(included_group) {
            push_diagnostic(
                DiagnosticKind::CyclicIncludeGroup {
                    group: included_group.clone(),
                },
                severity,
                *span,
                diagnostics,
            );
        } else if !visited.contains(included_group) {
            visiting.insert(included_group.clone());
            stack.push((included_group.clone(), 0));
        }
    }
}

fn validate_workspace_members(
    document_tree: &DocumentTree<'_>,
    manifest_path: Option<&Path>,
    rules: &tombi_config::PyprojectLintRules,
    diagnostics: &mut Vec<tombi_diagnostic::Diagnostic>,
) {
    if rules.missing_workspace_member == tombi_config::SeverityLevel::Off {
        return;
    }
    let Some(workspace_dir) = manifest_path.and_then(Path::parent) else {
        return;
    };
    let Some((_, Value::Array(members))) =
        dig_keys(document_tree, &["tool", "uv", "workspace", "members"])
    else {
        return;
    };

    for member in members.iter().filter_map(|value| match value {
        Value::String(member) => Some(member),
        _ => None,
    }) {
        // uv silently ignores member directories without a pyproject.toml, so only the literal
        // paths the user typed are checked; glob patterns are resolved by uv at build time.
        if is_glob_pattern(member.value()) {
            continue;
        }
        let member_path = Path::new(member.value());
        let member_dir = if member_path.is_absolute() {
            member_path.to_path_buf()
        } else {
            workspace_dir.join(member_path)
        };
        if !tombi_fs::is_file(&member_dir.join("pyproject.toml")) {
            push_diagnostic(
                DiagnosticKind::MissingWorkspaceMember {
                    member: member.value().to_owned(),
                },
                rules.missing_workspace_member,
                member.span(),
                diagnostics,
            );
        }
    }
}

fn is_glob_pattern(value: &str) -> bool {
    value.contains(['*', '?', '['])
}

/// Returns whether a changed `pyproject.toml` is referenced by a non-glob
/// `tool.uv.workspace.members` entry, so the workspace manifest can be re-diagnosed.
pub fn references_path(
    document_tree: &DocumentTree<'_>,
    manifest_path: &Path,
    changed_path: &Path,
) -> bool {
    let Some(workspace_dir) = manifest_path.parent() else {
        return false;
    };
    let Some((_, Value::Array(members))) =
        dig_keys(document_tree, &["tool", "uv", "workspace", "members"])
    else {
        return false;
    };
    let changed_path = tombi_fs::normalize(changed_path);
    members
        .iter()
        .filter_map(|value| match value {
            Value::String(member) => Some(member.value()),
            _ => None,
        })
        .filter(|value| !is_glob_pattern(value))
        .any(|value| {
            let member_path = Path::new(value);
            let member_dir = if member_path.is_absolute() {
                member_path.to_path_buf()
            } else {
                workspace_dir.join(member_path)
            };
            tombi_fs::normalize(&member_dir.join("pyproject.toml")) == changed_path
        })
}

fn validate_dependency_array(
    value: &Value<'_>,
    rules: &tombi_config::PyprojectLintRules,
    diagnostics: &mut Vec<tombi_diagnostic::Diagnostic>,
) {
    if rules.invalid_dependency_specifier == tombi_config::SeverityLevel::Off {
        return;
    }
    let Value::Array(dependencies) = value else {
        return;
    };
    for dependency in dependencies.iter().filter_map(|value| match value {
        Value::String(dependency) => Some(dependency),
        _ => None,
    }) {
        if crate::parse_requirement(dependency.value()).is_none() {
            push_diagnostic(
                DiagnosticKind::InvalidDependencySpecifier {
                    requirement: dependency.value().to_owned(),
                },
                rules.invalid_dependency_specifier,
                dependency.span(),
                diagnostics,
            );
        }
    }
}

fn check_duplicate_import_names(
    import_names: &[ImportName<'_>],
    severity: tombi_config::SeverityLevel,
    diagnostics: &mut Vec<tombi_diagnostic::Diagnostic>,
) {
    if severity == tombi_config::SeverityLevel::Off {
        return;
    }
    let mut seen = HashSet::new();
    for entry in import_names {
        if !seen.insert(entry.name) {
            push_diagnostic(
                DiagnosticKind::DuplicateImportName,
                severity,
                entry.span,
                diagnostics,
            );
        }
    }
}

#[derive(Debug, thiserror::Error)]
enum DiagnosticKind {
    #[error("dependency specifier `{requirement}` is not valid PEP 508")]
    InvalidDependencySpecifier { requirement: String },
    #[error("included dependency group `{group}` is not defined")]
    UndefinedIncludeGroup { group: String },
    #[error("dependency group `{group}` duplicates another group after normalization")]
    DuplicateDependencyGroup { group: String },
    #[error("extra name `{name}` duplicates another name after normalization")]
    DuplicateExtraName { name: String },
    #[error("dependency group include cycle reaches `{group}`")]
    CyclicIncludeGroup { group: String },
    #[error("unknown Trove classifier `{classifier}`")]
    UnknownTroveClassifier { classifier: String },
    #[error("author or maintainer name is not a valid RFC 822 email display name")]
    InvalidEmailDisplayName,
    #[error("entry point target contains a Python keyword `{identifier}`")]
    InvalidEntryPointTarget { identifier: String },
    #[error("`{identifier}` is not a valid Python identifier")]
    InvalidImportName { identifier: String },
    #[error("import name is declared more than once")]
    DuplicateImportName,
    #[error("an import name cannot be both exclusive and shared")]
    AmbiguousImportName,
    #[error("import name parent `{parent}` is not declared")]
    MissingImportNameParent { parent: String },
    #[error("workspace member `{member}` does not contain pyproject.toml")]
    MissingWorkspaceMember { member: String },
}

impl DiagnosticKind {
    fn code(&self) -> &'static str {
        match self {
            Self::InvalidDependencySpecifier { .. } => "pyproject/invalid-dependency-specifier",
            Self::UndefinedIncludeGroup { .. } => "pyproject/undefined-include-group",
            Self::DuplicateDependencyGroup { .. } => "pyproject/duplicate-dependency-group",
            Self::DuplicateExtraName { .. } => "pyproject/duplicate-extra-name",
            Self::CyclicIncludeGroup { .. } => "pyproject/cyclic-include-group",
            Self::UnknownTroveClassifier { .. } => "pyproject/unknown-classifier",
            Self::InvalidEmailDisplayName => "pyproject/invalid-email-display-name",
            Self::InvalidEntryPointTarget { .. } => "pyproject/invalid-entry-point-target",
            Self::InvalidImportName { .. } => "pyproject/invalid-import-name",
            Self::DuplicateImportName => "pyproject/duplicate-import-name",
            Self::AmbiguousImportName => "pyproject/ambiguous-import-name",
            Self::MissingImportNameParent { .. } => "pyproject/missing-import-name-parent",
            Self::MissingWorkspaceMember { .. } => "pyproject/missing-workspace-member",
        }
    }
}

static TROVE_CLASSIFIERS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    include_str!("../data/trove-classifiers.txt")
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect()
});

fn push_diagnostic(
    kind: DiagnosticKind,
    level: tombi_config::SeverityLevel,
    span: tombi_text::Span,
    diagnostics: &mut Vec<tombi_diagnostic::Diagnostic>,
) {
    let message = kind.to_string();
    let code = kind.code();
    match level {
        tombi_config::SeverityLevel::Error => {
            diagnostics.push(tombi_diagnostic::Diagnostic::new_error(message, code, span));
        }
        tombi_config::SeverityLevel::Warn => {
            diagnostics.push(tombi_diagnostic::Diagnostic::new_warning(
                message, code, span,
            ));
        }
        tombi_config::SeverityLevel::Off => {}
    }
}

struct ImportName<'a> {
    name: &'a str,
    span: tombi_text::Span,
}

fn read_import_names<'a>(value: Option<&'a Value<'a>>) -> Vec<ImportName<'a>> {
    let Some(Value::Array(array)) = value else {
        return Vec::new();
    };

    array
        .iter()
        .filter_map(|value| {
            let Value::String(value) = value else {
                return None;
            };
            let (name, modifier) = match value.value().split_once(';') {
                Some((name, modifier)) => (name.trim_end(), Some(modifier.trim())),
                None => (value.value(), None),
            };
            if modifier.is_some_and(|modifier| modifier != "private") {
                return None;
            }
            (!name.is_empty()).then_some(ImportName {
                name,
                span: value.span(),
            })
        })
        .collect()
}

fn is_python_keyword(value: &str) -> bool {
    matches!(
        value,
        "False"
            | "None"
            | "True"
            | "and"
            | "as"
            | "assert"
            | "async"
            | "await"
            | "break"
            | "class"
            | "continue"
            | "def"
            | "del"
            | "elif"
            | "else"
            | "except"
            | "finally"
            | "for"
            | "from"
            | "global"
            | "if"
            | "import"
            | "in"
            | "is"
            | "lambda"
            | "nonlocal"
            | "not"
            | "or"
            | "pass"
            | "raise"
            | "return"
            | "try"
            | "while"
            | "with"
            | "yield"
    )
}

fn validate_entry_point_table(
    value: &Value<'_>,
    rules: &tombi_config::PyprojectLintRules,
    diagnostics: &mut Vec<tombi_diagnostic::Diagnostic>,
) {
    if rules.invalid_entry_point_target == tombi_config::SeverityLevel::Off {
        return;
    }
    let Value::Table(entries) = value else {
        return;
    };
    for target in entries.values() {
        let Value::String(target) = target else {
            continue;
        };
        let reference = target.value().split('[').next().unwrap_or_default().trim();
        let mut parts = reference.splitn(2, ':');
        let module = parts.next().unwrap_or_default().trim();
        let object = parts.next().unwrap_or_default().trim();
        for identifier in module
            .split('.')
            .chain(object.split('.'))
            .filter(|part| is_python_keyword(part) || has_invalid_unicode_identifier_syntax(part))
        {
            push_diagnostic(
                DiagnosticKind::InvalidEntryPointTarget {
                    identifier: identifier.to_owned(),
                },
                rules.invalid_entry_point_target,
                target.span(),
                diagnostics,
            );
        }
    }
}

fn has_invalid_unicode_identifier_syntax(value: &str) -> bool {
    if value.is_ascii() {
        return false;
    }

    let mut characters = value.chars();
    let Some(first) = characters.next() else {
        return false;
    };
    let schema_accepts_shape = (first == '_' || first.is_ascii_alphabetic() || !first.is_ascii())
        && characters.all(|character| {
            character == '_' || character.is_ascii_alphanumeric() || !character.is_ascii()
        });
    schema_accepts_shape && !is_python_identifier(value)
}

fn is_python_identifier(value: &str) -> bool {
    let mut characters = value.chars();
    let Some(first) = characters.next() else {
        return false;
    };
    (first == '_' || unicode_ident::is_xid_start(first))
        && characters.all(|character| character == '_' || unicode_ident::is_xid_continue(character))
}

fn is_rfc822_display_name(value: &str) -> bool {
    !value.trim().is_empty() && value.chars().all(|character| !character.is_control())
}
