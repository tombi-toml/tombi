use itertools::Itertools;
use tombi_ast::DanglingCommentGroupOr;

use crate::format::blank_lines_before;

use super::Format;

impl Format for tombi_ast::Root {
    fn format(&self, f: &mut crate::Formatter) -> Result<(), std::fmt::Error> {
        f.reset();

        let dangling_comment_groups = self.dangling_comment_groups().collect_vec();
        let key_value_groups = self.key_value_groups().collect_vec();
        let dangling_comment_group_is_empty = dangling_comment_groups.is_empty();
        let key_value_groups_is_empty = key_value_groups.is_empty();

        let groups = itertools::chain!(
            dangling_comment_groups
                .into_iter()
                .map(DanglingCommentGroupOr::DanglingCommentGroup),
            key_value_groups
        )
        .collect_vec();

        groups.format(f)?;

        let table_or_array_of_tables = self.table_or_array_of_tables().collect_vec();

        if (!dangling_comment_group_is_empty || !key_value_groups_is_empty)
            && !table_or_array_of_tables.is_empty()
        {
            if key_value_groups_is_empty {
                f.write_blank_lines(
                    blank_lines_before(&table_or_array_of_tables[0])
                        .min(f.group_blank_lines_limit()),
                )?;
            } else {
                f.write_blank_lines(f.table_blank_lines())?;
            }
        }
        table_or_array_of_tables.format(f)?;

        Ok(())
    }
}

