use std::path::{Path, PathBuf};

use tombi_document_tree_syntax::{DocumentTree, Value};
use tombi_uri::{CatalogUri, SchemaUri};

/// Validate local schema and catalog paths configured in a Tombi config file.
pub fn validate(
    document_tree: &DocumentTree<'_>,
    source_path: Option<&Path>,
    options: &tombi_config::TombiLintFeatureTree,
) -> Vec<tombi_diagnostic::Diagnostic> {
    let rules = options.rules.unwrap_or_default();
    let mut diagnostics = Vec::new();
    if !schema_validation_enabled(document_tree) {
        return diagnostics;
    }

    if rules.missing_schema_path != tombi_config::SeverityLevel::Off {
        if let Some(Value::Array(schemas)) = document_tree.get("schemas") {
            for schema in schemas.values() {
                let Value::Table(schema) = schema else {
                    continue;
                };
                let Some(Value::String(path)) = schema.get("path") else {
                    continue;
                };
                let Some(path_to_check) = local_schema_path(path.value(), source_path) else {
                    continue;
                };
                if !tombi_fs::is_file(&path_to_check) {
                    push_diagnostic(
                        format!("schema path `{}` does not point to a file", path.value()),
                        "tombi/missing-schema-path",
                        rules.missing_schema_path,
                        path.span(),
                        &mut diagnostics,
                    );
                }
            }
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
            let Some(path_to_check) = local_catalog_path(path.value(), source_path) else {
                continue;
            };
            if !tombi_fs::is_file(&path_to_check) {
                push_diagnostic(
                    format!(
                        "schema catalog path `{}` does not point to a file",
                        path.value()
                    ),
                    "tombi/missing-catalog-path",
                    rules.missing_catalog_path,
                    path.span(),
                    &mut diagnostics,
                );
            }
        }
    }

    diagnostics
}

fn schema_validation_enabled(document_tree: &DocumentTree<'_>) -> bool {
    match tombi_document_tree_syntax::dig_keys(document_tree, &["schema", "enabled"]) {
        Some((_, Value::Boolean(enabled))) => enabled.value(),
        _ => true,
    }
}

fn local_schema_path(value: &str, source_path: Option<&Path>) -> Option<PathBuf> {
    if Path::new(value).is_absolute() {
        return Some(PathBuf::from(value));
    }
    match value.parse::<SchemaUri>() {
        Ok(uri) if uri.scheme() == "file" => uri.to_file_path().ok(),
        Ok(_) => None,
        Err(_) => relative_path(value, source_path),
    }
}

fn local_catalog_path(value: &str, source_path: Option<&Path>) -> Option<PathBuf> {
    if Path::new(value).is_absolute() {
        return Some(PathBuf::from(value));
    }
    match value.parse::<CatalogUri>() {
        Ok(uri) if uri.scheme() == "file" => uri.to_file_path().ok(),
        Ok(_) => None,
        Err(_) => relative_path(value, source_path),
    }
}

fn relative_path(value: &str, source_path: Option<&Path>) -> Option<PathBuf> {
    let path = Path::new(value);
    if path.is_absolute() {
        Some(path.to_path_buf())
    } else {
        source_path
            .and_then(Path::parent)
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
