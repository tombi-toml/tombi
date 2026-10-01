#[path = "algo.rs"]
pub(crate) mod algo;
mod api;
#[path = "array_of_tables_scope.rs"]
mod array_of_tables_scope;
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

pub use array_of_tables_scope::ArrayOfTablesScope;
pub use comment_directive::{
    DocumentCommentDirectives, SchemaDocumentCommentDirective, TombiDocumentCommentDirective,
    TombiValueCommentDirective,
};
pub use generated::*;
use itertools::Itertools;
pub use literal_value::LiteralValue;
pub use node::*;
pub use token::*;

use std::collections::HashMap;
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

/// Number of preceding `[[...]]` headers per header keys.
type ArrayOfTablesCounts = HashMap<Vec<String>, usize>;

fn collect_array_of_tables_counts(
    keys_iter: impl Iterator<Item = crate::Keys>,
    toml_version: TomlVersion,
) -> ArrayOfTablesCounts {
    keys_iter
        .map(|keys| {
            keys.keys()
                .map(|key| key.content_lossy(toml_version))
                .collect_vec()
        })
        .counts()
}

fn table_header_accessors(
    table: &crate::Table,
    array_of_tables_counts: &ArrayOfTablesCounts,
    toml_version: TomlVersion,
) -> Option<Vec<Accessor>> {
    let mut accessors = vec![];
    let mut header_keys = vec![];
    for key in table.header()?.keys() {
        let key_text = key.content_lossy(toml_version);
        accessors.push(Accessor::Key(key_text.clone()));
        header_keys.push(key_text);

        if let Some(index) = array_of_tables_counts
            .get(&header_keys)
            .map(|count| count - 1)
        {
            accessors.push(Accessor::Index(index));
        }
    }

    Some(accessors)
}

fn array_of_table_header_accessors(
    array_of_table: &crate::ArrayOfTable,
    array_of_tables_counts: &ArrayOfTablesCounts,
    toml_version: TomlVersion,
) -> Option<Vec<Accessor>> {
    let mut accessors = vec![];
    let mut header_keys = vec![];
    let keys = array_of_table.header()?.keys().collect_vec();
    let keys_len = keys.len();
    for key in keys {
        let key_text = key.content_lossy(toml_version);
        accessors.push(Accessor::Key(key_text.clone()));
        header_keys.push(key_text);

        if header_keys.len() == keys_len {
            break;
        }
        if let Some(index) = array_of_tables_counts
            .get(&header_keys)
            .map(|count| count - 1)
        {
            accessors.push(Accessor::Index(index));
        }
    }

    accessors.push(Accessor::Index(
        *array_of_tables_counts.get(&header_keys).unwrap_or(&0),
    ));

    Some(accessors)
}

impl GetHeaderAccessors for crate::Table {
    fn get_header_accessors(&self, toml_version: TomlVersion) -> Option<Vec<Accessor>> {
        let array_of_tables_counts = collect_array_of_tables_counts(
            self.parent_array_of_tables_keys(toml_version),
            toml_version,
        );

        table_header_accessors(self, &array_of_tables_counts, toml_version)
    }
}

impl GetHeaderAccessors for crate::ArrayOfTable {
    fn get_header_accessors(&self, toml_version: TomlVersion) -> Option<Vec<Accessor>> {
        let array_of_tables_counts =
            collect_array_of_tables_counts(self.parent_array_of_tables_keys(), toml_version);

        array_of_table_header_accessors(self, &array_of_tables_counts, toml_version)
    }
}

/// Computes header accessors for top-level tables visited in source order.
///
/// `get_header_accessors` rescans every preceding sibling for each table, which
/// is quadratic for a document with many tables. This tracker accumulates the
/// `[[...]]` headers seen so far and returns the same accessors in `O(1)`
/// amortized time per table.
#[derive(Debug, Default)]
pub struct HeaderAccessorsTracker {
    scope: ArrayOfTablesScope<ArrayOfTablesCounts>,
}

impl HeaderAccessorsTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the header accessors of `table_or_array_of_table` and records its header.
    ///
    /// Tables must be passed in source order, starting from the first table of the root.
    pub fn header_accessors(
        &mut self,
        table_or_array_of_table: &crate::TableOrArrayOfTable,
        toml_version: TomlVersion,
    ) -> Option<Vec<Accessor>> {
        let empty = ArrayOfTablesCounts::new();
        match table_or_array_of_table {
            crate::TableOrArrayOfTable::Table(table) => {
                let counts = self
                    .scope
                    .get(table.header().as_ref(), toml_version)
                    .unwrap_or(&empty);
                table_header_accessors(table, counts, toml_version)
            }
            crate::TableOrArrayOfTable::ArrayOfTable(array_of_table) => {
                let header = array_of_table.header();
                let counts = self
                    .scope
                    .get(header.as_ref(), TomlVersion::latest())
                    .unwrap_or(&empty);
                let accessors =
                    array_of_table_header_accessors(array_of_table, counts, toml_version);

                if let Some(counts) = self.scope.enter(header.as_ref())
                    && let Some(header) = header.as_ref()
                    && header
                        .keys()
                        .all(|key| key.try_to_content(TomlVersion::latest()).is_ok())
                {
                    let keys = header
                        .keys()
                        .map(|key| key.content_lossy(toml_version))
                        .collect_vec();
                    *counts.entry(keys).or_default() += 1;
                }

                accessors
            }
        }
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
