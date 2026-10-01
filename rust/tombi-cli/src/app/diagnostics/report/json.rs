use serde_json::json;

use super::{Finding, Report, ReportFormat, level_str, position_json};

/// Collects all files and writes a JSON array of diagnostics.
pub(in crate::app::diagnostics) struct JsonFormat;

impl ReportFormat for JsonFormat {
    type Range = tombi_text::Range;

    /// Columns count grapheme clusters, like the output of `pretty`.
    const ENCODING: tombi_text::EncodingKind = tombi_text::EncodingKind::GraphemeCluster;

    fn convert_range(range: tombi_text::Range) -> Self::Range {
        range
    }

    /// Renders a JSON array of diagnostics.
    ///
    /// Positions are 1-based and columns count grapheme clusters.
    /// `range` is `null` for file-level diagnostics.
    fn render(report: &Report<tombi_text::Range>) -> String {
        let diagnostics = report
            .findings
            .iter()
            .map(|finding| diagnostic(report, finding))
            .collect::<Vec<_>>();

        let mut output = serde_json::to_string_pretty(&diagnostics).unwrap_or_default();
        output.push('\n');
        output
    }
}

/// A diagnostic as an element of the JSON array, which is also a line of JSON Lines.
pub(super) fn diagnostic(
    report: &Report<tombi_text::Range>,
    finding: &Finding<tombi_text::Range>,
) -> serde_json::Value {
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
    use super::super::{FileProblem, FileReport, tests::*};
    use super::*;

    test_report! {
        #[test]
        fn json_diagnostics(
            JsonFormat,
            [clean_file("clean.toml"), lint_file("a.toml")],
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
            JsonFormat,
            [not_formatted_file("a.toml")],
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
            JsonFormat,
            [FileReport {
                problem: Some(FileProblem::Io("File not found".to_owned())),
                ..clean_file("a.toml")
            }],
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
            JsonFormat,
            [clean_file("a.toml")],
        ) -> Ok("[]\n");
    }
}
