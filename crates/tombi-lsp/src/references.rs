use crate::{Backend, location::into_lsp_locations};

pub async fn try_get_reference_locations(
    backend: &Backend,
    definitions: Option<Vec<tombi_extension::Location>>,
) -> Result<Option<Vec<tower_lsp_server::gen_lsp_types::Location>>, tower_lsp_server::jsonrpc::Error>
{
    let Some(definitions) = definitions else {
        return Ok(None);
    };

    let locations = into_lsp_locations(backend, definitions).await?;
    Ok((!locations.is_empty()).then_some(locations))
}
