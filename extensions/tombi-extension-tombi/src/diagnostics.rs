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
                    kind: "schema",
                    code: "tombi/missing-schema-path",
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
                    kind: "schema catalog",
                    code: "tombi/missing-catalog-path",
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
        Some(Value::String(reference)) => vec![reference],
        Some(Value::Array(references)) => references
            .values()
            .iter()
            .filter_map(|value| match value {
                Value::String(reference) => Some(reference),
                _ => None,
            })
            .collect(),
        _ => return,
    };

    for reference in references {
        match tombi_config::resolve_local_extends(reference.value(), source_path) {
            Ok(target) => {
                report_if_missing(
                    reference,
                    target,
                    Reference {
                        kind: "extends",
                        code: "tombi/missing-extends-path",
                        severity,
                    },
                    schema_store,
                    diagnostics,
                )
                .await
            }
            Err(reason) => diagnostics.push(tombi_diagnostic::Diagnostic::new_error(
                format!("invalid extends `{}`: {reason}", reference.value()),
                "config/invalid-extends",
                reference.span(),
            )),
        }
    }
}

struct Reference {
    kind: &'static str,
    code: &'static str,
    severity: tombi_config::SeverityLevel,
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
    if reference.severity == tombi_config::SeverityLevel::Off {
        return;
    }
    let message = match target {
        ExtendsTarget::File(path) if !tombi_fs::is_file(&path) => format!(
            "{} path `{}` does not point to a file",
            reference.kind,
            value.value()
        ),
        ExtendsTarget::Remote(uri) if schema_store.remote_file_is_missing(&uri).await => {
            format!("{} URL `{}` does not exist", reference.kind, value.value())
        }
        _ => return,
    };
    push_diagnostic(
        message,
        reference.code,
        reference.severity,
        value.span(),
        diagnostics,
    );
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

fn push_diagnostic(
    message: String,
    code: &'static str,
    severity: tombi_config::SeverityLevel,
    span: tombi_text::Span,
    diagnostics: &mut Vec<tombi_diagnostic::Diagnostic>,
) {
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
