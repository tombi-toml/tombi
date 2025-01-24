use crate::Format;
use itertools::Itertools;
use std::fmt::Write;

impl Format for ast::ArrayOfTables {
    fn fmt(&self, f: &mut crate::Formatter) -> Result<(), std::fmt::Error> {
        let header = self.header().unwrap();

        self.header_leading_comments().collect::<Vec<_>>().fmt(f)?;

        write!(f, "[[{header}]]")?;

        if let Some(comment) = self.header_tailing_comment() {
            comment.fmt(f)?;
        }

        let key_values = self.key_values().collect_vec();

        if key_values.is_empty() {
            let dangling_comments = self.dangling_comments();

            if !dangling_comments.is_empty() {
                write!(f, "{}", f.line_ending())?;
                dangling_comments.fmt(f)?;
            }

            return Ok(());
        } else {
            write!(f, "{}", f.line_ending())?;

            self.begin_dangling_comments().fmt(f)?;

            for (i, key_value) in key_values.into_iter().enumerate() {
                if i != 0 {
                    write!(f, "{}", f.line_ending())?;
                }
                key_value.fmt(f)?;
            }

            self.end_dangling_comments().fmt(f)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::test_format;

    test_format! {
        #[test]
        fn array_of_table_only_header(
            r#"[[package]]"#
        ) -> Ok(source);
    }

    test_format! {
        #[test]
        fn array_of_table_only_header_with_basic_string_key(
            r#"[[dependencies."unicase"]]"#
        ) -> Ok(source);
    }

    test_format! {
        #[test]
        fn array_of_table_only_header_nexted_keys(
            r#"[[dependencies.unicase]]"#
        ) -> Ok(source);
    }

    test_format! {
        #[test]
        fn array_of_table(
            r#"
            [[package]]
            name = "toml-rs"
            version = "0.4.0"
            "#
        ) -> Ok(source);
    }

    test_format! {
        #[test]
        fn array_of_table_with_full_comment1(
            r#"
            # header leading comment1
            # header leading comment2
            [[header]]  # header tailing comment
            # table begin dangling comment group 1-1
            # table begin dangling comment group 1-2

            # table begin dangling comment group 2-1
            # table begin dangling comment group 2-2
            # table begin dangling comment group 2-3

            # table begin dangling comment group 3-1

            # key value leading comment1
            # key value leading comment2
            key = "value"  # key tailing comment
            "#
        ) -> Ok(source);
    }
}
