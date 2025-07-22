#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CompletionKind {
    Boolean,
    Integer,
    Float,
    String,
    OffsetDateTime,
    LocalDateTime,
    LocalDate,
    LocalTime,
    Array,
    Table,
    Key,
    MagicTrigger,
    CommentDirective,
}

impl CompletionKind {
    pub fn is_literal(&self) -> bool {
        matches!(
            self,
            Self::Boolean
                | Self::Integer
                | Self::Float
                | Self::String
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
            CompletionKind::Boolean => tower_lsp::lsp_types::CompletionItemKind::ENUM_MEMBER,
            CompletionKind::Integer => tower_lsp::lsp_types::CompletionItemKind::VALUE,
            CompletionKind::Float => tower_lsp::lsp_types::CompletionItemKind::VALUE,
            CompletionKind::String => tower_lsp::lsp_types::CompletionItemKind::TEXT,
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
            CompletionKind::CommentDirective => tower_lsp::lsp_types::CompletionItemKind::KEYWORD,
        }
    }
}
