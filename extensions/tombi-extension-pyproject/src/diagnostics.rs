use std::{
    collections::{HashMap, HashSet},
    sync::LazyLock,
};

use tombi_document_tree_syntax::{DocumentTree, Value, dig_keys};

use crate::normalize_pep503_name;

/// Validate the pyproject metadata rules that cannot be expressed by JSON Schema.
pub fn validate(
    document_tree: &DocumentTree<'_>,
    options: &tombi_config::PyprojectLintFeatureTree,
) -> Vec<tombi_diagnostic::Diagnostic> {
    let rules = options.rules.unwrap_or_default();
    let mut diagnostics = Vec::new();
    validate_dependency_specifiers(document_tree, &rules, &mut diagnostics);
    validate_extra_names(document_tree, &rules, &mut diagnostics);

    let Some((_, Value::Table(project))) = dig_keys(document_tree, &["project"]) else {
        return diagnostics;
    };

    let import_names = read_import_names(project.get("import-names"));
    let import_namespaces = read_import_names(project.get("import-namespaces"));

    if let Some(Value::Array(classifiers)) = project.get("classifiers") {
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

    check_duplicate_import_names(&import_names, rules.duplicate_import_name, &mut diagnostics);
    check_duplicate_import_names(
        &import_namespaces,
        rules.duplicate_import_name,
        &mut diagnostics,
    );

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

    diagnostics
}

fn validate_extra_names(
    document_tree: &DocumentTree<'_>,
    rules: &tombi_config::PyprojectLintRules,
    diagnostics: &mut Vec<tombi_diagnostic::Diagnostic>,
) {
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
                rules.duplicate_extra_name,
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

    validate_include_groups(document_tree, rules, diagnostics);
}

fn validate_include_groups(
    document_tree: &DocumentTree<'_>,
    rules: &tombi_config::PyprojectLintRules,
    diagnostics: &mut Vec<tombi_diagnostic::Diagnostic>,
) {
    let Some((_, Value::Table(groups))) = dig_keys(document_tree, &["dependency-groups"]) else {
        return;
    };

    let mut normalized_groups = HashSet::new();
    for key in groups.keys() {
        let normalized_name = normalize_pep503_name(key.value());
        if !normalized_groups.insert(normalized_name) {
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
    for (key, group) in groups.key_values() {
        let Value::Array(dependencies) = group else {
            continue;
        };
        let source_group = normalize_pep503_name(key.value());
        for dependency in dependencies.values() {
            let Value::Table(include_group) = dependency else {
                continue;
            };
            let Some(Value::String(name)) = include_group.get("include-group") else {
                continue;
            };
            let included_group = normalize_pep503_name(name.value());
            if !normalized_groups.contains(&included_group) {
                push_diagnostic(
                    DiagnosticKind::UndefinedIncludeGroup {
                        group: name.value().to_owned(),
                    },
                    rules.undefined_include_group,
                    name.span(),
                    diagnostics,
                );
            } else {
                includes
                    .entry(source_group.clone())
                    .or_default()
                    .push((included_group, name.span()));
            }
        }
    }

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

    if let Some(included_groups) = includes.get(group) {
        for (included_group, span) in included_groups {
            if visiting.contains(included_group) {
                push_diagnostic(
                    DiagnosticKind::CyclicIncludeGroup {
                        group: included_group.clone(),
                    },
                    severity,
                    *span,
                    diagnostics,
                );
            } else {
                report_include_cycles(
                    included_group,
                    includes,
                    visited,
                    visiting,
                    severity,
                    diagnostics,
                );
            }
        }
    }

    visiting.remove(group);
    visited.insert(group.to_owned());
}

fn validate_dependency_array(
    value: &Value<'_>,
    rules: &tombi_config::PyprojectLintRules,
    diagnostics: &mut Vec<tombi_diagnostic::Diagnostic>,
) {
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
