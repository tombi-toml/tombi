#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Deserialize)]
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
        match url::Url::from_file_path(&path) {
            Ok(url) => Ok(Self(url)),
            Err(_) => Err(crate::Error::InvalidSchemaUrl {
                schema_url: path.as_ref().to_string_lossy().to_string(),
            }),
        }
    }
}

impl std::ops::Deref for SchemaUrl {
    type Target = url::Url;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for SchemaUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
