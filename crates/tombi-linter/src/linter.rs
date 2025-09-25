use std::borrow::Cow;

use itertools::Either;
use tombi_config::TomlVersion;
use tombi_diagnostic::{Diagnostic, SetDiagnostics};
use tombi_document_tree::IntoDocumentTreeAndErrors;

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
        let (source_schema, tombi_document_comment_directive) = if let Some(parsed) =
            tombi_parser::parse_document_header_comments(source).cast::<tombi_ast::Root>()
        {
            let root = parsed.tree();
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
        } else {
            (None, None)
        };

        if let Some(tombi_document_comment_directive) = &tombi_document_comment_directive {
            if let Some(lint) = &tombi_document_comment_directive.lint {
                if lint.disabled() == Some(true) {
                    // Only skip linting if there are no validation errors
                    if self.diagnostics.is_empty() {
                        match self.source_uri_or_path.map(|path| match path {
                            Either::Left(url) => url.to_string(),
                            Either::Right(path) => path.to_string_lossy().to_string(),
                        }) {
                            Some(source_url_or_path) => {
                                tracing::info!(
                                    "Skip linting for \"{source_url_or_path}\" due to `lint.disable`"
                                );
                            }
                            None => {
                                tracing::info!("Skip linting for stdin due to `lint.disable`");
                            }
                        }
                        return Ok(());
                    }
                }
            }
        }

        self.toml_version = tombi_document_comment_directive
            .as_ref()
            .and_then(|directive| directive.toml_version)
            .unwrap_or_else(|| {
                source_schema
                    .as_ref()
                    .and_then(|schema| {
                        schema
                            .root_schema
                            .as_ref()
                            .and_then(|root| root.toml_version())
                    })
                    .unwrap_or(self.toml_version)
            });

        let (root, errors) = tombi_parser::parse(source, self.toml_version).into_root_and_errors();
        for error in errors {
            error.set_diagnostics(&mut self.diagnostics);
        }

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

            tracing::trace!("document_tree: {:#?}", document_tree);

            let schema_context = tombi_schema_store::SchemaContext {
                toml_version: self.toml_version,
                root_schema: source_schema
                    .as_ref()
                    .and_then(|source_schema| source_schema.root_schema.as_ref()),
                sub_schema_uri_map: source_schema
                    .as_ref()
                    .map(|source_schema| &source_schema.sub_schema_uri_map),
                store: self.schema_store,
                strict: tombi_document_comment_directive
                    .as_ref()
                    .and_then(|directive| {
                        directive.schema.as_ref().and_then(|schema| schema.strict)
                    }),
            };

            if let Err(diagnostics) =
                tombi_validator::validate(document_tree, source_schema.as_ref(), &schema_context)
                    .await
            {
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
}
