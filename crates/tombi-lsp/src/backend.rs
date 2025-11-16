use std::sync::Arc;

use ahash::AHashMap;
use itertools::Either;
use tombi_comment_directive::document::TombiDocumentDirectiveContent;
use tombi_config::{Config, TomlVersion};
use tombi_text::IntoLsp;
use tower_lsp::lsp_types::{
    request::{
        GotoDeclarationParams, GotoDeclarationResponse, GotoTypeDefinitionParams,
        GotoTypeDefinitionResponse,
    },
    CodeActionParams, CodeActionResponse, CompletionParams, CompletionResponse,
    DidChangeConfigurationParams, DidChangeTextDocumentParams, DidChangeWatchedFilesParams,
    DidCloseTextDocumentParams, DidOpenTextDocumentParams, DidSaveTextDocumentParams,
    DocumentDiagnosticParams, DocumentDiagnosticReportResult, DocumentLink, DocumentLinkParams,
    DocumentSymbolParams, DocumentSymbolResponse, FoldingRange, FoldingRangeParams,
    GotoDefinitionParams, GotoDefinitionResponse, Hover, HoverParams, InitializeParams,
    InitializeResult, InitializedParams, SemanticTokensParams, SemanticTokensResult,
    TextDocumentIdentifier, Url,
};

use crate::{
    config_manager::{ConfigManager, ConfigSchemaStore},
    document::DocumentSource,
    goto_definition::into_definition_locations,
    goto_type_definition::into_type_definition_locations,
    handler::{
        handle_associate_schema, handle_code_action, handle_completion, handle_diagnostic,
        handle_did_change, handle_did_change_configuration, handle_did_change_watched_files,
        handle_did_close, handle_did_open, handle_did_save, handle_document_link,
        handle_document_symbol, handle_folding_range, handle_formatting, handle_get_status,
        handle_get_toml_version, handle_goto_declaration, handle_goto_definition,
        handle_goto_type_definition, handle_hover, handle_initialize, handle_initialized,
        handle_refresh_cache, handle_semantic_tokens_full, handle_shutdown, handle_update_config,
        handle_update_schema, push_diagnostics, AssociateSchemaParams, GetStatusResponse,
        GetTomlVersionResponse, RefreshCacheParams, TomlVersionSource,
    },
};

use tombi_text::EncodingKind;

#[derive(Debug)]
pub struct Backend {
    #[allow(dead_code)]
    pub client: tower_lsp::Client,
    pub capabilities: Arc<tokio::sync::RwLock<BackendCapabilities>>,
    pub document_sources: Arc<tokio::sync::RwLock<AHashMap<tombi_uri::Uri, DocumentSource>>>,
    pub config_manager: Arc<ConfigManager>,
}

#[derive(Debug)]
pub struct BackendCapabilities {
    pub encoding_kind: EncodingKind,
    pub diagnostic_mode: DiagnosticMode,
}

/// Diagnostic Type
///
/// Many editors, such as VSCode, adopt the Pull diagnostic mode, but some specific editors adopt the Push mode.
/// Therefore, it is necessary to support both modes.
///
/// See: https://github.com/tombi-toml/tombi/issues/711
///
/// For WorkspaceDiagnostic, Tombi supports only the Push model in order to avoid CPU spikes.
///
/// See: https://github.com/tombi-toml/tombi/issues/1070
///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticMode {
    Push,
    Pull,
}

#[derive(Debug, Clone, Default)]
pub struct Options {
    pub offline: Option<bool>,
    pub no_cache: Option<bool>,
}

impl Backend {
    #[inline]
    pub fn new(client: tower_lsp::Client, options: &Options) -> Self {
        Self {
            client,
            capabilities: Arc::new(tokio::sync::RwLock::new(BackendCapabilities {
                encoding_kind: EncodingKind::default(),
                diagnostic_mode: DiagnosticMode::Push,
            })),
            document_sources: Default::default(),
            config_manager: Arc::new(ConfigManager::new(options)),
        }
    }

