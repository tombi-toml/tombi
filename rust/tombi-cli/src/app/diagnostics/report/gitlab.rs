use std::fmt::Write;

use serde_json::json;
use sha2::{Digest, Sha256};
use tombi_diagnostic::Level;

use super::{Finding, Report, ReportFormat, position_json};

/// Collects all files and writes a GitLab Code Quality report.
pub(in crate::app::diagnostics) struct GitlabFormat;

impl ReportFormat for GitlabFormat {
    type Range = tombi_text::Range;

    /// Columns count grapheme clusters, like the output of `pretty`.
    const ENCODING: tombi_text::EncodingKind = tombi_text::EncodingKind::GraphemeCluster;

    fn convert_range(range: tombi_text::Range) -> Self::Range {
        range
    }

    /// Renders a GitLab Code Quality report.
    ///
    /// See <https://docs.gitlab.com/ci/testing/code_quality/#code-quality-report-format>.
    fn render(report: &Report<tombi_text::Range>) -> String {
        let issues = report
            .findings
            .iter()
            .map(|finding| {
                let path = report.file(finding).project_path_or_absolute();
                let location = match finding.range {
                    Some(range) => json!({
                        "path": path,
                        "positions": {
                            "begin": position_json(range.start),
                            "end": position_json(range.end),
                        },
                    }),
                    None => json!({
                        "path": path,
                        "lines": { "begin": 1 },
                    }),
                };

                json!({
                    "description": finding.message,
                    "check_name": finding.code,
                    "fingerprint": fingerprint(&path, finding),
                    "severity": match finding.level {
                        Level::ERROR => "major",
                        Level::WARNING => "minor",
                    },
                    "location": location,
                })
            })
            .collect::<Vec<_>>();

        let mut output = serde_json::to_string_pretty(&issues).unwrap_or_default();
        output.push('\n');
        output
    }
}

/// A fingerprint that is stable across line changes and tombi releases.
///
/// Line numbers are excluded so that a finding keeps its identity when code above it moves.
/// Identical findings in the same file are told apart by their order.
fn fingerprint(path: &str, finding: &Finding<tombi_text::Range>) -> String {
    let mut hasher = Sha256::new();
    for part in [
        path,
        finding.code,
        finding.message,
        &finding.occurrence.to_string(),
    ] {
        hasher.update(part.as_bytes());
        hasher.update([0]);
    }
    hasher
        .finalize()
        .iter()
        .fold(String::with_capacity(64), |mut hex, byte| {
            let _ = write!(hex, "{byte:02x}");
            hex
        })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use tombi_diagnostic::Diagnostic;

    use super::super::{FileReport, tests::*};
    use super::*;
    use tombi_text::EncodingKind;

    test_report! {
        #[test]
        fn gitlab_diagnostics(
            GitlabFormat,
            [clean_file("clean.toml"), lint_file("sub/a.toml"), not_formatted_file("b.toml")],
        ) -> Ok(r#"[
  {
    "description": "File is not formatted",
    "check_name": "not-formatted",
    "fingerprint": "da5f387228d6c666ff72458df32f65cf33716523f43e79410b45876daaabbd9b",
    "severity": "major",
    "location": {
      "path": "b.toml",
      "lines": {
        "begin": 1
      }
    }
  },
  {
    "description": "expected '='",
    "check_name": "expected-equal",
    "fingerprint": "8ac9d44d573eaff5646cc5d84cf802c6205bee624255b6608b189044f40c357f",
    "severity": "major",
    "location": {
      "path": "sub/a.toml",
      "positions": {
        "begin": {
          "line": 1,
          "column": 1
        },
        "end": {
          "line": 1,
          "column": 4
        }
      }
    }
  },
  {
    "description": "unused key",
    "check_name": "key-unused",
    "fingerprint": "e87030ae0a27c1f904ee971f6211b4ea7c746bdf1bde655d519bcd88d6f5f3d7",
    "severity": "minor",
    "location": {
      "path": "sub/a.toml",
      "positions": {
        "begin": {
          "line": 2,
          "column": 1
        },
        "end": {
          "line": 2,
          "column": 2
        }
      }
    }
  }
]
"#);
    }

    test_report! {
        #[test]
        fn gitlab_no_diagnostics(
            GitlabFormat,
            [clean_file("a.toml")],
        ) -> Ok("[]\n");
    }

    fn fingerprints(files: Vec<FileReport>) -> Vec<String> {
        let root = test_root();
        let files = collect::<GitlabFormat>(files);
        let report = Report::new(&files, true, &root, &root);
        report
            .findings
            .iter()
            .map(|finding| fingerprint(&report.file(finding).project_path_or_absolute(), finding))
            .collect()
    }

    fn duplicated_key_file(lines: &[u32]) -> FileReport {
        FileReport {
            path: Some(PathBuf::from("a.toml")),
            diagnostics: reported(
                &"a\n".repeat(10),
                EncodingKind::GraphemeCluster,
                lines
                    .iter()
                    .map(|line| {
                        Diagnostic::new_error(
                            "duplicate key",
                            "key-duplicated",
                            span(&"a\n".repeat(10), (*line, 0), (*line, 1)),
                        )
                    })
                    .collect(),
            ),
            problem: None,
        }
    }

    #[test]
    fn fingerprint_ignores_line_numbers() {
        pretty_assertions::assert_eq!(
            fingerprints(vec![duplicated_key_file(&[1])]),
            fingerprints(vec![duplicated_key_file(&[5])])
        );
    }

    #[test]
    fn fingerprint_distinguishes_duplicates() {
        let fingerprints = fingerprints(vec![duplicated_key_file(&[1, 3])]);
        assert_ne!(fingerprints[0], fingerprints[1]);
    }
}
