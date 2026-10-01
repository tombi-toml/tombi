use tombi_ast_syntax::AstNode;
use tombi_document_tree_syntax::TryIntoDocumentTree;
use tombi_schema_store::SchemaContext;

mod change;
mod edit;
mod rule;

use change::Change;
use edit::Edit;

/// Applies the formatter rewrite rules to `root`.
///
/// Returns the rewritten source, or `None` when nothing has to be rewritten.
/// The caller parses the returned source again, since the new syntax tree
/// has to borrow the new source.
pub(crate) async fn edit<'a>(
    root: tombi_ast_syntax::Root<'_>,
    source_path: Option<&'a std::path::Path>,
    schema_context: &'a SchemaContext<'a>,
) -> Option<String> {
    let decoded = root.decode_strings(schema_context.toml_version);
    let Ok(document_tree) = root.try_into_document_tree(schema_context.toml_version, &decoded)
    else {
        return None;
    };
    let current_schema = schema_context
        .root_schema
        .and_then(|document_schema| document_schema.as_current_schema());

    let document_value = tombi_document_tree_syntax::Value::from(document_tree);
    let changes = root
        .edit(
            &document_value,
            &[],
            source_path,
            current_schema.as_ref(),
            schema_context,
        )
        .await;
    if changes.is_empty() {
        return None;
    }

    match change::apply(root.syntax(), changes) {
        Ok(source) => Some(source),
        Err(error) => {
            log::error!("failed to apply formatter source rewrite: {error:?}");
            None
        }
    }
}
