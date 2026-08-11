mod catalog_uri;
mod schema_uri;

pub use catalog_uri::CatalogUri;
pub use schema_uri::SchemaUri;
pub use url::ParseError;

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
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Uri(url::Url);

impl Uri {
    #[inline]
    #[allow(clippy::result_unit_err)]
    pub fn from_file_path<P: AsRef<std::path::Path>>(path: P) -> Result<Self, ()> {
        url_from_file_path(path).map(Self)
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
        Self(url)
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
        Ok(Self(url::Url::from_str(s)?))
    }
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
