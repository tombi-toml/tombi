use crate::{AbsPath, AbsPathBuf, Error, FileSystem, FileType, Metadata};
use dashmap::DashMap;
use std::sync::Arc;

/// In-memory file system implementation for testing
///
/// This implementation stores files in memory using a DashMap,
/// which provides concurrent access safety.
#[derive(Clone, Default)]
pub struct InMemoryFileSystem {
    files: Arc<DashMap<AbsPathBuf, Vec<u8>>>,
}

impl FileSystem for InMemoryFileSystem {
    async fn read(&self, path: &AbsPath) -> Result<Vec<u8>, Error> {
        let key = AbsPathBuf::new_unchecked(path.to_path_buf());
        self.files
            .get(&key)
            .map(|entry| entry.value().clone())
            .ok_or_else(|| Error::NotFound {
                path: path.to_path_buf(),
            })
    }

    async fn write(&self, path: &AbsPath, contents: &[u8]) -> Result<(), Error> {
        self.files.insert(
            AbsPathBuf::new_unchecked(path.to_path_buf()),
            contents.to_vec(),
        );
        Ok(())
    }

    async fn exists(&self, path: &AbsPath) -> Result<bool, Error> {
        let key = AbsPathBuf::new_unchecked(path.to_path_buf());
        Ok(self.files.contains_key(&key))
    }

    async fn remove(&self, path: &AbsPath) -> Result<(), Error> {
        let key = AbsPathBuf::new_unchecked(path.to_path_buf());
        self.files.remove(&key).ok_or_else(|| Error::NotFound {
            path: path.to_path_buf(),
        })?;
        Ok(())
    }

    async fn create_dir_all(&self, _path: &AbsPath) -> Result<(), Error> {
        // No-op for in-memory filesystem
        Ok(())
    }

    async fn metadata(&self, path: &AbsPath) -> Result<Metadata, Error> {
        let key = AbsPathBuf::new_unchecked(path.to_path_buf());
        let data = self.files.get(&key).ok_or_else(|| Error::NotFound {
            path: path.to_path_buf(),
        })?;

        Ok(Metadata {
            size: data.len() as u64,
            modified: Some(0), // Fixed timestamp for testing
            file_type: FileType::File,
            readonly: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_inmemory_basic() {
        let fs = InMemoryFileSystem::default();
        let path = AbsPathBuf::assert(PathBuf::from("/test.txt"));

        // Write and read
        fs.write(path.as_path(), b"test").await.unwrap();
        let content = fs.read(path.as_path()).await.unwrap();
        assert_eq!(content, b"test");
    }

    #[tokio::test]
    async fn test_inmemory_not_found() {
        let fs = InMemoryFileSystem::default();
        let path = AbsPathBuf::assert(PathBuf::from("/nonexistent.txt"));

        let result = fs.read(path.as_path()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_inmemory_metadata() {
        let fs = InMemoryFileSystem::default();
        let path = AbsPathBuf::assert(PathBuf::from("/test.txt"));

        fs.write(path.as_path(), b"hello").await.unwrap();
        let meta = fs.metadata(path.as_path()).await.unwrap();

        assert_eq!(meta.size, 5);
        assert_eq!(meta.file_type, FileType::File);
        assert!(!meta.readonly);
    }
}
