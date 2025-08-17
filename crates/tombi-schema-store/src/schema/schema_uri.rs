use std::str::FromStr;

#[derive(Clone, PartialEq, Eq, Hash, serde::Deserialize)]
pub struct SchemaUri(tombi_uri::Uri);

impl SchemaUri {
    #[inline]
    pub fn new(uri: tombi_uri::Uri) -> Self {
        Self(uri)
    }

    #[inline]
    pub fn parse(uri: &str) -> Result<Self, crate::Error> {
        match tombi_uri::Uri::from_str(uri) {
            Ok(uri) => Ok(Self(uri)),
            Err(_) => Err(crate::Error::InvalidSchemaUri {
                schema_uri: uri.to_string(),
            }),
        }
    }

    #[inline]
    pub fn from_file_path<P: AsRef<std::path::Path>>(path: P) -> Result<Self, crate::Error> {
        match tombi_uri::Uri::from_file_path(&path) {
            Ok(uri) => Ok(Self(uri)),
            Err(_) => Err(crate::Error::InvalidSchemaUri {
                schema_uri: path.as_ref().to_string_lossy().to_string(),
            }),
        }
    }
}

impl std::fmt::Debug for SchemaUri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::ops::Deref for SchemaUri {
    type Target = tombi_uri::Uri;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for SchemaUri {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<tombi_uri::Uri> for SchemaUri {
    fn from(uri: tombi_uri::Uri) -> Self {
        Self(uri)
    }
}

impl From<SchemaUri> for tombi_uri::Uri {
    fn from(schema_uri: SchemaUri) -> Self {
        schema_uri.0
    }
}

impl std::fmt::Display for SchemaUri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
