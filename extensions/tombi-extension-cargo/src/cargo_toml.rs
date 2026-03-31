use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};

use tokio::sync::RwLock;
use tombi_ast::AstNode;
use tombi_config::TomlVersion;
use tombi_document_tree::{TryIntoDocumentTree, dig_keys};
use tombi_extension::file_cache_version;
use tombi_hashmap::HashMap;

const MAX_DID_OPEN_CARGO_TOML_CACHE_ENTRIES: usize = 128;

#[derive(Clone)]
struct CachedCargoTomlDocumentTree {
    version: Option<u64>,
    document_tree: tombi_document_tree::DocumentTree,
}

#[derive(Clone)]
struct CachedWorkspaceCargoTomlLookup {
    version: Option<u64>,
    workspace_cargo_toml_path: Option<PathBuf>,
}

static DID_OPEN_CARGO_TOML_CACHE: LazyLock<RwLock<HashMap<PathBuf, CachedCargoTomlDocumentTree>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));
static DID_OPEN_WORKSPACE_CARGO_TOML_LOOKUP_CACHE: LazyLock<
    RwLock<HashMap<PathBuf, CachedWorkspaceCargoTomlLookup>>,
> = LazyLock::new(|| RwLock::new(HashMap::new()));

#[derive(Debug, Clone)]
pub(crate) struct CrateLocation {
    pub(crate) cargo_toml_path: std::path::PathBuf,
    pub(crate) package_name_key_range: tombi_text::Range,
}

impl From<CrateLocation> for Option<tombi_extension::DefinitionLocation> {
    fn from(crate_location: CrateLocation) -> Self {
        let Ok(uri) = tombi_uri::Uri::from_file_path(&crate_location.cargo_toml_path) else {
            return None;
        };

        Some(tombi_extension::DefinitionLocation {
            uri,
            range: crate_location.package_name_key_range,
        })
    }
}

pub(crate) fn load_cargo_toml(
    cargo_toml_path: &Path,
    toml_version: TomlVersion,
) -> Option<(tombi_ast::Root, tombi_document_tree::DocumentTree)> {
    let toml_text = std::fs::read_to_string(cargo_toml_path).ok()?;
    let root = tombi_ast::Root::cast(tombi_parser::parse(&toml_text).into_syntax_node())?;

    Some((
        root.clone(),
        root.try_into_document_tree(toml_version).ok()?,
    ))
}

fn canonicalize_or_original_sync(path: PathBuf) -> PathBuf {
    path.canonicalize().unwrap_or(path)
}

fn insert_bounded_cargo_toml_cache(
    cache: &mut HashMap<PathBuf, CachedCargoTomlDocumentTree>,
    path: PathBuf,
    value: CachedCargoTomlDocumentTree,
) {
    if !cache.contains_key(&path)
        && cache.len() >= MAX_DID_OPEN_CARGO_TOML_CACHE_ENTRIES
        && let Some(evicted_path) = cache.keys().next().cloned()
    {
        cache.remove(&evicted_path);
    }

    cache.insert(path, value);
}

fn insert_bounded_workspace_lookup_cache(
    cache: &mut HashMap<PathBuf, CachedWorkspaceCargoTomlLookup>,
    path: PathBuf,
    value: CachedWorkspaceCargoTomlLookup,
) {
    if !cache.contains_key(&path)
        && cache.len() >= MAX_DID_OPEN_CARGO_TOML_CACHE_ENTRIES
        && let Some(evicted_path) = cache.keys().next().cloned()
    {
        cache.remove(&evicted_path);
    }

    cache.insert(path, value);
}

async fn load_cached_cargo_toml_document_tree(
    cargo_toml_path: PathBuf,
    toml_version: TomlVersion,
) -> Option<(PathBuf, tombi_document_tree::DocumentTree)> {
    let canonicalized_path = canonicalize_or_original_sync(cargo_toml_path);
    let version = file_cache_version(&canonicalized_path);

    {
        let cache = DID_OPEN_CARGO_TOML_CACHE.read().await;
        if let Some(cached_cargo_toml) = cache.get(&canonicalized_path)
            && cached_cargo_toml.version == version
        {
            return Some((canonicalized_path, cached_cargo_toml.document_tree.clone()));
        }
    }

    let parsed_document_tree = tokio::task::spawn_blocking({
        let canonicalized_path = canonicalized_path.clone();
        move || {
            load_cargo_toml(&canonicalized_path, toml_version)
                .map(|(_, document_tree)| document_tree)
        }
    })
    .await
    .ok()
    .flatten()?;

    {
        let mut cache = DID_OPEN_CARGO_TOML_CACHE.write().await;
        insert_bounded_cargo_toml_cache(
            &mut cache,
            canonicalized_path.clone(),
            CachedCargoTomlDocumentTree {
                version,
                document_tree: parsed_document_tree.clone(),
            },
        );
    }

    Some((canonicalized_path, parsed_document_tree))
}

