use std::borrow::Cow;

use itertools::Either;
use tombi_config::TomlVersion;
use tombi_diagnostic::{Diagnostic, SetDiagnostics};
use tombi_document_tree::IntoDocumentTreeAndErrors;
use tombi_severity_level::SeverityLevelDefaultWarn;

use crate::lint::Lint;

pub struct Linter<'a> {
    toml_version: TomlVersion,
    options: Cow<'a, crate::LintOptions>,
    source_text: Cow<'a, str>,
    source_uri_or_path: Option<Either<&'a tombi_uri::Uri, &'a std::path::Path>>,
    schema_store: &'a tombi_schema_store::SchemaStore,
    pub(crate) diagnostics: Vec<tombi_diagnostic::Diagnostic>,
}

impl<'a> Linter<'a> {
    pub fn new(
        toml_version: TomlVersion,
        options: &'a crate::LintOptions,
        source_uri_or_path: Option<Either<&'a tombi_uri::Uri, &'a std::path::Path>>,
        schema_store: &'a tombi_schema_store::SchemaStore,
    ) -> Self {
        Self {
            toml_version,
            options: Cow::Borrowed(options),
            source_text: Cow::Borrowed(""),
            source_uri_or_path,
            schema_store,
            diagnostics: Vec::new(),
        }
    }

    pub async fn lint(mut self, source: &str) -> Result<(), Vec<Diagnostic>> {
        self.source_text = Cow::Borrowed(source);

        let (root, errors) = tombi_parser::parse(source).into_root_and_errors();
        for error in errors {
            error.set_diagnostics(&mut self.diagnostics);
        }

        let (source_schema, tombi_document_comment_directive) = {
            let (source_schema, error_with_range) =
                tombi_schema_store::lint_source_schema_from_ast(
                    &root,
                    self.source_uri_or_path,
                    self.schema_store,
                )
                .await;
            if let Some((err, range)) = error_with_range {
                self.diagnostics
                    .push(tombi_diagnostic::Diagnostic::new_warning(
                        err.to_string(),
                        err.code(),
                        range,
                    ));
            };

            let (tombi_document_comment_directive, diagnostics) =
                tombi_validator::comment_directive::get_tombi_document_comment_directive_and_diagnostics(&root).await;
            self.diagnostics.extend(diagnostics);

            (source_schema, tombi_document_comment_directive)
        };

        if let Some(tombi_document_comment_directive) = &tombi_document_comment_directive
            && let Some(lint) = &tombi_document_comment_directive.lint
            && lint.disabled.unwrap_or_default()
        {
            // Only skip linting if there are no validation errors
            if self.diagnostics.is_empty() {
                match self.source_uri_or_path.map(|path| match path {
                    Either::Left(url) => url.to_string(),
                    Either::Right(path) => path.to_string_lossy().to_string(),
                }) {
                    Some(source_url_or_path) => {
                        log::info!(
                            "Skip linting for \"{source_url_or_path}\" due to `lint.disable`"
                        );
                    }
                    None => {
                        log::info!("Skip linting for stdin due to `lint.disable`");
                    }
                }
                return Ok(());
            }
        }

        self.toml_version = tombi_document_comment_directive
            .as_ref()
            .and_then(|directive| directive.toml_version)
            .unwrap_or_else(|| {
                source_schema
                    .as_ref()
                    .and_then(|schema| schema.toml_version())
                    .unwrap_or(self.toml_version)
            });

        {
            root.lint(&mut self).await;
        }

        if self
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.level() == tombi_diagnostic::Level::ERROR)
            .count()
            == 0
        {
            let (document_tree, errors) =
                root.into_document_tree_and_errors(self.toml_version).into();

            errors.set_diagnostics(&mut self.diagnostics);

            log::trace!("document_tree: {:#?}", document_tree);

            let schema_context = tombi_schema_store::SchemaContext::from_source_schema(
                self.toml_version,
                source_schema.as_ref(),
                self.schema_store,
                tombi_document_comment_directive
                    .as_ref()
                    .and_then(|directive| {
                        directive.schema.as_ref().and_then(|schema| schema.strict)
                    }),
            );

            if let Err(diagnostics) =
                tombi_validator::validate(document_tree, source_schema.as_ref(), &schema_context)
                    .await
            {
                let diagnostics = self.apply_lint_rules_to_diagnostics(diagnostics);
                self.diagnostics.extend(diagnostics);
            }
        }

        if self.diagnostics.is_empty() {
            Ok(())
        } else {
            Err(self.diagnostics)
        }
    }

    pub fn source_text(&self) -> &str {
        self.source_text.as_ref()
    }

    #[inline]
    #[allow(dead_code)]
    pub(crate) fn toml_version(&self) -> TomlVersion {
        self.toml_version
    }

    #[inline]
    #[allow(dead_code)]
    pub(crate) fn options(&self) -> &crate::LintOptions {
        &self.options
    }

    #[inline]
    pub(crate) fn extend_diagnostics(&mut self, diagnostics: impl SetDiagnostics) {
        diagnostics.set_diagnostics(&mut self.diagnostics);
    }

    fn apply_lint_rules_to_diagnostics(&self, diagnostics: Vec<Diagnostic>) -> Vec<Diagnostic> {
        let Some(key_empty_severity) = self
            .options
            .rules
            .as_ref()
            .and_then(|rules| rules.key_empty)
            .filter(|severity| *severity != SeverityLevelDefaultWarn::default())
        else {
            return diagnostics;
        };

        let key_empty_level = tombi_severity_level::SeverityLevel::from(key_empty_severity);

        diagnostics
            .into_iter()
            .filter_map(|d| {
                if d.code() != tombi_validator::DiagnosticKind::KeyEmpty.code() {
                    return Some(d);
                }
                match key_empty_level {
                    tombi_severity_level::SeverityLevel::Off => None,
                    tombi_severity_level::SeverityLevel::Warn => Some(Diagnostic::new_warning(
                        d.message().to_string(),
                        d.code().to_string(),
                        d.range(),
                    )),
                    tombi_severity_level::SeverityLevel::Error => Some(Diagnostic::new_error(
                        d.message().to_string(),
                        d.code().to_string(),
                        d.range(),
                    )),
                }
            })
            .collect()
    }
}
