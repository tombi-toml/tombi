use std::{future::Future, sync::Arc};

use tokio::sync::RwLock;
use tombi_hashmap::HashMap;

#[derive(Debug)]
struct CacheEntry {
    value: Arc<serde_json::Value>,
    version: Option<u64>,
}

static JSON_CACHE: std::sync::LazyLock<RwLock<HashMap<String, CacheEntry>>> =
    std::sync::LazyLock::new(|| RwLock::new(HashMap::new()));

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

    let loaded = loader().await;
    let mut cache = JSON_CACHE.write().await;

    match loaded {
        Some(value) => {
            let value = Arc::new(value);
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
    }
}

#[cfg(test)]
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
}
