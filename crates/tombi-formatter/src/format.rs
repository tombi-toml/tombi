mod array_of_table;
mod comment;
mod key;
mod key_value;
mod root;
mod table;
mod value;

use std::fmt::Write;

use crate::types::AlignmentWidth;

pub trait Format {
    fn format(&self, f: &mut crate::Formatter) -> Result<(), std::fmt::Error>;
}

fn write_trailing_comment_alignment_space(
    f: &mut crate::Formatter,
    trailing_comment_alignment_width: AlignmentWidth,
) -> Result<(), std::fmt::Error> {
    write!(
        f,
        "{}",
        " ".repeat(trailing_comment_alignment_width.value() as usize - f.current_line_width())
    )?;
    Ok(())
}
