use std::fmt::Write;

use tombi_ast::{AstNode, DanglingCommentGroupOr, LeadingComment, TrailingComment};

use super::{Format, has_empty_line_before};

impl Format for tombi_ast::DanglingCommentGroup {
    fn format(&self, f: &mut crate::Formatter) -> Result<(), std::fmt::Error> {
        if f.skip_comment() {
            return Ok(());
        }

        for (i, comment) in self.comments().enumerate() {
            if i != 0 {
                write!(f, "{}", f.line_ending())?;
            }
            f.write_indent()?;
            format_comment(f, comment.as_ref(), i == 0)?;
        }
        Ok(())
    }
}

impl Format for Vec<tombi_ast::DanglingCommentGroup> {
    fn format(&self, f: &mut crate::Formatter) -> Result<(), std::fmt::Error> {
        if f.skip_comment() {
            return Ok(());
        }

        for (i, group) in self.iter().enumerate() {
            if i != 0 {
                write!(f, "{}", f.line_ending())?;
                write!(f, "{}", f.line_ending())?;
            }
            group.format(f)?;
        }

        Ok(())
    }
}

impl<T: Format + AstNode> Format for Vec<DanglingCommentGroupOr<T>> {
    fn format(&self, f: &mut crate::Formatter) -> Result<(), std::fmt::Error> {
        let mut has_written_group = false;

        for group in self {
            match group {
                DanglingCommentGroupOr::DanglingCommentGroup(comment_group) => {
                    if f.skip_comment() {
                        continue;
                    }
                    if has_written_group {
                        if has_empty_line_before(comment_group) {
                            write!(f, "{}", f.line_ending())?;
                        }
                        write!(f, "{}", f.line_ending())?;
                    } else {
                        has_written_group = true;
                    }
                    comment_group.format(f)?;
                }
                DanglingCommentGroupOr::ItemGroup(item_group) => {
                    if has_written_group {
                        write!(f, "{}", f.line_ending())?;
                        write!(f, "{}", f.line_ending())?;
                    } else {
                        has_written_group = true;
                    }
                    item_group.format(f)?;
                }
            }
        }

        Ok(())
    }
}

impl Format for Vec<LeadingComment> {
    fn format(&self, f: &mut crate::Formatter) -> Result<(), std::fmt::Error> {
        if f.skip_comment() {
            return Ok(());
        }

        for (i, comment) in self.iter().enumerate() {
            f.write_indent()?;
            if i == 0 {
                format_comment(f, comment.as_ref(), true)?;
            } else {
                format_comment(f, comment.as_ref(), false)?;
            }
            write!(f, "{}", f.line_ending())?;
        }
        Ok(())
    }
}

impl Format for TrailingComment {
    #[inline]
    fn format(&self, f: &mut crate::Formatter) -> Result<(), std::fmt::Error> {
        if f.skip_comment() {
            return Ok(());
        }

        write!(f, "{}", f.trailing_comment_space())?;
        format_comment(f, self.as_ref(), true)
    }
}

