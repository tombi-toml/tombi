use text_position::TextPosition;
use text_size::TextSize;
use tower_lsp::lsp_types::{DocumentFormattingParams, Position, Range, TextEdit};

use crate::{server::backend::Backend, toml};

pub async fn handle_formatting(
    _backend: &Backend,
    DocumentFormattingParams { text_document, .. }: DocumentFormattingParams,
) -> Result<Option<Vec<TextEdit>>, tower_lsp::jsonrpc::Error> {
    let source = toml::try_load(&text_document.uri)?;

    if let Ok(new_text) = formatter::format(&source) {
        Ok(Some(vec![TextEdit {
            range: Range::new(
                Position::new(0, 0),
                TextPosition::from_source(&source, TextSize::new(source.len() as u32)).into(),
            ),
            new_text,
        }]))
    } else {
        Ok(None)
    }
}
