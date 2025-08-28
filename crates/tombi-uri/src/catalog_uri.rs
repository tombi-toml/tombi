#[derive(Clone, PartialEq, Eq, Hash)]
pub struct CatalogUri(crate::Uri);

impl CatalogUri {
    #[allow(clippy::result_unit_err)]
    pub fn from_file_path<P: AsRef<std::path::Path>>(path: P) -> Result<Self, ()> {
        crate::Uri::from_file_path(path).map(Self)
    }

    #[allow(clippy::result_unit_err)]
    pub fn to_file_path(&self) -> Result<std::path::PathBuf, ()> {
        crate::Uri::to_file_path(self)
    }
}

impl std::fmt::Debug for CatalogUri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Display for CatalogUri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::ops::Deref for CatalogUri {
    type Target = crate::Uri;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for CatalogUri {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<url::Url> for CatalogUri {
    fn from(url: url::Url) -> Self {
        Self(url.into())
    }
}

impl From<CatalogUri> for url::Url {
    fn from(uri: CatalogUri) -> Self {
        uri.0 .0
    }
}

impl From<crate::Uri> for CatalogUri {
    fn from(uri: crate::Uri) -> Self {
        Self(uri)
    }
}

impl From<CatalogUri> for crate::Uri {
    fn from(catalog_uri: CatalogUri) -> Self {
        catalog_uri.0
    }
}

impl std::str::FromStr for CatalogUri {
    type Err = crate::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        crate::Uri::from_str(s).map(Self)
    }
}
