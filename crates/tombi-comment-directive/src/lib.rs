mod error;
mod root;

pub use error::Error;
pub use root::RootCommentDirective;
use tombi_ast::AstNode;
use tombi_diagnostic::SetDiagnostics;
use tombi_document::IntoDocument;
use tombi_document_tree::IntoDocumentTreeAndErrors;
use tombi_toml_version::TomlVersion;

#[cfg(feature = "serde")]
pub fn get_root_comment_directive(root: &tombi_ast::Root) -> Option<RootCommentDirective> {
    try_get_root_comment_directive(root).ok().flatten()
}

#[cfg(feature = "serde")]
pub fn try_get_root_comment_directive(
    root: &tombi_ast::Root,
) -> Result<Option<RootCommentDirective>, Vec<tombi_diagnostic::Diagnostic>> {
    use serde::Deserialize;

    let mut total_diagnostics = Vec::new();
    if let Some(tombi_directives) = root.tombi_directives() {
        const COMMENT_DIRECTIVE_TOML_VERSION: TomlVersion = TomlVersion::V1_0_0;

        for (tombi_directive, tombi_directive_range) in tombi_directives {
            let parsed = tombi_parser::parse(&tombi_directive, COMMENT_DIRECTIVE_TOML_VERSION);
            // Check if there are any parsing errors
            if !parsed.errors.is_empty() {
                let mut diagnostics = Vec::new();
                for error in parsed.errors {
                    error.set_diagnostics(&mut diagnostics);
                }
                total_diagnostics.extend(diagnostics.into_iter().map(|diagnostic| {
                    into_directive_diagnostic(&diagnostic, tombi_directive_range)
                }));
                continue;
            }
            let root =
                tombi_ast::Root::cast(parsed.syntax_node()).expect("AST Root must be present");

            let (document_tree, errors) = root
                .into_document_tree_and_errors(COMMENT_DIRECTIVE_TOML_VERSION)
                .into();

            // Check for errors during document tree construction
            if !errors.is_empty() {
                let mut diagnostics = Vec::new();
                for error in errors {
                    error.set_diagnostics(&mut diagnostics);
                }
                total_diagnostics.extend(diagnostics.into_iter().map(|diagnostic| {
                    into_directive_diagnostic(&diagnostic, tombi_directive_range)
                }));
            }

            if let Ok(directive) = RootCommentDirective::deserialize(
                &document_tree.into_document(COMMENT_DIRECTIVE_TOML_VERSION),
            ) {
                return Ok(Some(directive));
            }
        }
    }

    if !total_diagnostics.is_empty() {
        return Err(total_diagnostics);
    } else {
        Ok(None)
    }
}

fn into_directive_diagnostic(
    diagnostic: &tombi_diagnostic::Diagnostic,
    tombi_directive_range: tombi_text::Range,
) -> tombi_diagnostic::Diagnostic {
    tombi_diagnostic::Diagnostic::new_warning(
        diagnostic.message(),
        diagnostic.code(),
        tombi_text::Range::new(
            tombi_directive_range.start
                + tombi_text::RelativePosition::from(diagnostic.range().start),
            tombi_directive_range.start
                + tombi_text::RelativePosition::from(diagnostic.range().end),
        ),
    )
}
