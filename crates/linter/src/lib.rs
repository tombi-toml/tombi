mod error;
mod lint;
mod linter;
mod rule;
mod validation;

pub use config::LintOptions;
use diagnostic::Diagnostic;
pub use error::Error;
pub use error::ErrorKind;
use lint::Lint;
pub use linter::Linter;
use rule::Rule;
pub use validation::Validate;
