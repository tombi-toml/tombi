use super::JsonCatalogSchema;

pub const DEFAULT_CATALOG_URL: &str = "https://json.schemastore.org/api/json/catalog.json";

#[derive(Debug, Clone, serde::Deserialize)]
pub struct JsonCatalog {
    pub schemas: Vec<JsonCatalogSchema>,
}
