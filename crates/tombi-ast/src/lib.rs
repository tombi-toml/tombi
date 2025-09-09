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

pub trait GetHeaderSchemarAccessors {
    fn get_header_accessor(&self, toml_version: TomlVersion) -> Option<Vec<Accessor>>;
}

impl GetHeaderSchemarAccessors for crate::Table {
    fn get_header_accessor(&self, toml_version: TomlVersion) -> Option<Vec<Accessor>> {
        let array_of_tables_keys = self
            .array_of_tables_keys()
            .map(|keys| keys.into_iter().collect_vec())
            .counts();

        let mut accessors = vec![];
        let mut header_keys = vec![];
        for key in self.header()?.keys() {
            accessors.push(Accessor::Key(key.to_raw_text(toml_version)));
            header_keys.push(key);

            if let Some(new_index) = array_of_tables_keys.get(&header_keys) {
                accessors.push(Accessor::Index(*new_index));
            }
        }

        Some(accessors)
    }
}

impl GetHeaderSchemarAccessors for crate::ArrayOfTable {
    fn get_header_accessor(&self, toml_version: TomlVersion) -> Option<Vec<Accessor>> {
        let array_of_tables_keys = self
            .array_of_tables_keys()
            .map(|keys| keys.into_iter().collect_vec())
            .counts();

        let mut accessors = vec![];
        let mut header_keys = vec![];
        for key in self.header()?.keys() {
            accessors.push(Accessor::Key(key.to_raw_text(toml_version)));
            header_keys.push(key);

            if let Some(new_index) = array_of_tables_keys.get(&header_keys) {
                accessors.push(Accessor::Index(*new_index));
            }
        }

        accessors.push(Accessor::Index(
            *array_of_tables_keys.get(&header_keys).unwrap_or(&0),
        ));

        Some(accessors)
    }
}
