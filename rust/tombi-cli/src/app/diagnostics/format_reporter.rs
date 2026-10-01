use std::io::Write;

use super::FileReport;

/// Writes the diagnostics of checked files in a format.
///
/// A reporter either writes each file as soon as it is recorded,
/// or collects all files and writes a single report in [`FormatReporter::finish`].
pub(super) trait FormatReporter: Send {
    /// The unit of the columns of the ranges in the recorded [`FileReport`]s.
    fn encoding(&self) -> tombi_text::EncodingKind;

    /// Records the result of a checked file.
    fn record(&mut self, file: FileReport, writer: &mut dyn Write) -> std::io::Result<()>;

    /// Records an error that is not a diagnostic, such as a file that could not be opened.
    fn record_runtime_error(&mut self);

    /// Writes what has been collected.
    fn finish(&mut self, writer: &mut dyn Write) -> std::io::Result<()>;

    /// Whether [`FileProblem`](super::report::FileProblem)s are reported as diagnostics.
    ///
    /// If not, the errors that caused them are printed to stderr instead.
    fn reports_file_problems(&self) -> bool;
}
