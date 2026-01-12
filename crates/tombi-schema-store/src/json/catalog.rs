use super::JsonCatalogSchema;

pub const DEFAULT_CATALOG_URL: &str = concat!(
    "https://",
    tombi_uri::schemastore_hostname!(),
    "/api/json/catalog.json"
);

#[derive(Debug, Clone, serde::Deserialize)]
pub struct JsonCatalog {
    pub schemas: Vec<JsonCatalogSchema>,
}
