#[derive(Debug, Clone, Copy)]
pub enum StringKind {
    Basic,
    MultiLineBasic,
    Literal,
    MultiLineLiteral,
}

#[derive(Debug, Clone)]
pub struct StringNode<'a> {
    pub kind: StringKind,
    pub value: &'a str,
    pub syntax: &'a lexer::SyntaxElement,
}

impl<'a> crate::FromSyntax<'a> for StringNode<'a> {
    fn from_syntax(syntax: &'a lexer::SyntaxElement) -> Result<Self, Vec<crate::Error>> {
        use lexer::Token::*;

        let kind = match syntax.kind() {
            BASIC_STRING => StringKind::Basic,
            MULTI_LINE_BASIC_STRING => StringKind::MultiLineBasic,
            LITERAL_STRING => StringKind::Literal,
            MULTI_LINE_LITERAL_STRING => StringKind::MultiLineLiteral,
            _ => unreachable!("invalid string kind: {syntax:#?}"),
        };

        if let Some(value) = syntax.as_token().map(|t| t.text()) {
            Ok(Self {
                kind,
                value,
                syntax,
            })
        } else {
            Err(vec![crate::Error::InvalidStringValue {
                syntax: syntax.clone(),
            }])
        }
    }
}
