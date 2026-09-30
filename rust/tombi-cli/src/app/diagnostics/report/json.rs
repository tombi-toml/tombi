use serde_json::json;

use super::{CollectedFiles, FileReport, Finding, Report, level_str, position_json};
use crate::app::diagnostics::format_reporter::FormatReporter;

/// Collects all files and writes a JSON array of diagnostics.
#[derive(Debug, Default)]
pub(in crate::app::diagnostics) struct JsonReporter(CollectedFiles);

impl FormatReporter for JsonReporter {
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

/// Renders a JSON array of diagnostics.
///
/// Positions are 1-based and columns count Unicode code points.
/// `range` is `null` for file-level diagnostics.
fn render(report: &Report) -> String {
    let diagnostics = report
        .findings
        .iter()
        .map(|finding| diagnostic(report, finding))
        .collect::<Vec<_>>();

    let mut output = serde_json::to_string_pretty(&diagnostics).unwrap_or_default();
    output.push('\n');
    output
}

/// A diagnostic as an element of the JSON array, which is also a line of JSON Lines.
pub(super) fn diagnostic(report: &Report, finding: &Finding) -> serde_json::Value {
    json!({
        "path": report.file(finding).display_path,
        "level": level_str(finding.level),
        "code": finding.code,
        "message": finding.message,
        "range": finding.range.map(|range| json!({
            "start": position_json(range.start),
            "end": position_json(range.end),
        })),
    })
}

#[cfg(test)]
mod tests {
    use super::super::{FileProblem, tests::*};
    use super::*;

    test_report! {
        #[test]
        fn json_diagnostics(
            [clean_file("clean.toml"), lint_file("a.toml")],
            render,
        ) -> Ok(r#"[
  {
    "path": "a.toml",
    "level": "error",
    "code": "expected-equal",
    "message": "expected '='",
    "range": {
      "start": {
        "line": 1,
        "column": 1
      },
      "end": {
        "line": 1,
        "column": 4
      }
    }
  },
  {
    "path": "a.toml",
    "level": "warning",
    "code": "key-unused",
    "message": "unused key",
    "range": {
      "start": {
        "line": 2,
        "column": 1
      },
      "end": {
        "line": 2,
        "column": 2
      }
    }
  }
]
"#);
    }

    test_report! {
        #[test]
        fn json_not_formatted(
            [not_formatted_file("a.toml")],
            render,
        ) -> Ok(r#"[
  {
    "path": "a.toml",
    "level": "error",
    "code": "not-formatted",
    "message": "File is not formatted",
    "range": null
  }
]
"#);
    }

    test_report! {
        #[test]
        fn json_io_error(
            [FileReport {
                problem: Some(FileProblem::Io("File not found".to_owned())),
                ..clean_file("a.toml")
            }],
            render,
        ) -> Ok(r#"[
  {
    "path": "a.toml",
    "level": "error",
    "code": "io-error",
    "message": "File not found",
    "range": null
  }
]
"#);
    }

    test_report! {
        #[test]
        fn json_no_diagnostics(
            [clean_file("a.toml")],
            render,
        ) -> Ok("[]\n");
    }
}
