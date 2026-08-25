use std::{borrow::Cow, str::FromStr};

use tombi_schema_store::get_tombi_schemastore_content;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DocumentLink {
    pub target: tombi_uri::Uri,
    pub range: tombi_text::Range,
    pub tooltip: Cow<'static, str>,
}

pub fn get_tombi_github_uri(uri: &tombi_uri::Uri) -> Option<tombi_uri::Uri> {
    if uri.scheme() == "tombi" {
        if get_tombi_schemastore_content(uri).is_some() {
            let version = env!("CARGO_PKG_VERSION");
            let branch = if version == "0.0.0-dev" {
                "main".to_string()
            } else {
                format!("refs/tags/v{version}")
            };
            let mut host = uri.host_str().unwrap();
            if host == tombi_uri::old_schemastore_hostname!() {
                host = tombi_uri::schemastore_hostname!();
            }

            if uri.path().ends_with("/json/catalog.json") {
                tombi_uri::Uri::from_str(&format!(
                    "https://raw.githubusercontent.com/tombi-toml/tombi/{branch}/{host}/api/json/catalog.json",
                ))
                .ok()
            } else if let Some(schema_filename) = uri
                .path_segments()
                .and_then(|mut segments| segments.next_back())
            {
                tombi_uri::Uri::from_str(&format!(
                    "https://raw.githubusercontent.com/tombi-toml/tombi/{branch}/{host}/{schema_filename}",
                )).ok()
            } else {
                None
            }
        } else {
            None
        }
    } else {
        Some(uri.clone())
    }
}

pub fn get_schema_link_uri(uri: &tombi_uri::Uri, position: tombi_text::Position) -> tombi_uri::Uri {
    if uri.scheme() == "tombi" {
        return get_tombi_github_uri(uri).unwrap_or_else(|| uri.clone());
    }
    if !matches!(uri.scheme(), "http" | "https") {
        return uri.clone();
    }

    let mut cache_uri = uri.clone();
    cache_uri.set_fragment(None);
    let Some(cache_file_path) = tombi_cache::get_existing_cache_file_path(&cache_uri) else {
        return uri.clone();
    };
    let Ok(mut cache_file_uri) = tombi_uri::Uri::from_file_path(cache_file_path) else {
        return uri.clone();
    };
    cache_file_uri.set_fragment(Some(&format!(
        "L{},{}",
        position.line + 1,
        position.column + 1
    )));
    cache_file_uri
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use tombi_test_lib::TestCacheHome;

    use super::get_schema_link_uri;

    #[tokio::test(flavor = "current_thread")]
    async fn cached_http_schema_links_to_local_position() {
        let _cache_home = TestCacheHome::new();
        let schema_uri =
            tombi_uri::Uri::from_str("https://example.com/cached.schema.json#/properties/value")
                .unwrap();
        let mut cache_uri = schema_uri.clone();
        cache_uri.set_fragment(None);
        let cache_file_path = tombi_cache::get_cache_file_path(&cache_uri).await.unwrap();
        tokio::fs::create_dir_all(cache_file_path.parent().unwrap())
            .await
            .unwrap();
        tokio::fs::write(&cache_file_path, "{}").await.unwrap();

        let link_uri = get_schema_link_uri(&schema_uri, tombi_text::Position::new(2, 4));

        assert_eq!(link_uri.scheme(), "file");
        assert_eq!(link_uri.fragment(), Some("L3,5"));
        assert_eq!(link_uri.to_file_path().unwrap(), cache_file_path);
    }

    #[test]
    fn uncached_and_unknown_schema_links_are_unchanged() {
        let _cache_home = TestCacheHome::new();
        let position = tombi_text::Position::new(2, 4);
        for schema_uri in [
            "https://example.com/uncached.schema.json",
            "tombi://unknown.example/unknown.json",
        ] {
            let schema_uri = tombi_uri::Uri::from_str(schema_uri).unwrap();
            assert_eq!(get_schema_link_uri(&schema_uri, position), schema_uri);
        }
    }

    #[test]
    fn built_in_schema_links_to_github() {
        let schema_uri =
            tombi_uri::Uri::from_str("tombi://www.schemastore.org/tombi.json").unwrap();

        let link_uri = get_schema_link_uri(&schema_uri, tombi_text::Position::default());

        assert_eq!(link_uri.scheme(), "https");
        assert_eq!(link_uri.host_str(), Some("raw.githubusercontent.com"));
        assert!(link_uri.path().ends_with("/www.schemastore.org/tombi.json"));
    }
}
