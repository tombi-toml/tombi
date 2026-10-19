mod error;
mod options;
pub use error::Error;
pub use options::{DEFAULT_CACHE_TTL_DURATION, DEFAULT_CACHE_TTL_SECONDS, Options};
pub const CACHE_INDEX_FILE_NAME: &str = "__index__.json";

#[cfg(not(target_arch = "wasm32"))]
pub async fn get_tombi_cache_dir_path() -> Option<std::path::PathBuf> {
    ensure_cache_dir(tombi_cache_dir_path()?).await
}

#[cfg(target_arch = "wasm32")]
pub async fn get_tombi_cache_dir_path() -> Option<std::path::PathBuf> {
    None
}

#[cfg(not(target_arch = "wasm32"))]
fn tombi_cache_dir_path() -> Option<std::path::PathBuf> {
    if let Some(tombi_cache_home) = std::env::var_os("TOMBI_CACHE_HOME") {
        return Some(std::path::PathBuf::from(tombi_cache_home));
    }

    if let Some(xdg_cache_home) = std::env::var_os("XDG_CACHE_HOME") {
        let mut cache_dir_path = std::path::PathBuf::from(xdg_cache_home);
        cache_dir_path.push("tombi");

        return Some(cache_dir_path);
    }

    if let Some(home_dir) = dirs::home_dir() {
        let mut cache_dir_path = home_dir;
        cache_dir_path.push(".cache");
        cache_dir_path.push("tombi");
        return Some(cache_dir_path);
    }

    None
}

/// Resolve the configured cache location without creating it.
#[cfg(not(target_arch = "wasm32"))]
pub fn try_get_tombi_cache_dir_path() -> Result<std::path::PathBuf, crate::Error> {
    let path = tombi_cache_dir_path().ok_or(crate::Error::CacheDirectoryUnavailable)?;
    if path.as_os_str().is_empty() {
        return Err(crate::Error::UnsafeCacheDirectory {
            cache_dir_path: path,
        });
    }
    std::path::absolute(&path).map_err(|error| crate::Error::CacheDirectoryResolveFailed {
        cache_dir_path: path,
        reason: error.to_string(),
    })
}

/// Remove every entry within the cache directory while preserving the directory.
#[cfg(not(target_arch = "wasm32"))]
pub async fn clean_cache() -> Result<(), crate::Error> {
    // Remove a trailing separator before checking the final path component for a symlink.
    let path: std::path::PathBuf = try_get_tombi_cache_dir_path()?.components().collect();
    let metadata = match tokio::fs::symlink_metadata(&path).await {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(crate::Error::CacheDirectoryRemoveFailed {
                cache_dir_path: path,
                reason: error.to_string(),
            });
        }
    };
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(crate::Error::UnsafeCacheDirectory {
            cache_dir_path: path,
        });
    }

    let canonical = tokio::fs::canonicalize(&path).await.map_err(|error| {
        crate::Error::CacheDirectoryResolveFailed {
            cache_dir_path: path.clone(),
            reason: error.to_string(),
        }
    })?;
    let current_dir = std::env::current_dir()
        .and_then(std::fs::canonicalize)
        .map_err(|error| crate::Error::CacheDirectoryResolveFailed {
            cache_dir_path: path.clone(),
            reason: error.to_string(),
        })?;
    let home_path = dirs::home_dir();
    let home_dir = home_path
        .as_ref()
        .map(|home| {
            canonicalize_allow_missing(home).map_err(|error| {
                crate::Error::CacheDirectoryResolveFailed {
                    cache_dir_path: home.to_path_buf(),
                    reason: error.to_string(),
                }
            })
        })
        .transpose()?;
    let default_cache_dir = home_path
        .map(|home| {
            let path = home.join(".cache");
            canonicalize_allow_missing(&path).map_err(|error| {
                crate::Error::CacheDirectoryResolveFailed {
                    cache_dir_path: path,
                    reason: error.to_string(),
                }
            })
        })
        .transpose()?;
    let xdg_dir = std::env::var_os("XDG_CACHE_HOME")
        .filter(|value| !value.is_empty())
        .map(|value| {
            canonicalize_allow_missing(std::path::Path::new(&value)).map_err(|error| {
                crate::Error::CacheDirectoryResolveFailed {
                    cache_dir_path: std::path::PathBuf::from(value),
                    reason: error.to_string(),
                }
            })
        })
        .transpose()?;
    if is_unsafe_clean_target(
        &canonical,
        &current_dir,
        home_dir.as_deref(),
        default_cache_dir.as_deref(),
        xdg_dir.as_deref(),
    ) {
        return Err(crate::Error::UnsafeCacheDirectory {
            cache_dir_path: path,
        });
    }

    let mut entries = tokio::fs::read_dir(&path).await.map_err(|error| {
        crate::Error::CacheDirectoryRemoveFailed {
            cache_dir_path: path.clone(),
            reason: error.to_string(),
        }
    })?;
    while let Some(entry) =
        entries
            .next_entry()
            .await
            .map_err(|error| crate::Error::CacheDirectoryRemoveFailed {
                cache_dir_path: path.clone(),
                reason: error.to_string(),
            })?
    {
        let entry_path = entry.path();
        let file_type =
            entry
                .file_type()
                .await
                .map_err(|error| crate::Error::CacheDirectoryRemoveFailed {
                    cache_dir_path: entry_path.clone(),
                    reason: error.to_string(),
                })?;
        let result = if file_type.is_dir() {
            tokio::fs::remove_dir_all(&entry_path).await
        } else {
            tokio::fs::remove_file(&entry_path).await
        };
        result.map_err(|error| crate::Error::CacheDirectoryRemoveFailed {
            cache_dir_path: entry_path,
            reason: error.to_string(),
        })?;
    }
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn is_unsafe_clean_target(
    canonical: &std::path::Path,
    current_dir: &std::path::Path,
    home_dir: Option<&std::path::Path>,
    default_cache_dir: Option<&std::path::Path>,
    xdg_dir: Option<&std::path::Path>,
) -> bool {
    canonical.parent().is_none()
        || current_dir.starts_with(canonical)
        || home_dir.is_some_and(|home| home.starts_with(canonical))
        || default_cache_dir.is_some_and(|cache| cache.starts_with(canonical))
        || xdg_dir.is_some_and(|xdg| xdg.starts_with(canonical))
}

