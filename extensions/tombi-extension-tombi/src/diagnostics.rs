use std::path::{Path, PathBuf};

use tombi_config::ExtendsTarget;
use tombi_document_tree_syntax::{DocumentTree, Value};
use tombi_uri::Uri;

/// Validate `extends` and the schema and catalog paths configured in a Tombi config file.
///
/// All three share one check: a local file must exist, and a remote file must not be
/// answered with 404 or 410.
pub async fn validate(
    document_tree: &DocumentTree<'_>,
    source_path: Option<&Path>,
    options: &tombi_config::TombiLintFeatureTree,
    schema_store: &tombi_schema_store::SchemaStore,
) -> Vec<tombi_diagnostic::Diagnostic> {
    let mut diagnostics = Vec::new();
    let rules = options.rules.unwrap_or_default();
    // A malformed `extends` fails config loading whatever the lint settings are.
    if let Some(source_path) = source_path {
        validate_extends(
            document_tree,
            source_path,
            rules.missing_extends_path,
            schema_store,
            &mut diagnostics,
        )
        .await;
    }

    if rules == tombi_config::TombiLintRules::off() {
        return diagnostics;
    }

    if !schema_validation_enabled(document_tree) {
        return diagnostics;
    }

    if rules.missing_schema_path != tombi_config::SeverityLevel::Off
        && let Some(Value::Array(schemas)) = document_tree.get("schemas")
    {
        for schema in schemas.values() {
            let Value::Table(schema) = schema else {
                continue;
            };
            let Some(Value::String(path)) = schema.get("path") else {
                continue;
            };
            check_reference(
                path,
                source_path,
                Reference {
                    kind: ReferenceKind::Schema,
                    severity: rules.missing_schema_path,
                },
                schema_store,
                &mut diagnostics,
            )
            .await;
        }
    }

    if rules.missing_catalog_path != tombi_config::SeverityLevel::Off
        && let Some((_, Value::Array(paths))) =
            tombi_document_tree_syntax::dig_keys(document_tree, &["schema", "catalog", "paths"])
    {
        for path in paths.values().iter().filter_map(|value| match value {
            Value::String(path) => Some(path),
            _ => None,
        }) {
            check_reference(
                path,
                source_path,
                Reference {
                    kind: ReferenceKind::SchemaCatalog,
                    severity: rules.missing_catalog_path,
                },
                schema_store,
                &mut diagnostics,
            )
            .await;
        }
    }

    diagnostics
}

async fn validate_extends(
    document_tree: &DocumentTree<'_>,
    source_path: &Path,
    severity: tombi_config::SeverityLevel,
    schema_store: &tombi_schema_store::SchemaStore,
    diagnostics: &mut Vec<tombi_diagnostic::Diagnostic>,
) {
    let references = match document_tree.get("extends") {
        Some(Value::Array(references)) => references,
        Some(value) => {
            push_error(DiagnosticKind::ExtendsNotAnArray, value.span(), diagnostics);
            return;
        }
        None => return,
    };

    for value in references.values() {
        let Value::String(reference) = value else {
            push_error(
                DiagnosticKind::ExtendsEntryNotAString,
                value.span(),
                diagnostics,
            );
            continue;
        };
        let target = match tombi_config::resolve_local_extends(reference.value(), source_path) {
            Ok(target) => target,
            Err(reason) => {
                push_error(
                    DiagnosticKind::InvalidExtends {
                        reference: reference.value().to_owned(),
                        reason,
                    },
                    reference.span(),
                    diagnostics,
                );
                continue;
            }
        };

        // A missing target is governed by `missing-extends-path`, which can be turned off.
        if target_is_missing(&target, schema_store).await {
            report_missing(
                reference,
                Reference {
                    kind: ReferenceKind::Extends,
                    severity,
                },
                &target,
                diagnostics,
            );
        } else if let Err(error) = serde_tombi::config::check_extends_reference(
            reference.value(),
            source_path,
            schema_store.http_client(),
        )
        .await
        {
            push_error(
                DiagnosticKind::ExtendsLoadFailed {
                    reference: reference.value().to_owned(),
                    error,
                },
                reference.span(),
                diagnostics,
            );
        }
    }
}

struct Reference {
    kind: ReferenceKind,
    severity: tombi_config::SeverityLevel,
}

#[derive(Debug, Clone, Copy)]
enum ReferenceKind {
    Extends,
    Schema,
    SchemaCatalog,
}

impl std::fmt::Display for ReferenceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Extends => "extends",
            Self::Schema => "schema",
            Self::SchemaCatalog => "schema catalog",
        })
    }
}

