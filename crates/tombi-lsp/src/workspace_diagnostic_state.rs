/// Workspace diagnostic state management
///
/// This module centrally manages workspace diagnostic-related state (file modification time tracking and throttling).
/// By grouping related functionality, it improves code cohesion and maintainability.
use crate::mtime_tracker::MtimeTracker;
use crate::workspace_diagnostics_throttle::WorkspaceDiagnosticsThrottle;

/// Structure for managing workspace diagnostic state
///
/// This structure has the following responsibilities:
/// - Managing file modification time (mtime) tracking
/// - Managing workspace diagnostic execution throttling
#[derive(Debug, Clone)]
pub struct WorkspaceDiagnosticState {
    mtime_tracker: MtimeTracker,
    throttle: WorkspaceDiagnosticsThrottle,
}

impl WorkspaceDiagnosticState {
    pub fn new() -> Self {
        Self {
            mtime_tracker: MtimeTracker::new(),
            throttle: WorkspaceDiagnosticsThrottle::new(),
        }
    }

    pub fn mtime_tracker(&self) -> &MtimeTracker {
        &self.mtime_tracker
    }

    pub fn throttle(&self) -> &WorkspaceDiagnosticsThrottle {
        &self.throttle
    }

    pub async fn clear(&self) {
        self.mtime_tracker.clear().await;
        self.throttle.clear().await;
    }
}
