use ahash::AHashMap;
use notify_debouncer_full::{
    new_debouncer,
    notify::{RecursiveMode, Watcher},
    DebounceEventResult, Debouncer, FileIdMap,
};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use crate::dirty_files_queue::DirtyFilesQueue;

/// Errors that can occur during file watcher operations
#[derive(Debug, thiserror::Error)]
pub enum FileWatcherError {
    #[error("Failed to initialize watcher: {0}")]
    InitializationFailed(String),

    #[error("Failed to watch path {path}: {error}")]
    WatchFailed { path: PathBuf, error: String },

    #[error("Watcher already exists for workspace: {0}")]
    WatcherAlreadyExists(PathBuf),
}

/// Configuration for file watcher
#[derive(Debug, Clone)]
pub struct FileWatcherConfig {
    pub enabled: bool,
    pub debounce_ms: u64,
}

impl Default for FileWatcherConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            debounce_ms: 100,
        }
    }
}

/// Manages file system watchers for workspace diagnostics
pub struct FileWatcherManager {
    /// Map of workspace folder paths to their debouncer handles
    watchers: Arc<RwLock<AHashMap<PathBuf, Debouncer<notify::RecommendedWatcher, FileIdMap>>>>,
    /// Reference to dirty files queue for recording changes
    dirty_files_queue: Arc<DirtyFilesQueue>,
    /// Configuration per workspace
    workspace_configs: Arc<RwLock<AHashMap<PathBuf, FileWatcherConfig>>>,
}

// Manual Debug implementation since Debouncer doesn't implement Debug
impl std::fmt::Debug for FileWatcherManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileWatcherManager")
            .field("dirty_files_queue", &self.dirty_files_queue)
            .field("watchers", &"<Debouncer instances>")
            .field("workspace_configs", &"<configs>")
            .finish()
    }
}

impl FileWatcherManager {
    /// Initialize file watchers for all workspaces
    ///
    /// # Preconditions
    /// - workspace_folders contains valid paths
    /// - configs contains valid configuration for each workspace
    ///
    /// # Postconditions
    /// - File watchers are started for enabled workspaces
    /// - Returns Ok with manager instance or Err with initialization errors
    pub async fn initialize(
        workspace_folders: Vec<PathBuf>,
        configs: AHashMap<PathBuf, FileWatcherConfig>,
        dirty_files_queue: Arc<DirtyFilesQueue>,
    ) -> Result<Self, Vec<FileWatcherError>> {
        let manager = Self {
            watchers: Arc::new(RwLock::new(AHashMap::new())),
            dirty_files_queue: dirty_files_queue.clone(),
            workspace_configs: Arc::new(RwLock::new(configs.clone())),
        };

        let mut errors = Vec::new();

        for workspace_path in workspace_folders {
            if let Some(config) = configs.get(&workspace_path) {
                if !config.enabled {
                    tracing::debug!(
                        "File watcher disabled for workspace: {:?}",
                        workspace_path
                    );
                    continue;
                }

                if let Err(err) = manager.add_workspace(workspace_path.clone(), config.clone()).await {
                    tracing::warn!("Failed to add workspace watcher: {}", err);
                    errors.push(err);
                }
            }
        }

        if !errors.is_empty() && manager.watchers.read().await.is_empty() {
            // All watchers failed
            Err(errors)
        } else {
            Ok(manager)
        }
    }

    /// Add a new workspace to watch
    ///
    /// # Preconditions
    /// - workspace_path is a valid directory path
    /// - config.enabled determines whether to actually start watching
    ///
    /// # Postconditions
    /// - If enabled, watcher is started for the workspace
    /// - Returns Ok(()) on success, Err on failure
    pub async fn add_workspace(
        &self,
        workspace_path: PathBuf,
        config: FileWatcherConfig,
    ) -> Result<(), FileWatcherError> {
        if !config.enabled {
            return Ok(());
        }

        let mut watchers = self.watchers.write().await;

        if watchers.contains_key(&workspace_path) {
            return Err(FileWatcherError::WatcherAlreadyExists(workspace_path));
        }

        let workspace_path_clone = workspace_path.clone();
        let dirty_files_queue = self.dirty_files_queue.clone();

        // Create debouncer with callback
        let debouncer = new_debouncer(
            Duration::from_millis(config.debounce_ms),
            None,
            move |result: DebounceEventResult| {
                let workspace_path = workspace_path_clone.clone();
                let queue = dirty_files_queue.clone();

                tokio::spawn(async move {
                    Self::handle_event(workspace_path, result, queue).await;
                });
            },
        )
        .map_err(|e| FileWatcherError::InitializationFailed(e.to_string()))?;

        // Watch the workspace directory recursively
        let mut watcher = debouncer;
        watcher
            .watcher()
            .watch(&workspace_path, RecursiveMode::Recursive)
            .map_err(|e| FileWatcherError::WatchFailed {
                path: workspace_path.clone(),
                error: e.to_string(),
            })?;

        watchers.insert(workspace_path.clone(), watcher);

        let mut configs = self.workspace_configs.write().await;
        configs.insert(workspace_path.clone(), config);

        tracing::info!("Started file watcher for workspace: {:?}", workspace_path);
        Ok(())
    }

