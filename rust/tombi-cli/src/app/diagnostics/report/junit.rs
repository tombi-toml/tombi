use std::fmt::Write;

use tombi_diagnostic::Level;

use super::{CollectedFiles, FileReport, Report, level_str};
use crate::app::diagnostics::format_reporter::FormatReporter;

/// Collects all files and writes a JUnit XML report.
#[derive(Debug, Default)]
pub(in crate::app::diagnostics) struct JunitReporter(CollectedFiles);

impl FormatReporter for JunitReporter {
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

/// Renders a JUnit XML report.
///
/// Each checked file is a test case. Its findings are aggregated into a single `<failure>`,
/// because consumers such as GitLab keep only the first of test cases with the same name.
fn render(report: &Report) -> String {
    let tests = report.files.len().max(1);
    let failures = (0..report.files.len())
        .filter(|index| !report.findings_of(*index).is_empty())
        .count();

    let mut output = String::new();
    let _ = writeln!(output, r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    let _ = writeln!(
        output,
        r#"<testsuites name="tombi" tests="{tests}" failures="{failures}" errors="0">"#
    );
    let _ = writeln!(
        output,
        r#"  <testsuite name="tombi" tests="{tests}" failures="{failures}" errors="0" skipped="0">"#
    );

    // Some consumers such as Jenkins reject a report without test cases.
    if report.files.is_empty() {
        let _ = writeln!(
            output,
            r#"    <testcase name="No files checked" classname="tombi"/>"#
        );
    }

    for (index, file) in report.files.iter().enumerate() {
        let findings = report.findings_of(index);
        let path = escape(&file.display_path);
        let mut attributes = format!(r#"name="{path}" classname="tombi" file="{path}""#);

        if findings.is_empty() {
            let _ = writeln!(output, "    <testcase {attributes}/>");
            continue;
        }

        if let Some(range) = findings.iter().find_map(|finding| finding.range) {
            let _ = write!(attributes, r#" line="{}""#, range.start.line);
        }
        let failure_type = if findings.iter().any(|finding| finding.level == Level::ERROR) {
            "error"
        } else {
            "warning"
        };
        let message = match findings.len() {
            1 => findings[0].message.to_owned(),
            n => format!("{n} problems"),
        };
        let body = findings
            .iter()
            .map(|finding| {
                let location = match finding.range {
                    Some(range) => format!(
                        "{}:{}:{}",
                        file.display_path, range.start.line, range.start.column
                    ),
                    None => file.display_path.clone(),
                };
                format!(
                    "{location}: {} [{}] {}",
                    level_str(finding.level),
                    finding.code,
                    finding.message
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        let _ = writeln!(output, "    <testcase {attributes}>");
        let _ = writeln!(
            output,
            r#"      <failure type="{failure_type}" message="{}">{}</failure>"#,
            escape(&message),
            escape(&body)
        );
        let _ = writeln!(output, "    </testcase>");
    }

    let _ = writeln!(output, "  </testsuite>");
    let _ = writeln!(output, "</testsuites>");
    output
}

/// Escapes text for XML attributes and content, replacing characters that XML 1.0 does not allow.
fn escape(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for c in value.chars() {
        match c {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&apos;"),
            '\t' | '\n' | '\r' => escaped.push(c),
            '\u{0}'..='\u{1F}' | '\u{FFFE}' | '\u{FFFF}' => escaped.push('\u{FFFD}'),
            c => escaped.push(c),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use tombi_diagnostic::Diagnostic;

    use super::super::{FileReport, tests::*};
    use super::*;

    test_report! {
        #[test]
        fn junit_diagnostics(
            [clean_file("clean.toml"), lint_file("a.toml")],
            render,
        ) -> Ok(r#"<?xml version="1.0" encoding="UTF-8"?>
<testsuites name="tombi" tests="2" failures="1" errors="0">
  <testsuite name="tombi" tests="2" failures="1" errors="0" skipped="0">
    <testcase name="a.toml" classname="tombi" file="a.toml" line="1">
      <failure type="error" message="2 problems">a.toml:1:1: error [expected-equal] expected &apos;=&apos;
a.toml:2:1: warning [key-unused] unused key</failure>
    </testcase>
    <testcase name="clean.toml" classname="tombi" file="clean.toml"/>
  </testsuite>
</testsuites>
"#);
    }

    test_report! {
        #[test]
        fn junit_not_formatted(
            [not_formatted_file("a.toml")],
            render,
        ) -> Ok(r#"<?xml version="1.0" encoding="UTF-8"?>
<testsuites name="tombi" tests="1" failures="1" errors="0">
  <testsuite name="tombi" tests="1" failures="1" errors="0" skipped="0">
    <testcase name="a.toml" classname="tombi" file="a.toml">
      <failure type="error" message="File is not formatted">a.toml: error [not-formatted] File is not formatted</failure>
    </testcase>
  </testsuite>
</testsuites>
"#);
    }

    test_report! {
        #[test]
        fn junit_escapes_xml(
            [FileReport {
                diagnostics: vec![Diagnostic::new_warning("<\"&\u{1b}\">", "code", range((0, 0), (0, 1)))],
                ..clean_file("a&b.toml")
            }],
            render,
        ) -> Ok("<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<testsuites name=\"tombi\" tests=\"1\" failures=\"1\" errors=\"0\">
  <testsuite name=\"tombi\" tests=\"1\" failures=\"1\" errors=\"0\" skipped=\"0\">
    <testcase name=\"a&amp;b.toml\" classname=\"tombi\" file=\"a&amp;b.toml\" line=\"1\">
      <failure type=\"warning\" message=\"&lt;&quot;&amp;\u{FFFD}&quot;&gt;\">a&amp;b.toml:1:1: warning [code] &lt;&quot;&amp;\u{FFFD}&quot;&gt;</failure>
    </testcase>
  </testsuite>
</testsuites>
");
    }

    test_report! {
        #[test]
        fn junit_no_files(
            [],
            render,
        ) -> Ok(r#"<?xml version="1.0" encoding="UTF-8"?>
<testsuites name="tombi" tests="1" failures="0" errors="0">
  <testsuite name="tombi" tests="1" failures="0" errors="0" skipped="0">
    <testcase name="No files checked" classname="tombi"/>
  </testsuite>
</testsuites>
"#);
    }
}
