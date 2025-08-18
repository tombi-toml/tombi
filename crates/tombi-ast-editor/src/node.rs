use tombi_parser::parse_as;
use tombi_toml_version::TomlVersion;

pub fn make_comma() -> tombi_syntax::SyntaxNode {
    parse_as::<tombi_ast::Comma>(",", TomlVersion::default()).into_syntax_node_mut()
}

pub fn make_comma_with_trailing_comment(
    trailing_comment: &tombi_ast::TrailingComment,
) -> tombi_syntax::SyntaxNode {
    parse_as::<tombi_ast::Comma>(
        &format!(",{}", trailing_comment.syntax().text()),
        TomlVersion::default(),
    )
    .into_syntax_node_mut()
}
