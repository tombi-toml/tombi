use std::{future::Future, path::Path, sync::Arc, time::UNIX_EPOCH};

use tokio::sync::{Mutex as AsyncMutex, RwLock};
use tombi_hashmap::HashMap;

#[derive(Debug)]
struct CacheEntry {
    value: Arc<serde_json::Value>,
    version: Option<u64>,
}

static JSON_CACHE: std::sync::LazyLock<RwLock<HashMap<String, CacheEntry>>> =
    std::sync::LazyLock::new(|| RwLock::new(HashMap::new()));
static JSON_CACHE_IN_FLIGHT: std::sync::LazyLock<AsyncMutex<HashMap<String, Arc<AsyncMutex<()>>>>> =
    std::sync::LazyLock::new(|| AsyncMutex::new(HashMap::new()));
const MAX_JSON_CACHE_ENTRIES: usize = 128;

pub fn file_cache_version(file_path: &Path) -> Option<u64> {
    let metadata = std::fs::metadata(file_path).ok()?;
    let modified = metadata.modified().ok()?;
    let duration = modified.duration_since(UNIX_EPOCH).ok()?;
    let modified_millis = u64::try_from(duration.as_millis()).ok()?;

    Some(modified_millis ^ metadata.len().wrapping_mul(0x9E37_79B1_85EB_CA87))
}

pub async fn get_or_load_json<F, Fut>(
    key: &str,
    version: Option<u64>,
    loader: F,
) -> Option<Arc<serde_json::Value>>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Option<serde_json::Value>>,
{
    {
        let cache = JSON_CACHE.read().await;
        if let Some(entry) = cache.get(key)
            && (version.is_none() || entry.version == version)
        {
            return Some(Arc::clone(&entry.value));
        }
    }

    let in_flight_lock = {
        let mut in_flight = JSON_CACHE_IN_FLIGHT.lock().await;
        Arc::clone(
            in_flight
                .entry(key.to_string())
                .or_insert_with(|| Arc::new(AsyncMutex::new(()))),
        )
    };
    let _in_flight_guard = in_flight_lock.lock().await;

    {
        let cache = JSON_CACHE.read().await;
        if let Some(entry) = cache.get(key)
            && (version.is_none() || entry.version == version)
        {
            return Some(Arc::clone(&entry.value));
        }
    }

    let loaded = loader().await;
    let mut cache = JSON_CACHE.write().await;

    let result = match loaded {
        Some(value) => {
            let value = Arc::new(value);
            if !cache.contains_key(key)
                && cache.len() >= MAX_JSON_CACHE_ENTRIES
                && let Some(evicted_key) = cache.keys().next().cloned()
            {
                cache.remove(&evicted_key);
            }
            cache.insert(
                key.to_string(),
                CacheEntry {
                    value: Arc::clone(&value),
                    version,
                },
            );
            Some(value)
        }
        None => cache.get(key).map(|entry| Arc::clone(&entry.value)),
    };

    drop(cache);

    let mut in_flight = JSON_CACHE_IN_FLIGHT.lock().await;
    if in_flight
        .get(key)
        .is_some_and(|existing_lock| Arc::ptr_eq(existing_lock, &in_flight_lock))
    {
        in_flight.remove(key);
    }

    result
}

