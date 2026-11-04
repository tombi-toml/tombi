use std::path::{Path, PathBuf};

use tombi_config::{NagiSqlLintRules, SeverityLevel};
use tombi_document_tree_syntax::{DocumentTree, Value, dig_keys};

/// Validate Nagi SQL workspace member references.
pub fn validate_nagi_sql(
    document_tree: &DocumentTree<'_>,
    source_path: Option<&Path>,
    rules: NagiSqlLintRules,
) -> Vec<tombi_diagnostic::Diagnostic> {
    if rules.missing_member_config == SeverityLevel::Off {
        return Vec::new();
    }
    let Some(root) = source_path.and_then(crate::workspace::config_root) else {
        return Vec::new();
    };
    let Some((_, Value::Array(members))) = dig_keys(document_tree, &["workspace", "members"])
    else {
        return Vec::new();
    };

    members
        .values()
        .iter()
        .filter_map(|member| {
            let Value::String(member) = member else {
                return None;
            };
            let pattern_path = resolve_path(root, member.value());
            let matches = tombi_fs::glob(&pattern_path.to_string_lossy()).ok()?;
            let found = matches.into_iter().any(|path| resolve_config_path(&path));
            if found {
                return None;
            }

            let message = format!(
                "workspace member `{}` does not match a Nagi configuration file",
                member.value()
            );
            Some(match rules.missing_member_config {
                SeverityLevel::Error => tombi_diagnostic::Diagnostic::new_error(
                    message,
                    "nagi-sql/missing-member-config",
                    member.span(),
                ),
                SeverityLevel::Warn => tombi_diagnostic::Diagnostic::new_warning(
                    message,
                    "nagi-sql/missing-member-config",
                    member.span(),
                ),
                SeverityLevel::Off => unreachable!(),
            })
        })
        .collect()
}

fn resolve_config_path(path: &Path) -> bool {
    if matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some("nagi.toml" | ".nagi.toml")
    ) {
        return tombi_fs::is_file(path);
    }
    if !tombi_fs::is_dir(path) {
        return false;
    }
    ["nagi.toml", ".nagi.toml", ".config/nagi.toml"]
        .iter()
        .any(|name| tombi_fs::is_file(&path.join(name)))
}

fn resolve_path(root: &Path, path: &str) -> PathBuf {
    let path = Path::new(path);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}
