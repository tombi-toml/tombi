use std::{collections::HashSet, sync::LazyLock};

use tombi_document_tree_syntax::{DocumentTree, Value, dig_keys};

/// Validate the pyproject metadata rules that cannot be expressed by JSON Schema.
pub fn validate(
    document_tree: &DocumentTree<'_>,
    rules: &tombi_config::PyprojectLintFeatureTree,
) -> Vec<tombi_diagnostic::Diagnostic> {
    let mut diagnostics = Vec::new();
    validate_dependency_specifiers(document_tree, rules, &mut diagnostics);

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
                    classifier.span(),
                    rules,
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
                if !name.value().contains(',') && !is_rfc822_display_name(name.value()) {
                    push_diagnostic(
                        DiagnosticKind::InvalidEmailDisplayName,
                        name.span(),
                        rules,
                        &mut diagnostics,
                    );
                }
            }
        }
    }

    for field in ["scripts", "gui-scripts"] {
        if let Some(value) = project.get(field) {
            validate_entry_point_table(value, rules, &mut diagnostics);
        }
    }
    if let Some(Value::Table(groups)) = project.get("entry-points") {
        for entries in groups.values() {
            validate_entry_point_table(entries, rules, &mut diagnostics);
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
                entry.span,
                rules,
                &mut diagnostics,
            );
        }
    }

    check_duplicates(&import_names, rules, &mut diagnostics);
    check_duplicates(&import_namespaces, rules, &mut diagnostics);

    let exclusive = import_names
        .iter()
        .map(|entry| entry.name)
        .collect::<HashSet<_>>();
    for entry in &import_namespaces {
        if exclusive.contains(entry.name) {
            push_diagnostic(
                DiagnosticKind::AmbiguousImportName,
                entry.span,
                rules,
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
                    entry.span,
                    rules,
                    &mut diagnostics,
                );
            }
            parent.push('.');
            parent.push_str(part);
        }
    }

    diagnostics
}

fn validate_dependency_specifiers(
    document_tree: &DocumentTree<'_>,
    rules: &tombi_config::PyprojectLintFeatureTree,
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
}

fn validate_dependency_array(
    value: &Value<'_>,
    rules: &tombi_config::PyprojectLintFeatureTree,
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
                dependency.span(),
                rules,
                diagnostics,
            );
        }
    }
}

fn check_duplicates(
    import_names: &[ImportName<'_>],
    rules: &tombi_config::PyprojectLintFeatureTree,
    diagnostics: &mut Vec<tombi_diagnostic::Diagnostic>,
) {
    let mut seen = HashSet::new();
    for entry in import_names {
        if !seen.insert(entry.name) {
            push_diagnostic(
                DiagnosticKind::DuplicateImportName,
                entry.span,
                rules,
                diagnostics,
            );
        }
    }
}

#[derive(Debug, thiserror::Error)]
enum DiagnosticKind {
    #[error("dependency specifier `{requirement}` is not valid PEP 508")]
    InvalidDependencySpecifier { requirement: String },
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
    span: tombi_text::Span,
    rules: &tombi_config::PyprojectLintFeatureTree,
    diagnostics: &mut Vec<tombi_diagnostic::Diagnostic>,
) {
    let message = kind.to_string();
    let code = kind.code();
    match rules.severity(code) {
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
            let (name, modifier) = value
                .value()
                .split_once(';')
                .map_or((value.value(), None), |(name, modifier)| {
                    (name, Some(modifier.trim()))
                });
            if modifier.is_some_and(|modifier| modifier != "private") {
                return None;
            }
            let name = name.trim();
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
    rules: &tombi_config::PyprojectLintFeatureTree,
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
                target.span(),
                rules,
                diagnostics,
            );
        }
    }
}

fn has_invalid_unicode_identifier_syntax(value: &str) -> bool {
    if !value.chars().any(|character| !character.is_ascii()) {
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
    let bytes = value.as_bytes();
    let mut index = 0;
    let mut has_word = false;

    while index < bytes.len() {
        while bytes
            .get(index)
            .is_some_and(|byte| matches!(byte, b' ' | b'\t'))
        {
            index += 1;
        }
        if index == bytes.len() {
            break;
        }

        has_word = true;
        if bytes[index] == b'"' {
            index += 1;
            let mut closed = false;
            while index < bytes.len() {
                match bytes[index] {
                    b'"' => {
                        index += 1;
                        closed = true;
                        break;
                    }
                    b'\\' => {
                        index += 1;
                        if bytes.get(index).is_none_or(|byte| !byte.is_ascii()) {
                            return false;
                        }
                        index += 1;
                    }
                    b'\r' | b'\n' => return false,
                    byte if byte.is_ascii() => index += 1,
                    _ => return false,
                }
            }
            if !closed
                || bytes
                    .get(index)
                    .is_some_and(|byte| !matches!(byte, b' ' | b'\t'))
            {
                return false;
            }
        } else {
            let start = index;
            while let Some(byte) = bytes.get(index) {
                if matches!(byte, b' ' | b'\t') {
                    break;
                }
                if !byte.is_ascii_graphic() || b"()<>@,;:\\\".[]".contains(byte) {
                    return false;
                }
                index += 1;
            }
            if index == start {
                return false;
            }
        }
    }

    has_word
}
