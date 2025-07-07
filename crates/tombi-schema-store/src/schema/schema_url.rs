use tombi_url::url_from_file_path;

#[derive(Clone, PartialEq, Eq, Hash, serde::Deserialize)]
pub struct SchemaUrl(url::Url);

impl SchemaUrl {
    #[inline]
    pub fn new(url: url::Url) -> Self {
        Self(url)
    }

    #[inline]
    pub fn parse(url: &str) -> Result<Self, crate::Error> {
        match url::Url::parse(url) {
            Ok(url) => Ok(Self(url)),
            Err(_) => Err(crate::Error::InvalidSchemaUrl {
                schema_url: url.to_string(),
            }),
        }
    }

    #[inline]
    pub fn from_file_path<P: AsRef<std::path::Path>>(path: P) -> Result<Self, crate::Error> {
        match url_from_file_path(&path) {
            Ok(url) => Ok(Self(url)),
            Err(_) => Err(crate::Error::InvalidSchemaUrl {
                schema_url: path.as_ref().to_string_lossy().to_string(),
            }),
        }
    }
}

impl std::fmt::Debug for SchemaUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::ops::Deref for SchemaUrl {
    type Target = url::Url;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for SchemaUrl {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<url::Url> for SchemaUrl {
    fn from(url: url::Url) -> Self {
        Self(url)
    }
}

impl From<SchemaUrl> for url::Url {
    fn from(schema_url: SchemaUrl) -> Self {
        schema_url.0
    }
}

impl std::fmt::Display for SchemaUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
