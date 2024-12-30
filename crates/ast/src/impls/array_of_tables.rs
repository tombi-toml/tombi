use crate::{support, ArrayOfTables, AstChildren, AstNode};
use syntax::SyntaxKind::*;
use syntax::T;
use toml_version::TomlVersion;

impl crate::ArrayOfTables {
    pub fn header_leading_comments(&self) -> impl Iterator<Item = crate::Comment> {
        support::node::leading_comments(self.syntax().children_with_tokens())
    }

    pub fn header_tailing_comment(&self) -> Option<crate::Comment> {
        support::node::tailing_comment(self.syntax().children_with_tokens(), T!("]]"))
    }

    pub fn begin_dangling_comments(&self) -> Vec<Vec<crate::Comment>> {
        support::node::begin_dangling_comments(
            self.syntax()
                .children_with_tokens()
                .skip_while(|node| !matches!(node.kind(), T!("]]")))
                .skip_while(|node| !matches!(node.kind(), LINE_BREAK)),
        )
    }

    pub fn end_dangling_comments(&self) -> Vec<Vec<crate::Comment>> {
        support::node::end_dangling_comments(self.syntax().children_with_tokens())
    }

    pub fn array_of_tables_keys(&self) -> impl Iterator<Item = AstChildren<crate::Key>> + '_ {
        support::node::prev_siblings_nodes(self)
            .map(|node: ArrayOfTables| node.header().unwrap().keys())
            .take_while(
                |keys| match (self.header().unwrap().keys().next(), keys.clone().next()) {
                    (Some(a), Some(b)) => match (
                        a.try_to_raw_text(TomlVersion::latest()),
                        b.try_to_raw_text(TomlVersion::latest()),
                    ) {
                        (Ok(a), Ok(b)) => a == b,
                        _ => false,
                    },
                    _ => false,
                },
            )
            .filter(|keys| self.header().unwrap().keys().starts_with(keys))
    }
}
