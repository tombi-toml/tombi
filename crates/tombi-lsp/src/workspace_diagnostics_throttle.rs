use std::sync::Arc;
use std::time::SystemTime;

use tokio::sync::RwLock;

/// Throttle management for WorkspaceDiagnostics
#[derive(Debug, Clone)]
pub struct WorkspaceDiagnosticsThrottle {
    /// Last completion time
    last_completion: Arc<RwLock<Option<SystemTime>>>,
}

impl WorkspaceDiagnosticsThrottle {
    /// Create a new throttle management instance
    pub fn new() -> Self {
        Self {
            last_completion: Arc::new(RwLock::new(None)),
        }
    }

    /// Check if diagnostics should be skipped based on throttle interval
    ///
    /// # Arguments
    /// - throttle_seconds: Throttle interval in seconds.
    ///   - 0: Run only once (first execution), then always skip
    ///   - >0: Skip if within interval, allow if interval has passed
    ///
    /// # Returns
    /// - Ok((should_skip, elapsed_secs)):
    ///   - should_skip: Whether to skip execution
    ///   - elapsed_secs: Elapsed seconds since last execution (None for first run)
    /// - Err: Lock acquisition failure
    pub async fn should_skip_by_throttle(
        &self,
        throttle_seconds: u64,
    ) -> Result<(bool, Option<f64>), String> {
        let last_completion = self.last_completion.read().await;

        match *last_completion {
            None => {
                // Don't skip on first execution
                Ok((false, None))
            }
            Some(last_time) => {
                let now = SystemTime::now();
                let elapsed = now
                    .duration_since(last_time)
                    .map_err(|e| format!("Failed to calculate elapsed time: {}", e))?;

                let elapsed_secs = elapsed.as_secs_f64();

                // Determine skip behavior based on throttle_seconds
                let should_skip = if throttle_seconds == 0 {
                    // throttle_seconds=0: always skip after first execution
                    true
                } else {
                    // throttle_seconds>0: skip only if within interval
                    elapsed_secs < throttle_seconds as f64
                };

                Ok((should_skip, Some(elapsed_secs)))
            }
        }
    }

    /// Record completion time
    ///
    /// # Postconditions
    /// - last_completion is updated with the current time
    pub async fn record_completion(&self) {
        let mut last_completion = self.last_completion.write().await;
        *last_completion = Some(SystemTime::now());
    }

    /// Clear records (for testing)
    pub async fn clear(&self) {
        let mut last_completion = self.last_completion.write().await;
        *last_completion = None;
    }
}

impl Default for WorkspaceDiagnosticsThrottle {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_new_throttle_is_empty() {
        let throttle = WorkspaceDiagnosticsThrottle::new();
        let last_completion = throttle.last_completion.read().await;
        assert!(last_completion.is_none());
    }

    #[tokio::test]
    async fn test_should_skip_returns_false_on_first_call() {
        let throttle = WorkspaceDiagnosticsThrottle::new();
        let (should_skip, elapsed_secs) = throttle.should_skip_by_throttle(5).await.unwrap();

        assert!(!should_skip, "Should not skip on first call");
        assert!(
            elapsed_secs.is_none(),
            "Elapsed time should be None on first call"
        );
    }

    #[tokio::test]
    async fn test_should_skip_returns_true_within_interval() {
        let throttle = WorkspaceDiagnosticsThrottle::new();

        // Record first execution
        throttle.record_completion().await;

        // Short wait (100ms)
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Check with 5 second interval
        let (should_skip, elapsed_secs) = throttle.should_skip_by_throttle(5).await.unwrap();

        assert!(should_skip, "Should skip within interval");
        assert!(elapsed_secs.is_some(), "Elapsed time should be returned");
        assert!(
            elapsed_secs.unwrap() < 5.0,
            "Elapsed time should be less than 5 seconds"
        );
    }

    #[tokio::test]
    async fn test_should_skip_returns_false_after_interval() {
        let throttle = WorkspaceDiagnosticsThrottle::new();

        // Record first execution
        throttle.record_completion().await;

        // Long wait (1.1 seconds, interval is 1 second)
        tokio::time::sleep(Duration::from_millis(1100)).await;

        // Check with 1 second interval
        let (should_skip, elapsed_secs) = throttle.should_skip_by_throttle(1).await.unwrap();

        assert!(!should_skip, "Should not skip after interval");
        assert!(elapsed_secs.is_some(), "Elapsed time should be returned");
        assert!(
            elapsed_secs.unwrap() >= 1.0,
            "Elapsed time should be at least 1 second"
        );
    }

    #[tokio::test]
    async fn test_throttle_zero_runs_once_only() {
        let throttle = WorkspaceDiagnosticsThrottle::new();

        // First call should not skip
        let (should_skip, elapsed_secs) = throttle.should_skip_by_throttle(0).await.unwrap();
        assert!(
            !should_skip,
            "Should not skip on first call even when throttle_seconds=0"
        );
        assert!(
            elapsed_secs.is_none(),
            "Elapsed time should be None on first call"
        );

        // Record first execution
        throttle.record_completion().await;

        // Short wait
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Second call should always skip when throttle_seconds=0
        let (should_skip, elapsed_secs) = throttle.should_skip_by_throttle(0).await.unwrap();

        assert!(
            should_skip,
            "Should always skip after first execution when throttle_seconds=0"
        );
        assert!(
            elapsed_secs.is_some(),
            "Elapsed time should be returned when throttle_seconds=0"
        );
    }

    #[tokio::test]
    async fn test_record_completion_updates_time() {
        let throttle = WorkspaceDiagnosticsThrottle::new();

        // Initially None
        {
            let last_completion = throttle.last_completion.read().await;
            assert!(last_completion.is_none());
        }

        // Record
        throttle.record_completion().await;

        // Now Some
        {
            let last_completion = throttle.last_completion.read().await;
            assert!(last_completion.is_some());
        }

        // Wait and record again
        let first_time = {
            let last_completion = throttle.last_completion.read().await;
            last_completion.unwrap()
        };

        tokio::time::sleep(Duration::from_millis(100)).await;
        throttle.record_completion().await;

        // Updated
        let second_time = {
            let last_completion = throttle.last_completion.read().await;
            last_completion.unwrap()
        };

        assert!(second_time > first_time, "Time should be updated");
    }

    #[tokio::test]
    async fn test_elapsed_seconds_calculation() {
        let throttle = WorkspaceDiagnosticsThrottle::new();

        throttle.record_completion().await;
        tokio::time::sleep(Duration::from_millis(500)).await;

        let (_, elapsed_secs) = throttle.should_skip_by_throttle(10).await.unwrap();
        let elapsed = elapsed_secs.unwrap();

        // Should be approximately 0.5 seconds (with tolerance of 0.4-0.7 seconds)
        assert!(
            elapsed >= 0.4 && elapsed <= 0.7,
            "Elapsed time calculation should be accurate: {}",
            elapsed
        );
    }

    #[tokio::test]
    async fn test_clear_resets_state() {
        let throttle = WorkspaceDiagnosticsThrottle::new();

        // Record
        throttle.record_completion().await;
        {
            let last_completion = throttle.last_completion.read().await;
            assert!(last_completion.is_some());
        }

        // Clear
        throttle.clear().await;
        {
            let last_completion = throttle.last_completion.read().await;
            assert!(last_completion.is_none(), "Should be None after clear");
        }
    }
}
