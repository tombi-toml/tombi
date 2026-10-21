use crate::channels::CondaQueryScope;

/// Fetches the available versions of a conda package, newest first.
///
/// Only sharded repodata (CEP-16) is used. Channels without a shard index yield `None`
/// instead of falling back to the full `repodata.json`, which would be loaded into memory.
#[cfg(feature = "native")]
pub(crate) async fn fetch_package_versions(
    package_name: &str,
    scope: &CondaQueryScope,
    offline: bool,
    cache_options: Option<&tombi_cache::Options>,
) -> Option<Vec<String>> {
    native::fetch_package_versions(package_name, scope, offline, cache_options).await
}

#[cfg(not(feature = "native"))]
pub(crate) async fn fetch_package_versions(
    _package_name: &str,
    _scope: &CondaQueryScope,
    _offline: bool,
    _cache_options: Option<&tombi_cache::Options>,
) -> Option<Vec<String>> {
    None
}

#[cfg(feature = "native")]
mod native {
    use std::{
        collections::HashMap,
        path::PathBuf,
        str::FromStr,
        sync::{LazyLock, Mutex},
    };

    use itertools::Itertools;
    use rattler_conda_types::{MatchSpec, PackageName};
    use rattler_repodata_gateway::{ChannelConfig, Gateway, SourceConfig, fetch::CacheAction};
    use reqwest_middleware::{
        Middleware, Next,
        reqwest::{Request, Response, Url},
    };

    use crate::channels::CondaQueryScope;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    enum CacheMode {
        Default,
        NoCache,
        Offline,
    }

    impl CacheMode {
        fn source_config(self) -> SourceConfig {
            let (cache_action, missing_shards_are_empty) = match self {
                CacheMode::Default => (CacheAction::CacheOrFetch, false),
                CacheMode::NoCache => (CacheAction::NoCache, false),
                CacheMode::Offline => (CacheAction::ForceCacheOnly, true),
            };
            SourceConfig {
                sharded_enabled: true,
                cache_action,
                missing_shards_are_empty,
                ..SourceConfig::default()
            }
        }
    }

    type GatewayKey = (Option<PathBuf>, CacheMode);

    /// Gateways are reused so that fetched shards stay cached in memory between requests.
    static GATEWAYS: LazyLock<Mutex<HashMap<GatewayKey, Gateway>>> =
        LazyLock::new(Default::default);

    fn gateway(cache_mode: CacheMode) -> Gateway {
        let cache_dir = tombi_cache::try_get_tombi_cache_dir_path()
            .ok()
            .map(|cache_dir| cache_dir.join("rattler"));

        let mut gateways = GATEWAYS.lock().unwrap_or_else(|err| err.into_inner());
        gateways
            .entry((cache_dir.clone(), cache_mode))
            .or_insert_with(|| {
                let client = reqwest_middleware::ClientBuilder::new(
                    reqwest_middleware::reqwest::Client::new(),
                )
                .with(ShardedOnlyGuard)
                .build();

                let mut builder =
                    Gateway::builder()
                        .with_client(client)
                        .with_channel_config(ChannelConfig {
                            default: cache_mode.source_config(),
                            per_channel: Default::default(),
                        });
                if let Some(cache_dir) = cache_dir {
                    builder.set_cache_dir(cache_dir);
                }
                builder.finish()
            })
            .clone()
    }

