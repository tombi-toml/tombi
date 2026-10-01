use tower_lsp::lsp_types::{
    DocumentSymbol, DocumentSymbolParams, DocumentSymbolResponse, SymbolKind,
};

use crate::backend::Backend;

pub async fn handle_document_symbol(
    backend: &Backend,
    params: DocumentSymbolParams,
) -> Result<Option<DocumentSymbolResponse>, tower_lsp::jsonrpc::Error> {
    log::info!("handle_document_symbol");
    log::trace!("{:?}", params);

    let DocumentSymbolParams { text_document, .. } = params;

    let text_document_uri = text_document.uri.into();

    let Some(document_source) = backend.document_source(&text_document_uri) else {
        return Ok(None);
    };

    let document_tree = document_source.document_tree();
    let mut cursor = document_source
        .line_index()
        .cursor(document_source.encoding_kind());

    let symbols = create_symbols(document_tree, &mut cursor);

    Ok(Some(DocumentSymbolResponse::Nested(symbols)))
}

fn create_symbols(
    tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    cursor: &mut tombi_text::LineIndexCursor,
) -> Vec<DocumentSymbol> {
    let mut symbols: Vec<DocumentSymbol> = vec![];

    for (key, value) in tree.key_values() {
        symbols_for_value(key.to_string(), value, None, cursor, &mut symbols);
    }

    symbols
}

#[allow(deprecated)]
fn symbols_for_value(
    mut name: String,
    value: &tombi_document_tree_syntax::Value<'_>,
    parent_key_span: Option<tombi_text::Span>,
    cursor: &mut tombi_text::LineIndexCursor,
    symbols: &mut Vec<DocumentSymbol>,
) {
    use tombi_document_tree_syntax::Value::*;

    // If the key is empty, set the name to "\"\"" for avoiding the empty key error.
    // See: https://github.com/tombi-toml/tombi/pull/1090
    if name.is_empty() {
        name = "\"\"".to_string();
    }

    let value_span = value.symbol_span();
    let span = if let Some(parent_key_span) = parent_key_span {
        parent_key_span + value_span
    } else {
        value_span
    };

    // Converted before the children, so that the cursor mostly moves forward.
    let range = cursor.lsp_range(span);
    let selection_range = range;

    match value {
        Boolean { .. } => {
            symbols.push(DocumentSymbol {
                name,
                kind: SymbolKind::BOOLEAN,
                range,
                selection_range,
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
                range,
                selection_range,
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
                range,
                selection_range,
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
                range,
                selection_range,
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
                    Some(value.symbol_span()),
                    cursor,
                    &mut children,
                );
            }

            symbols.push(DocumentSymbol {
                name,
                kind: SymbolKind::ARRAY,
                range,
                selection_range,
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
                    Some(key.span()),
                    cursor,
                    &mut children,
                );
            }

            symbols.push(DocumentSymbol {
                name,
                kind: SymbolKind::OBJECT,
                range,
                selection_range,
                children: Some(children),
                detail: None,
                deprecated: None,
                tags: None,
            });
        }
        Incomplete { .. } => {}
    }
}
