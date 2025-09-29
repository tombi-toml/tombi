use std::cmp::Ordering;

use itertools::{EitherOrBoth, Itertools};
use tombi_accessor::Accessor;
use tombi_toml_version::TomlVersion;

use crate::{AstChildren, AstNode};

impl crate::Key {
    pub fn token(&self) -> Option<tombi_syntax::SyntaxToken> {
        match self {
            Self::BareKey(key) => key.token(),
            Self::BasicString(key) => key.token(),
            Self::LiteralString(key) => key.token(),
        }
    }

    pub fn accessor(&self, toml_version: TomlVersion) -> Accessor {
        Accessor::Key(self.to_raw_text(toml_version))
    }

    pub fn to_raw_text(&self, toml_version: TomlVersion) -> String {
        self.try_to_raw_text(toml_version)
            .unwrap_or_else(|_| self.syntax().text().to_string())
    }

    pub fn try_to_raw_text(
        &self,
        toml_version: TomlVersion,
    ) -> Result<String, tombi_toml_text::ParseError> {
        match self {
            Self::BareKey(key) => tombi_toml_text::try_from_bare_key(key.token().unwrap().text()),
            Self::BasicString(key) => {
                tombi_toml_text::try_from_basic_string(key.token().unwrap().text(), toml_version)
            }
            Self::LiteralString(key) => {
                tombi_toml_text::try_from_literal_string(key.token().unwrap().text())
            }
        }
    }

    pub fn range(&self) -> tombi_text::Range {
        match self {
            Self::BareKey(key) => key.range(),
            Self::BasicString(key) => key.range(),
            Self::LiteralString(key) => key.range(),
        }
    }
}

impl PartialOrd for crate::Key {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (
            self.try_to_raw_text(TomlVersion::latest()),
            other.try_to_raw_text(TomlVersion::latest()),
        ) {
            (Ok(a), Ok(b)) => Some(a.cmp(&b)),
            _ => None,
        }
    }
}

impl crate::Keys {
    pub fn accessors(&self, toml_version: TomlVersion) -> Vec<Accessor> {
        self.keys()
            .map(|key| key.accessor(toml_version))
            .collect_vec()
    }
}

impl AstChildren<crate::Key> {
    pub fn starts_with(&self, other: &AstChildren<crate::Key>) -> bool {
        self.clone().zip_longest(other.clone()).all(|m| match m {
            EitherOrBoth::Left(_) => true,
            EitherOrBoth::Right(_) => false,
            EitherOrBoth::Both(a, b) => {
                match (
                    a.try_to_raw_text(TomlVersion::latest()),
                    b.try_to_raw_text(TomlVersion::latest()),
                ) {
                    (Ok(a), Ok(b)) => a == b,
                    _ => false,
                }
            }
        })
    }

    pub fn same_as(&self, other: &AstChildren<crate::Key>) -> bool {
        (self.clone().count() == other.clone().count()) && self.starts_with(other)
    }

    pub fn rev(self) -> impl Iterator<Item = crate::Key> {
        self.collect_vec().into_iter().rev()
    }
}
