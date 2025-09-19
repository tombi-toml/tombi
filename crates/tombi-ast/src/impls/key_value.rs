use tombi_accessor::Accessor;
use tombi_toml_version::TomlVersion;

use crate::AstNode;

impl crate::KeyValue {
    pub fn comment_directives(
        &self,
    ) -> impl Iterator<Item = crate::TombiValueCommentDirective> + '_ {
        itertools::chain!(
            self.leading_comments()
                .filter_map(|comment| comment.get_tombi_value_directive()),
            self.trailing_comment()
                .into_iter()
                .filter_map(|comment| comment.get_tombi_value_directive()),
        )
    }

    pub fn get_accessors(&self, toml_version: TomlVersion) -> Option<Vec<Accessor>> {
        self.keys().map(|keys| keys.accessors(toml_version))
    }
}