/// Every diagnostic that the Tombi extension reports.
#[derive(thiserror::Error, Debug)]
enum DiagnosticKind {
    #[error("{}", tombi_config::EXTENDS_NOT_AN_ARRAY)]
    ExtendsNotAnArray,
    #[error("{}", tombi_config::EXTENDS_ENTRY_NOT_A_STRING)]
    ExtendsEntryNotAString,
    #[error("invalid extends `{reference}`: {reason}")]
    InvalidExtends { reference: String, reason: String },
    #[error("invalid extends `{reference}`: {error}")]
    ExtendsLoadFailed {
        reference: String,
        error: tombi_config::Error,
    },
    #[error("{kind} path `{path}` does not exist")]
    MissingPath { kind: ReferenceKind, path: String },
    #[error("{kind} URL `{url}` does not exist")]
    MissingUrl { kind: ReferenceKind, url: String },
}

impl DiagnosticKind {
    fn code(&self) -> &'static str {
        match self {
            Self::ExtendsNotAnArray
            | Self::ExtendsEntryNotAString
            | Self::InvalidExtends { .. }
            | Self::ExtendsLoadFailed { .. } => "config/invalid-extends",
            Self::MissingPath { kind, .. } | Self::MissingUrl { kind, .. } => match kind {
                ReferenceKind::Extends => "tombi/missing-extends-path",
                ReferenceKind::Schema => "tombi/missing-schema-path",
                ReferenceKind::SchemaCatalog => "tombi/missing-catalog-path",
            },
        }
    }
}

async fn check_reference(
    value: &tombi_document_tree_syntax::String<'_>,
    source_path: Option<&Path>,
    reference: Reference,
    schema_store: &tombi_schema_store::SchemaStore,
    diagnostics: &mut Vec<tombi_diagnostic::Diagnostic>,
) {
    if let Some(target) = reference_target(value.value(), source_path) {
        report_if_missing(value, target, reference, schema_store, diagnostics).await;
    }
}

async fn report_if_missing(
    value: &tombi_document_tree_syntax::String<'_>,
    target: ExtendsTarget,
    reference: Reference,
    schema_store: &tombi_schema_store::SchemaStore,
    diagnostics: &mut Vec<tombi_diagnostic::Diagnostic>,
) {
    if reference.severity != tombi_config::SeverityLevel::Off
        && target_is_missing(&target, schema_store).await
    {
        report_missing(value, reference, &target, diagnostics);
    }
}

async fn target_is_missing(
    target: &ExtendsTarget,
    schema_store: &tombi_schema_store::SchemaStore,
) -> bool {
    match target {
        ExtendsTarget::File(path) => !tombi_fs::is_file(path),
        ExtendsTarget::Remote(uri) => schema_store.remote_file_is_missing(uri).await,
    }
}

fn report_missing(
    value: &tombi_document_tree_syntax::String<'_>,
    reference: Reference,
    target: &ExtendsTarget,
    diagnostics: &mut Vec<tombi_diagnostic::Diagnostic>,
) {
    if reference.severity == tombi_config::SeverityLevel::Off {
        return;
    }
    let value_text = value.value().to_owned();
    let kind = match target {
        ExtendsTarget::File(_) => DiagnosticKind::MissingPath {
            kind: reference.kind,
            path: value_text,
        },
        ExtendsTarget::Remote(_) => DiagnosticKind::MissingUrl {
            kind: reference.kind,
            url: value_text,
        },
    };
    push_diagnostic(kind, reference.severity, value.span(), diagnostics);
}

fn schema_validation_enabled(document_tree: &DocumentTree<'_>) -> bool {
    match tombi_document_tree_syntax::dig_keys(document_tree, &["schema", "enabled"]) {
        Some((_, Value::Boolean(enabled))) => enabled.value(),
        _ => true,
    }
}

/// Where a schema or catalog reference points to.
///
/// Anything other than a local path, `file://` or `http(s)://` is left unchecked.
fn reference_target(value: &str, source_path: Option<&Path>) -> Option<ExtendsTarget> {
    if Path::new(value).is_absolute() {
        return Some(ExtendsTarget::File(PathBuf::from(value)));
    }
    match value.parse::<Uri>() {
        Ok(uri) if uri.scheme() == "file" => uri.to_file_path().ok().map(ExtendsTarget::File),
        Ok(uri) => tombi_config::validate_remote_uri(uri)
            .ok()
            .map(ExtendsTarget::Remote),
        Err(_) => relative_path(value, source_path).map(ExtendsTarget::File),
    }
}

fn relative_path(value: &str, source_path: Option<&Path>) -> Option<PathBuf> {
    let path = Path::new(value);
    if path.is_absolute() {
        Some(path.to_path_buf())
    } else {
        source_path
            .and_then(tombi_config::config_base_dir)
            .map(|parent| parent.join(path))
    }
}

fn push_error(
    kind: DiagnosticKind,
    span: tombi_text::Span,
    diagnostics: &mut Vec<tombi_diagnostic::Diagnostic>,
) {
    push_diagnostic(kind, tombi_config::SeverityLevel::Error, span, diagnostics);
}

fn push_diagnostic(
    kind: DiagnosticKind,
    severity: tombi_config::SeverityLevel,
    span: tombi_text::Span,
    diagnostics: &mut Vec<tombi_diagnostic::Diagnostic>,
) {
    let message = kind.to_string();
    let code = kind.code();
    match severity {
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