#[cfg(test)]
#[allow(clippy::await_holding_lock)]
mod tests {
    use std::sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicUsize, Ordering},
    };

    use super::*;

    fn test_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    async fn clear_cache() {
        JSON_CACHE.write().await.clear();
        JSON_CACHE_IN_FLIGHT.lock().await.clear();
    }

    #[tokio::test(flavor = "current_thread")]
    async fn returns_cached_value_when_version_matches() {
        let _guard = test_lock()
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        clear_cache().await;

        let counter = Arc::new(AtomicUsize::new(0));
        let key = "test:matching-version";

        let first = get_or_load_json(key, Some(1), {
            let counter = counter.clone();
            move || async move {
                counter.fetch_add(1, Ordering::SeqCst);
                Some(serde_json::json!({"name": "first"}))
            }
        })
        .await;
        let second = get_or_load_json(key, Some(1), {
            let counter = counter.clone();
            move || async move {
                counter.fetch_add(1, Ordering::SeqCst);
                Some(serde_json::json!({"name": "second"}))
            }
        })
        .await;

        assert_eq!(
            first.as_deref(),
            Some(&serde_json::json!({"name": "first"}))
        );
        assert_eq!(
            second.as_deref(),
            Some(&serde_json::json!({"name": "first"}))
        );
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn reloads_when_version_changes() {
        let _guard = test_lock()
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        clear_cache().await;

        let key = "test:version-change";

        assert_eq!(
            get_or_load_json(key, Some(1), || async { Some(serde_json::json!(1)) })
                .await
                .as_deref(),
            Some(&serde_json::json!(1))
        );
        assert_eq!(
            get_or_load_json(key, Some(2), || async { Some(serde_json::json!(2)) })
                .await
                .as_deref(),
            Some(&serde_json::json!(2))
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn keeps_existing_value_when_version_is_none() {
        let _guard = test_lock()
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        clear_cache().await;

        let key = "test:none-version";

        assert_eq!(
            get_or_load_json(key, Some(1), || async { Some(serde_json::json!("cached")) })
                .await
                .as_deref(),
            Some(&serde_json::json!("cached"))
        );
        assert_eq!(
            get_or_load_json(key, None, || async { Some(serde_json::json!("new")) })
                .await
                .as_deref(),
            Some(&serde_json::json!("cached"))
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn separates_entries_by_key() {
        let _guard = test_lock()
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        clear_cache().await;

        assert_eq!(
            get_or_load_json(
                "cargo:inlay_hint.lockfile:/tmp/a/Cargo.lock",
                Some(1),
                || async { Some(serde_json::json!("a")) }
            )
            .await
            .as_deref(),
            Some(&serde_json::json!("a"))
        );
        assert_eq!(
            get_or_load_json(
                "cargo:inlay_hint.lockfile:/tmp/b/Cargo.lock",
                Some(1),
                || async { Some(serde_json::json!("b")) }
            )
            .await
            .as_deref(),
            Some(&serde_json::json!("b"))
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn returns_existing_value_when_reload_fails() {
        let _guard = test_lock()
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        clear_cache().await;

        let key = "test:reload-failure";

        assert_eq!(
            get_or_load_json(key, Some(1), || async { Some(serde_json::json!("cached")) })
                .await
                .as_deref(),
            Some(&serde_json::json!("cached"))
        );
        assert_eq!(
            get_or_load_json(key, Some(2), || async { None })
                .await
                .as_deref(),
            Some(&serde_json::json!("cached"))
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn returns_none_for_missing_key_when_loader_fails() {
        let _guard = test_lock()
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        clear_cache().await;

        assert_eq!(
            get_or_load_json("test:missing", Some(1), || async { None }).await,
            None
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn evicts_old_entries_when_capacity_is_reached() {
        let _guard = test_lock()
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        clear_cache().await;

        for index in 0..=MAX_JSON_CACHE_ENTRIES {
            let key = format!("test:entry:{index}");
            let value = serde_json::json!(index);
            let _ = get_or_load_json(&key, Some(1), move || async move { Some(value) }).await;
        }

        assert_eq!(JSON_CACHE.read().await.len(), MAX_JSON_CACHE_ENTRIES);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn loads_each_key_once_while_a_load_is_in_flight() {
        let _guard = test_lock()
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        clear_cache().await;

        let key = "test:singleflight";
        let counter = Arc::new(AtomicUsize::new(0));
        let barrier = Arc::new(tokio::sync::Barrier::new(2));

        let first = tokio::spawn({
            let counter = Arc::clone(&counter);
            let barrier = Arc::clone(&barrier);
            async move {
                get_or_load_json(key, Some(1), move || async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    barrier.wait().await;
                    Some(serde_json::json!("cached"))
                })
                .await
            }
        });

        let second = tokio::spawn({
            let counter = Arc::clone(&counter);
            let barrier = Arc::clone(&barrier);
            async move {
                barrier.wait().await;
                get_or_load_json(key, Some(1), move || async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Some(serde_json::json!("reloaded"))
                })
                .await
            }
        });

        let (first, second) = tokio::join!(first, second);

        assert_eq!(
            first.expect("first task should succeed").as_deref(),
            Some(&serde_json::json!("cached"))
        );
        assert_eq!(
            second.expect("second task should succeed").as_deref(),
            Some(&serde_json::json!("cached"))
        );
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }
}
