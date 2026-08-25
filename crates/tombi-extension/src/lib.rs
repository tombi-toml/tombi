mod code_action;
mod completion;
mod document_link;
mod hover;
mod inlay_hint;
mod json_cache;
mod location;
#[doc(hidden)]
pub mod remote_cache;
mod text_edit;

pub use code_action::*;
pub use completion::*;
pub use document_link::*;
pub use hover::*;
pub use inlay_hint::*;
pub use json_cache::{file_cache_version, get_or_load_json};
pub use location::*;
pub use remote_cache::fetch_cached_remote_json;
pub use tombi_ast as ast;
pub use tombi_document_tree as document_tree;

// Export completion-specific TextEdit (uses tombi_text::Range internally)
pub use text_edit::TextEdit;
