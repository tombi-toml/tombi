/// Workspace diagnostic state management
///
/// This module centrally manages workspace diagnostic-related state (file modification time tracking and throttling).
/// By grouping related functionality, it improves code cohesion and maintainability.
use crate::dirty_files_queue::DirtyFilesQueue;
use crate::file_watcher_manager::{FileWatcherConfig, FileWatcherManager};
use crate::mtime_tracker::MtimeTracker;
use crate::workspace_diagnostics_throttle::WorkspaceDiagnosticsThrottle;
use ahash::AHashMap;
use std::path::PathBuf;
use std::sync::Arc;

/// Structure for managing workspace diagnostic state
///
/// This structure has the following responsibilities:
/// - Managing file modification time (mtime) tracking
/// - Managing workspace diagnostic execution throttling
/// - Managing dirty files queue for file watcher
/// - Managing file watcher lifecycle (optional)
#[derive(Debug, Clone)]
pub struct WorkspaceDiagnosticState {
    mtime_tracker: MtimeTracker,
    throttle: WorkspaceDiagnosticsThrottle,
    dirty_files_queue: Arc<DirtyFilesQueue>,
    file_watcher_manager: Option<Arc<FileWatcherManager>>,
}

impl WorkspaceDiagnosticState {
    pub fn new() -> Self {
        Self {
            mtime_tracker: MtimeTracker::new(),
            throttle: WorkspaceDiagnosticsThrottle::new(),
            dirty_files_queue: Arc::new(DirtyFilesQueue::new()),
            file_watcher_manager: None,
        }
    }

    pub fn mtime_tracker(&self) -> &MtimeTracker {
        &self.mtime_tracker
    }

    pub fn throttle(&self) -> &WorkspaceDiagnosticsThrottle {
        &self.throttle
    }

    pub fn dirty_files_queue(&self) -> &Arc<DirtyFilesQueue> {
        &self.dirty_files_queue
    }

    pub fn file_watcher_manager(&self) -> Option<&Arc<FileWatcherManager>> {
        self.file_watcher_manager.as_ref()
    }

    pub fn set_file_watcher_manager(&mut self, manager: Arc<FileWatcherManager>) {
        self.file_watcher_manager = Some(manager);
    }

    /// Initialize file watcher for workspaces
    ///
    /// # Preconditions
    /// - workspace_folders and configs are valid
    ///
    /// # Postconditions
    /// - file_watcher_manager is initialized (or remains None on error)
    /// - Logs initialization status
    pub async fn initialize_file_watcher(
        &mut self,
        workspace_folders: Vec<PathBuf>,
        configs: AHashMap<PathBuf, FileWatcherConfig>,
    ) {
        match FileWatcherManager::initialize(
            workspace_folders,
            configs,
            self.dirty_files_queue.clone(),
        )
        .await
        {
            Ok(manager) => {
                tracing::info!("File watcher manager initialized successfully");
                self.file_watcher_manager = Some(Arc::new(manager));
            }
            Err(errors) => {
                tracing::warn!(
                    "Failed to initialize file watcher manager ({} errors), falling back to full scan mode",
                    errors.len()
                );
                for error in errors {
                    tracing::debug!("File watcher initialization error: {}", error);
                }
                // Continue without file watcher - will use full scan
            }
        }
    }

    pub async fn clear(&self) {
        self.mtime_tracker.clear().await;
        self.throttle.clear().await;
        self.dirty_files_queue.clear_all().await;
        if let Some(manager) = &self.file_watcher_manager {
            manager.shutdown().await;
        }
    }
}
