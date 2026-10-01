use tombi_ast_syntax::AstToken;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ParseError {
    #[error(transparent)]
    String(#[from] tombi_toml_text::ParseError),
}

pub(crate) fn try_from_comment(value: &str) -> Result<String, ParseError> {
    let comment = tombi_toml_text::parse_literal_string(&value[1..], false)?;
    Ok(comment.into_owned())
}

pub(crate) fn try_new_comment(
    node: &tombi_ast_syntax::Comment<'_>,
) -> Result<String, crate::Error> {
    try_from_comment(node.syntax().text()).map_err(|error| crate::Error::ParseCommentError {
        error,
        span: node.syntax().span(),
    })
}
