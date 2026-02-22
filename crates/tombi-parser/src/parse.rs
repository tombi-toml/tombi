mod array;
mod array_of_table;
mod comma;
mod dangling_comment_group;
mod inline_table;
mod key;
mod key_value;
mod key_value_group;
mod key_value_with_comma_group;
mod root;
mod table;
mod value;
mod value_with_comma_group;

use crate::{parser::Parser, token_set::TS_LINE_END};

pub(crate) trait Parse {
    fn parse(p: &mut Parser<'_>);
}

fn invalid_line(p: &mut Parser<'_>, kind: crate::ErrorKind) {
    p.error(crate::Error::new(kind, p.current_range()));
    p.bump_any();
    while !p.at_ts(TS_LINE_END) {
        p.bump_any();
    }
}

fn is_group_separator(p: &mut Parser<'_>) -> bool {
    p.at_ts(TS_LINE_END) && p.nth_at_ts(1, TS_LINE_END)
}
