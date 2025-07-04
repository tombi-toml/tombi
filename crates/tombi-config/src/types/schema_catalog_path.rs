use super::OneOrMany;
use tombi_url::url_from_file_path;

pub const TOMBI_CATALOG_URL: &str = "tombi:///json/catalog.json";
pub const JSON_SCHEMA_STORE_CATALOG_URL: &str =
    "https://json.schemastore.org/api/json/catalog.json";

/// Generic value that can be either single or multiple
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct SchemaCatalogPath(String);

impl SchemaCatalogPath {
    #[inline]
    pub fn value(&self) -> &str {
        self.0.as_str()
    }

    pub fn try_to_catalog_url(
        &self,
        base_dir_path: Option<&std::path::Path>,
    ) -> Result<url::Url, url::ParseError> {
        match self.0.parse() {
            Ok(url) => Ok(url),
            Err(err) => match base_dir_path {
                Some(base_dir_path) => url_from_file_path(base_dir_path.join(&self.0)),
                None => url_from_file_path(&self.0),
            }
            .map_err(|_| err),
        }
    }
}

impl std::fmt::Display for SchemaCatalogPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for OneOrMany<SchemaCatalogPath> {
    fn default() -> Self {
        Self::One(JSON_SCHEMA_STORE_CATALOG_URL.into())
    }
}

impl From<&str> for SchemaCatalogPath {
    fn from(value: &str) -> Self {
        SchemaCatalogPath(value.to_string())
    }
}
