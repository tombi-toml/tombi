mod catalog_uri;
mod schema_uri;

pub use catalog_uri::CatalogUri;
pub use schema_uri::SchemaUri;
pub use url::{Host, ParseError};

#[macro_export]
macro_rules! schemastore_hostname {
    () => {
        "www.schemastore.org"
    };
}

#[macro_export]
macro_rules! old_schemastore_hostname {
    () => {
        "json.schemastore.org"
    };
}

#[macro_export]
macro_rules! comment_directive_schemastore_hostname {
    () => {
        "www.schemastore.tombi"
    };
}

#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize)]
pub struct Uri(url::Url);

impl Uri {
    #[inline]
    #[allow(clippy::result_unit_err)]
    pub fn from_file_path<P: AsRef<std::path::Path>>(path: P) -> Result<Self, ()> {
        url_from_file_path(path).map(normalize_url).map(Self)
    }

    #[inline]
    #[allow(clippy::result_unit_err)]
    pub fn to_file_path(&self) -> Result<std::path::PathBuf, ()> {
        url_to_file_path(self)
    }
}

impl std::fmt::Display for Uri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<url::Url> for Uri {
    fn from(url: url::Url) -> Self {
        Self(normalize_url(url))
    }
}

impl<'de> serde::Deserialize<'de> for Uri {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        <url::Url as serde::Deserialize>::deserialize(deserializer).map(Self::from)
    }
}

impl From<Uri> for url::Url {
    fn from(uri: Uri) -> Self {
        uri.0
    }
}

impl AsRef<url::Url> for Uri {
    fn as_ref(&self) -> &url::Url {
        &self.0
    }
}

impl AsRef<Uri> for url::Url {
    fn as_ref(&self) -> &Uri {
        unsafe { std::mem::transmute(self) }
    }
}

impl std::ops::Deref for Uri {
    type Target = url::Url;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Uri {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::str::FromStr for Uri {
    type Err = url::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from(url::Url::from_str(s)?))
    }
}

fn normalize_url(url: url::Url) -> url::Url {
    #[cfg(windows)]
    {
        normalize_windows_file_url(url)
    }

    #[cfg(not(windows))]
    {
        url
    }
}

#[cfg(any(windows, test))]
fn normalize_windows_file_url(url: url::Url) -> url::Url {
    let Some(path) = url.as_str().strip_prefix("file:///") else {
        return url;
    };

    let bytes = path.as_bytes();
    let Some(drive) = bytes.first().copied().filter(u8::is_ascii_alphabetic) else {
        return url;
    };
    let after_drive = if bytes.get(1) == Some(&b':') {
        2
    } else if bytes
        .get(1..4)
        .is_some_and(|colon| colon.eq_ignore_ascii_case(b"%3A"))
    {
        4
    } else {
        return url;
    };
    if !matches!(bytes.get(after_drive), None | Some(b'/' | b'?' | b'#')) {
        return url;
    }

    let normalized = format!(
        "file:///{}:{}",
        (drive as char).to_ascii_lowercase(),
        &path[after_drive..]
    );
    url::Url::parse(&normalized).expect("normalizing a file URI must preserve its validity")
}

#[cfg(any(
    unix,
    windows,
    target_os = "redox",
    target_os = "wasi",
    target_os = "hermit"
))]
#[inline]
#[allow(clippy::result_unit_err)]
fn url_from_file_path<P: AsRef<std::path::Path>>(path: P) -> Result<url::Url, ()> {
    url::Url::from_file_path(path)
}

#[cfg(not(any(
    unix,
    windows,
    target_os = "redox",
    target_os = "wasi",
    target_os = "hermit"
)))]
#[allow(clippy::result_unit_err)]
fn url_from_file_path<P: AsRef<std::path::Path>>(path: P) -> Result<url::Url, ()> {
    let path = path.as_ref().to_str().ok_or(())?;
    if !path.starts_with('/') {
        return Err(());
    }
    let mut url = url::Url::parse("file:///").map_err(|_| ())?;
    url.set_path(path);
    Ok(url)
}

#[cfg(any(
    unix,
    windows,
    target_os = "redox",
    target_os = "wasi",
    target_os = "hermit"
))]
#[inline]
#[allow(clippy::result_unit_err)]
fn url_to_file_path(url: &url::Url) -> Result<std::path::PathBuf, ()> {
    url.to_file_path()
}

#[cfg(not(any(
    unix,
    windows,
    target_os = "redox",
    target_os = "wasi",
    target_os = "hermit"
)))]
#[inline]
#[allow(clippy::result_unit_err)]
fn url_to_file_path(url: &url::Url) -> Result<std::path::PathBuf, ()> {
    if url.scheme() != "file" || !matches!(url.host_str(), None | Some("localhost")) {
        return Err(());
    }
    let path = percent_encoding::percent_decode_str(url.path())
        .decode_utf8()
        .map_err(|_| ())?;
    Ok(std::path::PathBuf::from(path.as_ref()))
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use rstest::rstest;
    use serde::Deserialize;

    use super::{Uri, normalize_windows_file_url};

    #[rstest]
    #[case("file:///C:/project/Cargo.toml", "file:///c:/project/Cargo.toml")]
    #[case("file:///C%3A/project/Cargo.toml", "file:///c:/project/Cargo.toml")]
    #[case("file:///c%3a/project/Cargo.toml", "file:///c:/project/Cargo.toml")]
    #[case("file://server/share/Cargo.toml", "file://server/share/Cargo.toml")]
    #[case("https://example.com/C%3A/project", "https://example.com/C%3A/project")]
    fn windows_file_url_has_one_canonical_spelling(#[case] input: &str, #[case] expected: &str) {
        let url = url::Url::parse(input).unwrap();
        assert_eq!(normalize_windows_file_url(url).as_str(), expected);
    }

    #[rstest]
    #[case("file:///C:/project/Cargo.toml", "file:///c%3A/project/Cargo.toml")]
    fn uri_constructors_follow_platform_canonicalization(
        #[case] path_url: &str,
        #[case] encoded_url: &str,
    ) {
        let path_uri = Uri::from_str(path_url).unwrap();
        let parsed_url = url::Url::parse(encoded_url).unwrap();
        let converted_uri = Uri::from(parsed_url);
        let deserialized_uri = Uri::deserialize(serde::de::value::StringDeserializer::<
            serde::de::value::Error,
        >::new(encoded_url.to_owned()))
        .unwrap();

        #[cfg(windows)]
        {
            assert_eq!(path_uri, converted_uri);
            assert_eq!(path_uri, deserialized_uri);
        }

        #[cfg(not(windows))]
        {
            assert_ne!(path_uri, converted_uri);
            assert_ne!(path_uri, deserialized_uri);
        }
    }

    #[cfg(windows)]
    #[test]
    fn from_file_path_normalizes_windows_drive_letter() {
        let uri = Uri::from_file_path(r"C:\project\Cargo.toml").unwrap();
        assert_eq!(uri.as_str(), "file:///c:/project/Cargo.toml");
    }
}
