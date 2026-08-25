#[path = "algo.rs"]
pub(crate) mod algo;
mod api;
#[path = "comment_directive.rs"]
pub(crate) mod comment_directive;
#[path = "generated.rs"]
mod generated;
#[path = "impls.rs"]
mod impls;
#[path = "literal_value.rs"]
mod literal_value;
#[path = "node.rs"]
mod node;
#[path = "support.rs"]
pub mod support;
#[path = "token.rs"]
mod token;

pub use comment_directive::{
    DocumentCommentDirectives, SchemaDocumentCommentDirective, TombiDocumentCommentDirective,
    TombiValueCommentDirective,
};
pub use generated::*;
use itertools::Itertools;
pub use literal_value::LiteralValue;
pub use node::*;
pub use token::*;

use std::fmt::Debug;
use tombi_accessor::Accessor;
use tombi_toml_version::TomlVersion;

pub trait AstNode
where
    Self: Debug,
{
    /// Number of blank source lines immediately preceding this TOML node.
    fn blank_lines_before(&self) -> u8 {
        let mut line_break_count = 0usize;
        let mut current = self.syntax().prev_sibling_or_token();

        while let Some(element) = current {
            match element.kind() {
                crate::SyntaxKind::WHITESPACE => current = element.prev_sibling_or_token(),
                crate::SyntaxKind::LINE_BREAK => {
                    line_break_count += 1;
                    current = element.prev_sibling_or_token();
                }
                _ => break,
            }
        }

        u8::try_from(line_break_count.saturating_sub(1)).unwrap_or(u8::MAX)
    }

    fn leading_comments(&self) -> impl Iterator<Item = crate::LeadingComment> {
        support::comment::leading_comments(self.syntax().child_elements())
    }

    fn trailing_comment(&self) -> Option<crate::TrailingComment> {
        self.syntax()
            .last_token()
            .and_then(crate::Comment::cast)
            .map(Into::into)
    }

    fn can_cast(kind: tombi_ast_syntax::SyntaxKind) -> bool
    where
        Self: Sized;

    fn cast(syntax: tombi_ast_syntax::SyntaxNode) -> Option<Self>
    where
        Self: Sized;

    fn syntax(&self) -> &tombi_ast_syntax::SyntaxNode;
}

/// Like `AstNode`, but wraps tokens rather than interior nodes.
pub trait AstToken {
    fn can_cast(token: tombi_ast_syntax::SyntaxKind) -> bool
    where
        Self: Sized;

    fn cast(syntax: tombi_ast_syntax::SyntaxToken) -> Option<Self>
    where
        Self: Sized;

    fn syntax(&self) -> &tombi_ast_syntax::SyntaxToken;

    fn text(&self) -> &str {
        self.syntax().text()
    }
}

pub trait GetHeaderAccessors {
    fn get_header_accessors(&self, toml_version: TomlVersion) -> Option<Vec<Accessor>>;
}

impl GetHeaderAccessors for crate::Table {
    fn get_header_accessors(&self, toml_version: TomlVersion) -> Option<Vec<Accessor>> {
        let array_of_tables_keys = self
            .parent_array_of_tables_keys(toml_version)
            .map(|keys| {
                keys.keys()
                    .map(|key| key.content_lossy(toml_version))
                    .collect_vec()
            })
            .counts();

        let mut accessors = vec![];
        let mut header_keys = vec![];
        for key in self.header()?.keys() {
            let key_text = key.content_lossy(toml_version);
            accessors.push(Accessor::Key(key_text.clone()));
            header_keys.push(key_text);

            if let Some(index) = array_of_tables_keys
                .get(&header_keys)
                .map(|count| count - 1)
            {
                accessors.push(Accessor::Index(index));
            }
        }

        Some(accessors)
    }
}

impl GetHeaderAccessors for crate::ArrayOfTable {
    fn get_header_accessors(&self, toml_version: TomlVersion) -> Option<Vec<Accessor>> {
        let array_of_tables_keys = self
            .parent_array_of_tables_keys()
            .map(|keys| {
                keys.keys()
                    .map(|key| key.content_lossy(toml_version))
                    .collect_vec()
            })
            .counts();

        let mut accessors = vec![];
        let mut header_keys = vec![];
        let keys = self.header()?.keys().collect_vec();
        let keys_len = keys.len();
        for key in keys {
            let key_text = key.content_lossy(toml_version);
            accessors.push(Accessor::Key(key_text.clone()));
            header_keys.push(key_text);

            if header_keys.len() == keys_len {
                break;
            }
            if let Some(index) = array_of_tables_keys
                .get(&header_keys)
                .map(|count| count - 1)
            {
                accessors.push(Accessor::Index(index));
            }
        }

        accessors.push(Accessor::Index(
            *array_of_tables_keys.get(&header_keys).unwrap_or(&0),
        ));

        Some(accessors)
    }
}

impl GetHeaderAccessors for crate::TableOrArrayOfTable {
    fn get_header_accessors(&self, toml_version: TomlVersion) -> Option<Vec<Accessor>> {
        match self {
            crate::TableOrArrayOfTable::Table(table) => table.get_header_accessors(toml_version),
            crate::TableOrArrayOfTable::ArrayOfTable(array_of_table) => {
                array_of_table.get_header_accessors(toml_version)
            }
        }
    }
}
