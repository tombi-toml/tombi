use tombi_text::EncodingKind;
use tower_lsp_server::gen_lsp_types::{
    ChangeNotifications, ClientCapabilities, ClientInfo, CodeActionProvider, CompletionOptions,
    DeclarationProvider, DefinitionProvider, DiagnosticOptions, DiagnosticProvider,
    DocumentFormattingProvider, DocumentLinkOptions, DocumentSymbolProvider, FileOperationFilter,
    FileOperationOptions, FileOperationPattern, FileOperationPatternKind,
    FileOperationRegistrationOptions, FoldingRangeProvider, Full, HoverProvider, InitializeParams,
    InitializeResult, InlayHintOptions, InlayHintProvider, ReferencesProvider, Save,
    SemanticTokensLegend, SemanticTokensOptions, ServerCapabilities, ServerCompletionItemOptions,
    ServerInfo, TextDocumentSync, TextDocumentSyncKind, TextDocumentSyncOptions,
    TypeDefinitionProvider, WorkDoneProgressOptions, WorkspaceFoldersServerCapabilities,
    WorkspaceOptions,
};

use crate::{
    Backend,
    backend::{BackendCapabilities, DiagnosticMode},
    initialization_options::InitializationOptions,
    semantic_tokens::SUPPORTED_TOKEN_TYPES,
};

pub async fn handle_initialize(
    backend: &Backend,
    params: InitializeParams,
) -> Result<InitializeResult, tower_lsp_server::jsonrpc::Error> {
    log::info!("handle_initialize");
    log::debug!("{:?}", params);

    let InitializeParams {
        capabilities: client_capabilities,
        client_info,
        initialization_options,
        ..
    } = params;

    if let Some(ClientInfo { name, version }) = client_info {
        let version = version.unwrap_or_default();
        log::info!("{name} version: {version}",);
    }

    let mut backend_capabilities = backend.capabilities.write().await;
    backend_capabilities.encoding_kind = negotiated_wide_encoding(&client_capabilities);
    backend_capabilities.workspace_diagnostic_refresh_support = client_capabilities
        .workspace
        .as_ref()
        .and_then(|workspace| workspace.diagnostics.as_ref())
        .and_then(|diagnostic| diagnostic.refresh_support)
        .unwrap_or_default();
    backend_capabilities.multiline_token_support = client_capabilities
        .text_document
        .as_ref()
        .and_then(|text_document| text_document.semantic_tokens.as_ref())
        .and_then(|semantic_tokens| semantic_tokens.multiline_token_support)
        .unwrap_or_default();
    backend_capabilities.semantic_tokens_settings =
        InitializationOptions::from_value(initialization_options).semantic_tokens;
    if let Some(text_document_capabilities) = client_capabilities.text_document.as_ref()
        && let Some(diagnostic_capabilities) = text_document_capabilities.diagnostic.as_ref()
        && diagnostic_capabilities.dynamic_registration == Some(true)
    {
        backend_capabilities.diagnostic_mode = DiagnosticMode::Pull;
    }

    log::debug!("backend_capabilities: {:?}", backend_capabilities);

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
            Some(WorkspaceOptions {
                workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                    supported: Some(true),
                    change_notifications: Some(ChangeNotifications::Bool(true)),
                }),
                file_operations: Some(FileOperationOptions {
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
                ..Default::default()
            })
        } else {
            None
        }
    });

    ServerCapabilities {
        position_encoding: Some(backend_capabilities.encoding_kind.into()),
        workspace,
        text_document_sync: Some(TextDocumentSync::Options(TextDocumentSyncOptions {
            open_close: Some(true),
            change: Some(TextDocumentSyncKind::Full),
            save: Some(Save::Bool(true)),
            ..Default::default()
        })),
        hover_provider: Some(HoverProvider::Bool(true)),
        inlay_hint_provider: Some(InlayHintProvider::InlayHintOptions(InlayHintOptions {
            resolve_provider: Some(false),
            work_done_progress_options: WorkDoneProgressOptions::default(),
        })),
        code_action_provider: Some(CodeActionProvider::Bool(true)),
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
            ]),
            completion_item: Some(ServerCompletionItemOptions {
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
        references_provider: Some(ReferencesProvider::Bool(true)),
        type_definition_provider: Some(TypeDefinitionProvider::Bool(true)),
        declaration_provider: Some(DeclarationProvider::Bool(true)),
        definition_provider: Some(DefinitionProvider::Bool(true)),
        document_symbol_provider: Some(DocumentSymbolProvider::Bool(true)),
        document_formatting_provider: Some(DocumentFormattingProvider::Bool(true)),
        folding_range_provider: Some(FoldingRangeProvider::Bool(true)),
        semantic_tokens_provider: Some(
            SemanticTokensOptions {
                legend: SemanticTokensLegend {
                    token_types: SUPPORTED_TOKEN_TYPES
                        .iter()
                        .map(|token_type| token_type.to_string())
                        .collect(),
                    token_modifiers: vec![],
                },
                full: Some(Full::Bool(true)),
                ..Default::default()
            }
            .into(),
        ),
        diagnostic_provider: if backend_capabilities.diagnostic_mode == DiagnosticMode::Pull {
            Some(DiagnosticProvider::DiagnosticOptions(DiagnosticOptions {
                identifier: Some("tombi".to_string()),
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

fn negotiated_wide_encoding(client_capabilities: &ClientCapabilities) -> EncodingKind {
    client_capabilities
        .general
        .as_ref()
        .and_then(|general| general.position_encodings.as_ref())
        .and_then(|encodings| {
            encodings
                .iter()
                .filter_map(|encoding| EncodingKind::try_from(encoding).ok())
                .next()
        })
        .unwrap_or_default()
}
