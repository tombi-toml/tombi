mod completion;
mod diagnostics;
mod document_link;
mod extends;
mod goto_definition;
mod hover;

pub use completion::completion;
pub use diagnostics::validate as validate_tombi;
pub use document_link::{DocumentLinkToolTip, document_link};
pub use goto_definition::goto_definition;
pub use hover::hover;
