use itertools::Itertools;
use tombi_text::IntoLsp;

use crate::{Backend, remote_file::open_remote_file};

pub async fn into_lsp_locations(
    backend: &Backend,
    locations: Vec<tombi_extension::Location>,
) -> Result<Vec<tower_lsp::lsp_types::Location>, tower_lsp::jsonrpc::Error> {
    if locations.is_empty() {
        return Ok(Vec::new());
    }

    let mut uri_set = tombi_hashmap::HashMap::new();
    for location in &locations {
        if let Ok(Some(remote_uri)) = open_remote_file(backend, &location.uri).await {
            uri_set.insert(location.uri.clone(), remote_uri);
        }
    }

    let document_sources = backend.document_sources.try_read().ok();

    let locations = locations
        .into_iter()
        .map(|mut location| {
            if let Some(remote_uri) = uri_set.get(&location.uri) {
                location.uri = remote_uri.clone();
            }
            let range = match document_sources
                .as_ref()
                .and_then(|ds| ds.get(&location.uri))
            {
                Some(document_source) => location.range.into_lsp(document_source.line_index()),
                None => tombi_text::convert_range_to_lsp(location.range),
            };
            tower_lsp::lsp_types::Location {
                uri: location.uri.into(),
                range,
            }
        })
        .collect_vec();

    Ok(locations)
}
