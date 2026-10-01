use std::fmt::Write;

use tombi_diagnostic::Level;

use super::{Report, ReportFormat, level_str};

/// Collects all files and writes a JUnit XML report.
pub(in crate::app::diagnostics) struct JunitFormat;

impl ReportFormat for JunitFormat {
    type Range = tombi_text::Range;

    /// Columns count grapheme clusters, like the output of `pretty`.
    const ENCODING: tombi_text::EncodingKind = tombi_text::EncodingKind::GraphemeCluster;

    fn convert_range(range: tombi_text::Range) -> Self::Range {
        range
    }

    /// Renders a JUnit XML report.
    ///
    /// Each checked file is a test case. Its findings are aggregated into a single `<failure>`,
    /// because consumers such as GitLab keep only the first of test cases with the same name.
    fn render(report: &Report<tombi_text::Range>) -> String {
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
                let _ = write!(attributes, r#" line="{}""#, range.start.line + 1);
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
                            file.display_path,
                            range.start.line + 1,
                            range.start.column + 1
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
    use tombi_text::EncodingKind;

    test_report! {
        #[test]
        fn junit_diagnostics(
            JunitFormat,
            [clean_file("clean.toml"), lint_file("a.toml")],
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
            JunitFormat,
            [not_formatted_file("a.toml")],
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
            JunitFormat,
            [FileReport {
                diagnostics: reported("a = 1\n", EncodingKind::GraphemeCluster, vec![Diagnostic::new_warning("<\"&\u{1b}\">", "code", span("a = 1\n", (0, 0), (0, 1)))]),
                ..clean_file("a&b.toml")
            }],
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
            JunitFormat,
            [],
        ) -> Ok(r#"<?xml version="1.0" encoding="UTF-8"?>
<testsuites name="tombi" tests="1" failures="0" errors="0">
  <testsuite name="tombi" tests="1" failures="0" errors="0" skipped="0">
    <testcase name="No files checked" classname="tombi"/>
  </testsuite>
</testsuites>
"#);
    }
}
