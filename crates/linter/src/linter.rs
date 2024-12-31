use std::borrow::Cow;

use crate::lint::Lint;
use ast::AstNode;
use config::TomlVersion;
use diagnostic::Diagnostic;
use diagnostic::SetDiagnostics;
use document_tree::TryIntoDocumentTree;
use itertools::Either;
use url::Url;

pub struct Linter<'a> {
    toml_version: TomlVersion,
    options: Cow<'a, crate::LintOptions>,
    schema_url_or_path: Option<Either<&'a Url, &'a std::path::Path>>,
    schema_store: &'a schema_store::SchemaStore,
    diagnostics: Vec<crate::Diagnostic>,
}

impl<'a> Linter<'a> {
    #[inline]
    pub fn new(
        toml_version: TomlVersion,
        options: &'a crate::LintOptions,
        schema_url_or_path: Option<Either<&'a Url, &'a std::path::Path>>,
        schema_store: &'a schema_store::SchemaStore,
    ) -> Self {
        Self {
            toml_version,
            options: Cow::Borrowed(options),
            schema_url_or_path,
            schema_store,
            diagnostics: Vec::new(),
        }
    }

    pub async fn lint(mut self, source: &str) -> Result<(), Vec<Diagnostic>> {
        let schema = match self.schema_url_or_path {
            Some(schema_url_or_path) => self.schema_store.get_schema(schema_url_or_path).await,
            None => None,
        };

        self.toml_version = schema
            .map(|s| s.toml_version())
            .flatten()
            .unwrap_or(self.toml_version);

        let p = parser::parse(source, self.toml_version);
        let mut errors = vec![];

        for err in p.errors() {
            err.set_diagnostic(&mut errors);
        }

        if errors.is_empty() {
            let Some(root) = ast::Root::cast(p.into_syntax_node()) else {
                unreachable!("Root node is always present");
            };

            root.lint(&mut self);

            if let Err(errs) = root.try_into_document_tree(self.toml_version) {
                for err in errs {
                    err.set_diagnostic(&mut errors);
                }
            }

            errors.extend(self.into_diagnostics());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
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
    pub(crate) fn into_diagnostics(self) -> Vec<crate::Diagnostic> {
        self.diagnostics
    }

    #[inline]
    pub(crate) fn add_diagnostic(&mut self, diagnostic: crate::Diagnostic) {
        self.diagnostics.push(diagnostic);
    }
}
