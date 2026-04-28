use std::fmt::Write;

use itertools::Itertools;
use tombi_ast::DanglingCommentGroupOr;

use crate::{Format, format::filter_map_unique_keys};

impl Format for tombi_ast::ArrayOfTable {
    fn format(&self, f: &mut crate::Formatter) -> Result<(), std::fmt::Error> {
        let header = self.header().unwrap();
        let toml_version = f.toml_version();

        if f.indent_sub_tables() {
            filter_map_unique_keys(
                header.keys(),
                self.parent_table_or_array_of_table_keys(toml_version),
                toml_version,
            )
            .for_each(|_| f.inc_indent());
        }

        self.header_leading_comments().collect_vec().format(f)?;

        f.write_indent()?;
        write!(f, "[[{header}]]")?;

        if let Some(comment) = self.header_trailing_comment() {
            comment.format(f)?;
        }

        if f.indent_table_key_values() {
            f.inc_indent();
        }

        let key_value_groups = itertools::chain!(
            self.dangling_comment_groups()
                .map(DanglingCommentGroupOr::DanglingCommentGroup),
            self.key_value_groups()
        )
        .collect_vec();
        if !key_value_groups.is_empty() {
            f.write_line_ending()?;
            key_value_groups.format(f)?;
        }

        if f.indent_table_key_values() {
            f.dec_indent();
        }

        if f.indent_sub_tables() {
            filter_map_unique_keys(
                header.keys(),
                self.parent_table_or_array_of_table_keys(toml_version),
                toml_version,
            )
            .for_each(|_| f.dec_indent());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tombi_config::FormatRules;

    use crate::{Formatter, test_format};

    test_format! {
        #[tokio::test]
        async fn array_of_table_only_header(
            r#"[[package]]"#
        ) -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn array_of_table_only_header_with_basic_string_key(
            r#"[[dependencies."unicase"]]"#
        ) -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn array_of_table_only_header_nested_keys(
            r#"[[dependencies.unicase]]"#
        ) -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn array_of_table(
            r#"
            [[package]]
            name = "toml-rs"
            version = "0.4.0"
            "#
        ) -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn array_of_table_removes_trailing_commas(
            r#"
            [[package]]
            name = "toml-rs",
            version = "0.4.0",
            "#
        ) -> Ok(
            r#"
            [[package]]
            name = "toml-rs"
            version = "0.4.0"
            "#
        )
    }

    test_format! {
        #[tokio::test]
        async fn array_of_table_group_blank_lines_limit_between_dangling_comments_and_first_key_value(
            r#"
            [[dependencies]]
            # aaa



            tombi-schema-store.workspace = true
            "#,
            FormatOptions {
                rules: Some(FormatRules {
                    group_blank_lines_limit: Some(1.try_into().unwrap()),
                    ..Default::default()
                }),
            }
        ) -> Ok(
            r#"
            [[dependencies]]
            # aaa

            tombi-schema-store.workspace = true
            "#
        )
    }

    test_format! {
        #[tokio::test]
        async fn array_of_table_group_blank_lines_limit_between_dangling_comments_and_first_key_value_two(
            r#"
            [[dependencies]]
            # aaa



            tombi-schema-store.workspace = true
            "#,
            FormatOptions {
                rules: Some(FormatRules {
                    group_blank_lines_limit: Some(2.try_into().unwrap()),
                    ..Default::default()
                }),
            }
        ) -> Ok(
            r#"
            [[dependencies]]
            # aaa


            tombi-schema-store.workspace = true
            "#
        )
    }

    test_format! {
        #[tokio::test]
        async fn array_of_table_moves_comma_trailing_comment_to_key_value(
            r#"
            [[package]]
            name = "toml-rs", # comma trailing comment
            version = "0.4.0"
            "#
        ) -> Ok(
            r#"
            [[package]]
            name = "toml-rs"  # comma trailing comment
            version = "0.4.0"
            "#
        )
    }

    test_format! {
        #[tokio::test]
        async fn array_of_table_keeps_comma_when_comma_has_leading_comment(
            r#"
            [[package]]
            name = "toml-rs"
            # comma leading comment
            , # comma trailing comment
            version = "0.4.0"
            "#
        ) -> Ok(
            r#"
            [[package]]
            name = "toml-rs"
            # comma leading comment
            ,  # comma trailing comment
            version = "0.4.0"
            "#
        )
    }

    test_format! {
        #[tokio::test]
        async fn array_of_table_keeps_comma_when_both_key_value_and_comma_have_trailing_comments(
            r#"
            [[package]]
            name = "toml-rs" # key trailing comment
            , # comma trailing comment
            version = "0.4.0"
            "#
        ) -> Ok(
            r#"
            [[package]]
            name = "toml-rs"  # key trailing comment
            ,  # comma trailing comment
            version = "0.4.0"
            "#
        )
    }

    test_format! {
        #[tokio::test]
        async fn array_of_table_keeps_comma_when_both_key_value_and_comma_have_trailing_comments_with_indent(
            r#"
            [[package]]
            name = "toml-rs" # key trailing comment
            , # comma trailing comment
            version = "0.4.0"
            "#,
            FormatOptions {
                rules: Some(FormatRules {
                    indent_table_key_value_pairs: Some(true),
                    ..Default::default()
                }),
            }
        ) -> Ok(
            r#"
            [[package]]
              name = "toml-rs"  # key trailing comment
              ,  # comma trailing comment
              version = "0.4.0"
            "#
        )
    }

    test_format! {
        #[tokio::test]
        async fn array_of_table_with_full_comment1(
            r#"
            # header leading comment1
            # header leading comment2
            [[header]]  # header trailing comment
            # table begin dangling comment group 1-1
            # table begin dangling comment group 1-2

            # table begin dangling comment group 2-1
            # table begin dangling comment group 2-2
            # table begin dangling comment group 2-3

            # table begin dangling comment group 3-1

            # key value leading comment1
            # key value leading comment2
            key = "value"  # key trailing comment
            "#
        ) -> Ok(source)
    }
}
