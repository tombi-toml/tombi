pub mod algo;
mod comment_directive;
mod generated;
mod impls;
mod literal_value;
mod node;
pub mod support;

pub use comment_directive::{
    SchemaDocumentCommentDirective, TombiDocumentCommentDirective, TombiValueCommentDirective,
};
pub use generated::*;
use itertools::Itertools;
pub use literal_value::LiteralValue;
pub use node::*;
use std::{fmt::Debug, marker::PhantomData};
use tombi_accessor::Accessor;
use tombi_toml_version::TomlVersion;

pub trait AstNode
where
    Self: Debug,
{
    fn leading_comments(&self) -> impl Iterator<Item = crate::LeadingComment> {
        support::node::leading_comments(self.syntax().children_with_tokens())
    }

    fn trailing_comment(&self) -> Option<crate::TrailingComment> {
        self.syntax()
            .last_token()
            .and_then(crate::Comment::cast)
            .map(Into::into)
    }

    fn can_cast(kind: tombi_syntax::SyntaxKind) -> bool
    where
        Self: Sized;

    fn cast(syntax: tombi_syntax::SyntaxNode) -> Option<Self>
    where
        Self: Sized;

    fn syntax(&self) -> &tombi_syntax::SyntaxNode;

    fn clone_for_update(&self) -> Self
    where
        Self: Sized,
    {
        Self::cast(self.syntax().clone_for_update()).unwrap()
    }
}

/// Like `AstNode`, but wraps tokens rather than interior nodes.
pub trait AstToken {
    fn can_cast(token: tombi_syntax::SyntaxKind) -> bool
    where
        Self: Sized;

    fn cast(syntax: tombi_syntax::SyntaxToken) -> Option<Self>
    where
        Self: Sized;

    fn syntax(&self) -> &tombi_syntax::SyntaxToken;

    fn text(&self) -> &str {
        self.syntax().text()
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AstChildren<N> {
    inner: tombi_syntax::SyntaxNodeChildren,
    ph: PhantomData<N>,
}

impl<N> AstChildren<N> {
    fn new(parent: &tombi_syntax::SyntaxNode) -> Self {
        AstChildren {
            inner: parent.children(),
            ph: PhantomData,
        }
    }
}

impl<N: AstNode> Iterator for AstChildren<N> {
    type Item = N;
    fn next(&mut self) -> Option<N> {
        self.inner.find_map(N::cast)
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
                keys.into_iter()
                    .map(|key| key.to_raw_text(toml_version))
                    .collect_vec()
            })
            .counts();

        let mut accessors = vec![];
        let mut header_keys = vec![];
        for key in self.header()?.keys() {
            let key_text = key.to_raw_text(toml_version);
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
            .parrent_array_of_tables_keys()
            .map(|keys| {
                keys.into_iter()
                    .map(|key| key.to_raw_text(toml_version))
                    .collect_vec()
            })
            .counts();

        let mut accessors = vec![];
        let mut header_keys = vec![];
        for key in self.header()?.keys() {
            let key_text = key.to_raw_text(toml_version);
            accessors.push(Accessor::Key(key_text.clone()));
            header_keys.push(key_text);

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