#[cfg(not(target_arch = "wasm32"))]
fn canonicalize_allow_missing(path: &std::path::Path) -> std::io::Result<std::path::PathBuf> {
    let mut ancestor = std::path::absolute(path)?;
    let mut missing = Vec::new();
    loop {
        match std::fs::canonicalize(&ancestor) {
            Ok(mut resolved) => {
                for component in missing.into_iter().rev() {
                    resolved.push(component);
                }
                return Ok(resolved);
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                match std::fs::symlink_metadata(&ancestor) {
                    Ok(_) => return Err(error),
                    Err(metadata_error)
                        if metadata_error.kind() == std::io::ErrorKind::NotFound => {}
                    Err(metadata_error) => return Err(metadata_error),
                }
                missing.push(ancestor.file_name().ok_or(error)?.to_os_string());
                ancestor.pop();
            }
            Err(error) => return Err(error),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn get_cache_file_path(cache_file_uri: &tombi_uri::Uri) -> Option<std::path::PathBuf> {
    Some(cache_file_path(
        get_tombi_cache_dir_path().await?,
        cache_file_uri,
    ))
}

#[cfg(target_arch = "wasm32")]
pub async fn get_cache_file_path(_cache_file_uri: &tombi_uri::Uri) -> Option<std::path::PathBuf> {
    None
}

#[cfg(not(target_arch = "wasm32"))]
fn cache_file_path(
    mut cache_dir_path: std::path::PathBuf,
    cache_file_uri: &tombi_uri::Uri,
) -> std::path::PathBuf {
    cache_dir_path.push(cache_file_uri.scheme());
    if let Some(host) = cache_file_uri.host() {
        cache_dir_path.push(host.to_string());
    }
    if let Some(path_segments) = cache_file_uri.path_segments() {
        for segment in path_segments {
            cache_dir_path.push(segment);
        }
    }
    if matches!(cache_file_uri.scheme(), "http" | "https")
        && !cache_file_uri.path().ends_with(".json")
    {
        cache_dir_path.push(CACHE_INDEX_FILE_NAME);
    }

    cache_dir_path
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_existing_cache_file_path(cache_file_uri: &tombi_uri::Uri) -> Option<std::path::PathBuf> {
    let cache_file_path = cache_file_path(tombi_cache_dir_path()?, cache_file_uri);
    cache_file_path.is_file().then_some(cache_file_path)
}

#[cfg(target_arch = "wasm32")]
pub fn get_existing_cache_file_path(
    _cache_file_uri: &tombi_uri::Uri,
) -> Option<std::path::PathBuf> {
    None
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn read_from_cache(
    cache_file_path: Option<&std::path::Path>,
    options: Option<&Options>,
) -> Result<Option<String>, crate::Error> {
    if options
        .and_then(|options| options.no_cache)
        .unwrap_or_default()
    {
        return Ok(None);
    }

    if let Some(cache_file_path) = cache_file_path
        && cache_file_path.is_file()
    {
        let cache_ttl = options
            .map(|opts| opts.cache_ttl)
            .unwrap_or_else(|| Options::default().cache_ttl);
        if let Some(ttl) = cache_ttl {
            let Ok(metadata) = tokio::fs::metadata(cache_file_path).await else {
                return Ok(None);
            };
            if let Ok(modified) = metadata.modified()
                && let Ok(elapsed) = modified.elapsed()
                && elapsed > ttl
            {
                return Ok(None);
            }
        }
        return Ok(Some(
            tokio::fs::read_to_string(&cache_file_path)
                .await
                .map_err(|err| crate::Error::CacheFileReadFailed {
                    cache_file_path: cache_file_path.to_path_buf(),
                    reason: err.to_string(),
                })?,
        ));
    }

    Ok(None)
}

#[cfg(target_arch = "wasm32")]
pub async fn read_from_cache(
    _cache_file_path: Option<&std::path::Path>,
    _options: Option<&Options>,
) -> Result<Option<String>, crate::Error> {
    Ok(None)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn save_to_cache(
    cache_file_path: Option<&std::path::Path>,
    bytes: &[u8],
) -> Result<(), crate::Error> {
    if let Some(cache_file_path) = cache_file_path {
        if !cache_file_path.is_file() {
            let Some(cache_dir_path) = cache_file_path.parent() else {
                return Err(crate::Error::CacheFileParentDirectoryNotFound {
                    cache_file_path: cache_file_path.to_owned(),
                });
            };

            if let Err(err) = tokio::fs::create_dir_all(cache_dir_path).await {
                return Err(crate::Error::CacheFileSaveFailed {
                    cache_file_path: cache_file_path.to_owned(),
                    reason: err.to_string(),
                });
            }
        }
        use tokio::io::AsyncWriteExt;

        let mut options = tokio::fs::OpenOptions::new();
        options.create(true).write(true).truncate(true);
        #[cfg(unix)]
        options.mode(0o600);
        let write_result = async {
            let mut file = options.open(cache_file_path).await?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                file.set_permissions(std::fs::Permissions::from_mode(0o600))
                    .await?;
            }
            file.write_all(bytes).await?;
            file.flush().await
        }
        .await;
        if let Err(err) = write_result {
            return Err(crate::Error::CacheFileSaveFailed {
                cache_file_path: cache_file_path.to_owned(),
                reason: err.to_string(),
            });
        }
    }

    Ok(())
}

#[cfg(target_arch = "wasm32")]
pub async fn save_to_cache(
    _cache_file_path: Option<&std::path::Path>,
    _bytes: &[u8],
) -> Result<(), crate::Error> {
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn refresh_cache() -> Result<bool, crate::Error> {
    if let Some(cache_dir_path) = get_tombi_cache_dir_path().await {
        // Remove all contents of the cache directory but keep the directory itself
        if let Ok(mut entries) = tokio::fs::read_dir(&cache_dir_path).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                if let Ok(file_type) = entry.file_type().await
                    && file_type.is_dir()
                {
                    let path = entry.path();
                    if let Err(err) = tokio::fs::remove_dir_all(&path).await {
                        return Err(crate::Error::CacheDirectoryRemoveFailed {
                            cache_dir_path: path,
                            reason: err.to_string(),
                        });
                    }
                }
            }
        }
        return Ok(true);
    }

    Ok(false)
}

#[cfg(target_arch = "wasm32")]
pub async fn refresh_cache() -> Result<bool, crate::Error> {
    Ok(false)
}

#[cfg(not(target_arch = "wasm32"))]
async fn ensure_cache_dir(cache_dir_path: std::path::PathBuf) -> Option<std::path::PathBuf> {
    if let Err(error) = tokio::fs::create_dir_all(&cache_dir_path).await {
        log::warn!("failed to create cache directory: {error}");
        return None;
    }

    Some(cache_dir_path)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use tombi_test_lib::TestCacheHome;

    #[tokio::test(flavor = "current_thread")]
    async fn missing_cache_does_not_create_directory() {
        let cache_home = TestCacheHome::new();
        let path = try_get_tombi_cache_dir_path().unwrap();
        assert_eq!(path, cache_home.tombi_cache_dir_path());
        assert!(!path.exists());
        clean_cache().await.unwrap();
        assert!(!path.exists());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn removes_files_and_directories_but_keeps_root() {
        let cache_home = TestCacheHome::new();
        let path = cache_home.tombi_cache_dir_path();
        std::fs::create_dir_all(path.join("nested")).unwrap();
        std::fs::write(path.join("catalog.json"), "catalog").unwrap();
        std::fs::write(path.join("nested/schema.json"), "schema").unwrap();

        clean_cache().await.unwrap();

        assert!(path.is_dir());
        assert_eq!(std::fs::read_dir(&path).unwrap().count(), 0);
    }

    #[cfg(unix)]
    #[tokio::test(flavor = "current_thread")]
    async fn removes_child_symlink_without_following_it() {
        use std::os::unix::fs::symlink;

        let cache_home = TestCacheHome::new();
        let path = cache_home.tombi_cache_dir_path();
        let outside = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(&path).unwrap();
        std::fs::write(outside.path().join("keep.json"), "keep").unwrap();
        symlink(outside.path(), path.join("link")).unwrap();

        clean_cache().await.unwrap();

        assert!(!path.join("link").exists());
        assert_eq!(
            std::fs::read_to_string(outside.path().join("keep.json")).unwrap(),
            "keep"
        );
    }

    #[cfg(unix)]
    #[tokio::test(flavor = "current_thread")]
    async fn rejects_filesystem_root() {
        let _cache_home = TestCacheHome::with_tombi_cache_home(Some(std::path::Path::new("/")));
        assert!(matches!(
            clean_cache().await,
            Err(Error::UnsafeCacheDirectory { .. })
        ));
    }

    #[cfg(unix)]
    #[test]
    fn rejects_protected_roots() {
        use std::path::Path;

        let root = Path::new("/");
        let cwd = Path::new("/work/project");
        let home = Path::new("/users/me");
        let default_cache_parent = Path::new("/users/me/.cache");
        let xdg = Path::new("/tmp/xdg");
        for target in [
            root,
            Path::new("/work"),
            cwd,
            Path::new("/users"),
            home,
            default_cache_parent,
            Path::new("/tmp"),
            xdg,
        ] {
            assert!(is_unsafe_clean_target(
                target,
                cwd,
                Some(home),
                Some(default_cache_parent),
                Some(xdg),
            ));
        }
        assert!(!is_unsafe_clean_target(
            Path::new("/tmp/xdg/tombi"),
            cwd,
            Some(home),
            Some(default_cache_parent),
            Some(xdg),
        ));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn cleans_custom_cache_without_touching_sibling() {
        let temp = tempfile::tempdir().unwrap();
        let custom = temp.path().join("custom");
        std::fs::create_dir(&custom).unwrap();
        let cached = custom.join("cached.txt");
        let sibling = temp.path().join("keep.txt");
        std::fs::write(&cached, "cache").unwrap();
        std::fs::write(&sibling, "keep").unwrap();
        let _cache_home = TestCacheHome::with_tombi_cache_home(Some(&custom));

        clean_cache().await.unwrap();
        assert!(custom.is_dir());
        assert!(!cached.exists());
        assert_eq!(std::fs::read_to_string(&sibling).unwrap(), "keep");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn rejects_cache_root_containing_uncreated_xdg_home() {
        let temp = tempfile::tempdir().unwrap();
        let custom = temp.path().join("custom");
        std::fs::create_dir(&custom).unwrap();
        let sentinel = custom.join("keep.txt");
        std::fs::write(&sentinel, "keep").unwrap();
        let _cache_home = TestCacheHome::with_tombi_cache_home(Some(&custom));
        unsafe { std::env::set_var("XDG_CACHE_HOME", custom.join("uncreated")) };

        assert!(matches!(
            clean_cache().await,
            Err(Error::UnsafeCacheDirectory { .. })
        ));
        assert_eq!(std::fs::read_to_string(&sentinel).unwrap(), "keep");
    }

    #[cfg(unix)]
    #[tokio::test(flavor = "current_thread")]
    async fn rejects_unresolved_xdg_symlink() {
        use std::os::unix::fs::symlink;

        let temp = tempfile::tempdir().unwrap();
        let custom = temp.path().join("custom");
        std::fs::create_dir(&custom).unwrap();
        let sentinel = custom.join("keep.txt");
        std::fs::write(&sentinel, "keep").unwrap();
        let _cache_home = TestCacheHome::with_tombi_cache_home(Some(&custom));
        let link = temp.path().join("xdg-link");
        symlink(custom.join("uncreated"), &link).unwrap();
        unsafe { std::env::set_var("XDG_CACHE_HOME", &link) };

        assert!(matches!(
            clean_cache().await,
            Err(Error::CacheDirectoryResolveFailed { .. })
        ));
        assert_eq!(std::fs::read_to_string(&sentinel).unwrap(), "keep");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn cleans_explicit_conventional_cache_home() {
        let cache_home = TestCacheHome::new();
        let conventional = cache_home.tombi_cache_dir_path();
        std::fs::create_dir(&conventional).unwrap();
        let sentinel = conventional.join("cached.json");
        std::fs::write(&sentinel, "cache").unwrap();
        unsafe { std::env::set_var("TOMBI_CACHE_HOME", &conventional) };

        clean_cache().await.unwrap();
        assert!(!sentinel.exists());
    }

    #[cfg(unix)]
    #[tokio::test(flavor = "current_thread")]
    async fn rejects_root_symlink_with_trailing_slash() {
        use std::os::unix::fs::symlink;

        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join("target");
        let link = temp.path().join("link");
        std::fs::create_dir(&target).unwrap();
        std::fs::write(target.join("keep.txt"), "keep").unwrap();
        symlink(&target, &link).unwrap();
        let path = std::path::PathBuf::from(format!("{}/", link.display()));
        let _cache_home = TestCacheHome::with_tombi_cache_home(Some(&path));

        assert!(matches!(
            clean_cache().await,
            Err(Error::UnsafeCacheDirectory { .. })
        ));
        assert_eq!(
            std::fs::read_to_string(target.join("keep.txt")).unwrap(),
            "keep"
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn appends_index_file_to_non_json_http_paths() {
        let _cache_home = TestCacheHome::new();
        let uri = tombi_uri::Uri::from_str("https://crates.io/api/v1/crates/countme").unwrap();

        let cache_path = get_cache_file_path(&uri).await.unwrap();

        assert_eq!(cache_path.file_name().unwrap(), CACHE_INDEX_FILE_NAME);
        assert_eq!(cache_path.parent().unwrap().file_name().unwrap(), "countme");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn preserves_json_http_paths() {
        let _cache_home = TestCacheHome::new();
        let uri =
            tombi_uri::Uri::from_str("https://www.schemastore.org/api/json/catalog.json").unwrap();

        let cache_path = get_cache_file_path(&uri).await.unwrap();

        assert_eq!(cache_path.file_name().unwrap(), "catalog.json");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn preserves_non_http_paths() {
        let _cache_home = TestCacheHome::new();
        let uri = tombi_uri::Uri::from_str("file:///tmp/example.toml").unwrap();

        let cache_path = get_cache_file_path(&uri).await.unwrap();

        assert_eq!(cache_path.file_name().unwrap(), "example.toml");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn prefers_tombi_cache_home_over_xdg_cache_home() {
        let tombi_cache_home = tempfile::tempdir().unwrap();
        let _cache_home = TestCacheHome::with_tombi_cache_home(Some(tombi_cache_home.path()));

        let cache_path = get_tombi_cache_dir_path().await.unwrap();

        assert_eq!(cache_path, tombi_cache_home.path());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn falls_back_to_xdg_cache_home_when_tombi_cache_home_is_unset() {
        let cache_home = TestCacheHome::new();

        let cache_path = get_tombi_cache_dir_path().await.unwrap();

        assert_eq!(cache_path, cache_home.tombi_cache_dir_path());
    }
}
