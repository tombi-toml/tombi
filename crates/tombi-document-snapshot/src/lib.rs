use std::sync::Arc;

use tombi_ast_syntax::AstNode as _;
use tombi_diagnostic::SetDiagnostics;
use tombi_document_tree_syntax::IntoDocumentTreeAndErrors;
use tombi_parser::ParseResult;
use tombi_text::LineIndex;

self_cell::self_cell!(
    /// A text with the result of parsing it, which borrows the text.
    struct ParsedCell {
        owner: Box<str>,

        #[covariant]
        dependent: ParseResult,
    }
);

/// A text and the result of parsing it.
///
/// It is the part of a [`DocumentSnapshot`] that does not depend on the TOML version,
/// so it is shared when only the TOML version changes.
pub struct ParsedText(ParsedCell);

impl ParsedText {
    pub fn parse(text: impl Into<Box<str>>) -> Self {
        Self(ParsedCell::new(text.into(), |text| {
            tombi_parser::parse(text)
        }))
    }

    pub fn text(&self) -> &str {
        self.0.borrow_owner()
    }

    pub fn parsed(&self) -> &ParseResult<'_> {
        self.0.borrow_dependent()
    }

    pub fn root(&self) -> tombi_ast_syntax::Root<'_> {
        self.parsed().root()
    }
}

/// The owner of a [`DocumentTree`](tombi_document_tree_syntax::DocumentTree):
/// what the tree borrows from.
struct TreeOwner {
    parsed: Arc<ParsedText>,
    /// The escaped strings decoded for the TOML version of the tree.
    decoded: tombi_ast_syntax::DecodedTextResolver,
}

/// What is built from the [`TreeOwner`]. It borrows the owner.
struct Analyzed<'a> {
    /// Parsed AST (always exists, even with errors)
    ast: tombi_ast_syntax::Root<'a>,

    /// Parsed DocumentTree (always exists)
    document_tree: tombi_document_tree_syntax::DocumentTree<'a>,

    /// DocumentTree generation errors (empty if no errors)
    document_tree_errors: Vec<tombi_diagnostic::Diagnostic>,
}

self_cell::self_cell!(
    struct AnalyzedCell {
        owner: TreeOwner,

        #[covariant]
        dependent: Analyzed,
    }
);

/// A text, the syntax tree, and the document tree built from it.
///
/// The text and everything built from it are borrowed from the same owner, so a clone of a
/// snapshot shares the owner with one [`Arc`] instead of reference-counting every node.
/// It is `'static`, so it can be moved into a future, which a borrowed tree cannot.
#[derive(Clone)]
pub struct DocumentSnapshot {
    cell: Arc<AnalyzedCell>,

    pub toml_version: tombi_config::TomlVersion,

    /// AST generation errors (empty if no errors)
    ast_errors: Arc<[tombi_diagnostic::Diagnostic]>,
}

impl std::fmt::Debug for DocumentSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DocumentSnapshot")
            .field("toml_version", &self.toml_version)
            .finish_non_exhaustive()
    }
}

impl DocumentSnapshot {
    pub fn new(parsed: ParsedText, toml_version: tombi_config::TomlVersion) -> Self {
        Self::from_parsed(Arc::new(parsed), toml_version)
    }

    /// Parses `text` and builds the snapshot of it.
    pub fn parse(text: impl Into<Box<str>>, toml_version: tombi_config::TomlVersion) -> Self {
        Self::new(ParsedText::parse(text), toml_version)
    }

    fn from_parsed(parsed: Arc<ParsedText>, toml_version: tombi_config::TomlVersion) -> Self {
        let mut ast_errors = Vec::with_capacity(parsed.parsed().errors.len());
        for error in parsed.parsed().errors.iter().cloned() {
            error.set_diagnostics(&mut ast_errors);
        }
        let decoded = parsed.root().decode_strings(toml_version);

        Self {
            cell: Arc::new(AnalyzedCell::new(TreeOwner { parsed, decoded }, |owner| {
                let ast = owner.parsed.root();
                let (document_tree, document_tree_errors) =
                    build_document_tree(ast, toml_version, &owner.decoded);
                Analyzed {
                    ast,
                    document_tree,
                    document_tree_errors,
                }
            })),
            toml_version,
            ast_errors: ast_errors.into(),
        }
    }

