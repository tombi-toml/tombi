mod catalog_uri;
mod schema_uri;

pub use catalog_uri::CatalogUri;
pub use schema_uri::SchemaUri;
pub use url::ParseError;

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Uri(url::Url);

impl Uri {
    #[allow(clippy::result_unit_err)]
    pub fn from_file_path<P: AsRef<std::path::Path>>(path: P) -> Result<Self, ()> {
        url_from_file_path(path).map(Self)
    }

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
fn url_from_file_path<P: AsRef<std::path::Path>>(_path: P) -> Result<url::Url, ()> {
    Err(())
}

#[cfg(any(
    unix,
    windows,
    target_os = "redox",
    target_os = "wasi",
    target_os = "hermit"
))]
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
#[allow(clippy::result_unit_err)]
fn url_to_file_path(_url: &url::Url) -> Result<std::path::PathBuf, ()> {
    Err(())
}