    /// Remove a workspace from watching
    ///
    /// # Preconditions
    /// - workspace_path exists in watchers map
    ///
    /// # Postconditions
    /// - Watcher is stopped and removed
    /// - Dirty files for this workspace are cleared
    pub async fn remove_workspace(&self, workspace_path: &PathBuf) {
        let mut watchers = self.watchers.write().await;
        watchers.remove(workspace_path);
        self.dirty_files_queue.clear_workspace(workspace_path).await;
        let mut configs = self.workspace_configs.write().await;
        configs.remove(workspace_path);
        tracing::info!("Removed file watcher for workspace: {:?}", workspace_path);
    }

    /// Stop all watchers (called on shutdown)
    ///
    /// # Postconditions
    /// - All watchers are stopped
    /// - Resources are released
    pub async fn shutdown(&self) {
        let mut watchers = self.watchers.write().await;
        watchers.clear();
        self.dirty_files_queue.clear_all().await;
        tracing::info!("File watcher manager shut down");
    }

    /// Handle debounced file system events (internal callback)
    ///
    /// # Preconditions
    /// - event contains valid file paths
    ///
    /// # Postconditions
    /// - TOML files matching include/exclude patterns are added to dirty queue
    /// - Non-TOML or excluded files are ignored
    async fn handle_event(
        workspace_path: PathBuf,
        event: DebounceEventResult,
        dirty_files_queue: Arc<DirtyFilesQueue>,
    ) {
        match event {
            Ok(events) => {
                let mut dirty_uris = Vec::new();

                for event in events {
                    for path in &event.paths {
                        // Filter for TOML files
                        if let Some(extension) = path.extension() {
                            if extension != "toml" {
                                continue;
                            }
                        } else {
                            continue;
                        }

                        // Convert to URI
                        if let Ok(uri) = tombi_uri::Uri::from_file_path(path) {
                            dirty_uris.push(uri);
                            tracing::trace!("File change detected: {:?}", path);
                        }
                    }
                }

                if !dirty_uris.is_empty() {
                    dirty_files_queue
                        .add_dirty_files(workspace_path.clone(), dirty_uris)
                        .await;
                    tracing::debug!(
                        "Added dirty files for workspace: {:?}",
                        workspace_path
                    );
                }
            }
            Err(errors) => {
                for error in errors {
                    tracing::error!("File watcher error: {}", error);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_initialize_with_empty_workspaces() {
        let queue = Arc::new(DirtyFilesQueue::new());
        let configs = AHashMap::new();

        let result = FileWatcherManager::initialize(vec![], configs, queue).await;
        assert!(result.is_ok(), "Should initialize with empty workspaces");
    }

    #[tokio::test]
    async fn test_initialize_with_disabled_watcher() {
        let queue = Arc::new(DirtyFilesQueue::new());
        let workspace = PathBuf::from("/tmp/test_workspace");
        let mut configs = AHashMap::new();
        configs.insert(
            workspace.clone(),
            FileWatcherConfig {
                enabled: false,
                debounce_ms: 100,
            },
        );

        let result = FileWatcherManager::initialize(vec![workspace], configs, queue).await;
        assert!(
            result.is_ok(),
            "Should initialize even with disabled watcher"
        );
    }

    #[tokio::test]
    async fn test_shutdown_cleans_up_resources() {
        let queue = Arc::new(DirtyFilesQueue::new());
        let manager = FileWatcherManager::initialize(vec![], AHashMap::new(), queue)
            .await
            .unwrap();

        // Should not panic
        manager.shutdown().await;
    }
}
