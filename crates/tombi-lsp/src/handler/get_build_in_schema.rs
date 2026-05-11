use std::str::FromStr;

use tombi_schema_store::get_tombi_schemastore_content;
use tombi_uri::SchemaUri;

use crate::Backend;

pub async fn handle_get_build_in_schema(
    _backend: &Backend,
    params: GetBuildInSchemaParams,
) -> Result<Option<String>, tower_lsp::jsonrpc::Error> {
    log::info!("handle_get_build_in_schema");
    log::trace!("{:?}", params);

    let Ok(schema_uri) = SchemaUri::from_str(&params.uri) else {
        return Ok(None);
    };

    Ok(get_tombi_schemastore_content(&schema_uri).map(ToString::to_string))
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetBuildInSchemaParams {
    pub uri: String,
}
