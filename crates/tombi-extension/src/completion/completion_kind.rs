#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CompletionKind {
    Boolean,
    Integer,
    Float,
    String,
    Enum,
    OffsetDateTime,
    LocalDateTime,
    LocalDate,
    LocalTime,
    Array,
    Table,
    Key,
    MagicTrigger,
    CommentDirective,
    File,
}

impl CompletionKind {
    pub fn is_literal(&self) -> bool {
        matches!(
            self,
            Self::Boolean
                | Self::Integer
                | Self::Float
                | Self::String
                | Self::Enum
                | Self::OffsetDateTime
                | Self::LocalDateTime
                | Self::LocalDate
                | Self::LocalTime
        )
    }
}

impl From<CompletionKind> for tower_lsp::lsp_types::CompletionItemKind {
    fn from(kind: CompletionKind) -> Self {
        // NOTE: All TOML completions are CompletionItemKind::VALUE,
        //       but some are assigned different types to make it easier to distinguish by symbols.
        match kind {
            CompletionKind::Boolean => tower_lsp::lsp_types::CompletionItemKind::CONSTANT,
            CompletionKind::Integer => tower_lsp::lsp_types::CompletionItemKind::VALUE,
            CompletionKind::Float => tower_lsp::lsp_types::CompletionItemKind::VALUE,
            CompletionKind::String => tower_lsp::lsp_types::CompletionItemKind::TEXT,
            CompletionKind::Enum => tower_lsp::lsp_types::CompletionItemKind::ENUM_MEMBER,
            // NOTE: Event is related to time
            CompletionKind::OffsetDateTime => tower_lsp::lsp_types::CompletionItemKind::EVENT,
            CompletionKind::LocalDateTime => tower_lsp::lsp_types::CompletionItemKind::EVENT,
            CompletionKind::LocalDate => tower_lsp::lsp_types::CompletionItemKind::EVENT,
            CompletionKind::LocalTime => tower_lsp::lsp_types::CompletionItemKind::EVENT,
            CompletionKind::Array => tower_lsp::lsp_types::CompletionItemKind::STRUCT,
            CompletionKind::Table => tower_lsp::lsp_types::CompletionItemKind::STRUCT,
            CompletionKind::Key => tower_lsp::lsp_types::CompletionItemKind::FIELD,
            // NOTE: To give a writing taste close to method chaining
            CompletionKind::MagicTrigger => tower_lsp::lsp_types::CompletionItemKind::METHOD,
            // Document directives such as `schema` / `tombi` should stay visible
            // even when editors hide keyword suggestions for TOML.
            CompletionKind::CommentDirective => tower_lsp::lsp_types::CompletionItemKind::FIELD,
            CompletionKind::File => tower_lsp::lsp_types::CompletionItemKind::FILE,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CompletionKind;

    #[test]
    fn comment_directive_is_not_keyword() {
        assert_eq!(
            tower_lsp::lsp_types::CompletionItemKind::from(CompletionKind::CommentDirective),
            tower_lsp::lsp_types::CompletionItemKind::FIELD
        );
    }
}
