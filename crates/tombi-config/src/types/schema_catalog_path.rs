use tombi_wasm_compat::url::url_from_file_path;
use super::OneOrMany;

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
        base_dirpath: Option<&std::path::Path>,
    ) -> Result<url::Url, url::ParseError> {
        match self.0.parse() {
            Ok(url) => Ok(url),
            Err(err) => match base_dirpath {
                Some(base_dirpath) => url_from_file_path(base_dirpath.join(&self.0)),
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

impl Default for SchemaCatalogPath {
    fn default() -> Self {
        SchemaCatalogPath("https://www.schemastore.org/api/json/catalog.json".to_string())
    }
}

impl Default for OneOrMany<SchemaCatalogPath> {
    fn default() -> Self {
        Self::One(SchemaCatalogPath::default())
    }
}
