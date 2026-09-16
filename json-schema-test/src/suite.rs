use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
};

use anyhow::{Context, Result, bail};
use flate2::read::GzDecoder;
use tar::Archive;

/// Pinned upstream commit of json-schema-org/JSON-Schema-Test-Suite.
pub const SUITE_COMMIT: &str = include_str!("SUITE_PIN");

const SUITE_ARCHIVE_URL: &str = "https://github.com/json-schema-org/JSON-Schema-Test-Suite/archive";

pub fn suite_commit() -> &'static str {
    SUITE_COMMIT.trim()
}

pub fn vendor_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("vendor")
}

pub fn suite_dir() -> PathBuf {
    vendor_dir().join("JSON-Schema-Test-Suite")
}

pub fn metaschema_cache_dir() -> PathBuf {
    vendor_dir().join("metaschema-cache")
}

pub fn ensure_suite() -> Result<PathBuf> {
    let vendor = vendor_dir();
    let suite = suite_dir();
    let pin_path = vendor.join("COMMIT");
    let commit = suite_commit();

    let suite_ready = suite.join("tests").is_dir()
        && pin_path
            .exists()
            .then(|| fs::read_to_string(&pin_path).ok())
            .flatten()
            .is_some_and(|pinned| pinned.trim() == commit);

    if !suite_ready {
        if vendor.exists() {
            fs::remove_dir_all(&vendor).with_context(|| {
                format!(
                    "failed to remove stale suite vendor at {}",
                    vendor.display()
                )
            })?;
        }
        fs::create_dir_all(&vendor)
            .with_context(|| format!("failed to create vendor dir {}", vendor.display()))?;

        eprintln!("Fetching JSON-Schema-Test-Suite @ {commit} ...");
        download_and_extract(commit, &vendor, &suite)?;
        fs::write(&pin_path, format!("{commit}\n"))
            .with_context(|| format!("failed to write pin file {}", pin_path.display()))?;
        eprintln!("Suite ready at {}", suite.display());
    }

    ensure_official_metaschemas()?;
    Ok(suite)
}

/// Official dialect metaschemas, stored in the same on-disk layout as `tombi-cache`
/// so `SchemaStore` can resolve them offline via `TOMBI_CACHE_HOME`.
const OFFICIAL_METASCHEMA_URIS: &[&str] = &[
    "http://json-schema.org/draft-07/schema",
    "https://json-schema.org/draft-07/schema",
    "https://json-schema.org/draft/2019-09/schema",
    "https://json-schema.org/draft/2019-09/meta/core",
    "https://json-schema.org/draft/2019-09/meta/applicator",
    "https://json-schema.org/draft/2019-09/meta/validation",
    "https://json-schema.org/draft/2019-09/meta/meta-data",
    "https://json-schema.org/draft/2019-09/meta/format",
    "https://json-schema.org/draft/2019-09/meta/content",
    "https://json-schema.org/draft/2020-12/schema",
    "https://json-schema.org/draft/2020-12/meta/core",
    "https://json-schema.org/draft/2020-12/meta/applicator",
    "https://json-schema.org/draft/2020-12/meta/validation",
    "https://json-schema.org/draft/2020-12/meta/meta-data",
    "https://json-schema.org/draft/2020-12/meta/format-annotation",
    "https://json-schema.org/draft/2020-12/meta/content",
    "https://json-schema.org/draft/2020-12/meta/unevaluated",
];

