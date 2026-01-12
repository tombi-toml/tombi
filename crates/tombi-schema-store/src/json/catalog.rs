use super::JsonCatalogSchema;

pub const DEFAULT_CATALOG_URL: &str = concat!(
    "https://",
    tombi_uri::schemastore_hostname!(),
    "/api/json/catalog.json"
);

#[derive(Debug, Clone, serde::Deserialize)]
pub struct JsonCatalog {
    pub catalog_uri: tombi_uri::CatalogUri,
    pub schemas: Vec<JsonCatalogSchema>,
}
