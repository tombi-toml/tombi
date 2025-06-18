use std::sync::Arc;

use ahash::AHashMap;
use itertools::Either;
use tombi_config::{Config, TomlVersion};
use tombi_diagnostic::{Diagnostic, SetDiagnostics};
use tombi_document_tree::TryIntoDocumentTree;
use tombi_schema_store::SourceSchema;
use tombi_syntax::SyntaxNode;
use tower_lsp::{
    lsp_types::{
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
    },
    LanguageServer,
};

use crate::{
    document::DocumentSource,
    goto_definition::into_definition_locations,
    handler::{
        handle_associate_schema, handle_code_action, handle_completion, handle_diagnostic,
        handle_did_change, handle_did_change_configuration, handle_did_change_watched_files,
        handle_did_close, handle_did_open, handle_did_save, handle_document_link,
        handle_document_symbol, handle_folding_range, handle_formatting, handle_get_toml_version,
        handle_goto_declaration, handle_goto_definition, handle_goto_type_definition, handle_hover,
        handle_initialize, handle_initialized, handle_semantic_tokens_full, handle_shutdown,
        handle_update_config, handle_update_schema, AssociateSchemaParams, GetTomlVersionResponse,
    },
};

#[derive(Debug)]
pub struct Backend {
    #[allow(dead_code)]
    pub client: tower_lsp::Client,
    pub document_sources: Arc<tokio::sync::RwLock<AHashMap<Url, DocumentSource>>>,
    pub config_path: Option<std::path::PathBuf>,
    config: Arc<tokio::sync::RwLock<Config>>,
    pub schema_store: tombi_schema_store::SchemaStore,
}

#[derive(Debug, Clone, Default)]
pub struct Options {
    pub offline: Option<bool>,
}

impl Backend {
    #[inline]
    pub fn new(client: tower_lsp::Client, options: &Options) -> Self {
        let (config, config_path) = match serde_tombi::config::load_with_path() {
            Ok((config, config_path)) => (config, config_path),
            Err(err) => {
                tracing::error!("{err}");
                (Config::default(), None)
            }
        };

        let options = tombi_schema_store::Options {
            offline: options.offline,
            strict: config.schema.as_ref().and_then(|schema| schema.strict()),
        };

        Self {
            client,
            document_sources: Default::default(),
            config_path,
            config: Arc::new(tokio::sync::RwLock::new(config)),
            schema_store: tombi_schema_store::SchemaStore::new_with_options(options),
        }
    }

    #[inline]
    async fn get_parsed(
        &self,
        text_document_uri: &Url,
    ) -> Option<tombi_parser::Parsed<SyntaxNode>> {
        let document_source = self.document_sources.read().await;
        let document_source = match document_source.get(text_document_uri) {
            Some(document_source) => document_source,
            None => {
                tracing::warn!("document not found: {}", text_document_uri);
                return None;
            }
        };

        let source_schema = if let Some(parsed) =
            tombi_parser::parse_document_header_comments(&document_source.text)
                .cast::<tombi_ast::Root>()
        {
            match self
                .schema_store
                .resolve_source_schema_from_ast(
                    &parsed.tree(),
                    Some(Either::Left(text_document_uri)),
                )
                .await
            {
                Ok(Some(schema)) => Some(schema),
                Ok(None) => None,
                Err(_) => None,
            }
        } else {
            None
        };

        let toml_version = source_schema
            .as_ref()
            .and_then(|schema| {
                schema
                    .root_schema
                    .as_ref()
                    .and_then(|root| root.toml_version())
            })
            .unwrap_or(self.config.read().await.toml_version.unwrap_or_default());

        Some(tombi_parser::parse(&document_source.text, toml_version))
    }

    #[inline]
    pub async fn get_incomplete_ast(&self, text_document_uri: &Url) -> Option<tombi_ast::Root> {
        self.get_parsed(text_document_uri)
            .await?
            .cast::<tombi_ast::Root>()
            .map(|root| root.tree())
    }

    #[inline]
    pub async fn try_get_ast(
        &self,
        text_document_uri: &Url,
    ) -> Option<Result<tombi_ast::Root, Vec<Diagnostic>>> {
        Some(
            self.get_parsed(text_document_uri)
                .await?
                .try_into_root()
                .map_err(|errors| {
                    let mut diagnostics = vec![];
                    for error in errors {
                        error.set_diagnostics(&mut diagnostics);
                    }

                    diagnostics
                }),
        )
    }

    #[inline]
    pub async fn get_incomplete_document_tree(
        &self,
        text_document_uri: &Url,
    ) -> Option<tombi_document_tree::DocumentTree> {
        let root = self.get_incomplete_ast(text_document_uri).await?;

        let source_schema = self
            .schema_store
            .resolve_source_schema_from_ast(&root, Some(Either::Left(text_document_uri)))
            .await
            .ok()
            .flatten();

        let (toml_version, _) = self.source_toml_version(source_schema.as_ref()).await;

        root.try_into_document_tree(toml_version).ok()
    }

    #[inline]
    pub async fn config(&self) -> Config {
        self.config.read().await.clone()
    }

    #[inline]
    pub async fn update_workspace_config(&self, workspace_config_url: &Url, config: Config) {
        tracing::info!("Updated workspace config: {workspace_config_url}");

        *self.config.write().await = config;
    }

    pub async fn source_toml_version(
        &self,
        source_schema: Option<&SourceSchema>,
    ) -> (TomlVersion, &'static str) {
        if let Some(SourceSchema {
            root_schema: Some(document_schema),
            ..
        }) = source_schema
        {
            if let Some(toml_version) = document_schema.toml_version() {
                return (toml_version, "schema");
            }
        }

        if let Some(toml_version) = self.config.read().await.toml_version {
            return (toml_version, "config");
        }

        (TomlVersion::default(), "default")
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
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
        handle_did_change_watched_files(params).await
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
        handle_completion(self, params).await.map(|response| {
            response
                .map(|items| CompletionResponse::Array(items.into_iter().map(Into::into).collect()))
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
        handle_hover(self, params)
            .await
            .map(|response| response.map(Into::into))
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

    async fn diagnostic(
        &self,
        params: DocumentDiagnosticParams,
    ) -> Result<DocumentDiagnosticReportResult, tower_lsp::jsonrpc::Error> {
        handle_diagnostic(self, params).await
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
        into_definition_locations(self, handle_goto_type_definition(self, params).await?).await
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
}

impl Backend {
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
}
