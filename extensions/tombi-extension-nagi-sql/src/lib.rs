mod completion;
mod goto_declaration;
mod goto_definition;
mod references;
mod workspace;

pub use completion::completion;
pub use goto_declaration::goto_declaration;
pub use goto_definition::{get_current_declaration, goto_definition};
pub use references::{references, references_enabled};
pub use workspace::is_nagi_config;
