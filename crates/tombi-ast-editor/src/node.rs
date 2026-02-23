use itertools::Itertools;
use tombi_ast::AstNode;
use tombi_parser::parse_as;

pub fn make_comma() -> tombi_syntax::SyntaxNode {
    parse_as::<tombi_ast::Comma>(",").into_syntax_node_mut()
}

pub fn make_comma_with_trailing_comment(
    trailing_comment: &tombi_ast::TrailingComment,
) -> tombi_syntax::SyntaxNode {
    parse_as::<tombi_ast::Comma>(&format!(",{}", trailing_comment.syntax().text()))
        .into_syntax_node_mut()
}

pub fn make_dangling_comment_group_from_leading_comments(
    comments: &[tombi_ast::LeadingComment],
) -> Option<tombi_syntax::SyntaxNode> {
    if comments.is_empty() {
        return None;
    }

    let text = comments
        .iter()
        .map(|comment| comment.syntax().text().to_string())
        .join("\n");
    let root = tombi_ast::Root::cast(
        tombi_parser::parse_as::<tombi_ast::Root>(&text).into_syntax_node_mut(),
    )?;
    root.dangling_comment_groups()
        .next()
        .map(|group| group.syntax().clone())
}
