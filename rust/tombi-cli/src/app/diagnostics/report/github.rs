use std::fmt::Write;

use super::{CollectedFiles, FileReport, Report, level_str};
use crate::app::diagnostics::format_reporter::FormatReporter;

/// Collects all files and writes a GitHub Actions workflow commands.
#[derive(Debug, Default)]
pub(in crate::app::diagnostics) struct GithubReporter(CollectedFiles);

impl FormatReporter for GithubReporter {
    fn record(
        &mut self,
        file: FileReport,
        _writer: &mut dyn std::io::Write,
    ) -> std::io::Result<()> {
        self.0.record(file);
        Ok(())
    }

    fn record_runtime_error(&mut self) {
        self.0.record_runtime_error();
    }

    fn finish(&mut self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
        self.0.finish(writer, render)
    }

    fn reports_file_problems(&self) -> bool {
        true
    }
}

/// Renders GitHub Actions workflow commands, one annotation per line.
///
/// See <https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-commands>.
fn render(report: &Report) -> String {
    let mut output = String::new();

    for finding in &report.findings {
        let file = report.file(finding);
        let command = level_str(finding.level);

        // The runner converts absolute paths in the workspace to repository-relative paths.
        let mut properties = vec![format!(
            "file={}",
            escape_property(&file.absolute_path.to_string_lossy())
        )];
        let location = match finding.range {
            Some(range) => {
                properties.push(format!("line={}", range.start.line));
                properties.push(format!("endLine={}", range.end.line));
                // The runner drops columns of annotations spanning multiple lines.
                if range.start.line == range.end.line {
                    properties.push(format!("col={}", range.start.column));
                    properties.push(format!("endColumn={}", range.end.column));
                }
                format!(
                    "{}:{}:{}",
                    file.display_path, range.start.line, range.start.column
                )
            }
            None => file.display_path.clone(),
        };
        properties.push(format!(
            "title={}",
            escape_property(&format!("tombi ({})", finding.code))
        ));

        let _ = writeln!(
            output,
            "::{command} {}::{}",
            properties.join(","),
            escape_data(&format!("{location}: {}", finding.message))
        );
    }

    output
}

fn escape_data(value: &str) -> String {
    value
        .replace('%', "%25")
        .replace('\r', "%0D")
        .replace('\n', "%0A")
}

fn escape_property(value: &str) -> String {
    escape_data(value).replace(':', "%3A").replace(',', "%2C")
}

#[cfg(test)]
mod tests {
    use tombi_diagnostic::Diagnostic;

    use super::super::{FileReport, tests::*};
    use super::*;

    #[cfg(unix)]
    test_report! {
        #[test]
        fn github_diagnostics(
            [clean_file("clean.toml"), lint_file("a.toml"), not_formatted_file("b.toml")],
            render,
        ) -> Ok(concat!(
            "::error file=/project/a.toml,line=1,endLine=1,col=1,endColumn=4,title=tombi (expected-equal)::a.toml:1:1: expected '='\n",
            "::warning file=/project/a.toml,line=2,endLine=2,col=1,endColumn=2,title=tombi (key-unused)::a.toml:2:1: unused key\n",
            "::error file=/project/b.toml,title=tombi (not-formatted)::b.toml: File is not formatted\n",
        ));
    }

    #[cfg(unix)]
    test_report! {
        #[test]
        fn github_multiline_range_has_no_columns(
            [FileReport {
                source: "a = \"\"\"\n\"\"\"\n".to_owned(),
                diagnostics: vec![Diagnostic::new_error("invalid", "invalid", range((0, 4), (1, 3)))],
                ..clean_file("a.toml")
            }],
            render,
        ) -> Ok("::error file=/project/a.toml,line=1,endLine=2,title=tombi (invalid)::a.toml:1:5: invalid\n");
    }

    #[cfg(unix)]
    test_report! {
        #[test]
        fn github_escapes_data_and_properties(
            [FileReport {
                diagnostics: vec![Diagnostic::new_error("100%\r\nsure: a, b", "a:b,c", range((0, 0), (0, 1)))],
                ..clean_file("a,b:c.toml")
            }],
            render,
        ) -> Ok("::error file=/project/a%2Cb%3Ac.toml,line=1,endLine=1,col=1,endColumn=2,title=tombi (a%3Ab%2Cc)::a,b:c.toml:1:1: 100%25%0D%0Asure: a, b\n");
    }

    test_report! {
        #[test]
        fn github_no_diagnostics(
            [clean_file("a.toml")],
            render,
        ) -> Ok("");
    }
}