fn ensure_official_metaschemas() -> Result<()> {
    let cache_root = metaschema_cache_dir();
    fs::create_dir_all(&cache_root).with_context(|| {
        format!(
            "failed to create metaschema cache dir {}",
            cache_root.display()
        )
    })?;

    let mut missing = Vec::new();
    for uri in OFFICIAL_METASCHEMA_URIS {
        let path = cache_file_path_for_uri(&cache_root, uri);
        if !path.is_file() {
            missing.push((*uri, path));
        }
    }
    if missing.is_empty() {
        return Ok(());
    }

    eprintln!(
        "Fetching {} official JSON Schema metaschema(s) for offline suite runs ...",
        missing.len()
    );
    for (uri, path) in missing {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create metaschema dir {}", parent.display()))?;
        }
        let download_uri = canonical_metaschema_download_uri(uri);
        let status = Command::new("curl")
            .args(["-fsSL", "-o"])
            .arg(&path)
            .arg(download_uri.as_ref())
            .status()
            .context("failed to spawn curl; install curl to fetch metaschemas")?;
        if !status.success() {
            bail!("curl failed to download {download_uri} for {uri} (status {status})");
        }
    }
    Ok(())
}

fn canonical_metaschema_download_uri(uri: &str) -> std::borrow::Cow<'_, str> {
    if let Some(path) = uri.strip_prefix("http://json-schema.org/") {
        return std::borrow::Cow::Owned(format!("https://json-schema.org/{path}"));
    }
    std::borrow::Cow::Borrowed(uri)
}

fn cache_file_path_for_uri(cache_root: &Path, uri: &str) -> PathBuf {
    // Mirrors `tombi_cache::cache_file_path` for http(s) URIs without a `.json` suffix.
    let parsed =
        tombi_schema_store::SchemaUri::from_str(uri).expect("official metaschema URI must parse");
    let mut path = cache_root.to_path_buf();
    path.push(parsed.scheme());
    if let Some(host) = parsed.host() {
        path.push(host.to_string());
    }
    if let Some(segments) = parsed.path_segments() {
        for segment in segments {
            path.push(segment);
        }
    }
    if matches!(parsed.scheme(), "http" | "https") && !parsed.path().ends_with(".json") {
        path.push("__index__.json");
    }
    path
}
fn download_and_extract(commit: &str, vendor: &Path, suite: &Path) -> Result<()> {
    let url = format!("{SUITE_ARCHIVE_URL}/{commit}.tar.gz");
    let archive_path = vendor.join("suite.tar.gz");

    let status = Command::new("curl")
        .args(["-fsSL", "-o"])
        .arg(&archive_path)
        .arg(&url)
        .status()
        .context("failed to spawn curl; install curl to fetch the test suite")?;
    if !status.success() {
        bail!("curl failed to download {url} (status {status})");
    }

    let file = fs::File::open(&archive_path)
        .with_context(|| format!("failed to open {}", archive_path.display()))?;
    let decoder = GzDecoder::new(file);
    let mut archive = Archive::new(decoder);
    archive
        .unpack(vendor)
        .context("failed to unpack suite archive")?;

    let extracted = vendor.join(format!("JSON-Schema-Test-Suite-{commit}"));
    if !extracted.is_dir() {
        // GitHub short-sha archives still use the full name from the request.
        let found = fs::read_dir(vendor)?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .find(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.starts_with("JSON-Schema-Test-Suite-"))
                    && path.is_dir()
            });
        match found {
            Some(path) => fs::rename(&path, suite).with_context(|| {
                format!("failed to rename {} -> {}", path.display(), suite.display())
            })?,
            None => bail!(
                "extracted suite directory not found under {}",
                vendor.display()
            ),
        }
    } else {
        fs::rename(&extracted, suite).with_context(|| {
            format!(
                "failed to rename {} -> {}",
                extracted.display(),
                suite.display()
            )
        })?;
    }

    let _ = fs::remove_file(&archive_path);
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Draft {
    #[value(name = "draft7")]
    Draft7,
    #[value(name = "draft2019-09")]
    Draft2019_09,
    #[value(name = "draft2020-12")]
    Draft2020_12,
}