fn format_comment(
    f: &mut crate::Formatter,
    comment: &tombi_ast::Comment,
    strip_leading_spaces: bool,
) -> Result<(), std::fmt::Error> {
    let comment_string = comment.to_string();
    {
        // For the purpose of reading the JSON Schema path defined in the file by taplo,
        // we format in a different style from the tombi comment style.
        if let Some(schema_uri) = comment_string.strip_prefix("#:schema ") {
            return write!(f, "#:schema {}", schema_uri.trim());
        }

        if let Some(content) = comment_string.strip_prefix("#:tombi ") {
            let formatted = f.format_tombi_comment_directive_content(content)?;
            return write!(f, "#:tombi {formatted}");
        }

        if let Some(comment_directive) = comment.get_tombi_value_directive() {
            let formatted = f.format_tombi_comment_directive_content(&comment_directive.content)?;
            return write!(f, "# tombi: {formatted}");
        }
    }

    let mut iter = comment_string.trim_ascii_end().chars();

    // write '#' character
    write!(f, "{}", iter.next().unwrap())?;

    if let Some(mut c) = iter.next() {
        // For https://crates.io/crates/document-features crate, the comment starts with '#' or '!'.
        {
            if matches!(c, '#' | '!') {
                write!(f, "{c}")?;
                if let Some(next) = iter.next() {
                    c = next;
                } else {
                    return Ok(());
                }
            }
        }

        write!(f, " ")?;

        if c != ' ' && c != '\t' {
            write!(f, "{c}")?;
        } else if strip_leading_spaces {
            for c in iter.by_ref() {
                if c != ' ' && c != '\t' {
                    write!(f, "{c}")?;
                    break;
                }
            }
        }
    }

    write!(f, "{}", iter.as_str())
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use tombi_ast::AstNode;

    use crate::{Formatter, test_format};

    test_format! {
        #[tokio::test]
        async fn test_only_comment1(
            r#"
            # comment1
            # comment2
            "#,
            TomlVersion::V1_0_0
        ) -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn test_only_comment2(
            r#"
            # comment1
            # comment2

            # comment3
            # comment4
            "#,
            TomlVersion::V1_0_0
        ) -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn comment_without_space(r"#comment") -> Ok("# comment")
    }

    test_format! {
        #[tokio::test]
        async fn empty_comment(r"#") -> Ok(source)
    }

    test_format! {
        // Reference: https://crates.io/crates/document-features
        #[tokio::test]
        async fn empty_comment_document_features(r"#!") -> Ok(source)
    }

    test_format! {
        // Reference: https://crates.io/crates/document-features
        #[tokio::test]
        async fn empty_comment_document_features2(r"##") -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn only_space_comment1(r"# ") -> Ok(r"#")
    }

    test_format! {
        #[tokio::test]
        async fn only_long_space_comment(r"#      ") -> Ok(r"#")
    }

    #[test]
    fn format_to_string_without_comment_keeps_item_groups() {
        let source = textwrap::dedent(
            r#"
            # comment1
            # comment2

            key1 = 1
            key2 = 2
            "#,
        )
        .trim()
        .to_string();

        let root = tombi_ast::Root::cast(tombi_parser::parse(&source).into_syntax_node()).unwrap();
        let groups = root.key_value_groups().collect_vec();

        let schema_store = tombi_schema_store::SchemaStore::new();
        let options = tombi_config::FormatOptions::default();
        let source_path = tombi_test_lib::project_root_path().join("test.toml");
        let mut formatter = Formatter::new(
            tombi_config::TomlVersion::V1_0_0,
            &options,
            Some(itertools::Either::Right(source_path.as_path())),
            &schema_store,
        );

        let formatted = formatter.format_to_string_without_comment(&groups).unwrap();

        assert!(
            formatted.contains("key1"),
            "item group content should be formatted even when comments are skipped"
        );
        assert!(
            formatted.contains("key2"),
            "item group content should be formatted even when comments are skipped"
        );
    }

    test_format! {
        // Reference: https://crates.io/crates/document-features
        #[tokio::test]
        async fn only_space_comment_document_features(r"#! ") -> Ok(r"#!")
    }

    test_format! {
        // Reference: https://crates.io/crates/document-features
        #[tokio::test]
        async fn only_space_comment_document_features2(r"## ") -> Ok(r"##")
    }

    test_format! {
        // Reference: https://crates.io/crates/document-features
        #[tokio::test]
        async fn only_long_space_comment_document_features(r"#!       ") -> Ok(r"#!")
    }

    test_format! {
        // Reference: https://crates.io/crates/document-features
        #[tokio::test]
        async fn only_long_space_comment_document_features2(r"##      ") -> Ok(r"##")
    }

    test_format! {
        #[tokio::test]
        async fn strip_prefix_space(r"#    hello") -> Ok(r"# hello")
    }

    test_format! {
        // Reference: https://crates.io/crates/document-features
        #[tokio::test]
        async fn strip_prefix_space_document_features(r"#!      hello") -> Ok(r"#! hello")
    }

    test_format! {
        // Reference: https://crates.io/crates/document-features
        #[tokio::test]
        async fn strip_prefix_space_document_features2(r"##      hello") -> Ok(r"## hello")
    }

    test_format! {
        // Reference: https://crates.io/crates/document-features
        #[tokio::test]
        async fn strip_prefix_space_document_features_double_bang(r"#!!  hello") -> Ok(r"#! !  hello")
    }

    test_format! {
        // Reference: https://crates.io/crates/document-features
        #[tokio::test]
        async fn strip_prefix_space_document_features_double_sharp(r"###  hello") -> Ok(r"## #  hello")
    }

    test_format! {
        #[tokio::test]
        async fn multiline_comment_with_ident(
            r#"
            # NOTE: Tombi preserves spaces at the beginning of a comment line.
            #       This allows for multi-line indentation to be preserved.
            "#
        ) -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn end_dangling_comment(
            r#"
            [dependencies]
            serde = "^1.0"
            # serde_json = "^1.0"
            # serde-yaml = "^0.10"
            "#
        ) -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn end_dangling_comment_starts_with_line_break(
            r#"
            key = "value"

            # end dangling comment1
            # end dangling comment2

            # end dangling comment3
            # end dangling comment4
            "#
        ) -> Ok(source)
    }

    test_format! {
        #[tokio::test]
        async fn end_dangling_comment_starts_with_multi_line_break(
            r#"
            key = "value"


            # end dangling comment1
            # end dangling comment2

            # end dangling comment3
            # end dangling comment4
            "#
        ) -> Ok(
            r#"
            key = "value"

            # end dangling comment1
            # end dangling comment2

            # end dangling comment3
            # end dangling comment4
            "#
        )
    }

    test_format! {
        #[tokio::test]
        async fn schema_comment(r"#:schema https://www.schemastore.org/pyproject.json") -> Ok(
            "#:schema https://www.schemastore.org/pyproject.json"
        )
    }

    test_format! {
        #[tokio::test]
        async fn schema_comment_with_space(r"#:schema  https://www.schemastore.org/pyproject.json  ") -> Ok(
            "#:schema https://www.schemastore.org/pyproject.json"
        )
    }

    test_format! {
        #[tokio::test]
        async fn tombi_comment_directive(r"#:tombi   toml-version   = 'v1.0.0'  ") -> Ok(
            "#:tombi toml-version = \"v1.0.0\""
        )
    }
}
