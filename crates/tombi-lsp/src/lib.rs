pub mod backend;
pub mod code_action;
mod comment_directive;
mod completion;
mod config_manager;
mod diagnostic;
mod document;
mod goto_definition;
mod goto_type_definition;
pub mod handler;
mod hover;
mod semantic_tokens;

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
