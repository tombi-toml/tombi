#![cfg(feature = "native")]

use std::path::PathBuf;
use tempfile::tempdir;
use tombi_file_system::{AbsPathBuf, FileSystem, NativeFileSystem};

#[tokio::test]
async fn test_native_read_write() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.txt");
    let abs_path = AbsPathBuf::assert(file_path);

    let fs = NativeFileSystem;

    // Write
    let content = b"Hello, World!";
    fs.write(abs_path.as_path(), content).await.unwrap();

    // Read
    let read_content = fs.read(abs_path.as_path()).await.unwrap();
    assert_eq!(read_content, content);
}

#[tokio::test]
async fn test_native_read_nonexistent() {
    let fs = NativeFileSystem;
    let abs_path = AbsPathBuf::assert(PathBuf::from("/nonexistent/file.txt"));

    let result = fs.read(abs_path.as_path()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_native_write_create_file() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("new_file.txt");
    let abs_path = AbsPathBuf::assert(file_path.clone());

    let fs = NativeFileSystem;

    let content = b"New content";
    fs.write(abs_path.as_path(), content).await.unwrap();

    // Verify file was created
    assert!(file_path.exists());

    // Verify content
    let read_content = fs.read(abs_path.as_path()).await.unwrap();
    assert_eq!(read_content, content);
}

#[tokio::test]
async fn test_native_write_overwrite() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("overwrite.txt");
    let abs_path = AbsPathBuf::assert(file_path);

    let fs = NativeFileSystem;

    // First write
    fs.write(abs_path.as_path(), b"First").await.unwrap();

    // Overwrite
    fs.write(abs_path.as_path(), b"Second").await.unwrap();

    // Verify overwrite
    let content = fs.read(abs_path.as_path()).await.unwrap();
    assert_eq!(content, b"Second");
}

#[tokio::test]
async fn test_native_exists() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("exists.txt");
    let abs_path = AbsPathBuf::assert(file_path.clone());

    let fs = NativeFileSystem;

    // File does not exist yet
    assert!(!fs.exists(abs_path.as_path()).await.unwrap());

    // Create file
    fs.write(abs_path.as_path(), b"test").await.unwrap();

    // File now exists
    assert!(fs.exists(abs_path.as_path()).await.unwrap());
}

#[tokio::test]
async fn test_native_remove() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("remove.txt");
    let abs_path = AbsPathBuf::assert(file_path.clone());

    let fs = NativeFileSystem;

    // Create file
    fs.write(abs_path.as_path(), b"test").await.unwrap();
    assert!(file_path.exists());

    // Remove file
    fs.remove(abs_path.as_path()).await.unwrap();
    assert!(!file_path.exists());
}

#[tokio::test]
async fn test_native_remove_nonexistent() {
    let fs = NativeFileSystem;
    let abs_path = AbsPathBuf::assert(PathBuf::from("/nonexistent/remove.txt"));

    let result = fs.remove(abs_path.as_path()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_native_create_dir_all() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path().join("parent").join("child").join("grandchild");
    let abs_path = AbsPathBuf::assert(dir_path.clone());

    let fs = NativeFileSystem;

    // Create nested directories
    fs.create_dir_all(abs_path.as_path()).await.unwrap();

    // Verify directory was created
    assert!(dir_path.exists());
    assert!(dir_path.is_dir());
}

#[tokio::test]
async fn test_native_metadata() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("metadata.txt");
    let abs_path = AbsPathBuf::assert(file_path.clone());

    let fs = NativeFileSystem;

    // Create file with content
    let content = b"Hello, metadata!";
    fs.write(abs_path.as_path(), content).await.unwrap();

    // Get metadata
    let metadata = fs.metadata(abs_path.as_path()).await.unwrap();

    assert_eq!(metadata.size, content.len() as u64);
    assert!(metadata.modified.is_some());
    assert_eq!(
        metadata.file_type,
        tombi_file_system::FileType::File
    );
}

#[tokio::test]
async fn test_native_metadata_directory() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path().join("test_dir");
    let abs_path = AbsPathBuf::assert(dir_path.clone());

    let fs = NativeFileSystem;

    // Create directory
    fs.create_dir_all(abs_path.as_path()).await.unwrap();

    // Get metadata
    let metadata = fs.metadata(abs_path.as_path()).await.unwrap();

    assert_eq!(
        metadata.file_type,
        tombi_file_system::FileType::Directory
    );
}