    #[inline]
    pub async fn is_diagnostic_mode_push(&self) -> bool {
        self.capabilities.read().await.diagnostic_mode == DiagnosticMode::Push
    }

    #[inline]
    pub async fn config(&self, text_document_uri: &tombi_uri::Uri) -> Config {
        self.config_manager
            .config_schema_store_for_uri(text_document_uri)
            .await
            .config
    }

    #[inline]
    pub async fn config_path(&self, text_document_uri: &Url) -> Option<std::path::PathBuf> {
        self.config_manager
            .get_config_path_for_url(text_document_uri)
            .await
    }

    #[inline]
    pub async fn text_document_toml_version(
        &self,
        text_document_uri: &tombi_uri::Uri,
        text: &str,
    ) -> TomlVersion {
        self.text_document_toml_version_and_source(text_document_uri, text)
            .await
            .0
    }

    pub async fn text_document_toml_version_and_source(
        &self,
        text_document_uri: &tombi_uri::Uri,
        text: &str,
    ) -> (TomlVersion, TomlVersionSource) {
        let ConfigSchemaStore {
            config,
            schema_store,
            ..
        } = self
            .config_manager
            .config_schema_store_for_uri(text_document_uri)
            .await;

        let source_schema = if let Some(parsed) =
            tombi_parser::parse_document_header_comments(text).cast::<tombi_ast::Root>()
        {
            let root = parsed.tree();
            if let Some(TombiDocumentDirectiveContent {
                toml_version: Some(toml_version),
                ..
            }) = tombi_validator::comment_directive::get_tombi_document_comment_directive(&root)
                .await
            {
                return (toml_version, TomlVersionSource::Comment);
            }

            match schema_store
                .resolve_source_schema_from_ast(&root, Some(Either::Left(text_document_uri)))
                .await
            {
                Ok(Some(schema)) => Some(schema),
                Ok(None) => None,
                Err(_) => None,
            }
        } else {
            None
        };

        if let Some(toml_version) = source_schema.as_ref().and_then(|schema| {
            schema
                .root_schema
                .as_ref()
                .and_then(|root| root.toml_version())
        }) {
            return (toml_version, TomlVersionSource::Schema);
        }

        if let Some(toml_version) = config.toml_version {
            return (toml_version, TomlVersionSource::Config);
        }

        (TomlVersion::default(), TomlVersionSource::Default)
    }

    #[inline]
    pub async fn get_status(
        &self,
        params: TextDocumentIdentifier,
    ) -> Result<GetStatusResponse, tower_lsp::jsonrpc::Error> {
        handle_get_status(self, params).await
    }

    #[inline]
    pub async fn get_toml_version(
        &self,
        params: TextDocumentIdentifier,
    ) -> Result<GetTomlVersionResponse, tower_lsp::jsonrpc::Error> {
        handle_get_toml_version(self, params).await
    }

    #[inline]
    pub async fn update_schema(
        &self,
        params: TextDocumentIdentifier,
    ) -> Result<bool, tower_lsp::jsonrpc::Error> {
        handle_update_schema(self, params).await
    }

    #[inline]
    pub async fn update_config(
        &self,
        params: TextDocumentIdentifier,
    ) -> Result<bool, tower_lsp::jsonrpc::Error> {
        handle_update_config(self, params).await
    }

    #[inline]
    pub async fn associate_schema(&self, params: AssociateSchemaParams) {
        handle_associate_schema(self, params).await
    }

    #[inline]
    pub async fn refresh_cache(
        &self,
        params: RefreshCacheParams,
    ) -> Result<bool, tower_lsp::jsonrpc::Error> {
        handle_refresh_cache(self, params).await
    }

    #[inline]
    pub async fn push_diagnostics(&self, text_document_uri: tombi_uri::Uri) {
        push_diagnostics(self, text_document_uri).await
    }
}

