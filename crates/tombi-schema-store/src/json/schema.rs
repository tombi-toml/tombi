use crate::SchemaUri;

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonCatalogSchema {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub file_match: Vec<String>,
    pub url: SchemaUri,
}
