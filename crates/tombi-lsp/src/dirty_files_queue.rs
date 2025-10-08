use ahash::{AHashMap, AHashSet};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Queue of files that have changed and need diagnostics
#[derive(Debug, Clone)]
pub struct DirtyFilesQueue {
    /// Map of workspace folder path to set of dirty file URIs
    dirty_files: Arc<RwLock<AHashMap<PathBuf, AHashSet<tombi_uri::Uri>>>>,
}

impl DirtyFilesQueue {
    /// Create a new empty DirtyFilesQueue
    pub fn new() -> Self {
        Self {
            dirty_files: Arc::new(RwLock::new(AHashMap::new())),
        }
    }

    /// Add dirty files for a workspace
    ///
    /// # Preconditions
    /// - workspace_path is valid
    /// - file_uris contains TOML file URIs that passed include/exclude filtering
    ///
    /// # Postconditions
    /// - Files are added to the workspace's dirty set
    /// - Duplicate URIs are automatically deduplicated by HashSet
    pub async fn add_dirty_files(
        &self,
        workspace_path: PathBuf,
        file_uris: Vec<tombi_uri::Uri>,
    ) {
        let mut dirty_files = self.dirty_files.write().await;
        let workspace_set = dirty_files.entry(workspace_path).or_insert_with(AHashSet::new);
        workspace_set.extend(file_uris);
    }

    /// Get and clear dirty files for a workspace
    ///
    /// # Preconditions
    /// - workspace_path is valid
    ///
    /// # Postconditions
    /// - Returns all dirty files for the workspace
    /// - Clears the dirty set for the workspace
    /// - Returns empty Vec if no dirty files exist
    pub async fn get_and_clear_dirty_files(
        &self,
        workspace_path: &PathBuf,
    ) -> Vec<tombi_uri::Uri> {
        let mut dirty_files = self.dirty_files.write().await;
        dirty_files
            .remove(workspace_path)
            .map(|set| set.into_iter().collect())
            .unwrap_or_default()
    }

    /// Clear dirty files for a specific workspace (e.g., on workspace removal)
    ///
    /// # Postconditions
    /// - All dirty files for workspace_path are removed
    pub async fn clear_workspace(&self, workspace_path: &PathBuf) {
        let mut dirty_files = self.dirty_files.write().await;
        dirty_files.remove(workspace_path);
    }

    /// Clear all dirty files (e.g., on LSP shutdown)
    ///
    /// # Postconditions
    /// - All dirty files across all workspaces are removed
    pub async fn clear_all(&self) {
        let mut dirty_files = self.dirty_files.write().await;
        dirty_files.clear();
    }

    /// Mark all files as dirty for initial scan
    ///
    /// # Preconditions
    /// - workspace_path is valid
    /// - file_uris contains all TOML files in workspace (from initial scan)
    ///
    /// # Postconditions
    /// - All files are marked dirty to trigger initial diagnostics
    pub async fn mark_initial_scan(
        &self,
        workspace_path: PathBuf,
        file_uris: Vec<tombi_uri::Uri>,
    ) {
        // Initial scan is the same as adding dirty files
        self.add_dirty_files(workspace_path, file_uris).await;
    }
}

impl Default for DirtyFilesQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_new_queue_is_empty() {
        let queue = DirtyFilesQueue::new();
        let workspace = PathBuf::from("/test/workspace");

