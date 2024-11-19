use ast::AstNode;

use crate::Format;
use std::fmt::Write;

use super::comment::{BeginDanglingComment, EndDanglingComment, LeadingComment, TailingComment};

impl Format for ast::Array {
    fn fmt(&self, f: &mut crate::Formatter) -> Result<(), std::fmt::Error> {
        if self.should_be_multiline() {
            format_multiline_array(self, f)
        } else {
            format_singleline_array(self, f)
        }
    }
}

fn format_multiline_array(
    array: &ast::Array,
    f: &mut crate::Formatter,
) -> Result<(), std::fmt::Error> {
    for comment in array.leading_comments() {
        LeadingComment(comment).fmt(f)?;
    }

    write!(f, "{}[{}", f.ident(), f.line_ending())?;

    f.inc_ident();

    array
        .inner_begin_dangling_comments()
        .map(BeginDanglingComment)
        .collect::<Vec<_>>()
        .fmt(f)?;

    for (i, (value, comma)) in array.values_with_comma().enumerate() {
        // value format
        {
            if i > 0 {
                write!(f, "{}", f.line_ending())?;
            }
            value.fmt(f)?;
        }

        // comma format
        {
            let (comma_leading_comments, comma_tailing_comment) = match comma {
                Some(comma) => (
                    comma.leading_comments().collect::<Vec<_>>(),
                    comma.tailing_comment(),
                ),
                None => (vec![], None),
            };

            if !comma_leading_comments.is_empty() {
                write!(f, "{}", f.line_ending())?;
                for comment in comma_leading_comments {
                    LeadingComment(comment).fmt(f)?;
                }
                write!(f, "{},", f.ident())?;
            } else if value.tailing_comment().is_some() {
                write!(f, "{}{},", f.line_ending(), f.ident())?;
            } else {
                write!(f, ",")?;
            }

            if let Some(comment) = comma_tailing_comment {
                TailingComment(comment).fmt(f)?;
            }
        }
    }

    array
        .inner_end_dangling_comments()
        .map(EndDanglingComment)
        .collect::<Vec<_>>()
        .fmt(f)?;

    f.dec_ident();

    write!(f, "{}{}]", f.line_ending(), f.ident())?;

    if let Some(comment) = array.tailing_comment() {
        TailingComment(comment).fmt(f)?;
    }

    Ok(())
}

fn format_singleline_array(
    array: &ast::Array,
    f: &mut crate::Formatter,
) -> Result<(), std::fmt::Error> {
    write!(
        f,
        "{}[{}",
        f.ident(),
        f.defs().singleline_array_bracket_inner_space()
    )?;

    f.with_reset_ident(|f| {
        let values = array.values().collect::<Vec<_>>();
        for (i, value) in values.iter().enumerate() {
            if i > 0 {
                write!(f, ",{}", f.defs().singleline_array_space_after_comma())?;
            }
            value.fmt(f)?;
        }

        Ok(())
    })?;

    write!(f, "{}]", f.defs().singleline_array_bracket_inner_space())?;

    if let Some(comment) = array.tailing_comment() {
        TailingComment(comment).fmt(f)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use rstest::rstest;

    crate::test_format! {
        #[test]
        fn singleline_array1(
            "array=[1,2,3]"
        ) -> Ok("array = [1, 2, 3]");

        #[test]
        fn singleline_array2(
            "array=[ 1 ]"
        ) -> Ok("array = [1]");

        #[test]
        fn singleline_array3(
            "array=[ 1, 2, 3 ]"
        ) -> Ok("array = [1, 2, 3]");

        #[test]
        fn singleline_array4(
            r#"colors = [ "red", "yellow", "green" ]"#
        ) -> Ok(r#"colors = ["red", "yellow", "green"]"#);

        #[test]
        fn singleline_array5(
            "nested_arrays_of_ints = [ [ 1, 2 ], [ 3, 4, 5 ] ]"
        ) -> Ok("nested_arrays_of_ints = [[1, 2], [3, 4, 5]]");

        #[test]
        fn singleline_array6(
            r#"nested_mixed_array = [ [ 1, 2 ], [ "a", "b", "c" ] ]"#
        ) -> Ok(r#"nested_mixed_array = [[1, 2], ["a", "b", "c"]]"#);

        #[test]
        fn singleline_array7(
            r#"string_array = [ "all", 'strings', """are the same""", '''type''' ]"#
        ) -> Ok(r#"string_array = ["all", 'strings', """are the same""", '''type''']"#);

        #[test]
        fn multiline_array1(
            "array = [1, 2, 3,]"
        ) -> Ok(
            r#"
            array = [
              1,
              2,
              3,
            ]
            "#
        );

        #[test]
        fn multiline_array2(
            "array = [1, ]"
        ) -> Ok(
            r#"
            array = [
              1,
            ]
            "#
        );

        #[test]
        // NOTE: Currently, This test is collect.
        //       In the future, by inserting a layer that rewrites the ast before formatting,
        //       when there is no value tailing comment and there is a comma tailing comment,
        //       we will add logic to swap them.
        fn multiline_array3(
            r#"
            array = [
              1  # comment
            ]
            "#
        ) -> Ok(
            r#"
            array = [
              1  # comment
              ,
            ]
            "#
        );

        #[test]
        fn multiline_array4(
            r#"
            array = [
              1,  # comment
            ]
            "#
        ) -> Ok(
            r#"
            array = [
              1,  # comment
            ]
            "#
        );

        #[test]
        fn multiline_array_with_full_comment(
            r#"
            # array leading comment1
            # array leading comment2
            array = [

              # inner array begin dangling comment1
              # inner array begin dangling comment2

              # value1 leading comment1
              # value1 leading comment2
              1 # value1 trailing comment
              , # value1 comma tailing comment
              2 # value2 trailing comment
              # value2 comma leading comment1
              # value2 comma leading comment2
              , # value2 comma tailing comment
              # value3 leading comment1
              # value3 leading comment2
              3 # value3 trailing comment
              # array end dangling comment1

              # array end dangling comment2

            ] # array tailing comment
            "#
        ) -> Ok(
            r#"
            # array leading comment1
            # array leading comment2
            array = [
              # inner array begin dangling comment1
              # inner array begin dangling comment2

              # value1 leading comment1
              # value1 leading comment2
              1  # value1 trailing comment
              ,  # value1 comma tailing comment
              2  # value2 trailing comment
              # value2 comma leading comment1
              # value2 comma leading comment2
              ,  # value2 comma tailing comment
              # value3 leading comment1
              # value3 leading comment2
              3  # value3 trailing comment
              ,

              # array end dangling comment1
              # array end dangling comment2
            ]  # array tailing comment
            "#
        );

        #[test]
        fn nested_multiline_array(
            "array = [ [1,2,3,], [4,5,6], [7,8,9,] ]"
        ) -> Ok(
            r#"
            array = [
              [
                1,
                2,
                3,
              ],
              [4, 5, 6],
              [
                7,
                8,
                9,
              ],
            ]
            "#
        );
    }

    #[rstest]
    #[case("[1, 2, 3,]", true)]
    #[case("[1, 2, 3]", false)]
    fn has_tailing_comma_after_last_value(#[case] source: &str, #[case] expected: bool) {
        let p = parser::parse_as::<ast::Array>(source);
        assert_eq!(p.errors(), []);

        let ast = ast::Array::cast(p.syntax_node()).unwrap();
        assert_eq!(ast.has_tailing_comma_after_last_value(), expected);
    }
}
