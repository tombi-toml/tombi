use std::path::Path;

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
    let mut changed_paths = vec![changed_path.clone()];
    if let Some(parent) = changed_path.parent() {
        changed_paths.push(parent.to_path_buf());
        if changed_path.ends_with(Path::new(".config/nagi.toml"))
            && let Some(workspace_member_root) = parent.parent()
        {
            changed_paths.push(workspace_member_root.to_path_buf());
        }
    }
    let Some((_, Value::Array(members))) = dig_keys(document_tree, &["workspace", "members"])
    else {
        return false;
    };

    members.values().iter().any(|member| {
        let Value::String(member) = member else {
            return false;
        };
        let member_path = Path::new(member.value());
        if !is_glob_pattern(member.value()) {
            let member_path = if member_path.is_absolute() {
                member_path.to_path_buf()
            } else {
                root.join(member_path)
            };
            return changed_paths.iter().any(|changed_path| {
                tombi_fs::normalize(changed_path) == tombi_fs::normalize(&member_path)
            });
        }

        let pattern = member_glob_pattern(root, member.value());
        let Some(glob_pattern) = glob::Pattern::new(&pattern).ok() else {
            return false;
        };
        if changed_paths
            .iter()
            .any(|changed_path| glob_pattern.matches_path(changed_path))
        {
            return true;
        }

        tombi_fs::glob(&pattern).is_ok_and(|matches| {
            matches.into_iter().any(|matched_path| {
                let matched_path = tombi_fs::normalize(&matched_path);
                changed_paths
                    .iter()
                    .any(|changed_path| tombi_fs::normalize(changed_path) == matched_path)
            })
        })
    })
}

fn is_glob_pattern(value: &str) -> bool {
    value.contains(['*', '?', '['])
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

#[cfg(test)]
mod tests {
    use tombi_ast_syntax::AstNode as _;
    use tombi_config::TomlVersion;
    use tombi_document_tree_syntax::TryIntoDocumentTree;

    use super::*;

    fn with_document_tree(source: &str, f: impl FnOnce(&DocumentTree<'_>)) {
        let parsed = tombi_parser::parse(source);
        let root = parsed.root();
        let toml_version = TomlVersion::default();
        let decoded = root.decode_strings(toml_version);
        let document_tree = root.try_into_document_tree(toml_version, &decoded).unwrap();
        f(&document_tree);
    }

    #[test]
    fn references_parent_relative_member_after_file_changes() {
        with_document_tree(
            "[workspace]\nmembers = [\"../shared\"]\n",
            |document_tree| {
                assert!(references_path(
                    document_tree,
                    Path::new("/project/workspace/nagi.toml"),
                    Path::new("/project/shared/nagi.toml"),
                ));
            },
        );
    }

    #[test]
    fn references_config_directory_member_after_file_changes() {
        with_document_tree(
            "[workspace]\nmembers = [\"members/app/.config\"]\n",
            |document_tree| {
                assert!(references_path(
                    document_tree,
                    Path::new("/project/workspace/nagi.toml"),
                    Path::new("/project/workspace/members/app/.config/nagi.toml"),
                ));
            },
        );
    }
}
