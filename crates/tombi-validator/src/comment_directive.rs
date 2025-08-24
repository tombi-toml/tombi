mod document;
mod value;

pub use document::*;
pub use value::*;

fn into_directive_diagnostic(
    diagnostic: &tombi_diagnostic::Diagnostic,
    content_range: tombi_text::Range,
) -> tombi_diagnostic::Diagnostic {
    tombi_diagnostic::Diagnostic::new_warning(
        diagnostic.message(),
        diagnostic.code(),
        tombi_text::Range::new(
            content_range.start + tombi_text::RelativePosition::from(diagnostic.range().start),
            content_range.start + tombi_text::RelativePosition::from(diagnostic.range().end),
        ),
    )
}
