use tombi_accessor::Accessor;
use tombi_toml_version::TomlVersion;

use crate::AstNode;

impl crate::KeyValue {
    /// Source range of this inline-table or array item, extended through its
    /// comma when one is present.
    pub fn item_range_with_comma(
        &self,
        position: tombi_text::Position,
    ) -> Option<tombi_text::Range> {
        for syntax_node in self.syntax().ancestors() {
            if let Some(group) = crate::KeyValueWithCommaGroup::cast(syntax_node.clone()) {
                for (item, comma) in group.key_values_with_comma() {
                    if item.syntax() == self.syntax() {
                        let start = item
                            .leading_comments()
                            .next()
                            .map(|comment| comment.syntax().range().start)
                            .or_else(|| item.keys().map(|keys| keys.range().start))?;
                        let end = item
                            .value()
                            .map(|value| value.range().end)
                            .unwrap_or(item.range().end);
                        let range = tombi_text::Range::new(start, end);
                        return Some(comma.map_or(range, |comma| range + comma.range()));
                    }
                }

                if let Some(range) = range_containing_position(
                    group
                        .key_values_with_comma()
                        .map(|(item, comma)| (item.range(), comma.map(|comma| comma.range()))),
                    position,
                ) {
                    return Some(range);
                }
            } else if let Some(group) = crate::ValueWithCommaGroup::cast(syntax_node)
                && let Some(range) = range_containing_position(
                    group
                        .value_or_key_values_with_comma()
                        .map(|(item, comma)| (item.range(), comma.map(|comma| comma.range()))),
                    position,
                )
            {
                return Some(range);
            }
        }

        None
    }

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

fn range_containing_position(
    items: impl IntoIterator<Item = (tombi_text::Range, Option<tombi_text::Range>)>,
    position: tombi_text::Position,
) -> Option<tombi_text::Range> {
    items.into_iter().find_map(|(item, comma)| {
        let range = comma.map_or(item, |comma| item + comma);
        range.contains(position).then_some(range)
    })
}
