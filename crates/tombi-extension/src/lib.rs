mod completion;
mod definition;
mod document_link;
mod hover;
mod remote_cache;
mod text_edit;

pub use completion::*;
pub use definition::*;
pub use document_link::*;
pub use hover::*;
pub use remote_cache::{fetch_cached_remote_json, get_remote_json_cache_file_path};

// Export completion-specific TextEdit (uses tombi_text::Range internally)
pub use text_edit::TextEdit;

// Re-export LSP types for code actions
pub use tower_lsp::lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, DocumentChanges, OneOf, WorkspaceEdit,
};

pub trait Extension {}
