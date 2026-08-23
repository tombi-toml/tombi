use std::path::{Component, Path, PathBuf};

#[derive(Debug)]
pub struct RunBlockingError {
    #[cfg(not(target_family = "wasm"))]
    source: tokio::task::JoinError,
}

impl std::fmt::Display for RunBlockingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("blocking task failed")
    }
}

impl std::error::Error for RunBlockingError {
    #[cfg(not(target_family = "wasm"))]
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

#[cfg(not(target_family = "wasm"))]
pub fn home_dir() -> Option<PathBuf> {
    dirs::home_dir()
}

#[cfg(target_family = "wasm")]
pub fn home_dir() -> Option<PathBuf> {
    None
}

#[derive(Debug, Clone)]
pub struct DirEntry {
    path: PathBuf,
    is_dir: bool,
}

impl DirEntry {
    #[inline]
    pub fn path(&self) -> &Path {
        &self.path
    }

    #[inline]
    pub fn is_dir(&self) -> bool {
        self.is_dir
    }
}

pub fn normalize(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            component => normalized.push(component.as_os_str()),
        }
    }
    normalized
}

#[cfg(not(target_family = "wasm"))]
pub fn set_file(_path: impl Into<PathBuf>, _text: impl Into<String>) {}

#[cfg(target_family = "wasm")]
pub fn set_file(path: impl Into<PathBuf>, text: impl Into<String>) {
    let path = normalize(&path.into());
    if let Some(parent) = path.parent() {
        create_dir_all(parent).expect("the in-memory filesystem cannot fail to create a directory");
    }
    files()
        .write()
        .unwrap_or_else(|error| error.into_inner())
        .insert(path, text.into());
}

#[cfg(not(target_family = "wasm"))]
pub fn create_dir_all(path: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(path)
}

#[cfg(target_family = "wasm")]
pub fn create_dir_all(path: &Path) -> std::io::Result<()> {
    let mut directories = directories()
        .write()
        .unwrap_or_else(|error| error.into_inner());
    let mut current = Some(normalize(path));
    while let Some(path) = current {
        if path.as_os_str().is_empty() {
            break;
        }
        current = path.parent().map(Path::to_path_buf);
        directories.insert(path);
    }
    Ok(())
}

#[cfg(not(target_family = "wasm"))]
pub fn remove_file(_path: &Path) {}

#[cfg(target_family = "wasm")]
pub fn remove_file(path: &Path) {
    files()
        .write()
        .unwrap_or_else(|error| error.into_inner())
        .remove(&normalize(path));
}

#[cfg(not(target_family = "wasm"))]
pub fn clear() {}

#[cfg(target_family = "wasm")]
pub fn clear() {
    files()
        .write()
        .unwrap_or_else(|error| error.into_inner())
        .clear();
    directories()
        .write()
        .unwrap_or_else(|error| error.into_inner())
        .clear();
}

#[cfg(not(target_family = "wasm"))]
pub fn read_to_string(path: &Path) -> std::io::Result<String> {
    std::fs::read_to_string(path)
}

#[cfg(target_family = "wasm")]
pub fn read_to_string(path: &Path) -> std::io::Result<String> {
    files()
        .read()
        .unwrap_or_else(|error| error.into_inner())
        .get(&normalize(path))
        .cloned()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "virtual file not found"))
}

pub async fn read_to_string_async(path: &Path) -> std::io::Result<String> {
    #[cfg(not(target_family = "wasm"))]
    {
        tokio::fs::read_to_string(path).await
    }
    #[cfg(target_family = "wasm")]
    {
        read_to_string(path)
    }
}

#[cfg(not(target_family = "wasm"))]
pub fn is_file(path: &Path) -> bool {
    path.is_file()
}

#[cfg(target_family = "wasm")]
pub fn is_file(path: &Path) -> bool {
    files()
        .read()
        .unwrap_or_else(|error| error.into_inner())
        .contains_key(&normalize(path))
}

#[cfg(not(target_family = "wasm"))]
pub fn is_dir(path: &Path) -> bool {
    path.is_dir()
}

#[cfg(target_family = "wasm")]
pub fn is_dir(path: &Path) -> bool {
    let path = normalize(path);
    if directories()
        .read()
        .unwrap_or_else(|error| error.into_inner())
        .contains(&path)
    {
        return true;
    }
    files()
        .read()
        .unwrap_or_else(|error| error.into_inner())
        .keys()
        .any(|candidate| candidate.starts_with(&path) && candidate != &path)
}

#[cfg(not(target_family = "wasm"))]
pub fn canonicalize(path: &Path) -> std::io::Result<PathBuf> {
    path.canonicalize()
}

#[cfg(target_family = "wasm")]
pub fn canonicalize(path: &Path) -> std::io::Result<PathBuf> {
    let path = normalize(path);
    (is_file(&path) || is_dir(&path))
        .then_some(path)
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "virtual path not found"))
}

