use tower_lsp::lsp_types::{
    ClientCapabilities, ClientInfo, CompletionOptions, CompletionOptionsCompletionItem,
    DiagnosticOptions, DiagnosticServerCapabilities, DocumentLinkOptions,
    FoldingRangeProviderCapability, HoverProviderCapability, InitializeParams, InitializeResult,
    OneOf, PositionEncodingKind, SemanticTokensFullOptions, SemanticTokensLegend,
    SemanticTokensOptions, ServerCapabilities, ServerInfo, TextDocumentSyncCapability,
    TextDocumentSyncKind, TextDocumentSyncOptions, TextDocumentSyncSaveOptions,
    TypeDefinitionProviderCapability, WorkDoneProgressOptions,
};

use crate::semantic_tokens::SUPPORTED_TOKEN_TYPES;

#[tracing::instrument(level = "debug", skip_all)]
pub async fn handle_initialize(
    params: InitializeParams,
) -> Result<InitializeResult, tower_lsp::jsonrpc::Error> {
    tracing::debug!("handle_initialize");
    tracing::trace!(?params);

    let InitializeParams {
        capabilities: client_capabilities,
        client_info,
        ..
    } = params;

    if let Some(ClientInfo { name, version }) = client_info {
        let version = version.unwrap_or_default();
        tracing::info!("{name} version: {version}",);
    }

    Ok(InitializeResult {
        server_info: Some(ServerInfo {
            name: String::from("Tombi LSP"),
            version: Some(env!("CARGO_PKG_VERSION").to_string()),
        }),
        capabilities: server_capabilities(&client_capabilities),
    })
}

pub fn server_capabilities(client_capabilities: &ClientCapabilities) -> ServerCapabilities {
    ServerCapabilities {
        position_encoding: Some(PositionEncodingKind::UTF16),
        text_document_sync: Some(TextDocumentSyncCapability::Options(
            TextDocumentSyncOptions {
                open_close: Some(true),
                change: Some(TextDocumentSyncKind::FULL),
                save: Some(TextDocumentSyncSaveOptions::Supported(true)),
                ..Default::default()
            },
        )),
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        completion_provider: Some(CompletionOptions {
            trigger_characters: Some(vec![
                ".".into(),
                ",".into(),
                "=".into(),
                "[".into(),
                "{".into(),
                " ".into(),
                "\n".into(),
            ]),
            completion_item: Some(CompletionOptionsCompletionItem {
                label_details_support: (|| -> _ {
                    client_capabilities
                        .text_document
                        .as_ref()?
                        .completion
                        .as_ref()?
                        .completion_item
                        .as_ref()?
                        .label_details_support
                })(),
            }),
            ..Default::default()
        }),
        document_link_provider: Some(DocumentLinkOptions {
            resolve_provider: Some(true),
            work_done_progress_options: WorkDoneProgressOptions::default(),
        }),
        type_definition_provider: Some(TypeDefinitionProviderCapability::Simple(true)),
        document_symbol_provider: Some(OneOf::Left(true)),
        document_formatting_provider: Some(OneOf::Left(true)),
        folding_range_provider: Some(FoldingRangeProviderCapability::Simple(true)),
        semantic_tokens_provider: Some(
            SemanticTokensOptions {
                legend: SemanticTokensLegend {
                    token_types: SUPPORTED_TOKEN_TYPES.to_vec(),
                    token_modifiers: vec![],
                },
                full: Some(SemanticTokensFullOptions::Bool(true)),
                ..Default::default()
            }
            .into(),
        ),
        diagnostic_provider: Some(DiagnosticServerCapabilities::Options(DiagnosticOptions {
            ..Default::default()
        })),

        ..Default::default()
    }
}
