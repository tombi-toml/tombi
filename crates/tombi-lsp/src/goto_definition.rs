use tower_lsp::lsp_types::GotoDefinitionResponse;

use crate::{Backend, location::into_lsp_locations};

pub async fn try_get_goto_definition_response(
    backend: &Backend,
    locations: Option<Vec<tombi_extension::Location>>,
) -> Result<Option<GotoDefinitionResponse>, tower_lsp::jsonrpc::Error> {
    let Some(locations) = locations else {
        return Ok(None);
    };

    let locations = into_lsp_locations(backend, locations).await?;

    match locations.len() {
        0 => Ok(None),
        1 => Ok(Some(GotoDefinitionResponse::Scalar(
            locations.into_iter().next().unwrap(),
        ))),
        _ => Ok(Some(GotoDefinitionResponse::Array(locations))),
    }
}
