use itertools::{Either, Itertools};
use tombi_glob::{MatchResult, matches_file_patterns};
use tombi_text::{IntoLsp, LineIndex};

use crate::{backend::Backend, config_manager::ConfigSchemaStore};

#[derive(Debug, Clone)]
pub struct DiagnosticsResult {
    pub diagnostics: Vec<tower_lsp::lsp_types::Diagnostic>,
    pub version: Option<i32>,
}

pub async fn get_diagnostics_result(
    backend: &Backend,
    text_document_uri: &tombi_uri::Uri,
) -> Option<DiagnosticsResult> {
    {
        let workspace_diagnostics_cache = backend.workspace_diagnostics_cache.read().await;
        if let Some(diagnostics_result) = workspace_diagnostics_cache.get(text_document_uri) {
            return Some(diagnostics_result.clone());
        }
    }

    let ConfigSchemaStore {
        config,
        schema_store,
        config_path,
    } = backend
        .config_manager
        .config_schema_store_for_uri(text_document_uri)
        .await;

    if !config
        .lsp
        .as_ref()
        .and_then(|lsp| lsp.diagnostic.as_ref())
        .and_then(|diagnostic| diagnostic.enabled)
        .unwrap_or_default()
        .value()
    {
        log::debug!("`lsp.diagnostic.enabled` is false");
        return None;
    }

    if let Ok(text_document_path) = tombi_uri::Uri::to_file_path(text_document_uri) {
        match matches_file_patterns(&text_document_path, config_path.as_deref(), &config) {
            MatchResult::Matched => {}
            MatchResult::IncludeNotMatched => {
                log::info!("Skip {text_document_path:?} because it is not in config.files.include");
                return None;
            }
            MatchResult::ExcludeMatched => {
                log::info!("Skip {text_document_path:?} because it is in config.files.exclude");
                return None;
            }
        }
    }

    let (text, version, toml_version, encoding_kind) = {
        let Ok(document_sources) = backend.document_sources.try_read() else {
            return None;
        };
        let document_source = document_sources.get(text_document_uri)?;
        (
            document_source.text_arc(),
            document_source.version,
            document_source.toml_version,
            document_source.line_index().encoding_kind,
        )
    };

    // Get lint options with override support
    let text_document_path = text_document_uri.to_file_path().ok();
    let Some(lint_options) = tombi_glob::get_lint_options(
        &config,
        text_document_path.as_deref(),
        config_path.as_deref(),
    ) else {
        log::debug!("Linting disabled for {:?} by override", text_document_path);
        return None;
    };

    let diagnostics = match tombi_linter::Linter::new(
        toml_version,
        &lint_options,
        Some(Either::Left(text_document_uri)),
        &schema_store,
    )
    .lint(text.as_ref())
    .await
    {
        Ok(_) => Vec::new(),
        Err(diagnostics) => {
            let line_index = LineIndex::new(text.as_ref(), encoding_kind);
            diagnostics
                .into_iter()
                .unique()
                .map(|diagnostic| diagnostic.into_lsp(&line_index))
                .collect_vec()
        }
    };

    let diagnostics_result = DiagnosticsResult {
        diagnostics,
        version,
    };

    {
        let mut workspace_diagnostics_cache = backend.workspace_diagnostics_cache.write().await;
        workspace_diagnostics_cache.set(text_document_uri.clone(), diagnostics_result.clone());
    }

    Some(diagnostics_result)
}
