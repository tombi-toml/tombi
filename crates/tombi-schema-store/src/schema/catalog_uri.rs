#[derive(Clone, PartialEq, Eq, Hash)]
pub struct CatalogUri(tombi_uri::Uri);

impl CatalogUri {
    #[inline]
    pub fn new(uri: tombi_uri::Uri) -> Self {
        Self(uri)
    }
}

impl std::ops::Deref for CatalogUri {
    type Target = tombi_uri::Uri;

    fn deref(&self) -> &Self::Target {
        &self.0
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

impl From<CatalogUri> for tombi_uri::Uri {
    fn from(catalog_uri: CatalogUri) -> Self {
        catalog_uri.0
    }
}