/// Primary workspace Cargo.toml lookup for Cargo extension features.
///
/// Most callers need the parsed AST and document tree, so this stays as the
/// crate-wide API surface. Cache-aware callers should use a narrower helper.
pub(crate) fn find_workspace_cargo_toml(
    cargo_toml_path: &Path,
    workspace_path: Option<&str>,
    toml_version: TomlVersion,
) -> Option<(
    std::path::PathBuf,
    tombi_ast::Root,
    tombi_document_tree::DocumentTree,
)> {
    if let Some(workspace_path) = workspace_path {
        let workspace_cargo_toml_path = tombi_extension_manifest::resolve_manifest_path(
            cargo_toml_path,
            Path::new(workspace_path),
            "Cargo.toml",
        )?;
        let canonicalized_path = workspace_cargo_toml_path.canonicalize().ok()?;
        let (root, document_tree) = load_cargo_toml(&canonicalized_path, toml_version)?;

        return document_tree.contains_key("workspace").then_some((
            canonicalized_path,
            root,
            document_tree,
        ));
    }

    let (workspace_cargo_toml_path, (root, document_tree)) =
        tombi_extension_manifest::find_ancestor_manifest(
            cargo_toml_path,
            "Cargo.toml",
            |path| load_cargo_toml(path, toml_version),
            |(_, tree)| tree.contains_key("workspace"),
        )?;

    Some((workspace_cargo_toml_path, root, document_tree))
}

/// Specialized did-open helper that only exposes the document tree and hides
/// the global Cargo.toml cache behind a narrower API.
pub(crate) async fn load_workspace_cargo_toml_cached(
    cargo_toml_path: &Path,
    workspace_path: Option<&str>,
    toml_version: TomlVersion,
) -> Option<(PathBuf, tombi_document_tree::DocumentTree)> {
    let cache_key = canonicalize_or_original_sync(cargo_toml_path.to_path_buf());
    let cache_version = file_cache_version(&cache_key);

    {
        let cache = DID_OPEN_WORKSPACE_CARGO_TOML_LOOKUP_CACHE.read().await;
        if let Some(cached_lookup) = cache.get(&cache_key)
            && cached_lookup.version == cache_version
        {
            if let Some(workspace_cargo_toml_path) = &cached_lookup.workspace_cargo_toml_path {
                if let Some((workspace_cargo_toml_path, document_tree)) =
                    load_cached_cargo_toml_document_tree(
                        workspace_cargo_toml_path.clone(),
                        toml_version,
                    )
                    .await
                {
                    if document_tree.contains_key("workspace") {
                        return Some((workspace_cargo_toml_path, document_tree));
                    }
                }
            } else {
                return None;
            }
        }
    }

    let workspace_cargo_toml = tokio::task::spawn_blocking({
        let cargo_toml_path = cache_key.clone();
        let workspace_path = workspace_path.map(str::to_owned);
        move || {
            find_workspace_cargo_toml(&cargo_toml_path, workspace_path.as_deref(), toml_version)
                .map(|(path, _root, document_tree)| (path, document_tree))
        }
    })
    .await
    .ok()
    .flatten();

    {
        let mut cache = DID_OPEN_WORKSPACE_CARGO_TOML_LOOKUP_CACHE.write().await;
        insert_bounded_workspace_lookup_cache(
            &mut cache,
            cache_key,
            CachedWorkspaceCargoTomlLookup {
                version: cache_version,
                workspace_cargo_toml_path: workspace_cargo_toml
                    .as_ref()
                    .map(|(workspace_cargo_toml_path, _)| workspace_cargo_toml_path.clone()),
            },
        );
    }

    if let Some((workspace_cargo_toml_path, workspace_document_tree)) = workspace_cargo_toml {
        let mut cache = DID_OPEN_CARGO_TOML_CACHE.write().await;
        insert_bounded_cargo_toml_cache(
            &mut cache,
            workspace_cargo_toml_path.clone(),
            CachedCargoTomlDocumentTree {
                version: file_cache_version(&workspace_cargo_toml_path),
                document_tree: workspace_document_tree.clone(),
            },
        );

        return Some((workspace_cargo_toml_path, workspace_document_tree));
    }

    None
}

pub(crate) fn find_path_crate_cargo_toml(
    cargo_toml_path: &Path,
    crate_path: &Path,
    toml_version: TomlVersion,
) -> Option<(
    std::path::PathBuf,
    tombi_ast::Root,
    tombi_document_tree::DocumentTree,
)> {
    let crate_cargo_toml_path =
        tombi_extension_manifest::resolve_manifest_path(cargo_toml_path, crate_path, "Cargo.toml")?;
    let canonicalized_path = crate_cargo_toml_path.canonicalize().ok()?;
    let (root, document_tree) = load_cargo_toml(&canonicalized_path, toml_version)?;

    Some((canonicalized_path, root, document_tree))
}

