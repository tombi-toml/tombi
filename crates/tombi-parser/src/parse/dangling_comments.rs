use tombi_syntax::SyntaxKind::*;

use crate::{parse::Parse, parser::Parser, token_set::TS_LINE_END};

impl Parse for tombi_ast::DanglingComments {
    fn parse(p: &mut Parser<'_>) {
        let m = p.start();

        loop {
            if p.at(COMMENT) {
                p.bump(COMMENT);
            } else if p.at_ts(TS_LINE_END) {
                if p.nth_at_ts(1, TS_LINE_END) {
                    p.bump_any(); // Consume the first LINE_BREAK
                    break;
                }
                p.bump(LINE_BREAK);
            } else {
                break;
            }
        }

        m.complete(p, DANGLING_COMMENTS);
    }
}

/// Check if we should parse a DANGLING_COMMENTS node
/// Returns true if current position has comments that form a dangling comments group.
/// A dangling comments group must be preceded by either:
/// - At least 2 consecutive LINE_BREAKs (empty line separator), OR
/// - The logical start of the document (only trivia before current comment), OR
/// - A single LINE_BREAK followed by EOF (end of file comments)
pub fn should_parse_dangling_comments(p: &Parser<'_>) -> bool {
    let mut n = 0;

    // Must start with a comment to be a dangling comments group
    loop {
        if p.nth_at(n, COMMENT) {
            n += 1;
        } else if p.nth_at_ts(n, TS_LINE_END) {
            if p.nth_at_ts(n + 1, TS_LINE_END) {
                return true;
            }
            n += 1;
        } else {
            return false;
        }
    }
}
