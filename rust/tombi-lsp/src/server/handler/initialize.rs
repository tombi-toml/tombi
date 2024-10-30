use tower_lsp::lsp_types::{
    ClientCapabilities, ClientInfo, DiagnosticOptions, DiagnosticServerCapabilities,
    DocumentOnTypeFormattingOptions, HoverProviderCapability, InitializeParams, InitializeResult,
    OneOf, PositionEncodingKind, SaveOptions, SemanticTokenModifier, SemanticTokensFullOptions,
    SemanticTokensLegend, SemanticTokensOptions, ServerCapabilities, ServerInfo,
    TextDocumentSyncCapability, TextDocumentSyncKind, TextDocumentSyncOptions,
    TextDocumentSyncSaveOptions,
};

use crate::converters::encoding::{negotiated_encoding, PositionEncoding, WideEncoding};

use super::semantic_tokens_full::TokenType;

pub fn handle_initialize(
    InitializeParams {
        capabilities: client_capabilities,
        client_info,
        ..
    }: InitializeParams,
) -> Result<InitializeResult, tower_lsp::jsonrpc::Error> {
    let _p = tracing::debug_span!("handle_initialize").entered();

    if let Some(ClientInfo { name, version }) = client_info {
        let version = version.unwrap_or_default();
        tracing::info!("Client {name} version: {version}",);
    }

    Ok(InitializeResult {
        server_info: Some(ServerInfo {
            name: String::from("Tombi LSP"),
            version: Some(crate::version().to_string()),
        }),
        capabilities: server_capabilities(&client_capabilities),
    })
}

pub fn server_capabilities(client_capabilities: &ClientCapabilities) -> ServerCapabilities {
    ServerCapabilities {
        position_encoding: Some(match negotiated_encoding(client_capabilities) {
            PositionEncoding::Utf8 => PositionEncodingKind::UTF8,
            PositionEncoding::Wide(wide) => match wide {
                WideEncoding::Utf16 => PositionEncodingKind::UTF16,
                WideEncoding::Utf32 => PositionEncodingKind::UTF32,
            },
        }),
        text_document_sync: Some(TextDocumentSyncCapability::Options(
            TextDocumentSyncOptions {
                open_close: Some(true),
                change: Some(TextDocumentSyncKind::INCREMENTAL),
                save: Some(SaveOptions::default().into()),
                ..Default::default()
            },
        )),
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        document_on_type_formatting_provider: Some(DocumentOnTypeFormattingOptions {
            first_trigger_character: "=".to_string(),
            more_trigger_character: Some(vec![
                ".".to_owned(),
                "[".to_owned(),
                "{".to_owned(),
                "(".to_owned(),
            ]),
        }),
        // completion_provider: Some(CompletionOptions {
        //     trigger_characters: Some(vec![
        //         ".".into(),
        //         "=".into(),
        //         "[".into(),
        //         "{".into(),
        //         ",".into(),
        //         "'".into(),
        //         "\"".into(),
        //     ]),
        //     completion_item: Some(CompletionOptionsCompletionItem {
        //         label_details_support: (|| -> _ {
        //             client_capabilities
        //                 .text_document
        //                 .as_ref()?
        //                 .completion
        //                 .as_ref()?
        //                 .completion_item
        //                 .as_ref()?
        //                 .label_details_support
        //         })(),
        //     }),
        //     ..Default::default()
        // }),
        // declaration_provider: Some(DeclarationCapability::Simple(true)),
        // definition_provider: Some(OneOf::Left(true)),
        // type_definition_provider: Some(TypeDefinitionProviderCapability::Simple(true)),
        // implementation_provider: Some(ImplementationProviderCapability::Simple(true)),
        // references_provider: Some(OneOf::Left(true)),
        document_symbol_provider: Some(OneOf::Left(true)),
        // workspace_symbol_provider: Some(OneOf::Left(true)),
        // code_lens_provider: Some(CodeLensOptions {
        //     resolve_provider: Some(true),
        // }),
        document_formatting_provider: Some(OneOf::Left(true)),
        // selection_range_provider: Some(SelectionRangeProviderCapability::Simple(true)),
        // folding_range_provider: Some(FoldingRangeProviderCapability::Simple(true)),
        // rename_provider: Some(OneOf::Right(RenameOptions {
        //     prepare_provider: Some(true),
        //     work_done_progress_options: WorkDoneProgressOptions {
        //         work_done_progress: None,
        //     },
        // })),
        // workspace: Some(WorkspaceServerCapabilities {
        //     workspace_folders: Some(WorkspaceFoldersServerCapabilities {
        //         supported: Some(true),
        //         change_notifications: Some(OneOf::Left(true)),
        //     }),
        //     file_operations: None,
        // }),
        // call_hierarchy_provider: Some(CallHierarchyServerCapability::Simple(true)),
        semantic_tokens_provider: Some(
            SemanticTokensOptions {
                legend: SemanticTokensLegend {
                    token_types: TokenType::LEGEND.to_vec(),
                    token_modifiers: vec![SemanticTokenModifier::READONLY],
                },
                full: Some(SemanticTokensFullOptions::Bool(true)),
                ..Default::default()
            }
            .into(),
        ),
        // inlay_hint_provider: Some(OneOf::Right(InlayHintServerCapabilities::Options(
        //     InlayHintOptions {
        //         work_done_progress_options: Default::default(),
        //         resolve_provider: Some(true),
        //     },
        // ))),
        diagnostic_provider: Some(DiagnosticServerCapabilities::Options(DiagnosticOptions {
            ..Default::default()
        })),

        ..Default::default()
    }
}