/// Get the workspace path from Cargo.toml
///
/// See: https://doc.rust-lang.org/cargo/reference/manifest.html#the-workspace-field
#[inline]
pub(crate) fn get_workspace_path(
    document_tree: &tombi_document_tree::DocumentTree,
) -> Option<&str> {
    dig_keys(document_tree, &["package", "workspace"]).and_then(|(_, workspace)| {
        if let tombi_document_tree::Value::String(workspace_path) = workspace {
            Some(workspace_path.value())
        } else {
            None
        }
    })
}

pub(crate) fn dependency_package_name<'a>(
    dependency_key: &'a str,
    dependency_value: &'a tombi_document_tree::Value,
) -> &'a str {
    match dependency_value {
        tombi_document_tree::Value::Table(table) => match table.get("package") {
            Some(tombi_document_tree::Value::String(package)) => package.value(),
            _ => dependency_key,
        },
        _ => dependency_key,
    }
}

pub(crate) fn get_uri_relative_to_cargo_toml(
    relative_path: &Path,
    cargo_toml_path: &Path,
) -> Option<tombi_uri::Uri> {
    tombi_extension_manifest::resolve_relative_file_uri(cargo_toml_path, relative_path)
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        sync::{Mutex, OnceLock},
        time::Duration,
    };

    use super::*;

    fn test_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn test_document_tree() -> tombi_document_tree::DocumentTree {
        let root = tombi_ast::Root::cast(
            tombi_parser::parse(
                r#"
                [package]
                name = "example"
                version = "0.1.0"
                "#,
            )
            .into_syntax_node(),
        )
        .expect("expected root");
        root.try_into_document_tree(TomlVersion::default())
            .expect("expected document tree")
    }

    async fn clear_caches() {
        DID_OPEN_CARGO_TOML_CACHE.write().await.clear();
        DID_OPEN_WORKSPACE_CARGO_TOML_LOOKUP_CACHE
            .write()
            .await
            .clear();
    }

    #[tokio::test(flavor = "current_thread")]
    async fn reload_workspace_lookup_when_cached_workspace_loses_workspace_table() {
        let _guard = test_lock()
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        clear_caches().await;

        let temp_dir = tempfile::tempdir().expect("expected temp dir");
        let workspace_cargo_toml_path = temp_dir.path().join("Cargo.toml");
        let member_dir = temp_dir.path().join("member");
        fs::create_dir(&member_dir).expect("expected member dir");
        let member_cargo_toml_path = member_dir.join("Cargo.toml");

        fs::write(
            &workspace_cargo_toml_path,
            r#"
            [workspace]
            members = ["member"]
            "#,
        )
        .expect("expected workspace Cargo.toml");
        fs::write(
            &member_cargo_toml_path,
            r#"
            [package]
            name = "member"
            version = "0.1.0"
            workspace = ".."
            "#,
        )
        .expect("expected member Cargo.toml");

        let first = load_workspace_cargo_toml_cached(
            &member_cargo_toml_path,
            Some(".."),
            TomlVersion::default(),
        )
        .await;
        assert!(first.is_some());

        std::thread::sleep(Duration::from_millis(5));
        fs::write(
            &workspace_cargo_toml_path,
            r#"
            [package]
            name = "workspace-root"
            version = "0.1.0"
            "#,
        )
        .expect("expected rewritten Cargo.toml");

        let second = load_workspace_cargo_toml_cached(
            &member_cargo_toml_path,
            Some(".."),
            TomlVersion::default(),
        )
        .await;
        assert!(second.is_none());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn keeps_did_open_caches_bounded() {
        let _guard = test_lock()
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        clear_caches().await;

        {
            let mut cargo_toml_cache = DID_OPEN_CARGO_TOML_CACHE.write().await;
            for index in 0..=MAX_DID_OPEN_CARGO_TOML_CACHE_ENTRIES {
                insert_bounded_cargo_toml_cache(
                    &mut cargo_toml_cache,
                    PathBuf::from(format!("/tmp/cargo-{index}/Cargo.toml")),
                    CachedCargoTomlDocumentTree {
                        version: Some(index as u64),
                        document_tree: test_document_tree(),
                    },
                );
            }
            assert_eq!(
                cargo_toml_cache.len(),
                MAX_DID_OPEN_CARGO_TOML_CACHE_ENTRIES
            );
        }

        {
            let mut workspace_lookup_cache =
                DID_OPEN_WORKSPACE_CARGO_TOML_LOOKUP_CACHE.write().await;
            for index in 0..=MAX_DID_OPEN_CARGO_TOML_CACHE_ENTRIES {
                insert_bounded_workspace_lookup_cache(
                    &mut workspace_lookup_cache,
                    PathBuf::from(format!("/tmp/member-{index}/Cargo.toml")),
                    CachedWorkspaceCargoTomlLookup {
                        version: Some(index as u64),
                        workspace_cargo_toml_path: Some(PathBuf::from(format!(
                            "/tmp/workspace-{index}/Cargo.toml"
                        ))),
                    },
                );
            }
            assert_eq!(
                workspace_lookup_cache.len(),
                MAX_DID_OPEN_CARGO_TOML_CACHE_ENTRIES
            );
        }
    }
}
