use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// Absolute path (owned version)
///
/// This type guarantees that the path is absolute at the type level.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AbsPathBuf(PathBuf);

impl AbsPathBuf {
    /// Create an absolute path with assertion
    ///
    /// # Panics
    ///
    /// Panics if the path is not absolute.
    pub fn assert(path: PathBuf) -> Self {
        assert!(path.is_absolute(), "Path must be absolute: {:?}", path);
        Self(path)
    }

    /// Create an absolute path without checking (caller's responsibility)
    ///
    /// # Safety
    ///
    /// The caller must ensure that the path is absolute.
    pub fn new_unchecked(path: PathBuf) -> Self {
        Self(path)
    }

    /// Convert to borrowed version
    pub fn as_path(&self) -> &AbsPath {
        AbsPath::new_unchecked(&self.0)
    }

    /// Convert to standard PathBuf
    pub fn to_path_buf(&self) -> PathBuf {
        self.0.clone()
    }
}

/// Absolute path (borrowed version)
///
/// This type guarantees that the path is absolute at the type level.
#[repr(transparent)]
pub struct AbsPath(Path);

impl AbsPath {
    pub(crate) fn new_unchecked(path: &Path) -> &Self {
        // SAFETY: AbsPath is repr(transparent) over Path
        unsafe { &*(path as *const Path as *const AbsPath) }
    }

    /// Get the parent directory
    pub fn parent(&self) -> Option<&AbsPath> {
        self.0.parent().map(Self::new_unchecked)
    }

    /// Get the file name
    pub fn file_name(&self) -> Option<&OsStr> {
        self.0.file_name()
    }

    /// Get the file extension
    pub fn extension(&self) -> Option<&OsStr> {
        self.0.extension()
    }

    /// Convert to PathBuf
    pub fn to_path_buf(&self) -> PathBuf {
        self.0.to_path_buf()
    }
}

impl AsRef<Path> for AbsPath {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl AsRef<Path> for AbsPathBuf {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

/// Virtual path (platform-independent, always `/` separated)
///
/// This type is used for test environments and WASM where platform-specific
/// path separators are not applicable.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VfsPath(String);

impl VfsPath {
    /// Create a virtual path from a `/` separated string
    ///
    /// # Panics
    ///
    /// Panics if the path does not start with `/`.
    pub fn new_virtual_path(path: impl Into<String>) -> Self {
        let path = path.into();
        assert!(path.starts_with('/'), "Virtual path must start with /: {}", path);
        Self(path)
    }

    /// Create a virtual path from a real absolute path
    ///
    /// Platform-specific separators are converted to `/`.
    pub fn new_real_path(path: &AbsPath) -> Self {
        let path_str = path.as_ref().to_string_lossy().replace('\\', "/");
        Self(path_str)
    }

    /// Join a path component
    pub fn join(&self, component: &str) -> Self {
        let mut new_path = self.0.clone();
        if !new_path.ends_with('/') {
            new_path.push('/');
        }
        new_path.push_str(component);
        Self(new_path)
    }

    /// Get the parent directory
    pub fn parent(&self) -> Option<Self> {
        let path = self.0.trim_end_matches('/');
        let parent = path.rsplit_once('/')?.0;
        if parent.is_empty() {
            // Root has no parent
            return None;
        }
        Some(Self(parent.to_string()))
    }

    /// Get the path as a string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "Path must be absolute")]
    fn test_abspathbuf_assert_panics_on_relative_path() {
        let relative = PathBuf::from("relative/path");
        AbsPathBuf::assert(relative);
    }

    #[test]
    fn test_abspathbuf_assert_succeeds_on_absolute_path() {
        let absolute = PathBuf::from("/absolute/path");
        let abs_path = AbsPathBuf::assert(absolute.clone());
        assert_eq!(abs_path.to_path_buf(), absolute);
    }

    #[test]
    fn test_abspath_parent() {
        let abs_path = AbsPathBuf::assert(PathBuf::from("/parent/child"));
        let parent = abs_path.as_path().parent().unwrap();
        assert_eq!(parent.as_ref(), Path::new("/parent"));
    }

    #[test]
    fn test_abspath_parent_returns_none_for_root() {
        let abs_path = AbsPathBuf::assert(PathBuf::from("/"));
        assert!(abs_path.as_path().parent().is_none());
    }

