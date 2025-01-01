mod diagnostic;
mod did_change;
mod did_change_configuration;
mod did_change_watched_files;
mod did_open;
mod did_save;
mod document_symbol;
mod folding_range;
mod formatting;
mod get_toml_version;
mod hover;
mod initialize;
mod initialized;
mod semantic_tokens_full;
mod shutdown;
mod update_config;
mod update_schema;

pub use diagnostic::handle_diagnostic;
pub use did_change::handle_did_change;
pub use did_change_configuration::handle_did_change_configuration;
pub use did_change_watched_files::handle_did_change_watched_files;
pub use did_open::handle_did_open;
pub use did_save::handle_did_save;
pub use document_symbol::handle_document_symbol;
pub use folding_range::handle_folding_range;
pub use formatting::handle_formatting;
pub use get_toml_version::handle_get_toml_version;
pub use hover::handle_hover;
pub use initialize::handle_initialize;
pub use initialized::handle_initialized;
pub use semantic_tokens_full::handle_semantic_tokens_full;
pub use shutdown::handle_shutdown;
pub use update_config::handle_update_config;
pub use update_schema::handle_update_schema;
