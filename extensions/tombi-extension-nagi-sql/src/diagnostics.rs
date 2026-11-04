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
            let pattern_path = member_glob_pattern(root, member.value());
            let matches = tombi_fs::glob(&pattern_path).ok()?;
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

pub fn references_path(
    document_tree: &DocumentTree<'_>,
    config_path: &Path,
    changed_path: &Path,
) -> bool {
    let Some(root) = crate::workspace::config_root(config_path) else {
        return false;
    };
    let changed_path = tombi_fs::normalize(changed_path);
    let Some((_, Value::Array(members))) = dig_keys(document_tree, &["workspace", "members"])
    else {
        return false;
    };

    members.values().iter().any(|member| {
        let Value::String(member) = member else {
            return false;
        };
        let pattern = member_glob_pattern(root, member.value());
        glob::Pattern::new(&pattern).ok().is_some_and(|pattern| {
            let config_root = member_config_root(&changed_path);
            pattern.matches_path(&changed_path) || pattern.matches_path(&config_root)
        })
    })
}

fn member_config_root(path: &Path) -> PathBuf {
    if path.ends_with(Path::new(".config/nagi.toml")) {
        return path
            .parent()
            .and_then(Path::parent)
            .unwrap_or(path)
            .to_path_buf();
    }
    if matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some("nagi.toml" | ".nagi.toml")
    ) {
        return path.parent().unwrap_or(path).to_path_buf();
    }
    path.to_path_buf()
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

fn member_glob_pattern(root: &Path, value: &str) -> String {
    if Path::new(value).is_absolute() {
        return value.to_owned();
    }
    let mut pattern = glob::Pattern::escape(&root.to_string_lossy());
    pattern.push(std::path::MAIN_SEPARATOR);
    pattern.push_str(value);
    pattern
}
