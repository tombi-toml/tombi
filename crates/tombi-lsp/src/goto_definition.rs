use tower_lsp_server::gen_lsp_types::{
    Declaration, DeclarationResponse, Definition, DefinitionResponse,
};

use crate::{Backend, location::into_lsp_locations};

pub async fn try_get_goto_definition_response(
    backend: &Backend,
    locations: Option<Vec<tombi_extension::Location>>,
) -> Result<Option<DefinitionResponse>, tower_lsp_server::jsonrpc::Error> {
    Ok(try_get_definition(backend, locations)
        .await?
        .map(DefinitionResponse::Definition))
}

pub async fn try_get_goto_declaration_response(
    backend: &Backend,
    locations: Option<Vec<tombi_extension::Location>>,
) -> Result<Option<DeclarationResponse>, tower_lsp_server::jsonrpc::Error> {
    Ok(try_get_definition(backend, locations)
        .await?
        .map(|definition| match definition {
            Definition::Location(location) => {
                DeclarationResponse::Declaration(Declaration::Location(location))
            }
            Definition::LocationList(locations) => {
                DeclarationResponse::Declaration(Declaration::LocationList(locations))
            }
        }))
}

async fn try_get_definition(
    backend: &Backend,
    locations: Option<Vec<tombi_extension::Location>>,
) -> Result<Option<Definition>, tower_lsp_server::jsonrpc::Error> {
    let Some(locations) = locations else {
        return Ok(None);
    };

    let locations = into_lsp_locations(backend, locations).await?;

    match locations.len() {
        0 => Ok(None),
        1 => Ok(Some(Definition::Location(
            locations.into_iter().next().unwrap(),
        ))),
        _ => Ok(Some(Definition::LocationList(locations))),
    }
}