#[tower_lsp::async_trait]
impl tower_lsp::LanguageServer for Backend {
    async fn initialize(
        &self,
        params: InitializeParams,
    ) -> Result<InitializeResult, tower_lsp::jsonrpc::Error> {
        handle_initialize(self, params).await
    }

    async fn initialized(&self, params: InitializedParams) {
        handle_initialized(self, params).await
    }

    async fn shutdown(&self) -> Result<(), tower_lsp::jsonrpc::Error> {
        handle_shutdown().await
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        handle_did_open(self, params).await
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        handle_did_close(self, params).await
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        handle_did_change(self, params).await
    }

    async fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams) {
        handle_did_change_watched_files(self, params).await
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        handle_did_save(self, params).await
    }

    async fn did_change_configuration(&self, params: DidChangeConfigurationParams) {
        handle_did_change_configuration(params).await
    }

    async fn completion(
        &self,
        params: CompletionParams,
    ) -> Result<Option<CompletionResponse>, tower_lsp::jsonrpc::Error> {
        let document_source = self.document_sources.read().await;
        let Some(document_source) = document_source.get(
            &params
                .text_document_position
                .text_document
                .uri
                .clone()
                .into(),
        ) else {
            return Ok(None);
        };

        handle_completion(self, params).await.map(|response| {
            response.map(|items| {
                CompletionResponse::Array(
                    items
                        .into_iter()
                        .map(|item| item.into_lsp(document_source.line_index()))
                        .collect(),
                )
            })
        })
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>, tower_lsp::jsonrpc::Error> {
        handle_semantic_tokens_full(self, params).await
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>, tower_lsp::jsonrpc::Error> {
        handle_document_symbol(self, params).await
    }

    async fn document_link(
        &self,
        params: DocumentLinkParams,
    ) -> Result<Option<Vec<DocumentLink>>, tower_lsp::jsonrpc::Error> {
        handle_document_link(self, params).await
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>, tower_lsp::jsonrpc::Error> {
        let document_source = self.document_sources.read().await;
        let Some(document_source) = document_source.get(
            &params
                .text_document_position_params
                .text_document
                .uri
                .clone()
                .into(),
        ) else {
            return Ok(None);
        };
        let line_index = document_source.line_index();

        handle_hover(self, params)
            .await
            .map(|response| response.map(|content| content.into_lsp(line_index)))
    }

    async fn folding_range(
        &self,
        params: FoldingRangeParams,
    ) -> Result<Option<Vec<FoldingRange>>, tower_lsp::jsonrpc::Error> {
        handle_folding_range(self, params).await
    }

    async fn formatting(
        &self,
        params: tower_lsp::lsp_types::DocumentFormattingParams,
    ) -> Result<Option<Vec<tower_lsp::lsp_types::TextEdit>>, tower_lsp::jsonrpc::Error> {
        handle_formatting(self, params).await
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>, tower_lsp::jsonrpc::Error> {
        into_definition_locations(self, handle_goto_definition(self, params).await?).await
    }

    async fn goto_type_definition(
        &self,
        params: GotoTypeDefinitionParams,
    ) -> Result<Option<GotoTypeDefinitionResponse>, tower_lsp::jsonrpc::Error> {
        into_type_definition_locations(self, handle_goto_type_definition(self, params).await?).await
    }

    async fn goto_declaration(
        &self,
        params: GotoDeclarationParams,
    ) -> Result<Option<GotoDeclarationResponse>, tower_lsp::jsonrpc::Error> {
        into_definition_locations(self, handle_goto_declaration(self, params).await?).await
    }

    async fn code_action(
        &self,
        params: CodeActionParams,
    ) -> Result<Option<CodeActionResponse>, tower_lsp::jsonrpc::Error> {
        handle_code_action(self, params).await
    }

    async fn diagnostic(
        &self,
        params: DocumentDiagnosticParams,
    ) -> Result<DocumentDiagnosticReportResult, tower_lsp::jsonrpc::Error> {
        handle_diagnostic(self, params).await
    }
}
