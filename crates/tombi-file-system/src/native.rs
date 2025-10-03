use crate::{AbsPath, Error, FileSystem, FileType, Metadata};
use std::time::SystemTime;
use tracing::{debug, error};

/// Native file system implementation using tokio::fs
pub struct NativeFileSystem;

impl FileSystem for NativeFileSystem {
    async fn read(&self, path: &AbsPath) -> Result<Vec<u8>, Error> {
        debug!("Reading file: {:?}", path.as_ref());
        tokio::fs::read(path.as_ref()).await.map_err(|e| {
            error!("Failed to read file {:?}: {}", path.as_ref(), e);
            if e.kind() == std::io::ErrorKind::NotFound {
                Error::NotFound {
                    path: path.to_path_buf(),
                }
            } else if e.kind() == std::io::ErrorKind::PermissionDenied {
                Error::PermissionDenied {
                    path: path.to_path_buf(),
                }
            } else {
                Error::IoError {
                    path: path.to_path_buf(),
                    source: e,
                }
            }
        })
    }

    async fn write(&self, path: &AbsPath, contents: &[u8]) -> Result<(), Error> {
        debug!(
            "Writing {} bytes to file: {:?}",
            contents.len(),
            path.as_ref()
        );
        tokio::fs::write(path.as_ref(), contents)
            .await
            .map_err(|e| {
                error!("Failed to write file {:?}: {}", path.as_ref(), e);
                if e.kind() == std::io::ErrorKind::PermissionDenied {
                    Error::PermissionDenied {
                        path: path.to_path_buf(),
                    }
                } else {
                    Error::IoError {
                        path: path.to_path_buf(),
                        source: e,
                    }
                }
            })
    }

    async fn exists(&self, path: &AbsPath) -> Result<bool, Error> {
        debug!("Checking if file exists: {:?}", path.as_ref());
        match tokio::fs::metadata(path.as_ref()).await {
            Ok(_) => Ok(true),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(Error::IoError {
                path: path.to_path_buf(),
                source: e,
            }),
        }
    }

    async fn remove(&self, path: &AbsPath) -> Result<(), Error> {
        debug!("Removing file: {:?}", path.as_ref());
        tokio::fs::remove_file(path.as_ref()).await.map_err(|e| {
            error!("Failed to remove file {:?}: {}", path.as_ref(), e);
            if e.kind() == std::io::ErrorKind::NotFound {
                Error::NotFound {
                    path: path.to_path_buf(),
                }
            } else {
                Error::IoError {
                    path: path.to_path_buf(),
                    source: e,
                }
            }
        })
    }

    async fn create_dir_all(&self, path: &AbsPath) -> Result<(), Error> {
        debug!("Creating directory (with parents): {:?}", path.as_ref());
        tokio::fs::create_dir_all(path.as_ref()).await.map_err(|e| {
            error!("Failed to create directory {:?}: {}", path.as_ref(), e);
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                Error::PermissionDenied {
                    path: path.to_path_buf(),
                }
            } else {
                Error::IoError {
                    path: path.to_path_buf(),
                    source: e,
                }
            }
        })
    }

    async fn metadata(&self, path: &AbsPath) -> Result<Metadata, Error> {
        debug!("Getting metadata for: {:?}", path.as_ref());
        let meta = tokio::fs::metadata(path.as_ref()).await.map_err(|e| {
            error!("Failed to get metadata for {:?}: {}", path.as_ref(), e);
            if e.kind() == std::io::ErrorKind::NotFound {
                Error::NotFound {
                    path: path.to_path_buf(),
                }
            } else {
                Error::IoError {
                    path: path.to_path_buf(),
                    source: e,
                }
            }
        })?;

        let modified = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
            .map(|d| d.as_secs());

        let file_type = if meta.is_dir() {
            FileType::Directory
        } else if meta.is_symlink() {
            FileType::Symlink
        } else {
            FileType::File
        };

        Ok(Metadata {
            size: meta.len(),
            modified,
            file_type,
            readonly: meta.permissions().readonly(),
        })
    }
}
