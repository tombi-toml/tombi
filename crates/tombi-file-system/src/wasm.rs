#![cfg(feature = "wasm")]

use crate::{AbsPath, Error, FileSystem, Metadata};

/// WASM file system implementation using WASI
///
/// This implementation uses WASI (WebAssembly System Interface) for file system operations.
/// Note: Full WASI implementation is complex and will be completed in a future iteration.
pub struct WasmFileSystem;

impl FileSystem for WasmFileSystem {
    async fn read(&self, _path: &AbsPath) -> Result<Vec<u8>, Error> {
        // TODO: Implement WASI file reading
        // For now, return Unimplemented error
        Err(Error::Unimplemented {
            feature: "WASM file read",
        })
    }

    async fn write(&self, _path: &AbsPath, _contents: &[u8]) -> Result<(), Error> {
        // TODO: Implement WASI file writing
        Err(Error::Unimplemented {
            feature: "WASM file write",
        })
    }

    async fn exists(&self, _path: &AbsPath) -> Result<bool, Error> {
        // TODO: Implement WASI file existence check
        Err(Error::Unimplemented {
            feature: "WASM file exists",
        })
    }

    async fn remove(&self, _path: &AbsPath) -> Result<(), Error> {
        // TODO: Implement WASI file removal
        Err(Error::Unimplemented {
            feature: "WASM file remove",
        })
    }

    async fn create_dir_all(&self, _path: &AbsPath) -> Result<(), Error> {
        // TODO: Implement WASI directory creation
        Err(Error::Unimplemented {
            feature: "WASM create_dir_all",
        })
    }

    async fn metadata(&self, _path: &AbsPath) -> Result<Metadata, Error> {
        // TODO: Implement WASI metadata retrieval
        Err(Error::Unimplemented {
            feature: "WASM metadata",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AbsPathBuf;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_wasm_fs_unimplemented() {
        let fs = WasmFileSystem;
        let path = AbsPathBuf::assert(PathBuf::from("/test.txt"));

        // All operations should return Unimplemented for now
        assert!(matches!(
            fs.read(path.as_path()).await,
            Err(Error::Unimplemented { .. })
        ));
        assert!(matches!(
            fs.write(path.as_path(), b"test").await,
            Err(Error::Unimplemented { .. })
        ));
        assert!(matches!(
            fs.exists(path.as_path()).await,
            Err(Error::Unimplemented { .. })
        ));
        assert!(matches!(
            fs.remove(path.as_path()).await,
            Err(Error::Unimplemented { .. })
        ));
        assert!(matches!(
            fs.create_dir_all(path.as_path()).await,
            Err(Error::Unimplemented { .. })
        ));
        assert!(matches!(
            fs.metadata(path.as_path()).await,
            Err(Error::Unimplemented { .. })
        ));
    }
}
