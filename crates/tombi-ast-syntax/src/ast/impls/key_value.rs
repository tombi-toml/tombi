use tombi_accessor::Accessor;
use tombi_toml_version::TomlVersion;

use crate::AstNode;

impl<'t> crate::KeyValue<'t> {
    /// Source span of this inline-table or array item, extended through its
    /// comma when one is present.
    pub fn item_span_with_comma(&self, offset: tombi_text::Offset) -> Option<tombi_text::Span> {
        for syntax_node in self.syntax().ancestors() {
            if let Some(group) = crate::KeyValueWithCommaGroup::cast(syntax_node) {
                for (item, comma) in group.key_values_with_comma() {
                    if item.syntax() == self.syntax() {
                        let start = item
                            .leading_comments()
                            .next()
                            .map(|comment| comment.syntax().span().start)
                            .or_else(|| item.keys().map(|keys| keys.span().start))?;
                        let end = item
                            .value()
                            .map(|value| value.span().end)
                            .unwrap_or(item.span().end);
                        let span = tombi_text::Span::new(start, end);
                        return Some(comma.map_or(span, |comma| span + comma.span()));
                    }
                }

                if let Some(span) = span_containing_offset(
                    group
                        .key_values_with_comma()
                        .map(|(item, comma)| (item.span(), comma.map(|comma| comma.span()))),
                    offset,
                ) {
                    return Some(span);
                }
            } else if let Some(group) = crate::ValueWithCommaGroup::cast(syntax_node)
                && let Some(span) = span_containing_offset(
                    group
                        .value_or_key_values_with_comma()
                        .map(|(item, comma)| (item.span(), comma.map(|comma| comma.span()))),
                    offset,
                )
            {
                return Some(span);
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

fn span_containing_offset(
    items: impl IntoIterator<Item = (tombi_text::Span, Option<tombi_text::Span>)>,
    offset: tombi_text::Offset,
) -> Option<tombi_text::Span> {
    items.into_iter().find_map(|(item, comma)| {
        let span = comma.map_or(item, |comma| item + comma);
        span.contains_inclusive(offset).then_some(span)
    })
}
