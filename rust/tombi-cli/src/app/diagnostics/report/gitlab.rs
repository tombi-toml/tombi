use std::fmt::Write;

use serde_json::json;
use sha2::{Digest, Sha256};
use tombi_diagnostic::Level;

use super::{CollectedFiles, FileReport, Finding, Report};
use crate::app::diagnostics::format_reporter::FormatReporter;

/// Collects all files and writes a GitLab Code Quality report.
#[derive(Debug, Default)]
pub(in crate::app::diagnostics) struct GitlabReporter(CollectedFiles);

impl FormatReporter for GitlabReporter {
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

/// Renders a GitLab Code Quality report.
///
/// See <https://docs.gitlab.com/ci/testing/code_quality/#code-quality-report-format>.
fn render(report: &Report) -> String {
    let issues = report
        .findings
        .iter()
        .map(|finding| {
            let path = report.file(finding).project_path_or_absolute();
            let location = match finding.range {
                Some(range) => json!({
                    "path": path,
                    "positions": {
                        "begin": range.start.to_json(),
                        "end": range.end.to_json(),
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

/// A fingerprint that is stable across line changes and tombi releases.
///
/// Line numbers are excluded so that a finding keeps its identity when code above it moves.
/// Identical findings in the same file are told apart by their order.
fn fingerprint(path: &str, finding: &Finding) -> String {
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

    test_report! {
        #[test]
        fn gitlab_diagnostics(
            [clean_file("clean.toml"), lint_file("sub/a.toml"), not_formatted_file("b.toml")],
            render,
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
            [clean_file("a.toml")],
            render,
        ) -> Ok("[]\n");
    }

    fn fingerprints(files: Vec<FileReport>) -> Vec<String> {
        let root = test_root();
        let files = collect(files);
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
            source: "\n".repeat(10),
            diagnostics: lines
                .iter()
                .map(|line| {
                    Diagnostic::new_error(
                        "duplicate key",
                        "key-duplicated",
                        range((*line, 0), (*line, 1)),
                    )
                })
                .collect(),
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