    #[test]
    fn test_abspath_file_name() {
        let abs_path = AbsPathBuf::assert(PathBuf::from("/path/to/file.txt"));
        assert_eq!(
            abs_path.as_path().file_name().unwrap(),
            OsStr::new("file.txt")
        );
    }

    #[test]
    fn test_abspath_extension() {
        let abs_path = AbsPathBuf::assert(PathBuf::from("/path/to/file.txt"));
        assert_eq!(
            abs_path.as_path().extension().unwrap(),
            OsStr::new("txt")
        );
    }

    #[test]
    fn test_abspath_extension_none_for_no_extension() {
        let abs_path = AbsPathBuf::assert(PathBuf::from("/path/to/file"));
        assert!(abs_path.as_path().extension().is_none());
    }

    #[test]
    fn test_abspath_as_ref() {
        let abs_path = AbsPathBuf::assert(PathBuf::from("/test/path"));
        let path_ref: &Path = abs_path.as_ref();
        assert_eq!(path_ref, Path::new("/test/path"));
    }

    #[test]
    fn test_abspathbuf_clone() {
        let abs_path = AbsPathBuf::assert(PathBuf::from("/test/path"));
        let cloned = abs_path.clone();
        assert_eq!(abs_path, cloned);
    }

    #[test]
    fn test_abspathbuf_eq() {
        let path1 = AbsPathBuf::assert(PathBuf::from("/test/path"));
        let path2 = AbsPathBuf::assert(PathBuf::from("/test/path"));
        assert_eq!(path1, path2);
    }

    // VfsPath tests
    #[test]
    #[should_panic(expected = "Virtual path must start with /")]
    fn test_vfspath_new_virtual_path_panics_without_leading_slash() {
        VfsPath::new_virtual_path("relative/path");
    }

    #[test]
    fn test_vfspath_new_virtual_path_succeeds() {
        let vfs_path = VfsPath::new_virtual_path("/virtual/path");
        assert_eq!(vfs_path.as_str(), "/virtual/path");
    }

    #[test]
    fn test_vfspath_new_real_path() {
        let abs_path = AbsPathBuf::assert(PathBuf::from("/real/path"));
        let vfs_path = VfsPath::new_real_path(abs_path.as_path());
        assert_eq!(vfs_path.as_str(), "/real/path");
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_vfspath_new_real_path_converts_backslashes() {
        let abs_path = AbsPathBuf::assert(PathBuf::from("C:\\Windows\\System32"));
        let vfs_path = VfsPath::new_real_path(abs_path.as_path());
        assert_eq!(vfs_path.as_str(), "C:/Windows/System32");
    }

    #[test]
    fn test_vfspath_join() {
        let vfs_path = VfsPath::new_virtual_path("/base");
        let joined = vfs_path.join("child");
        assert_eq!(joined.as_str(), "/base/child");
    }

    #[test]
    fn test_vfspath_join_with_trailing_slash() {
        let vfs_path = VfsPath::new_virtual_path("/base/");
        let joined = vfs_path.join("child");
        assert_eq!(joined.as_str(), "/base/child");
    }

    #[test]
    fn test_vfspath_parent() {
        let vfs_path = VfsPath::new_virtual_path("/parent/child");
        let parent = vfs_path.parent().unwrap();
        assert_eq!(parent.as_str(), "/parent");
    }

    #[test]
    fn test_vfspath_parent_of_root_returns_none() {
        let vfs_path = VfsPath::new_virtual_path("/");
        assert!(vfs_path.parent().is_none());
    }

    #[test]
    fn test_vfspath_parent_with_trailing_slash() {
        let vfs_path = VfsPath::new_virtual_path("/parent/child/");
        let parent = vfs_path.parent().unwrap();
        assert_eq!(parent.as_str(), "/parent");
    }

    #[test]
    fn test_vfspath_clone() {
        let vfs_path = VfsPath::new_virtual_path("/test/path");
        let cloned = vfs_path.clone();
        assert_eq!(vfs_path, cloned);
    }

    #[test]
    fn test_vfspath_eq() {
        let path1 = VfsPath::new_virtual_path("/test/path");
        let path2 = VfsPath::new_virtual_path("/test/path");
        assert_eq!(path1, path2);
    }
}