impl Format for Vec<tombi_ast::TableOrArrayOfTable> {
    fn format(&self, f: &mut crate::Formatter) -> Result<(), std::fmt::Error> {
        let mut header = Header::Root;
        for (i, table_or_array_of_table) in self.iter().enumerate() {
            if i != 0 {
                f.write_line_ending()?;
            }
            match table_or_array_of_table {
                tombi_ast::TableOrArrayOfTable::Table(table) => {
                    let header_keys = table.header().unwrap().keys();
                    let key_value_size = table.key_values().count();
                    let has_dangling_comments = table.dangling_comment_groups().next().is_some();

                    match header {
                        Header::Root => {}
                        Header::Table {
                            header_keys: pre_header_keys,
                            key_value_size,
                            has_dangling_comments,
                        }
                        | Header::ArrayOfTable {
                            header_keys: pre_header_keys,
                            key_value_size,
                            has_dangling_comments,
                        } => {
                            if key_value_size > 0
                                || !header_keys.starts_with(&pre_header_keys)
                                || has_dangling_comments
                            {
                                for _ in 0..f.table_blank_lines() {
                                    f.write_line_ending()?;
                                }
                            }
                        }
                    };
                    table.format(f)?;

                    header = Header::Table {
                        header_keys,
                        key_value_size,
                        has_dangling_comments,
                    };
                }
                tombi_ast::TableOrArrayOfTable::ArrayOfTable(array_of_table) => {
                    let header_keys = array_of_table.header().unwrap().keys();
                    let key_value_size = array_of_table.key_values().count();
                    let has_dangling_comments =
                        array_of_table.dangling_comment_groups().next().is_some();

                    match header {
                        Header::Root => {}
                        Header::Table {
                            header_keys: pre_header_keys,
                            key_value_size,
                            has_dangling_comments,
                        } => {
                            if key_value_size > 0
                                || !header_keys.starts_with(&pre_header_keys)
                                || has_dangling_comments
                            {
                                for _ in 0..f.table_blank_lines() {
                                    f.write_line_ending()?;
                                }
                            }
                        }
                        Header::ArrayOfTable {
                            header_keys: pre_header_keys,
                            key_value_size,
                            has_dangling_comments,
                        } => {
                            if key_value_size > 0
                                || !header_keys.starts_with(&pre_header_keys)
                                || pre_header_keys.same_as(&header_keys)
                                || has_dangling_comments
                            {
                                for _ in 0..f.table_blank_lines() {
                                    f.write_line_ending()?;
                                }
                            }
                        }
                    };

                    array_of_table.format(f)?;

                    header = Header::ArrayOfTable {
                        header_keys,
                        key_value_size,
                        has_dangling_comments,
                    };
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
enum Header {
    Root,

    Table {
        header_keys: tombi_ast::AstChildren<tombi_ast::Key>,
        key_value_size: usize,
        has_dangling_comments: bool,
    },

    ArrayOfTable {
        header_keys: tombi_ast::AstChildren<tombi_ast::Key>,
        key_value_size: usize,
        has_dangling_comments: bool,
    },
}

#[cfg(test)]
mod test {
    use tombi_config::FormatRules;

    use crate::{Formatter, test_format};

    test_format! {
        #[tokio::test]
        async fn empty_table_space_on_own_subtable(
            r#"
            [foo]
            [foo.bar]
            "#
        ) -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn root_group_blank_lines_limit_between_dangling_comments_and_first_key_value(
            r#"
            # aaa



            key = "value"
            "#,
            FormatOptions {
                rules: Some(FormatRules {
                    group_blank_lines_limit: Some(1.try_into().unwrap()),
                    ..Default::default()
                }),
            }
        ) -> Ok(
            r#"
            # aaa

            key = "value"
            "#
        )
    }

    test_format! {
        #[tokio::test]
        async fn root_group_blank_lines_limit_between_dangling_comments_and_first_key_value_two(
            r#"
            # aaa



            key = "value"
            "#,
            FormatOptions {
                rules: Some(FormatRules {
                    group_blank_lines_limit: Some(2.try_into().unwrap()),
                    ..Default::default()
                }),
            }
        ) -> Ok(
            r#"
            # aaa


            key = "value"
            "#
        )
    }

    test_format! {
        #[tokio::test]
        async fn empty_table_space_on_other_table(
            r#"
            [foo]

            [bar.baz]
            "#
        ) -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn table_blank_lines_does_not_expand_tight_parent_child_tables(
            r#"
            [foo]
            [foo.bar]
            "#,
            FormatOptions {
                rules: Some(FormatRules {
                    table_blank_lines: Some(3.into()),
                    ..Default::default()
                }),
            }
        ) -> Ok(
            r#"
            [foo]
            [foo.bar]
            "#
        )
    }

    test_format! {
        #[tokio::test]
        async fn empty_table_space_on_own_array_of_sub_tables(
            r#"
            [foo]
            [[foo.bar]]
            "#
        ) -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn empty_table_space_on_other_array_of_tables(
            r#"
            [foo]

            [[bar.baz]]
            "#
        ) -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn empty_table_space_on_other_array_of_tables_with_comments(
            r#"
            [foo]  # header table comment
            # table dangling comment 1-1
            # table dangling comment 1-2

            # table dangling comment 2-1
            # table dangling comment 2-2
            # table dangling comment 2-3

            # table dangling comment 3-1

            # table header leading comment1
            # table header leading comment2
            [[bar.baz]]
            "#
        ) -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn empty_array_of_tables_space_on_own_subtable(
            r#"
            [[foo]]
            [foo.bar]
            "#
        ) -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn empty_array_of_tables_space_on_own_subtable_with_comments(
            r#"
            [[foo]]  # header trailing comment
            # table dangling comment 1-1
            # table dangling comment 1-2

            # table dangling comment 2-1
            # table dangling comment 2-2
            # table dangling comment 2-3

            # table dangling comment 3-1

            # table header leading comment1
            # table header leading comment2
            [foo.bar]
            "#
        ) -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn empty_array_of_tables_space_on_other_subtable(
            r#"
            [[foo]]

            [bar.baz]
            "#
        ) -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn empty_array_of_tables_space_on_other_subtable_with_comments(
            r#"
            [[foo]]  # header trailing comment
            # table dangling comment 1-1
            # table dangling comment 1-2

            # table dangling comment 2-1
            # table dangling comment 2-2
            # table dangling comment 2-3

            # table dangling comment 3-1

            [bar.baz]
            "#
        ) -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn empty_array_of_tables_space_on_same_array_of_tables(
            r#"
            [[foo]]

            [[foo]]
            "#
        ) -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn empty_array_of_tables_space_on_same_array_of_tables_with_comment(
            r#"
            [[foo]]  # header trailing comment
            # table dangling comment 1-1
            # table dangling comment 1-2

            # table dangling comment 2-1
            # table dangling comment 2-2
            # table dangling comment 2-3

            # table dangling comment 3-1

            # table header leading comment1
            # table header leading comment2
            [[foo]]
            "#
        ) -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn only_dangling_comment1(
            r#"
            # root dangling comment
            "#
        ) -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn only_dangling_comment2(
            r#"
            # root dangling comment 1-1
            # root dangling comment 1-2

            # root dangling comment 2-1
            # root dangling comment 2-1
            # root dangling comment 2-3

            # root dangling comment 3-1
            "#
        ) -> Ok(source)
    }
}
