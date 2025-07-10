use crate::Backend;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssociateSchemaParams {
    uri: String,
    file_match: Vec<String>,
}

/// Handle the `tombi/associateSchema` request to associate a schema with a file match pattern.
///
/// This function is used to associate a schema URL with a file match pattern in the TOML Language Server.
///
/// In VSCode Extension, contributors can use this to associate a schema with specific files
/// by providing a regex match pattern.
///
/// ```json
/// {
///   "contributes": {
///     "tomlValidation": [
///       {
///         "regexMatch": "^.*foo.toml$",
///         "url": "https://json.schemastore.org/foo.json"
///       }
///     ]
///   }
/// }
/// ```
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
        .associate_schema(schema_url, params.file_match)
        .await;
}
