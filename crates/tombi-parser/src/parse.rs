mod array;
mod array_of_table;
mod comma;
mod dangling_comment_group;
mod inline_table;
mod key;
mod key_value;
mod root;
mod table;
mod value;

use crate::{parser::Parser, token_set::TS_LINE_END};

pub(crate) trait Parse {
    fn parse(p: &mut Parser<'_>);
}

pub fn invalid_line(p: &mut Parser<'_>, kind: crate::ErrorKind) {
    p.error(crate::Error::new(kind, p.current_range()));
    p.bump_any();
    while !p.at_ts(TS_LINE_END) {
        p.bump_any();
    }
}