impl Draft {
    pub fn all() -> &'static [Draft] {
        &[Draft::Draft7, Draft::Draft2019_09, Draft::Draft2020_12]
    }

    pub fn dir_name(self) -> &'static str {
        match self {
            Draft::Draft7 => "draft7",
            Draft::Draft2019_09 => "draft2019-09",
            Draft::Draft2020_12 => "draft2020-12",
        }
    }

    pub fn meta_schema_uri(self) -> &'static str {
        match self {
            Draft::Draft7 => "http://json-schema.org/draft-07/schema#",
            Draft::Draft2019_09 => "https://json-schema.org/draft/2019-09/schema",
            Draft::Draft2020_12 => "https://json-schema.org/draft/2020-12/schema",
        }
    }
}

pub fn list_required_suite_files(suite_root: &Path, draft: Draft) -> Result<Vec<PathBuf>> {
    let dir = suite_root.join("tests").join(draft.dir_name());
    if !dir.is_dir() {
        bail!("suite draft directory missing: {}", dir.display());
    }

    let mut files = Vec::new();
    for entry in fs::read_dir(&dir).with_context(|| format!("failed to read {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        // Definition A: required tests only (skip optional/).
        if path
            .components()
            .any(|component| component.as_os_str() == "optional")
        {
            continue;
        }
        files.push(path);
    }
    files.sort();
    Ok(files)
}

pub fn remotes_dir(suite_root: &Path) -> PathBuf {
    suite_root.join("remotes")
}

/// Rewrite `http://localhost:1234/...` remote refs to `file://` URIs under `remotes/`.
pub fn rewrite_localhost_remotes(value: &mut serde_json::Value, remotes: &Path) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, child) in map.iter_mut() {
                if matches!(
                    key.as_str(),
                    "$ref" | "$id" | "$recursiveRef" | "$dynamicRef"
                ) {
                    if let serde_json::Value::String(uri) = child {
                        if let Some(rewritten) = rewrite_localhost_uri(uri, remotes) {
                            *uri = rewritten;
                        }
                    }
                } else {
                    rewrite_localhost_remotes(child, remotes);
                }
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                rewrite_localhost_remotes(item, remotes);
            }
        }
        _ => {}
    }
}

fn rewrite_localhost_uri(uri: &str, remotes: &Path) -> Option<String> {
    const BASE: &str = "http://localhost:1234";

    let (without_fragment, fragment) = match uri.split_once('#') {
        Some((path, fragment)) => (path, Some(fragment)),
        None => (uri, None),
    };
    let (without_query, query) = match without_fragment.split_once('?') {
        Some((path, query)) => (path, Some(query)),
        None => (without_fragment, None),
    };

    let relative = if without_query == BASE {
        ""
    } else {
        without_query.strip_prefix(&format!("{BASE}/"))?
    };

    let path = if relative.is_empty() {
        remotes.to_path_buf()
    } else {
        remotes.join(relative)
    };
    let mut file_uri = tombi_schema_store::SchemaUri::from_file_path(&path)
        .ok()?
        .to_string();
    if let Some(query) = query {
        file_uri.push('?');
        file_uri.push_str(query);
    }
    if let Some(fragment) = fragment {
        file_uri.push('#');
        file_uri.push_str(fragment);
    }
    Some(file_uri)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn rewrite_preserves_json_pointer_fragment() {
        let remotes = PathBuf::from("/tmp/remotes");
        let rewritten = rewrite_localhost_uri(
            "http://localhost:1234/draft7/subSchemas.json#/definitions/integer",
            &remotes,
        )
        .expect("rewrite");
        let (path_part, fragment) = rewritten
            .split_once('#')
            .expect("rewritten URI should keep a fragment");
        assert!(
            path_part.ends_with("subSchemas.json"),
            "filesystem path should be the json file: {rewritten}"
        );
        assert_eq!(fragment, "/definitions/integer");
    }
}

pub fn inject_dialect_if_missing(schema: &mut serde_json::Value, draft: Draft) {
    let serde_json::Value::Object(map) = schema else {
        return;
    };
    if !map.contains_key("$schema") {
        map.insert(
            "$schema".to_string(),
            serde_json::Value::String(draft.meta_schema_uri().to_string()),
        );
    }
}
