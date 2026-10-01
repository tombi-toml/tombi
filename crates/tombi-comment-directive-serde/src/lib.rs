use serde::Deserialize;
use tombi_ast_syntax::AstNode as _;
use tombi_comment_directive::{
    TOMBI_COMMENT_DIRECTIVE_TOML_VERSION, TombiCommentDirectiveImpl,
    value::TombiValueDirectiveContent,
};
use tombi_document::IntoDocument;
use tombi_document_tree_syntax::TryIntoDocumentTree;

pub fn get_comment_directive_content<FormatRules, LintRules>(
    comment_directives: impl IntoIterator<Item = tombi_ast_syntax::TombiValueCommentDirective>,
) -> Option<TombiValueDirectiveContent<FormatRules, LintRules>>
where
    FormatRules: serde::de::DeserializeOwned,
    LintRules: serde::de::DeserializeOwned,
    TombiValueDirectiveContent<FormatRules, LintRules>: TombiCommentDirectiveImpl,
{
    let mut total_document_tree_table: Option<tombi_document_tree_syntax::Table> = None;

    // The document tree borrows the contents, the parse results and the decoded strings.
    let contents = comment_directives
        .into_iter()
        .map(|directive| directive.content)
        .collect::<Vec<_>>();
    let parsed = contents
        .iter()
        .map(|content| tombi_parser::parse(content))
        .collect::<Vec<_>>();
    let roots = parsed
        .iter()
        .map(|parsed| parsed.try_root().ok())
        .collect::<Option<Vec<_>>>()?;
    let decoded = roots
        .iter()
        .map(|root| root.decode_strings(TOMBI_COMMENT_DIRECTIVE_TOML_VERSION))
        .collect::<Vec<_>>();

    for (root, decoded) in roots.into_iter().zip(&decoded) {
        let document_tree = root
            .try_into_document_tree(TOMBI_COMMENT_DIRECTIVE_TOML_VERSION, decoded)
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
