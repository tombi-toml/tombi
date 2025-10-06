use tombi_text::IntoLsp;
use tower_lsp::lsp_types::{
    DocumentSymbol, DocumentSymbolParams, DocumentSymbolResponse, SymbolKind,
};

use crate::backend::Backend;

#[tracing::instrument(level = "debug", skip_all)]
pub async fn handle_document_symbol(
    backend: &Backend,
    params: DocumentSymbolParams,
) -> Result<Option<DocumentSymbolResponse>, tower_lsp::jsonrpc::Error> {
    tracing::info!("handle_document_symbol");
    tracing::trace!(?params);

    let DocumentSymbolParams { text_document, .. } = params;

    let text_document_uri = text_document.uri.into();

    let document_sources = backend.document_sources.read().await;
    let Some(document_source) = document_sources.get(&text_document_uri) else {
        return Ok(None);
    };

    let document_tree = document_source.document_tree();
    let line_index = document_source.line_index();

    let symbols = create_symbols(document_tree, line_index);

    Ok(Some(DocumentSymbolResponse::Nested(symbols)))
}

fn create_symbols(
    tree: &tombi_document_tree::DocumentTree,
    line_index: &tombi_text::LineIndex,
) -> Vec<DocumentSymbol> {
    let mut symbols: Vec<DocumentSymbol> = vec![];

    for (key, value) in tree.key_values() {
        symbols_for_value(key.to_string(), value, None, line_index, &mut symbols);
    }

    symbols
}

#[allow(deprecated)]
fn symbols_for_value(
    mut name: String,
    value: &tombi_document_tree::Value,
    parent_key_range: Option<tombi_text::Range>,
    line_index: &tombi_text::LineIndex,
    symbols: &mut Vec<DocumentSymbol>,
) {
    use tombi_document_tree::Value::*;

    // If the key is empty, set the name to "\"\"" for avoiding the empty key error.
    // See: https://github.com/tombi-toml/tombi/pull/1090
    if name.is_empty() {
        name = "\"\"".to_string();
    }

    let value_range = value.symbol_range();
    let range = if let Some(parent_key_range) = parent_key_range {
        parent_key_range + value_range
    } else {
        value_range
    };

    let selection_range = range;

    match value {
        Boolean { .. } => {
            symbols.push(DocumentSymbol {
                name,
                kind: SymbolKind::BOOLEAN,
                range: range.into_lsp(line_index),
                selection_range: selection_range.into_lsp(line_index),
                children: None,
                detail: None,
                deprecated: None,
                tags: None,
            });
        }
        Integer { .. } | Float { .. } => {
            symbols.push(DocumentSymbol {
                name,
                kind: SymbolKind::NUMBER,
                range: range.into_lsp(line_index),
                selection_range: selection_range.into_lsp(line_index),
                children: None,
                detail: None,
                deprecated: None,
                tags: None,
            });
        }
        String { .. } => {
            symbols.push(DocumentSymbol {
                name,
                kind: SymbolKind::STRING,
                range: range.into_lsp(line_index),
                selection_range: selection_range.into_lsp(line_index),
                children: None,
                detail: None,
                deprecated: None,
                tags: None,
            });
        }
        OffsetDateTime { .. } | LocalDateTime { .. } | LocalDate { .. } | LocalTime { .. } => {
            symbols.push(DocumentSymbol {
                name,
                kind: SymbolKind::STRING,
                range: range.into_lsp(line_index),
                selection_range: selection_range.into_lsp(line_index),
                children: None,
                detail: None,
                deprecated: None,
                tags: None,
            });
        }
        Array(array) => {
            let mut children = vec![];
            for (index, value) in array.values().iter().enumerate() {
                symbols_for_value(
                    format!("[{index}]"),
                    value,
                    Some(value.symbol_range()),
                    line_index,
                    &mut children,
                );
            }

            symbols.push(DocumentSymbol {
                name,
                kind: SymbolKind::ARRAY,
                range: range.into_lsp(line_index),
                selection_range: selection_range.into_lsp(line_index),
                children: Some(children),
                detail: None,
                deprecated: None,
                tags: None,
            });
        }
        Table(table) => {
            let mut children = vec![];
            for (key, value) in table.key_values() {
                symbols_for_value(
                    key.to_string(),
                    value,
                    Some(key.range()),
                    line_index,
                    &mut children,
                );
            }

            symbols.push(DocumentSymbol {
                name,
                kind: SymbolKind::OBJECT,
                range: range.into_lsp(line_index),
                selection_range: selection_range.into_lsp(line_index),
                children: Some(children),
                detail: None,
                deprecated: None,
                tags: None,
            });
        }
        Incomplete { .. } => {}
    }
}