    pub(super) async fn fetch_package_versions(
        package_name: &str,
        scope: &CondaQueryScope,
        offline: bool,
        cache_options: Option<&tombi_cache::Options>,
    ) -> Option<Vec<String>> {
        if scope.channels.is_empty() {
            return None;
        }

        let package_name = PackageName::from_str(package_name).ok()?;
        let cache_mode = if offline {
            CacheMode::Offline
        } else if cache_options.and_then(|options| options.no_cache) == Some(true) {
            CacheMode::NoCache
        } else {
            CacheMode::Default
        };

        let gateway = gateway(cache_mode);
        // Channels are queried separately: a gateway query fails as a whole when any of its
        // channels fails, so one channel without sharded repodata would hide all others.
        let repodata = futures::future::join_all(scope.channels.iter().map(|channel| {
            let gateway = &gateway;
            let package_name = &package_name;
            async move {
                gateway
                    .query(
                        [channel.clone()],
                        scope.subdirs.iter().copied(),
                        [MatchSpec::from(package_name.clone())],
                    )
                    .recursive(false)
                    .channel_notices(false)
                    .execute()
                    .await
                    .inspect_err(|err| {
                        log::debug!(
                            "failed to fetch sharded repodata for {} from {}: {err}",
                            package_name.as_source(),
                            channel.base_url
                        );
                    })
                    .ok()
            }
        }))
        .await;

        let versions = repodata
            .iter()
            .flatten()
            .flatten()
            .flat_map(|repodata| repodata.iter())
            .map(|record| &record.package_record.version)
            .unique()
            .sorted_by(|a, b| b.version().cmp(a.version()))
            .map(ToString::to_string)
            .dedup()
            .collect_vec();

        (!versions.is_empty()).then_some(versions)
    }

    /// Rejects every request for full repodata so that the gateway can never fall back
    /// from sharded repodata to `repodata.json`.
    struct ShardedOnlyGuard;

    #[async_trait::async_trait]
    impl Middleware for ShardedOnlyGuard {
        async fn handle(
            &self,
            request: Request,
            extensions: &mut http::Extensions,
            next: Next<'_>,
        ) -> reqwest_middleware::Result<Response> {
            if is_full_repodata_url(request.url()) {
                return Err(reqwest_middleware::Error::Middleware(anyhow::anyhow!(
                    "only sharded repodata is supported, refusing to fetch {}",
                    request.url()
                )));
            }
            next.run(request, extensions).await
        }
    }

    fn is_full_repodata_url(url: &Url) -> bool {
        let Some(file_name) = url
            .path_segments()
            .and_then(|mut segments| segments.next_back())
        else {
            return false;
        };

        file_name.ends_with(".jlap")
            || matches!(
                file_name,
                "repodata.json"
                    | "repodata.json.zst"
                    | "repodata.json.bz2"
                    | "current_repodata.json"
                    | "repodata_from_packages.json"
            )
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn blocks_full_repodata_urls() {
            for url in [
                "https://conda.anaconda.org/conda-forge/linux-64/repodata.json",
                "https://conda.anaconda.org/conda-forge/linux-64/repodata.json.zst",
                "https://conda.anaconda.org/conda-forge/linux-64/repodata.json.bz2",
                "https://conda.anaconda.org/conda-forge/linux-64/repodata.jlap",
                "https://conda.anaconda.org/conda-forge/noarch/current_repodata.json",
                "https://repo.prefix.dev/conda-forge/noarch/repodata_from_packages.json",
            ] {
                assert!(is_full_repodata_url(&Url::parse(url).unwrap()), "{url}");
            }
        }

        #[test]
        fn allows_sharded_repodata_urls() {
            for url in [
                "https://conda.anaconda.org/conda-forge/linux-64/repodata_shards.msgpack.zst",
                "https://conda.anaconda.org/conda-forge/linux-64/shards/0123abcd.msgpack.zst",
                "https://fast.prefix.dev/conda-forge/noarch/repodata_shards.msgpack.zst",
            ] {
                assert!(!is_full_repodata_url(&Url::parse(url).unwrap()), "{url}");
            }
        }
    }
}

#[cfg(all(test, feature = "native"))]
#[allow(clippy::await_holding_lock)]
mod tests {
    use rattler_conda_types::{Channel, ChannelConfig, Subdir};
    use tombi_test_lib::{PIXI_TEST_PACKAGE, PixiTestChannel, TestCacheHome};

    use super::*;

