use std::sync::Arc;
use std::time::SystemTime;

use ahash::AHashMap;
use tokio::sync::RwLock;

/// Mtime tracker for workspace diagnostics
#[derive(Debug, Clone)]
pub struct MtimeTracker {
    /// Mtime records: URI -> SystemTime
    mtimes: Arc<RwLock<AHashMap<tombi_uri::Uri, SystemTime>>>,
}

impl MtimeTracker {
    /// Create a new mtime tracker
    pub fn new() -> Self {
        Self {
            mtimes: Arc::new(RwLock::new(AHashMap::new())),
        }
    }

    /// Check mtime and return true if unchanged
    ///
    /// # Preconditions
    /// - uri is a valid file path
    ///
    /// # Postconditions
    /// - Returns true if mtime matches, false otherwise
    pub async fn should_skip(&self, uri: &tombi_uri::Uri, mtime: SystemTime) -> bool {
        let mtimes = self.mtimes.read().await;
        mtimes
            .get(uri)
            .map_or(false, |recorded_mtime| *recorded_mtime == mtime)
    }

    /// Record mtime
    ///
    /// # Preconditions
    /// - mtime is a valid timestamp
    ///
    /// # Postconditions
    /// - The mtime for the specified URI is recorded
    pub async fn record(&self, uri: tombi_uri::Uri, mtime: SystemTime) {
        let mut mtimes = self.mtimes.write().await;
        mtimes.insert(uri, mtime);
    }

    /// Remove mtime record for a specific URI
    pub async fn remove(&self, uri: &tombi_uri::Uri) {
        let mut mtimes = self.mtimes.write().await;
        mtimes.remove(uri);
    }

    /// Clear all mtime records
    pub async fn clear(&self) {
        let mut mtimes = self.mtimes.write().await;
        mtimes.clear();
    }
}

impl Default for MtimeTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_new_tracker_is_empty() {
        let tracker = MtimeTracker::new();
        let uri = tombi_uri::Uri::from_file_path("/tmp/test.toml").unwrap();
        let mtime = SystemTime::now();

        assert!(!tracker.should_skip(&uri, mtime).await);
    }

    #[tokio::test]
    async fn test_should_skip_returns_true_for_same_mtime() {
        let tracker = MtimeTracker::new();
        let uri = tombi_uri::Uri::from_file_path("/tmp/test.toml").unwrap();
        let mtime = SystemTime::now();

        tracker.record(uri.clone(), mtime).await;

        assert!(tracker.should_skip(&uri, mtime).await);
    }

    #[tokio::test]
    async fn test_should_skip_returns_false_for_different_mtime() {
        let tracker = MtimeTracker::new();
        let uri = tombi_uri::Uri::from_file_path("/tmp/test.toml").unwrap();
        let mtime1 = SystemTime::now();
        let mtime2 = mtime1 + Duration::from_secs(1);

        tracker.record(uri.clone(), mtime1).await;

        assert!(!tracker.should_skip(&uri, mtime2).await);
    }

    #[tokio::test]
    async fn test_should_skip_returns_false_for_unrecorded_uri() {
        let tracker = MtimeTracker::new();
        let uri = tombi_uri::Uri::from_file_path("/tmp/test.toml").unwrap();
        let mtime = SystemTime::now();

        assert!(!tracker.should_skip(&uri, mtime).await);
    }

    #[tokio::test]
    async fn test_record_updates_existing_mtime() {
        let tracker = MtimeTracker::new();
        let uri = tombi_uri::Uri::from_file_path("/tmp/test.toml").unwrap();
        let mtime1 = SystemTime::now();
        let mtime2 = mtime1 + Duration::from_secs(1);

        tracker.record(uri.clone(), mtime1).await;
        tracker.record(uri.clone(), mtime2).await;

        assert!(!tracker.should_skip(&uri, mtime1).await);
        assert!(tracker.should_skip(&uri, mtime2).await);
    }

    #[tokio::test]
    async fn test_remove_deletes_mtime_record() {
        let tracker = MtimeTracker::new();
        let uri = tombi_uri::Uri::from_file_path("/tmp/test.toml").unwrap();
        let mtime = SystemTime::now();

        tracker.record(uri.clone(), mtime).await;
        tracker.remove(&uri).await;

        assert!(!tracker.should_skip(&uri, mtime).await);
    }

    #[tokio::test]
    async fn test_remove_nonexistent_uri_does_not_error() {
        let tracker = MtimeTracker::new();
        let uri = tombi_uri::Uri::from_file_path("/tmp/test.toml").unwrap();

        tracker.remove(&uri).await; // Should not panic
    }

    #[tokio::test]
    async fn test_clear_removes_all_records() {
        let tracker = MtimeTracker::new();
        let uri1 = tombi_uri::Uri::from_file_path("/tmp/test1.toml").unwrap();
        let uri2 = tombi_uri::Uri::from_file_path("/tmp/test2.toml").unwrap();
        let mtime = SystemTime::now();

        tracker.record(uri1.clone(), mtime).await;
        tracker.record(uri2.clone(), mtime).await;
        tracker.clear().await;

        assert!(!tracker.should_skip(&uri1, mtime).await);
        assert!(!tracker.should_skip(&uri2, mtime).await);
    }
}
