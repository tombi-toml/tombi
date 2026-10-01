use std::path::PathBuf;

use super::{CollectedFile, FileReport, Report, ReportFormat, json};
use crate::app::diagnostics::format_reporter::FormatReporter;

/// A diagnostic per line, in the same form as the elements of the JSON array.
pub(in crate::app::diagnostics) struct JsonLinesFormat;

impl ReportFormat for JsonLinesFormat {
    type Range = tombi_text::Range;

    /// Columns count grapheme clusters, like the output of `pretty`.
    const ENCODING: tombi_text::EncodingKind = tombi_text::EncodingKind::GraphemeCluster;

    fn convert_range(range: tombi_text::Range) -> Self::Range {
        range
    }

    /// Renders a JSON object per line, in the same form as the elements of the JSON array.
    fn render(report: &Report<tombi_text::Range>) -> String {
        report
            .findings
            .iter()
            .map(|finding| format!("{}\n", json::diagnostic(report, finding)))
            .collect()
    }
}

/// Writes the diagnostics of each file as JSON Lines as soon as the file is recorded.
///
/// Files are written in the order they are checked, and diagnostics in a file are sorted.
pub(in crate::app::diagnostics) struct JsonLinesReporter {
    cwd: PathBuf,
}

impl JsonLinesReporter {
    pub(in crate::app::diagnostics) fn new() -> std::io::Result<Self> {
        Ok(Self {
            cwd: std::env::current_dir()?,
        })
    }
}

impl FormatReporter for JsonLinesReporter {
    fn encoding(&self) -> tombi_text::EncodingKind {
        JsonLinesFormat::ENCODING
    }

    fn record(&mut self, file: FileReport, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
        let files = [CollectedFile::new::<JsonLinesFormat>(file)];
        // Paths are relative to the current directory, so the project root is not used.
        let report = Report::new(&files, true, &self.cwd, &self.cwd);
        writer.write_all(JsonLinesFormat::render(&report).as_bytes())?;
        writer.flush()
    }

    fn record_runtime_error(&mut self) {}

    fn finish(&mut self, _writer: &mut dyn std::io::Write) -> std::io::Result<()> {
        Ok(())
    }

    fn reports_file_problems(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::*;
    use super::*;

    test_report! {
        #[test]
        fn json_lines_diagnostics(
            JsonLinesFormat,
            [clean_file("clean.toml"), lint_file("a.toml"), not_formatted_file("b.toml")],
        ) -> Ok(concat!(
            r#"{"path":"a.toml","level":"error","code":"expected-equal","message":"expected '='","range":{"start":{"line":1,"column":1},"end":{"line":1,"column":4}}}"#,
            "\n",
            r#"{"path":"a.toml","level":"warning","code":"key-unused","message":"unused key","range":{"start":{"line":2,"column":1},"end":{"line":2,"column":2}}}"#,
            "\n",
            r#"{"path":"b.toml","level":"error","code":"not-formatted","message":"File is not formatted","range":null}"#,
            "\n",
        ));
    }

    test_report! {
        #[test]
        fn json_lines_no_diagnostics(
            JsonLinesFormat,
            [clean_file("a.toml")],
        ) -> Ok("");
    }
}