    fn scope(channel: &PixiTestChannel) -> CondaQueryScope {
        let channel_config = ChannelConfig::default_with_root_dir(std::env::temp_dir());
        CondaQueryScope {
            channels: vec![Channel::from_str(channel.url(), &channel_config).unwrap()],
            subdirs: vec![Subdir::Linux64, Subdir::NoArch],
        }
    }

    #[tokio::test]
    async fn fetches_versions_newest_first_across_subdirs() {
        let _cache_home = TestCacheHome::new();
        let versions = fetch_package_versions(
            PIXI_TEST_PACKAGE,
            &scope(PixiTestChannel::sharded()),
            false,
            None,
        )
        .await;
        assert_eq!(
            versions,
            Some(
                ["2.0.0", "1.10.0", "1.2.0", "1.0.0", "0.9.0"]
                    .map(ToString::to_string)
                    .to_vec()
            )
        );
    }

    #[tokio::test]
    async fn offline_reads_cached_shards() {
        let _cache_home = TestCacheHome::new();
        let scope = scope(PixiTestChannel::sharded());
        assert!(
            fetch_package_versions(PIXI_TEST_PACKAGE, &scope, false, None)
                .await
                .is_some()
        );

        let versions = fetch_package_versions(PIXI_TEST_PACKAGE, &scope, true, None).await;
        assert_eq!(
            versions.and_then(|versions| versions.into_iter().next()),
            Some("2.0.0".to_string())
        );
    }

    #[tokio::test]
    async fn offline_without_cache_returns_none() {
        let _cache_home = TestCacheHome::new();
        let versions = fetch_package_versions(
            PIXI_TEST_PACKAGE,
            &scope(PixiTestChannel::sharded()),
            true,
            None,
        )
        .await;
        assert_eq!(versions, None);
    }

    #[tokio::test]
    async fn skips_channels_without_shards() {
        let _cache_home = TestCacheHome::new();
        let mut scope = scope(PixiTestChannel::without_shards());
        scope
            .channels
            .extend(self::scope(PixiTestChannel::sharded()).channels);

        let versions = fetch_package_versions(PIXI_TEST_PACKAGE, &scope, false, None).await;
        assert_eq!(
            versions.and_then(|versions| versions.into_iter().next()),
            Some("2.0.0".to_string())
        );
    }

    #[tokio::test]
    async fn never_falls_back_to_full_repodata() {
        let _cache_home = TestCacheHome::new();
        let channel = PixiTestChannel::without_shards();
        let versions =
            fetch_package_versions(PIXI_TEST_PACKAGE, &scope(channel), false, None).await;
        assert_eq!(versions, None);

        let requested_paths = channel.requested_paths();
        assert!(
            requested_paths
                .iter()
                .any(|path| path.ends_with("/repodata_shards.msgpack.zst")),
            "the shard index should have been requested: {requested_paths:?}"
        );
        assert!(
            !requested_paths
                .iter()
                .any(|path| path.contains("repodata.json") || path.ends_with(".jlap")),
            "full repodata must never be requested: {requested_paths:?}"
        );
    }
}

#[cfg(all(test, feature = "native"))]
mod network_tests {
    use rattler_conda_types::{Channel, ChannelConfig, Subdir};

    use super::*;

    fn scope(channel: &str) -> CondaQueryScope {
        let channel_config = ChannelConfig::default_with_root_dir(std::env::temp_dir());
        CondaQueryScope {
            channels: vec![Channel::from_str(channel, &channel_config).unwrap()],
            subdirs: vec![Subdir::Linux64, Subdir::NoArch],
        }
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn fetches_versions_from_sharded_conda_forge() {
        let versions = fetch_package_versions("python", &scope("conda-forge"), false, None)
            .await
            .expect("conda-forge serves sharded repodata");
        assert!(versions.iter().any(|version| version.starts_with("3.12.")));
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn refuses_channels_without_sharded_repodata() {
        let versions = fetch_package_versions(
            "python",
            &scope("https://repo.anaconda.com/pkgs/main"),
            false,
            None,
        )
        .await;
        assert_eq!(versions, None);
    }
}
