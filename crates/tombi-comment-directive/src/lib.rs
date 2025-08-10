mod error;
mod root;

pub use error::Error;
pub use root::RootCommentDirective;
use tombi_ast::AstNode;
use tombi_document::IntoDocument;
use tombi_document_tree::IntoDocumentTreeAndErrors;
use tombi_toml_version::TomlVersion;

#[cfg(feature = "serde")]
pub fn get_root_comment_directive(root: &tombi_ast::Root) -> Option<RootCommentDirective> {
    use serde::Deserialize;

    if let Some(tombi_directives) = root.tombi_directives() {
        const COMMENT_DIRECTIVE_TOML_VERSION: TomlVersion = TomlVersion::V1_0_0;

        for (tombi_directive, _) in tombi_directives {
            let parsed = tombi_parser::parse(&tombi_directive, COMMENT_DIRECTIVE_TOML_VERSION);
            // Check if there are any parsing errors
            if !parsed.errors.is_empty() {
                continue;
            }
            let root =
                tombi_ast::Root::cast(parsed.syntax_node()).expect("AST Root must be present");

            let (document_tree, errors) = root
                .into_document_tree_and_errors(COMMENT_DIRECTIVE_TOML_VERSION)
                .into();

            // Check for errors during document tree construction
            if !errors.is_empty() {
                continue;
            }
            return RootCommentDirective::deserialize(
                &document_tree.into_document(COMMENT_DIRECTIVE_TOML_VERSION),
            )
            .ok();
        }
    }
    None
}
