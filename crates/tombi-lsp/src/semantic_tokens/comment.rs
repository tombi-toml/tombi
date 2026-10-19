use tombi_ast_syntax::{AstToken, RegionMarker};

use crate::semantic_tokens::{AppendSemanticTokens, SemanticTokensBuilder, TokenType};

impl AppendSemanticTokens for tombi_ast_syntax::Comment<'_> {
    fn append_semantic_tokens(&self, builder: &mut SemanticTokensBuilder) {
        append_comment_tokens(self, builder, true);
    }
}

impl AppendSemanticTokens for tombi_ast_syntax::TrailingComment<'_> {
    fn append_semantic_tokens(&self, builder: &mut SemanticTokensBuilder) {
        append_comment_tokens(self.as_ref(), builder, false);
    }
}

fn append_comment_tokens(
    comment: &tombi_ast_syntax::Comment<'_>,
    builder: &mut SemanticTokensBuilder,
    is_standalone: bool,
) {
    if let Some(tombi_value_directive) = comment.get_tombi_value_directive() {
        builder.add_comment_directive(comment, tombi_value_directive.directive_span);
    } else if is_standalone && let Some(region_marker) = comment.region_marker() {
        let comment_text = comment.syntax().text();
        let marker_start = comment_text.len() - comment_text[1..].trim_start().len();
        let marker_len = match region_marker {
            RegionMarker::Start => "region".len(),
            RegionMarker::End => "endregion".len(),
        };
        let comment_start = comment.syntax().span().start;
        builder.add_comment_directive(
            comment,
            tombi_text::Span::new(
                comment_start + marker_start as u32,
                comment_start + (marker_start + marker_len) as u32,
            ),
        );
    } else {
        builder.add_token(TokenType::COMMENT, comment.syntax().span());
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn region_markers_are_keywords() {
        let source = "#:tombi toml-version = \"v1.0.0\"\n\n# region dependencies\n[dependencies]\nvalue = 1 # region inline\n# endregion dependencies\n# regionish\n# tombi: lint.rules = {}\n";
        let parsed = tombi_parser::parse(source);
        let mut builder = SemanticTokensBuilder::new(
            tombi_uri::Uri::from_str("file:///test.toml").unwrap(),
            parsed.line_index(),
            tombi_text::EncodingKind::Utf16,
        );
        parsed.root().append_semantic_tokens(&mut builder);

        let mut line = 0;
        let mut column = 0;
        let keywords: Vec<_> = builder
            .build()
            .into_iter()
            .filter_map(|token| {
                line += token.delta_line;
                column = if token.delta_line == 0 {
                    column + token.delta_start
                } else {
                    token.delta_start
                };
                (token.token_type == TokenType::KEYWORD as u32).then_some((line, column, token.length))
            })
            .collect();

        assert_eq!(keywords, [(0, 1, 6), (2, 2, 6), (5, 2, 9), (7, 2, 6)]);
    }

    #[test]
    fn region_markers_without_descriptions_have_no_empty_tokens() {
        let source = "# region\n# endregion\n";
        let parsed = tombi_parser::parse(source);
        let mut builder = SemanticTokensBuilder::new(
            tombi_uri::Uri::from_str("file:///test.toml").unwrap(),
            parsed.line_index(),
            tombi_text::EncodingKind::Utf16,
        );
        parsed.root().append_semantic_tokens(&mut builder);

        let tokens: Vec<_> = builder
            .build()
            .into_iter()
            .map(|token| (token.token_type, token.length))
            .collect();

        assert_eq!(
            tokens,
            [
                (TokenType::COMMENT as u32, 2),
                (TokenType::KEYWORD as u32, 6),
                (TokenType::COMMENT as u32, 2),
                (TokenType::KEYWORD as u32, 9),
            ]
        );
    }
}
