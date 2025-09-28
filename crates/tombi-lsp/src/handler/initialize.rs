use tower_lsp::lsp_types::{
    ClientCapabilities, ClientInfo, CodeActionProviderCapability, CompletionOptions,
    CompletionOptionsCompletionItem, DeclarationCapability, DiagnosticOptions,
    DiagnosticServerCapabilities, DocumentLinkOptions, FileOperationFilter, FileOperationPattern,
    FileOperationPatternKind, FileOperationRegistrationOptions, FoldingRangeProviderCapability,
    HoverProviderCapability, InitializeParams, InitializeResult, MessageType, OneOf,
    PositionEncodingKind, SemanticTokensFullOptions, SemanticTokensLegend, SemanticTokensOptions,
    ServerCapabilities, ServerInfo, TextDocumentSyncCapability, TextDocumentSyncKind,
    TextDocumentSyncOptions, TextDocumentSyncSaveOptions, TypeDefinitionProviderCapability,
    WorkDoneProgressOptions, WorkspaceFileOperationsServerCapabilities,
    WorkspaceFoldersServerCapabilities, WorkspaceServerCapabilities,
};

use crate::{
    backend::{BackendCapabilities, DiagnosticType},
    semantic_tokens::SUPPORTED_TOKEN_TYPES,
    Backend,
};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn handle_initialize(
    backend: &Backend,
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

    tracing::info!("Loading config...");
    if let Err(error) = backend.config_manager.load().await {
        let error_message = error.to_string();

        tracing::error!("{error_message}");

        backend
            .client
            .show_message(MessageType::ERROR, error_message)
            .await;
    }

    let mut backend_capabilities = backend.capabilities.write().await;
    if let Some(text_document_capabilities) = client_capabilities.text_document.as_ref() {
        if let Some(diagnostic_capabilities) = text_document_capabilities.diagnostic.as_ref() {
            if diagnostic_capabilities.dynamic_registration == Some(true) {
                backend_capabilities.diagnostic_type = DiagnosticType::Pull;
            }
        }
    }

    Ok(InitializeResult {
        server_info: Some(ServerInfo {
            name: String::from("Tombi LSP"),
            version: Some(env!("CARGO_PKG_VERSION").to_string()),
        }),
        capabilities: server_capabilities(client_capabilities, &backend_capabilities),
    })
}

pub fn server_capabilities(
    client_capabilities: ClientCapabilities,
    backend_capabilities: &BackendCapabilities,
) -> ServerCapabilities {
    let toml_file_operation_filter = FileOperationFilter {
        scheme: Some("file".to_string()),
        pattern: FileOperationPattern {
            glob: "**/*.toml".to_string(),
            matches: Some(FileOperationPatternKind::File),
            ..Default::default()
        },
    };

    let workspace = client_capabilities.workspace.and_then(|workspace| {
        if workspace.workspace_folders == Some(true) {
            Some(WorkspaceServerCapabilities {
                workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                    supported: Some(true),
                    change_notifications: Some(OneOf::Left(true)),
                }),
                file_operations: Some(WorkspaceFileOperationsServerCapabilities {
                    did_create: Some(FileOperationRegistrationOptions {
                        filters: vec![toml_file_operation_filter.clone()],
                    }),
                    did_rename: Some(FileOperationRegistrationOptions {
                        filters: vec![toml_file_operation_filter.clone()],
                    }),
                    did_delete: Some(FileOperationRegistrationOptions {
                        filters: vec![toml_file_operation_filter],
                    }),
                    ..Default::default()
                }),
            })
        } else {
            None
        }
    });

    ServerCapabilities {
        position_encoding: Some(PositionEncodingKind::UTF16),
        workspace,
        text_document_sync: Some(TextDocumentSyncCapability::Options(
            TextDocumentSyncOptions {
                open_close: Some(true),
                change: Some(TextDocumentSyncKind::FULL),
                save: Some(TextDocumentSyncSaveOptions::Supported(true)),
                ..Default::default()
            },
        )),
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
        completion_provider: Some(CompletionOptions {
            trigger_characters: Some(vec![
                ".".into(),
                ",".into(),
                "=".into(),
                ":".into(), // for schema directive `#:schema ...`
                "[".into(),
                "{".into(),
                " ".into(),
                "\"".into(),
                "'".into(),
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
        declaration_provider: Some(DeclarationCapability::Simple(true)),
        definition_provider: Some(OneOf::Left(true)),
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
        diagnostic_provider: if backend_capabilities.diagnostic_type == DiagnosticType::Pull {
            Some(DiagnosticServerCapabilities::Options(DiagnosticOptions {
                inter_file_dependencies: false,
                workspace_diagnostics: true,
                ..Default::default()
            }))
        } else {
            None
        },
        ..Default::default()
    }
}
