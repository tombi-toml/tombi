use std::borrow::Cow;

use itertools::Either;
use tombi_config::TomlVersion;
use tombi_diagnostic::{Diagnostic, SetDiagnostics};
use tombi_document_tree::IntoDocumentTreeAndErrors;
use url::Url;

use crate::lint::Lint;

pub struct Linter<'a> {
    toml_version: TomlVersion,
    options: Cow<'a, crate::LintOptions>,
    source_text: Cow<'a, str>,
    source_url_or_path: Option<Either<&'a Url, &'a std::path::Path>>,
    schema_store: &'a tombi_schema_store::SchemaStore,
    pub(crate) diagnostics: Vec<crate::Diagnostic>,
}

impl<'a> Linter<'a> {
    pub fn new(
        toml_version: TomlVersion,
        options: &'a crate::LintOptions,
        source_url_or_path: Option<Either<&'a Url, &'a std::path::Path>>,
        schema_store: &'a tombi_schema_store::SchemaStore,
    ) -> Self {
        Self {
            toml_version,
            options: Cow::Borrowed(options),
            source_text: Cow::Borrowed(""),
            source_url_or_path,
            schema_store,
            diagnostics: Vec::new(),
        }
    }

    pub async fn lint(mut self, source: &str) -> Result<(), Vec<Diagnostic>> {
        self.source_text = Cow::Borrowed(source);
        let source_schema = if let Some(parsed) =
            tombi_parser::parse_document_header_comments(source).cast::<tombi_ast::Root>()
        {
            match self
                .schema_store
                .resolve_source_schema_from_ast(&parsed.tree(), self.source_url_or_path)
                .await
            {
                Ok(Some(schema)) => Some(schema),
                Ok(None) => None,
                Err((err, range)) => {
                    self.diagnostics.push(Diagnostic::new_error(
                        err.to_string(),
                        err.code(),
                        range,
                    ));
                    None
                }
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
            .unwrap_or(self.toml_version);

        let (root, errors) = tombi_parser::parse(source, toml_version).into_root_and_errors();
        for error in errors {
            error.set_diagnostics(&mut self.diagnostics);
        }

        root.lint(&mut self);

        if self.diagnostics.is_empty() {
            let (document_tree, errors) = root.into_document_tree_and_errors(toml_version).into();

            errors.set_diagnostics(&mut self.diagnostics);

            if let Some(source_schema) = source_schema {
                let schema_context = tombi_schema_store::SchemaContext {
                    toml_version,
                    root_schema: source_schema.root_schema.as_ref(),
                    sub_schema_url_map: Some(&source_schema.sub_schema_url_map),
                    store: self.schema_store,
                };
                if let Err(schema_diagnostics) =
                    tombi_validator::validate(document_tree, &source_schema, &schema_context).await
                {
                    self.diagnostics.extend(schema_diagnostics);
                }
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
