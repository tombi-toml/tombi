use std::{path::Path, str::FromStr};

use itertools::Itertools;
use rattler_conda_types::{Channel, ChannelConfig, Subdir};
use tombi_document_tree_syntax::{DocumentTree, Value, dig_keys};

use crate::dependency::CondaDependency;

const DEFAULT_CHANNEL: &str = "conda-forge";

/// The channels and subdirs to look a conda dependency up in.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct CondaQueryScope {
    pub channels: Vec<Channel>,
    pub subdirs: Vec<Subdir>,
}

pub(crate) fn query_scope(
    document_tree: &DocumentTree<'_>,
    dependency: &CondaDependency<'_>,
    inline_channel: Option<&str>,
    manifest_dir: &Path,
) -> CondaQueryScope {
    let channel_config = ChannelConfig::default_with_root_dir(manifest_dir.to_path_buf());

    let channel_names = match inline_channel {
        Some(channel) => vec![channel],
        None => {
            let mut names = dependency
                .feature
                .map(|feature| string_values(document_tree, &["feature", feature, "channels"]))
                .unwrap_or_default();
            names.extend(workspace_values(document_tree, "channels"));
            if names.is_empty() {
                names.push(DEFAULT_CHANNEL);
            }
            names
        }
    };

    let channels = channel_names
        .into_iter()
        .unique()
        .filter_map(|name| match Channel::from_str(name, &channel_config) {
            Ok(channel) => Some(channel),
            Err(err) => {
                log::debug!("failed to parse pixi channel {name:?}: {err}");
                None
            }
        })
        // Only remote channels can serve sharded repodata; local channels would load
        // the whole `repodata.json` into memory.
        .filter(|channel| matches!(channel.base_url.url().scheme(), "http" | "https"))
        .collect();

    CondaQueryScope {
        channels,
        subdirs: subdirs(document_tree, dependency),
    }
}

fn subdirs(document_tree: &DocumentTree<'_>, dependency: &CondaDependency<'_>) -> Vec<Subdir> {
    let platforms = match dependency
        .target
        .and_then(|target| Subdir::from_str(target).ok())
    {
        Some(subdir) => vec![subdir],
        None => {
            let mut platforms = dependency
                .feature
                .map(|feature| string_values(document_tree, &["feature", feature, "platforms"]))
                .unwrap_or_default();
            if platforms.is_empty() {
                platforms = workspace_values(document_tree, "platforms");
            }
            platforms
                .into_iter()
                .filter_map(|platform| Subdir::from_str(platform).ok())
                .collect_vec()
        }
    };

    let platforms = if platforms.is_empty() {
        Subdir::current().into_iter().collect_vec()
    } else {
        platforms
    };

    platforms
        .into_iter()
        .chain(std::iter::once(Subdir::NoArch))
        .unique()
        .collect()
}

/// Reads `workspace.<key>`, falling back to the legacy `project.<key>`.
fn workspace_values<'a>(document_tree: &'a DocumentTree<'a>, key: &str) -> Vec<&'a str> {
    let values = string_values(document_tree, &["workspace", key]);
    if values.is_empty() {
        string_values(document_tree, &["project", key])
    } else {
        values
    }
}

/// Reads an array of strings or `{ channel = "..." }` tables.
fn string_values<'a>(document_tree: &'a DocumentTree<'a>, keys: &[&str]) -> Vec<&'a str> {
    let Some((_, Value::Array(array))) = dig_keys(document_tree, keys) else {
        return Vec::new();
    };

    array
        .values()
        .iter()
        .filter_map(|value| match value {
            Value::String(string) => Some(string.value()),
            Value::Table(table) => match table.get("channel") {
                Some(Value::String(string)) => Some(string.value()),
                _ => None,
            },
            _ => None,
        })
        .collect()
}
