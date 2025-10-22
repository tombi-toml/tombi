mod completion;
mod completion_text_edit;
mod definition;
mod document_link;

pub use completion::*;
pub use definition::*;
pub use document_link::*;

// Export completion-specific TextEdit (uses tombi_text::Range internally)
pub use completion_text_edit::TextEdit;

// Re-export LSP types for code actions
pub use tower_lsp::lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, DocumentChanges, OneOf, WorkspaceEdit,
};

pub trait Extension {}