        let files = queue.get_and_clear_dirty_files(&workspace).await;
        assert!(files.is_empty(), "New queue should have no dirty files");
    }

    #[tokio::test]
    async fn test_add_and_retrieve_dirty_files() {
        let queue = DirtyFilesQueue::new();
        let workspace = PathBuf::from("/test/workspace");
        let file1 = tombi_uri::Uri::from_file_path("/test/workspace/file1.toml").unwrap();
        let file2 = tombi_uri::Uri::from_file_path("/test/workspace/file2.toml").unwrap();

        // Add files
        queue
            .add_dirty_files(workspace.clone(), vec![file1.clone(), file2.clone()])
            .await;

        // Retrieve files
        let files = queue.get_and_clear_dirty_files(&workspace).await;
        assert_eq!(files.len(), 2, "Should have 2 dirty files");
        assert!(files.contains(&file1), "Should contain file1");
        assert!(files.contains(&file2), "Should contain file2");

        // After clearing, queue should be empty
        let files_after_clear = queue.get_and_clear_dirty_files(&workspace).await;
        assert!(
            files_after_clear.is_empty(),
            "Queue should be empty after clearing"
        );
    }

    #[tokio::test]
    async fn test_duplicate_uris_are_deduplicated() {
        let queue = DirtyFilesQueue::new();
        let workspace = PathBuf::from("/test/workspace");
        let file = tombi_uri::Uri::from_file_path("/test/workspace/file.toml").unwrap();

        // Add same file multiple times
        queue
            .add_dirty_files(workspace.clone(), vec![file.clone()])
            .await;
        queue
            .add_dirty_files(workspace.clone(), vec![file.clone()])
            .await;
        queue
            .add_dirty_files(workspace.clone(), vec![file.clone()])
            .await;

        // Should only have one file
        let files = queue.get_and_clear_dirty_files(&workspace).await;
        assert_eq!(
            files.len(),
            1,
            "Duplicate URIs should be automatically deduplicated"
        );
    }

    #[tokio::test]
    async fn test_multiple_workspaces_are_independent() {
        let queue = DirtyFilesQueue::new();
        let workspace1 = PathBuf::from("/test/workspace1");
        let workspace2 = PathBuf::from("/test/workspace2");
        let file1 = tombi_uri::Uri::from_file_path("/test/workspace1/file.toml").unwrap();
        let file2 = tombi_uri::Uri::from_file_path("/test/workspace2/file.toml").unwrap();

        // Add files to different workspaces
        queue
            .add_dirty_files(workspace1.clone(), vec![file1.clone()])
            .await;
        queue
            .add_dirty_files(workspace2.clone(), vec![file2.clone()])
            .await;

        // Retrieve files from workspace1
        let files1 = queue.get_and_clear_dirty_files(&workspace1).await;
        assert_eq!(files1.len(), 1, "Workspace1 should have 1 file");
        assert!(files1.contains(&file1), "Workspace1 should contain file1");

        // Retrieve files from workspace2
        let files2 = queue.get_and_clear_dirty_files(&workspace2).await;
        assert_eq!(files2.len(), 1, "Workspace2 should have 1 file");
        assert!(files2.contains(&file2), "Workspace2 should contain file2");
    }

    #[tokio::test]
    async fn test_clear_workspace() {
        let queue = DirtyFilesQueue::new();
        let workspace = PathBuf::from("/test/workspace");
        let file = tombi_uri::Uri::from_file_path("/test/workspace/file.toml").unwrap();

        // Add file
        queue
            .add_dirty_files(workspace.clone(), vec![file.clone()])
            .await;

        // Clear workspace
        queue.clear_workspace(&workspace).await;

        // Should be empty
        let files = queue.get_and_clear_dirty_files(&workspace).await;
        assert!(files.is_empty(), "Workspace should be cleared");
    }

    #[tokio::test]
    async fn test_clear_all() {
        let queue = DirtyFilesQueue::new();
        let workspace1 = PathBuf::from("/test/workspace1");
        let workspace2 = PathBuf::from("/test/workspace2");
        let file1 = tombi_uri::Uri::from_file_path("/test/workspace1/file.toml").unwrap();
        let file2 = tombi_uri::Uri::from_file_path("/test/workspace2/file.toml").unwrap();

        // Add files to both workspaces
        queue
            .add_dirty_files(workspace1.clone(), vec![file1.clone()])
            .await;
        queue
            .add_dirty_files(workspace2.clone(), vec![file2.clone()])
            .await;

        // Clear all
        queue.clear_all().await;

        // Both should be empty
        let files1 = queue.get_and_clear_dirty_files(&workspace1).await;
        let files2 = queue.get_and_clear_dirty_files(&workspace2).await;
        assert!(files1.is_empty(), "Workspace1 should be cleared");
        assert!(files2.is_empty(), "Workspace2 should be cleared");
    }

    #[tokio::test]
    async fn test_mark_initial_scan() {
        let queue = DirtyFilesQueue::new();
        let workspace = PathBuf::from("/test/workspace");
        let file1 = tombi_uri::Uri::from_file_path("/test/workspace/file1.toml").unwrap();
        let file2 = tombi_uri::Uri::from_file_path("/test/workspace/file2.toml").unwrap();

        // Mark initial scan
        queue
            .mark_initial_scan(workspace.clone(), vec![file1.clone(), file2.clone()])
            .await;

        // All files should be marked as dirty
        let files = queue.get_and_clear_dirty_files(&workspace).await;
        assert_eq!(files.len(), 2, "Initial scan should mark all files as dirty");
        assert!(files.contains(&file1), "Should contain file1");
        assert!(files.contains(&file2), "Should contain file2");
    }
}
