#[path = "algo.rs"]
pub(crate) mod algo;
mod api;
#[path = "comment_directive.rs"]
pub(crate) mod comment_directive;
#[path = "generated.rs"]
mod generated;
mod header_index;
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
pub(crate) use header_index::{HeaderIndex, header_info};
use itertools::Itertools;
pub use literal_value::LiteralValue;
pub use node::*;
pub use token::*;

use std::fmt::Debug;
use tombi_accessor::Accessor;
use tombi_toml_version::TomlVersion;

pub trait AstNode<'t>
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

    /// Decodes the escaped strings of the whole tree for `toml_version`.
    ///
    /// The document tree borrows the result, so the caller keeps it next to the tree.
    fn decode_strings(
        &self,
        toml_version: tombi_toml_version::TomlVersion,
    ) -> crate::DecodedTextResolver {
        self.syntax().decoded_text_resolver(toml_version)
    }

    fn leading_comments(&self) -> impl Iterator<Item = crate::LeadingComment<'t>> + use<'t, Self> {
        support::comment::leading_comments(self.syntax().child_elements())
    }

    fn trailing_comment(&self) -> Option<crate::TrailingComment<'t>> {
        self.syntax()
            .last_token()
            .and_then(crate::Comment::cast)
            .map(Into::into)
    }

    fn can_cast(kind: tombi_ast_syntax::SyntaxKind) -> bool
    where
        Self: Sized;

    fn cast(syntax: tombi_ast_syntax::SyntaxNode<'t>) -> Option<Self>
    where
        Self: Sized;

    fn syntax(&self) -> &tombi_ast_syntax::SyntaxNode<'t>;
}

/// Like `AstNode`, but wraps tokens rather than interior nodes.
pub trait AstToken<'t> {
    fn can_cast(token: tombi_ast_syntax::SyntaxKind) -> bool
    where
        Self: Sized;

    fn cast(syntax: tombi_ast_syntax::SyntaxToken<'t>) -> Option<Self>
    where
        Self: Sized;

    fn syntax(&self) -> &tombi_ast_syntax::SyntaxToken<'t>;

    fn text(&self) -> &'t str {
        self.syntax().text()
    }
}

pub trait GetHeaderAccessors {
    fn get_header_accessors(&self, toml_version: TomlVersion) -> Option<Vec<Accessor>>;
}

impl GetHeaderAccessors for crate::Table<'_> {
    fn get_header_accessors(&self, toml_version: TomlVersion) -> Option<Vec<Accessor>> {
        let prefix_counts = self.parent_array_of_tables_prefix_counts();

        let mut accessors = vec![];
        for (i, key) in self.header()?.keys().enumerate() {
            accessors.push(Accessor::Key(key.content_lossy(toml_version)));

            if let Some(index) = prefix_counts.get(i).and_then(|count| count.checked_sub(1)) {
                accessors.push(Accessor::Index(index));
            }
        }

        Some(accessors)
    }
}

impl GetHeaderAccessors for crate::ArrayOfTable<'_> {
    fn get_header_accessors(&self, toml_version: TomlVersion) -> Option<Vec<Accessor>> {
        let prefix_counts = self.parent_array_of_tables_prefix_counts();

        let mut accessors = vec![];
        let keys = self.header()?.keys().collect_vec();
        let keys_len = keys.len();
        for (i, key) in keys.into_iter().enumerate() {
            accessors.push(Accessor::Key(key.content_lossy(toml_version)));

            if i + 1 == keys_len {
                break;
            }
            if let Some(index) = prefix_counts.get(i).and_then(|count| count.checked_sub(1)) {
                accessors.push(Accessor::Index(index));
            }
        }

        accessors.push(Accessor::Index(
            prefix_counts.get(keys_len - 1).copied().unwrap_or_default(),
        ));

        Some(accessors)
    }
}

impl<'t> GetHeaderAccessors for crate::TableOrArrayOfTable<'t> {
    fn get_header_accessors(&self, toml_version: TomlVersion) -> Option<Vec<Accessor>> {
        match self {
            crate::TableOrArrayOfTable::Table(table) => table.get_header_accessors(toml_version),
            crate::TableOrArrayOfTable::ArrayOfTable(array_of_table) => {
                array_of_table.get_header_accessors(toml_version)
            }
        }
    }
}
