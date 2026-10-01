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

    let mut lsp_locations = Vec::with_capacity(locations.len());
    for mut location in locations {
        if let Some(remote_uri) = uri_set.get(&location.uri) {
            location.uri = remote_uri.clone();
        }
        let range = match location.range {
            // The extension converted the range with the text it was built from,
            // even if the file is open with unsaved changes.
            Some(range) => tower_lsp::lsp_types::Range::new(
                tower_lsp::lsp_types::Position::new(range.start.line, range.start.column),
                tower_lsp::lsp_types::Position::new(range.end.line, range.end.column),
            ),
            // A file that is not parsed is opened at its start.
            None => tower_lsp::lsp_types::Range::default(),
        };
        lsp_locations.push(tower_lsp::lsp_types::Location {
            uri: location.uri.into(),
            range,
        });
    }

    Ok(lsp_locations)
}
