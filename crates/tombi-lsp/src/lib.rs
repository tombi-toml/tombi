pub mod backend;
pub mod code_action;
mod comment_directive;
mod completion;
mod config_manager;
mod diagnostic;
mod document;
mod goto_definition;
mod goto_type_definition;
mod hover;
mod semantic_tokens;

pub mod handler {
    mod associate_schema;
    mod code_action;
    mod completion;
    mod diagnostic;
    mod did_change;
    mod did_change_configuration;
    mod did_change_watched_files;
    mod did_close;
    mod did_open;
    mod did_save;
    mod document_link;
    mod document_symbol;
    mod folding_range;
    mod formatting;
    mod get_status;
    mod get_toml_version;
    mod goto_declaration;
    mod goto_definition;
    mod goto_type_definition;
    mod hover;
    mod initialize;
    mod initialized;
    mod refresh_cache;
    mod semantic_tokens_full;
    mod shutdown;
    mod update_config;
    mod update_schema;
    mod workspace_diagnostic;

    pub use associate_schema::{handle_associate_schema, AssociateSchemaParams};
    pub use code_action::handle_code_action;
    pub use completion::handle_completion;
    pub use diagnostic::{handle_diagnostic, push_diagnostics};
    pub use did_change::handle_did_change;
    pub use did_change_configuration::handle_did_change_configuration;
    pub use did_change_watched_files::handle_did_change_watched_files;
    pub use did_close::handle_did_close;
    pub use did_open::handle_did_open;
    pub use did_save::handle_did_save;
    pub use document_link::handle_document_link;
    pub use document_symbol::handle_document_symbol;
    pub use folding_range::handle_folding_range;
    pub use formatting::handle_formatting;
    pub use get_status::{handle_get_status, GetStatusResponse};
    pub use get_toml_version::{
        handle_get_toml_version, GetTomlVersionResponse, TomlVersionSource,
    };
    pub use goto_declaration::handle_goto_declaration;
    pub use goto_definition::handle_goto_definition;
    pub use goto_type_definition::handle_goto_type_definition;
    pub use hover::{get_hover_keys_with_range, handle_hover};
    pub use initialize::handle_initialize;
    pub use initialized::handle_initialized;
    pub use refresh_cache::{handle_refresh_cache, RefreshCacheParams};
    pub use semantic_tokens_full::handle_semantic_tokens_full;
    pub use shutdown::handle_shutdown;
    pub use update_config::handle_update_config;
    pub use update_schema::handle_update_schema;
    pub use workspace_diagnostic::push_workspace_diagnostics;
}

pub use backend::Backend;
pub(crate) use comment_directive::{
    DOCUMENT_SCHEMA_DIRECTIVE_DESCRIPTION, DOCUMENT_SCHEMA_DIRECTIVE_TITLE,
    DOCUMENT_TOMBI_DIRECTIVE_DESCRIPTION, DOCUMENT_TOMBI_DIRECTIVE_TITLE,
};
pub use hover::HoverContent;

/// Run TOML Language Server
#[derive(Debug)]
#[cfg_attr(feature = "clap", derive(clap::Args))]
pub struct Args {}

pub async fn serve(_args: impl Into<Args>, offline: bool, no_cache: bool) {
    tracing::info!(
        "Tombi Language Server version \"{}\" will start.",
        env!("CARGO_PKG_VERSION")
    );

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = tower_lsp::LspService::build(|client| {
        Backend::new(
            client,
            &crate::backend::Options {
                offline: offline.then_some(true),
                no_cache: no_cache.then_some(true),
            },
        )
    })
    .custom_method("tombi/getStatus", Backend::get_status)
    .custom_method("tombi/getTomlVersion", Backend::get_toml_version)
    .custom_method("tombi/updateSchema", Backend::update_schema)
    .custom_method("tombi/updateConfig", Backend::update_config)
    .custom_method("tombi/associateSchema", Backend::associate_schema)
    .custom_method("tombi/refreshCache", Backend::refresh_cache)
    .finish();

    tower_lsp::Server::new(stdin, stdout, socket)
        .serve(service)
        .await;

    tracing::info!("Tombi LSP Server did shut down.");
}
