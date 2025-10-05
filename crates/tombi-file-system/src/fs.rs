use crate::{AbsPath, Error};

/// File metadata
#[derive(Debug, Clone)]
pub struct Metadata {
    /// File size in bytes
    pub size: u64,

    /// Last modified time (UNIX timestamp in seconds)
    pub modified: Option<u64>,

    /// File type
    pub file_type: FileType,

    /// Read-only flag
    pub readonly: bool,
}

/// File type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    /// Regular file
    File,
    /// Directory
    Directory,
    /// Symbolic link
    Symlink,
}

/// Platform-agnostic file system operations trait
///
/// This trait provides a unified interface for file system operations
/// across native and WASM environments.
pub trait FileSystem: Send + Sync {
    /// Read a file and return its contents as bytes
    ///
    /// # Errors
    ///
    /// Returns `Error::NotFound` if the file does not exist.
    /// Returns `Error::IoError` for other I/O errors.
    fn read(
        &self,
        path: &AbsPath,
    ) -> impl std::future::Future<Output = Result<Vec<u8>, Error>> + Send;

    /// Write bytes to a file
    ///
    /// # Errors
    ///
    /// Returns `Error::PermissionDenied` if write permission is denied.
    /// Returns `Error::IoError` for other I/O errors.
    fn write(
        &self,
        path: &AbsPath,
        contents: &[u8],
    ) -> impl std::future::Future<Output = Result<(), Error>> + Send;

    /// Check if a file exists
    ///
    /// # Errors
    ///
    /// Returns `Error::IoError` if the check fails.
    fn exists(
        &self,
        path: &AbsPath,
    ) -> impl std::future::Future<Output = Result<bool, Error>> + Send;

    /// Remove a file
    ///
    /// # Errors
    ///
    /// Returns `Error::NotFound` if the file does not exist.
    /// Returns `Error::IoError` for other I/O errors.
    fn remove(&self, path: &AbsPath)
        -> impl std::future::Future<Output = Result<(), Error>> + Send;

    /// Create all directories in the path
    ///
    /// # Errors
    ///
    /// Returns `Error::PermissionDenied` if directory creation is denied.
    /// Returns `Error::IoError` for other I/O errors.
    fn create_dir_all(
        &self,
        path: &AbsPath,
    ) -> impl std::future::Future<Output = Result<(), Error>> + Send;

    /// Get file metadata
    ///
    /// # Errors
    ///
    /// Returns `Error::NotFound` if the file does not exist.
    /// Returns `Error::IoError` for other I/O errors.
    fn metadata(
        &self,
        path: &AbsPath,
    ) -> impl std::future::Future<Output = Result<Metadata, Error>> + Send;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AbsPathBuf;
    use std::path::PathBuf;

    // Mock FileSystem implementation for testing
    struct MockFileSystem;

    impl FileSystem for MockFileSystem {
        async fn read(&self, _path: &AbsPath) -> Result<Vec<u8>, Error> {
            Ok(b"mock content".to_vec())
        }

        async fn write(&self, _path: &AbsPath, _contents: &[u8]) -> Result<(), Error> {
            Ok(())
        }

        async fn exists(&self, _path: &AbsPath) -> Result<bool, Error> {
            Ok(true)
        }

        async fn remove(&self, _path: &AbsPath) -> Result<(), Error> {
            Ok(())
        }

        async fn create_dir_all(&self, _path: &AbsPath) -> Result<(), Error> {
            Ok(())
        }

        async fn metadata(&self, _path: &AbsPath) -> Result<Metadata, Error> {
            Ok(Metadata {
                size: 100,
                modified: Some(1234567890),
                file_type: FileType::File,
                readonly: false,
            })
        }
    }

    #[tokio::test]
    async fn test_filesystem_trait_read() {
        let fs = MockFileSystem;
        let path = AbsPathBuf::assert(PathBuf::from("/test/file.txt"));
        let content = fs.read(path.as_path()).await.unwrap();
        assert_eq!(content, b"mock content");
    }

    #[tokio::test]
    async fn test_filesystem_trait_write() {
        let fs = MockFileSystem;
        let path = AbsPathBuf::assert(PathBuf::from("/test/file.txt"));
        let result = fs.write(path.as_path(), b"test").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_filesystem_trait_exists() {
        let fs = MockFileSystem;
        let path = AbsPathBuf::assert(PathBuf::from("/test/file.txt"));
        let exists = fs.exists(path.as_path()).await.unwrap();
        assert!(exists);
    }

    #[tokio::test]
    async fn test_filesystem_trait_remove() {
        let fs = MockFileSystem;
        let path = AbsPathBuf::assert(PathBuf::from("/test/file.txt"));
        let result = fs.remove(path.as_path()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_filesystem_trait_create_dir_all() {
        let fs = MockFileSystem;
        let path = AbsPathBuf::assert(PathBuf::from("/test/dir"));
        let result = fs.create_dir_all(path.as_path()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_filesystem_trait_metadata() {
        let fs = MockFileSystem;
        let path = AbsPathBuf::assert(PathBuf::from("/test/file.txt"));
        let metadata = fs.metadata(path.as_path()).await.unwrap();
        assert_eq!(metadata.size, 100);
        assert_eq!(metadata.modified, Some(1234567890));
        assert_eq!(metadata.file_type, FileType::File);
        assert!(!metadata.readonly);
    }

    #[test]
    fn test_filetype_eq() {
        assert_eq!(FileType::File, FileType::File);
        assert_ne!(FileType::File, FileType::Directory);
        assert_ne!(FileType::Directory, FileType::Symlink);
    }

    #[test]
    fn test_metadata_clone() {
        let metadata = Metadata {
            size: 100,
            modified: Some(1234567890),
            file_type: FileType::File,
            readonly: false,
        };
        let cloned = metadata.clone();
        assert_eq!(metadata.size, cloned.size);
        assert_eq!(metadata.modified, cloned.modified);
        assert_eq!(metadata.file_type, cloned.file_type);
        assert_eq!(metadata.readonly, cloned.readonly);
    }
}