pub async fn run_blocking<F, R>(function: F) -> Result<R, RunBlockingError>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    #[cfg(not(target_family = "wasm"))]
    {
        tokio::task::spawn_blocking(function)
            .await
            .map_err(|source| RunBlockingError { source })
    }
    #[cfg(target_family = "wasm")]
    {
        Ok(function())
    }
}

#[cfg(not(target_family = "wasm"))]
pub fn file_version(path: &Path) -> Option<u64> {
    use std::time::UNIX_EPOCH;

    let metadata = std::fs::metadata(path).ok()?;
    let modified = metadata.modified().ok()?;
    let duration = modified.duration_since(UNIX_EPOCH).ok()?;
    let modified_millis = u64::try_from(duration.as_millis()).ok()?;
    Some(modified_millis ^ metadata.len().wrapping_mul(0x9E37_79B1_85EB_CA87))
}

#[cfg(target_family = "wasm")]
pub fn file_version(path: &Path) -> Option<u64> {
    use std::hash::{Hash, Hasher};

    let files = files().read().unwrap_or_else(|error| error.into_inner());
    let text = files.get(&normalize(path))?;
    let mut hasher = std::hash::DefaultHasher::new();
    text.hash(&mut hasher);
    Some(hasher.finish())
}

#[cfg(not(target_family = "wasm"))]
pub fn glob(pattern: &str) -> Result<Vec<PathBuf>, glob::PatternError> {
    Ok(glob::glob(pattern)?.filter_map(Result::ok).collect())
}

#[cfg(target_family = "wasm")]
pub fn glob(pattern: &str) -> Result<Vec<PathBuf>, glob::PatternError> {
    let pattern = glob::Pattern::new(pattern)?;
    let files = files().read().unwrap_or_else(|error| error.into_inner());
    let mut paths = std::collections::HashSet::new();
    for file_path in files.keys() {
        paths.insert(file_path.clone());
        let mut parent = file_path.parent();
        while let Some(path) = parent {
            paths.insert(path.to_path_buf());
            parent = path.parent();
        }
    }
    Ok(paths
        .into_iter()
        .filter(|path| pattern.matches_path(path))
        .collect())
}

#[cfg(not(target_family = "wasm"))]
pub fn read_dir(path: &Path) -> std::io::Result<Vec<DirEntry>> {
    std::fs::read_dir(path)?
        .map(|entry| {
            let entry = entry?;
            Ok(DirEntry {
                is_dir: entry.file_type()?.is_dir(),
                path: entry.path(),
            })
        })
        .collect()
}

#[cfg(target_family = "wasm")]
pub fn read_dir(path: &Path) -> std::io::Result<Vec<DirEntry>> {
    let path = normalize(path);
    let files = files().read().unwrap_or_else(|error| error.into_inner());
    let directories = directories()
        .read()
        .unwrap_or_else(|error| error.into_inner());
    let mut entries = std::collections::HashMap::<PathBuf, bool>::new();

    for candidate in directories
        .iter()
        .filter(|candidate| candidate.starts_with(&path) && *candidate != &path)
    {
        let Ok(relative) = candidate.strip_prefix(&path) else {
            continue;
        };
        let Some(first) = relative.components().next() else {
            continue;
        };
        entries.insert(path.join(first.as_os_str()), true);
    }

    for candidate in files
        .keys()
        .filter(|candidate| candidate.starts_with(&path))
    {
        let Ok(relative) = candidate.strip_prefix(&path) else {
            continue;
        };
        let mut components = relative.components();
        let Some(first) = components.next() else {
            continue;
        };
        let entry_path = path.join(first.as_os_str());
        let is_dir = components.next().is_some();
        entries
            .entry(entry_path)
            .and_modify(|entry_is_dir| *entry_is_dir |= is_dir)
            .or_insert(is_dir);
    }
    Ok(entries
        .into_iter()
        .map(|(path, is_dir)| DirEntry { path, is_dir })
        .collect())
}

#[cfg(target_family = "wasm")]
fn files() -> &'static std::sync::RwLock<std::collections::HashMap<PathBuf, String>> {
    static FILES: std::sync::OnceLock<
        std::sync::RwLock<std::collections::HashMap<PathBuf, String>>,
    > = std::sync::OnceLock::new();
    FILES.get_or_init(Default::default)
}

#[cfg(target_family = "wasm")]
fn directories() -> &'static std::sync::RwLock<std::collections::HashSet<PathBuf>> {
    static DIRECTORIES: std::sync::OnceLock<std::sync::RwLock<std::collections::HashSet<PathBuf>>> =
        std::sync::OnceLock::new();
    DIRECTORIES.get_or_init(Default::default)
}
