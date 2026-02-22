use tombi_syntax::{SyntaxKind::*, T};

use super::{Parse, TS_LINE_END, invalid_line};
use crate::{
    ErrorKind::*,
    parser::Parser,
    support::{leading_comments, peek_leading_comments, trailing_comment},
    token_set::TS_NEXT_SECTION,
};

impl Parse for tombi_ast::Root {
    fn parse(p: &mut Parser<'_>) {
        let m = p.start();

        loop {
            while p.eat(LINE_BREAK) {}

            Vec::<tombi_ast::DanglingCommentGroup>::parse(p);

            let n = peek_leading_comments(p);
            if p.nth_at_ts(n, TS_NEXT_SECTION) {
                break;
            }

            tombi_ast::KeyValueGroup::parse(p);
            if !p.at_ts(TS_LINE_END) {
                invalid_line(p, ExpectedLineBreak);
            }
        }

        loop {
            while p.eat(LINE_BREAK) {}

            let n = peek_leading_comments(p);
            if p.nth_at(n, EOF) {
                break;
            } else if p.nth_at(n, T!("[[")) {
                tombi_ast::ArrayOfTable::parse(p);
            } else if p.nth_at(n, T!['[']) {
                tombi_ast::Table::parse(p);
            } else {
                unknwon_line(p);
            }
        }

        m.complete(p, ROOT);
    }
}

fn unknwon_line(p: &mut Parser<'_>) {
    let m = p.start();

    leading_comments(p);

    while !p.at_ts(TS_LINE_END) {
        p.bump_any();
    }
    p.error(crate::Error::new(UnknownLine, p.current_range()));

    trailing_comment(p);

    m.complete(p, ERROR);
}

#[cfg(test)]
mod test {
    use crate::test_parser;

    test_parser! {
        #[test]
        fn parses_root_begin_dangling_comments(
            r#"
            # begin dangling_comment1
            # begin dangling_comment2

            # table leading comment1
            # table leading comment2
            [table]
            "#
        ) -> Ok(_)
    }

    test_parser! {
        #[test]
        fn parses_root_dangling_comment(
            r#"
            # dangling comment
            "#
        ) -> Ok(_)
    }

    test_parser! {
        #[test]
        fn parses_root_dangling_comment_groups(
            r#"
            # dangling comment group 1
            # dangling comment group 1

            # dangling comment group 2
            # dangling comment group 2


            # dangling comment group 3
            # dangling comment group 3
            "#
        ) -> Ok(_)
    }

    test_parser! {
        #[test]
        fn parses_root_key_value_group_and_dangling_comment_groups(
            r#"
            key1 = "value1"
            key2 = "value2"
            # dangling comment group 1
            # dangling comment group 1

            # dangling comment group 2
            # dangling comment group 2

            key3 = "value3"
            key4 = "value4"

            # leading comment 1
            # leading comment 1
            key5 = "value5"
            # leading comment 2
            key6 = "value6"

            # dangling comment group 3
            # dangling comment group 3
            "#
        ) -> Ok(_)
    }

    test_parser! {
        #[test]
        fn parses_root_key_values_then_tables(
            r#"
            key1 = "value1"
            key2 = "value2"

            [table1]
            key3 = "value3"

            [[array_of_table1]]
            key4 = "value4"
            "#
        ) -> Ok(_)
    }

    test_parser! {
        #[test]
        fn parses_root_dangling_comments_then_tables(
            r#"
            # dangling comment group 1
            # dangling comment group 1

            # dangling comment group 2
            # dangling comment group 2

            # table leading comment
            [table1]
            key1 = "value1"
            "#
        ) -> Ok(_)
    }
}
