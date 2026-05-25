use std::sync::Arc;

use tombi_diagnostic::SetDiagnostics;
use tombi_text::{EncodingKind, LineIndex};

use tombi_document_tree::IntoDocumentTreeAndErrors;

#[derive(Debug, Clone)]
pub struct DocumentSource {
    /// The text of the document.
    text: Arc<str>,

    line_index: Arc<LineIndex>,

    /// The version of the document.
    ///
    /// If the file has never been opened in the editor, None will be entered.
    pub version: Option<i32>,

    pub toml_version: tombi_config::TomlVersion,

    /// Parsed AST (always exists, even with errors)
    ast: Arc<tombi_ast::Root>,

    /// AST generation errors (empty if no errors)
    ast_errors: Vec<tombi_diagnostic::Diagnostic>,

    /// Parsed DocumentTree (always exists)
    document_tree: Arc<tombi_document_tree::DocumentTree>,

    /// DocumentTree generation errors (empty if no errors)
    document_tree_errors: Vec<tombi_diagnostic::Diagnostic>,
}

impl DocumentSource {
    pub fn new(
        text: impl Into<String>,
        version: Option<i32>,
        toml_version: tombi_config::TomlVersion,
        encoding_kind: EncodingKind,
    ) -> Self {
        let text: Arc<str> = Arc::<str>::from(text.into());

        let (ast, errors) = tombi_parser::parse(text.as_ref()).into_root_and_errors();

        // Convert parser errors to diagnostics
        let mut ast_errors = Vec::with_capacity(errors.len());
        for error in errors {
            error.set_diagnostics(&mut ast_errors);
        }

        // Create DocumentTree from AST and collect DocumentTree errors
        let (document_tree, errors) = ast
            .clone()
            .into_document_tree_and_errors(toml_version)
            .into();

        let mut document_tree_errors = Vec::with_capacity(errors.len());
        for error in errors {
            error.set_diagnostics(&mut document_tree_errors);
        }

        Self {
            line_index: Arc::new(LineIndex::from_arc(Arc::clone(&text), encoding_kind)),
            text,
            version,
            toml_version,
            ast: Arc::new(ast),
            ast_errors,
            document_tree: Arc::new(document_tree),
            document_tree_errors,
        }
    }

    pub fn text(&self) -> &str {
        self.text.as_ref()
    }

    pub fn text_arc(&self) -> Arc<str> {
        Arc::clone(&self.text)
    }

    pub fn set_text(&mut self, text: impl Into<String>, toml_version: tombi_config::TomlVersion) {
        self.text = Arc::<str>::from(text.into());
        self.toml_version = toml_version;
        self.line_index = Arc::new(LineIndex::from_arc(
            Arc::clone(&self.text),
            self.line_index.encoding_kind,
        ));

        // Re-parse the text and collect errors
        let (ast, errors) = tombi_parser::parse(self.text.as_ref()).into_root_and_errors();
        self.ast = Arc::new(ast);

        // Convert parser errors to diagnostics
        self.ast_errors = Vec::with_capacity(errors.len());
        for error in errors {
            error.set_diagnostics(&mut self.ast_errors);
        }

        let (document_tree, errors) = self
            .ast
            .as_ref()
            .clone()
            .into_document_tree_and_errors(toml_version)
            .into();
        self.document_tree = Arc::new(document_tree);
        self.document_tree_errors = Vec::with_capacity(errors.len());
        for error in errors {
            error.set_diagnostics(&mut self.document_tree_errors);
        }
    }

    pub fn line_index(&self) -> &LineIndex {
        self.line_index.as_ref()
    }

    pub fn line_index_arc(&self) -> Arc<LineIndex> {
        Arc::clone(&self.line_index)
    }

    /// Get the parsed AST (always exists)
    pub fn ast(&self) -> Arc<tombi_ast::Root> {
        Arc::clone(&self.ast)
    }

    /// Get AST generation errors
    pub fn ast_errors(&self) -> &[tombi_diagnostic::Diagnostic] {
        &self.ast_errors
    }

    /// Get the parsed DocumentTree (always exists)
    pub fn document_tree(&self) -> Arc<tombi_document_tree::DocumentTree> {
        Arc::clone(&self.document_tree)
    }

    /// Get DocumentTree generation errors
    pub fn document_tree_errors(&self) -> &[tombi_diagnostic::Diagnostic] {
        &self.document_tree_errors
    }
}

#[cfg(test)]
mod tests {
    use tombi_config::TomlVersion;
    use tombi_text::EncodingKind;

    use super::DocumentSource;

    #[test]
    fn line_index_arc_keeps_original_text_alive() {
        let mut document_source = DocumentSource::new(
            "name = \"before\"\nversion = \"1.0.0\"",
            Some(1),
            TomlVersion::default(),
            EncodingKind::Utf16,
        );
        let line_index = document_source.line_index_arc();

        document_source.set_text("name = \"after\"", TomlVersion::default());

        assert_eq!(line_index.line_text(1), Some("version = \"1.0.0\""));
    }
}
