use tombi_syntax::{SyntaxKind::*, T};

use super::Parse;
use crate::{
    parse::{
        begin_dangling_comments, end_dangling_comments, invalid_line, leading_comments,
        peek_leading_comments, tailing_comment, TS_LINE_END,
    },
    parser::Parser,
    token_set::TS_NEXT_SECTION,
    ErrorKind::*,
};

impl Parse for tombi_ast::ArrayOfTable {
    fn parse(p: &mut Parser<'_>) {
        let m = p.start();

        leading_comments(p);

        assert!(p.at(T!("[[")));

        p.eat(T!("[["));

        tombi_ast::Keys::parse(p);

        if !p.eat(T!("]]")) {
            invalid_line(p, ExpectedDoubleBracketEnd);
        }

        tailing_comment(p);

        if !p.at_ts(TS_LINE_END) {
            invalid_line(p, ExpectedLineBreak);
        }
        p.eat(LINE_BREAK);

        begin_dangling_comments(p);

        loop {
            while p.eat(LINE_BREAK) {}
            let n = peek_leading_comments(p);

            if p.nth_at_ts(n, TS_NEXT_SECTION) {
                break;
            }

            tombi_ast::KeyValue::parse(p);

            if !p.at_ts(TS_LINE_END) {
                invalid_line(p, ExpectedLineBreak);
            }
        }

        end_dangling_comments(p, false);

        // NOTE: For easier calculation of the table range
        //       from the cursor position in the editor,
        //       consume whitespace until the next section.
        while p.eat(LINE_BREAK) {}

        m.complete(p, ARRAY_OF_TABLE);
    }
}

#[cfg(test)]
mod test {
    use crate::{test_parser, ErrorKind::*};

    test_parser! {
        #[test]
        fn invalid_array_of_table1(
            r#"
            [[]]
            key1 = 1
            key2 = 2
            "#
        ) -> Err([SyntaxError(ExpectedKey, 0:2..0:3)])
    }

    test_parser! {
        #[test]
        fn invalid_array_of_table2(
            r#"
            [[aaa.]]
            key1 = 1
            key2 = 2
            "#
        ) -> Err([SyntaxError(ForbiddenKeysLastPeriod, 0:6..0:7)])
    }

    test_parser! {
        #[test]
        fn invalid_array_of_table3(
            r#"
            [[aaa.bbb
            key1 = 1
            key2 = 2
            "#
        ) -> Err([SyntaxError(ExpectedDoubleBracketEnd, 0:9..1:0)])
    }

    test_parser! {
        #[test]
        fn invalid_array_of_table4(
            r#"
            [[aaa.bbb]
            key1 = 1
            key2 = 2
            "#
        ) -> Err([SyntaxError(ExpectedDoubleBracketEnd, 0:9..0:10)])
    }

    test_parser! {
        #[test]
        fn invalid_array_of_table5(
            r#"
            [[aaa.bbb]]
            key1 = 1 INVALID COMMENT
            key2 = 2
            "#
        ) -> Err([SyntaxError(ExpectedLineBreak, 1:9..1:16)])
    }
}
