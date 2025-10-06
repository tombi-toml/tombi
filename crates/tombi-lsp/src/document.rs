use tombi_diagnostic::SetDiagnostics;
use tombi_text::{EncodingKind, LineIndex};

use tombi_document_tree::IntoDocumentTreeAndErrors;

#[derive(Debug, Clone)]
pub struct DocumentSource {
    /// The text of the document.
    text: String,

    line_index: LineIndex<'static>,

    /// The version of the document.
    ///
    /// If the file has never been opened in the editor, None will be entered.
    pub version: Option<i32>,

    pub toml_version: tombi_config::TomlVersion,

    /// Parsed AST (always exists, even with errors)
    ast: tombi_ast::Root,

    /// AST generation errors (empty if no errors)
    ast_errors: Vec<tombi_diagnostic::Diagnostic>,

    /// Parsed DocumentTree (always exists)
    document_tree: tombi_document_tree::DocumentTree,

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
        let text = text.into();

        let (ast, errors) = tombi_parser::parse(&text, toml_version).into_root_and_errors();

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

        let text_ref = unsafe { std::mem::transmute::<&str, &'static str>(text.as_str()) };

        Self {
            text,
            line_index: LineIndex::new(text_ref, encoding_kind),
            version,
            toml_version,
            ast,
            ast_errors,
            document_tree,
            document_tree_errors,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn set_text(&mut self, text: impl Into<String>, toml_version: tombi_config::TomlVersion) {
        self.text = text.into();
        let text_ref = unsafe { std::mem::transmute::<&str, &'static str>(self.text.as_str()) };
        self.toml_version = toml_version;
        self.line_index = LineIndex::new(text_ref, self.line_index.encoding_kind);

        // Re-parse the text and collect errors
        let (ast, errors) = tombi_parser::parse(&self.text, toml_version).into_root_and_errors();
        self.ast = ast;

        // Convert parser errors to diagnostics
        self.ast_errors = Vec::with_capacity(errors.len());
        for error in errors {
            error.set_diagnostics(&mut self.ast_errors);
        }

        let (document_tree, errors) = self
            .ast
            .clone()
            .into_document_tree_and_errors(toml_version)
            .into();
        self.document_tree = document_tree;
        self.document_tree_errors = Vec::with_capacity(errors.len());
        for error in errors {
            error.set_diagnostics(&mut self.document_tree_errors);
        }
    }

    pub fn line_index(&self) -> &LineIndex<'static> {
        &self.line_index
    }

    /// Get the parsed AST (always exists)
    pub fn ast(&self) -> &tombi_ast::Root {
        &self.ast
    }

    /// Get AST generation errors
    pub fn ast_errors(&self) -> &[tombi_diagnostic::Diagnostic] {
        &self.ast_errors
    }

    /// Get the parsed DocumentTree (always exists)
    pub fn document_tree(&self) -> &tombi_document_tree::DocumentTree {
        &self.document_tree
    }

    /// Get DocumentTree generation errors
    pub fn document_tree_errors(&self) -> &[tombi_diagnostic::Diagnostic] {
        &self.document_tree_errors
    }
}
