use std::path::PathBuf;
use tombi_file_system::{AbsPathBuf, FileSystem, InMemoryFileSystem};

#[tokio::test]
async fn test_memory_read_write() {
    let fs = InMemoryFileSystem::default();
    let path = AbsPathBuf::assert(PathBuf::from("/test/file.txt"));

    // Write
    let content = b"Hello, Memory!";
    fs.write(path.as_path(), content).await.unwrap();

    // Read
    let read_content = fs.read(path.as_path()).await.unwrap();
    assert_eq!(read_content, content);
}

#[tokio::test]
async fn test_memory_read_nonexistent() {
    let fs = InMemoryFileSystem::default();
    let path = AbsPathBuf::assert(PathBuf::from("/nonexistent.txt"));

    let result = fs.read(path.as_path()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_memory_exists() {
    let fs = InMemoryFileSystem::default();
    let path = AbsPathBuf::assert(PathBuf::from("/test.txt"));

    // File does not exist yet
    assert!(!fs.exists(path.as_path()).await.unwrap());

    // Write file
    fs.write(path.as_path(), b"test").await.unwrap();

    // File now exists
    assert!(fs.exists(path.as_path()).await.unwrap());
}

#[tokio::test]
async fn test_memory_remove() {
    let fs = InMemoryFileSystem::default();
    let path = AbsPathBuf::assert(PathBuf::from("/remove.txt"));

    // Write file
    fs.write(path.as_path(), b"test").await.unwrap();
    assert!(fs.exists(path.as_path()).await.unwrap());

    // Remove file
    fs.remove(path.as_path()).await.unwrap();
    assert!(!fs.exists(path.as_path()).await.unwrap());
}

#[tokio::test]
async fn test_memory_write_overwrite() {
    let fs = InMemoryFileSystem::default();
    let path = AbsPathBuf::assert(PathBuf::from("/overwrite.txt"));

    // First write
    fs.write(path.as_path(), b"First").await.unwrap();

    // Overwrite
    fs.write(path.as_path(), b"Second").await.unwrap();

    // Verify overwrite
    let content = fs.read(path.as_path()).await.unwrap();
    assert_eq!(content, b"Second");
}

#[tokio::test]
async fn test_memory_concurrent_access() {
    use std::sync::Arc;

    let fs = Arc::new(InMemoryFileSystem::default());
    let mut handles = vec![];

    // Spawn multiple tasks writing to different files
    for i in 0..10 {
        let fs = Arc::clone(&fs);
        let handle = tokio::spawn(async move {
            let path = AbsPathBuf::assert(PathBuf::from(format!("/file_{}.txt", i)));
            let content = format!("Content {}", i);
            fs.write(path.as_path(), content.as_bytes()).await.unwrap();

            // Read back
            let read = fs.read(path.as_path()).await.unwrap();
            assert_eq!(read, content.as_bytes());
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify all files exist
    for i in 0..10 {
        let path = AbsPathBuf::assert(PathBuf::from(format!("/file_{}.txt", i)));
        assert!(fs.exists(path.as_path()).await.unwrap());
    }
}
