use tombi_schema_store::SchemaSpec;
use crate::Backend;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssociateSchemaParams {
    uri: String,
    file_match: Vec<String>,
}

#[tracing::instrument(level = "debug", skip_all)]
pub async fn handle_associate_schema(backend: &Backend, params: AssociateSchemaParams) {
    tracing::info!("handle_associate_schema");
    tracing::trace!(?params);

    let Ok(schema_url) = tombi_schema_store::SchemaUrl::parse(&params.uri) else {
        tracing::error!("Invalid schema URL");
        return;
    };

    backend
        .schema_store
        .associate_schema(SchemaSpec::Url(schema_url), params.file_match)
        .await;
}
