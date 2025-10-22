use tombi_syntax::{SyntaxKind::*, T};

use crate::{
    parse::{
        begin_dangling_comments, end_dangling_comments, leading_comments, peek_leading_comments,
        trailing_comment, Parse,
    },
    parser::Parser,
    ErrorKind::*,
};

impl Parse for tombi_ast::Array {
    fn parse(p: &mut Parser<'_>) {
        let m = p.start();

        leading_comments(p);

        assert!(p.at(T!['[']));

        p.eat(T!['[']);

        begin_dangling_comments(p);

        loop {
            while p.eat(LINE_BREAK) {}

            let n = peek_leading_comments(p);
            if p.nth_at(n, EOF) || p.nth_at(n, T![']']) {
                break;
            }

            tombi_ast::Value::parse(p);

            let n = peek_leading_comments(p);
            if p.nth_at(n, T![,]) {
                tombi_ast::Comma::parse(p);
            } else if !p.nth_at(n, T![']']) {
                p.error(crate::Error::new(ExpectedComma, p.current_range()));
                p.bump_any();
            }
        }

        end_dangling_comments(p, true);

        if !p.eat(T![']']) {
            p.error(crate::Error::new(ExpectedBracketEnd, p.current_range()));
        }

        trailing_comment(p);

        m.complete(p, ARRAY);
    }
}

#[cfg(test)]
mod test {
    use crate::{test_parser, ErrorKind::*};

    test_parser! {
        #[test]
        fn empty_array("key = []") -> Ok(_)
    }

    test_parser! {
        #[test]
        fn number_array("key = [1, 2]") -> Ok(_)
    }

    test_parser! {
        #[test]
        fn number_array_with_trailing_comma("key = [1, 2,]") -> Ok(_)
    }

    test_parser! {
        #[test]
        fn array_only_key("key = [key]") -> Err([
            SyntaxError(ExpectedValue, 0:7..0:10),
        ])
    }

    test_parser! {
        #[test]
        fn array_only_key_dot("key = [key.]") -> Err([
            SyntaxError(ExpectedValue, 0:7..0:10),
            SyntaxError(ExpectedComma, 0:10..0:11),
        ])
    }
}