    /// A snapshot that builds only the document tree for `toml_version`, reusing the parsed AST.
    pub fn with_toml_version(&self, toml_version: tombi_config::TomlVersion) -> Self {
        Self::from_parsed(Arc::clone(&self.cell.borrow_owner().parsed), toml_version)
    }

    pub fn text(&self) -> &str {
        self.parsed_text().text()
    }

    fn parsed_text(&self) -> &ParsedText {
        &self.cell.borrow_owner().parsed
    }

    /// The parse result of the text, to lint or format it without parsing it again.
    pub fn parsed(&self) -> &ParseResult<'_> {
        self.parsed_text().parsed()
    }

    /// The pool of escaped strings that the document tree borrows.
    pub fn decoded(&self) -> &tombi_ast_syntax::DecodedTextResolver {
        &self.cell.borrow_owner().decoded
    }

    pub fn line_index(&self) -> &LineIndex<'_> {
        self.parsed().line_index()
    }

    /// Get the parsed AST (always exists)
    pub fn ast(&self) -> tombi_ast_syntax::Root<'_> {
        self.cell.borrow_dependent().ast
    }

    /// Get AST generation errors
    pub fn ast_errors(&self) -> &[tombi_diagnostic::Diagnostic] {
        &self.ast_errors
    }

    /// Get the parsed DocumentTree (always exists)
    pub fn document_tree(&self) -> &tombi_document_tree_syntax::DocumentTree<'_> {
        &self.cell.borrow_dependent().document_tree
    }

    /// Get DocumentTree generation errors
    pub fn document_tree_errors(&self) -> &[tombi_diagnostic::Diagnostic] {
        &self.cell.borrow_dependent().document_tree_errors
    }
}

fn build_document_tree<'a>(
    ast: tombi_ast_syntax::Root<'a>,
    toml_version: tombi_config::TomlVersion,
    decoded: &'a tombi_ast_syntax::DecodedTextResolver,
) -> (
    tombi_document_tree_syntax::DocumentTree<'a>,
    Vec<tombi_diagnostic::Diagnostic>,
) {
    let (document_tree, errors) = ast
        .into_document_tree_and_errors(toml_version, decoded)
        .into();
    let mut document_tree_errors = Vec::with_capacity(errors.len());
    for error in errors {
        error.set_diagnostics(&mut document_tree_errors);
    }
    (document_tree, document_tree_errors)
}

#[cfg(test)]
mod tests {
    use tombi_config::TomlVersion;

    use super::DocumentSnapshot;

    #[test]
    fn snapshot_keeps_its_own_text_after_the_next_snapshot_is_built() {
        let snapshot = DocumentSnapshot::parse(
            "name = \"before\"\nversion = \"1.0.0\"",
            TomlVersion::default(),
        );
        let next = DocumentSnapshot::parse("name = \"after\"", TomlVersion::default());

        assert_eq!(
            snapshot.line_index().line_text(1),
            Some("version = \"1.0.0\"")
        );
        assert_eq!(next.text(), "name = \"after\"");
    }

    #[test]
    fn toml_version_change_shares_the_parse_result() {
        let snapshot = DocumentSnapshot::parse("name = \"a\"", TomlVersion::default());

        let next = snapshot.with_toml_version(TomlVersion::V1_0_0);

        assert!(std::ptr::eq(snapshot.parsed(), next.parsed()));
        assert_eq!(next.toml_version, TomlVersion::V1_0_0);
    }

    #[test]
    fn snapshot_can_be_moved_to_a_static_future() {
        fn assert_static<T: 'static + Send + Sync>(_: &T) {}

        let snapshot = DocumentSnapshot::parse("name = \"a\"", TomlVersion::default());
        assert_static(&snapshot);
    }
}
