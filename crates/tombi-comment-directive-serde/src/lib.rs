use serde::Deserialize;
use tombi_comment_directive::{
    value::TombiValueDirectiveContent, TombiCommentDirectiveImpl,
    TOMBI_COMMENT_DIRECTIVE_TOML_VERSION,
};
use tombi_document::IntoDocument;
use tombi_document_tree::TryIntoDocumentTree;

pub fn get_comment_directive_content<FormatRules, LintRules>(
    comment_directives: impl IntoIterator<Item = tombi_ast::TombiValueCommentDirective>,
) -> Option<TombiValueDirectiveContent<FormatRules, LintRules>>
where
    FormatRules: serde::de::DeserializeOwned,
    LintRules: serde::de::DeserializeOwned,
    TombiValueDirectiveContent<FormatRules, LintRules>: TombiCommentDirectiveImpl,
{
    let mut total_document_tree_table: Option<tombi_document_tree::Table> = None;

    for tombi_ast::TombiValueCommentDirective { content, .. } in comment_directives {
        let root = tombi_parser::parse(&content, TOMBI_COMMENT_DIRECTIVE_TOML_VERSION)
            .try_into_root()
            .ok()?;

        let document_tree = root
            .try_into_document_tree(TOMBI_COMMENT_DIRECTIVE_TOML_VERSION)
            .ok()?;

        if let Some(total_document_tree_table) = total_document_tree_table.as_mut() {
            total_document_tree_table.merge(document_tree.into()).ok()?;
        } else {
            total_document_tree_table = Some(document_tree.into());
        }
    }

    total_document_tree_table.and_then(|table| {
        TombiValueDirectiveContent::<FormatRules, LintRules>::deserialize(
            &table.into_document(TOMBI_COMMENT_DIRECTIVE_TOML_VERSION),
        )
        .ok()
    })
}
