#[derive(Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct SchemaUri(crate::Uri);

impl SchemaUri {
    #[allow(clippy::result_unit_err)]
    pub fn from_file_path<P: AsRef<std::path::Path>>(path: P) -> Result<Self, ()> {
        crate::Uri::from_file_path(path).map(Self)
    }

    #[allow(clippy::result_unit_err)]
    pub fn to_file_path(&self) -> Result<std::path::PathBuf, ()> {
        crate::Uri::to_file_path(self)
    }
}

impl std::fmt::Debug for SchemaUri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Display for SchemaUri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::ops::Deref for SchemaUri {
    type Target = crate::Uri;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for SchemaUri {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<url::Url> for SchemaUri {
    fn from(url: url::Url) -> Self {
        Self(url.into())
    }
}

impl From<SchemaUri> for url::Url {
    fn from(uri: SchemaUri) -> Self {
        uri.0.0
    }
}

impl From<crate::Uri> for SchemaUri {
    fn from(uri: crate::Uri) -> Self {
        Self(uri)
    }
}

impl From<SchemaUri> for crate::Uri {
    fn from(schema_uri: SchemaUri) -> Self {
        schema_uri.0
    }
}

impl std::str::FromStr for SchemaUri {
    type Err = crate::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        crate::Uri::from_str(s).map(Self)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::SchemaUri;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_schema_uri_from_file_path_with_fragment() {
        let uri_text = "file://./schema.json#/definitions/TableValue";
        let schema_uri = SchemaUri::from_str(uri_text).unwrap();
        assert_eq!(
            schema_uri.to_string(),
            format!("file://./schema.json#/definitions/TableValue")
        );
        assert_eq!(schema_uri.fragment(), Some("/definitions/TableValue"))
    }
}
