use std::collections::BTreeSet;
use std::path::Path;

use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};
use serde_json::json;
use tombi_text::EncodingKind;

use super::{CollectedFiles, FileReport, Report, level_str, to_slash};
use crate::app::diagnostics::format_reporter::FormatReporter;

const SRCROOT: &str = "%SRCROOT%";

/// Characters that are not allowed in a URI path segment.
const PATH_SEGMENT: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'%')
    .add(b'/')
    .add(b'<')
    .add(b'>')
    .add(b'?')
    .add(b'[')
    .add(b'\\')
    .add(b']')
    .add(b'^')
    .add(b'`')
    .add(b'{')
    .add(b'|')
    .add(b'}');

/// Collects all files and writes a SARIF 2.1.0 report.
///
/// Columns count UTF-16 code units, as declared by `columnKind: utf16CodeUnits`.
#[derive(Debug)]
pub(in crate::app::diagnostics) struct SarifReporter(CollectedFiles);

impl Default for SarifReporter {
    fn default() -> Self {
        Self(CollectedFiles::new(EncodingKind::Utf16))
    }
}

impl FormatReporter for SarifReporter {
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

/// Renders a SARIF 2.1.0 report.
///
/// Columns count UTF-16 code units, as declared by `columnKind`.
/// Artifact URIs are relative to `%SRCROOT%`, the project root.
fn render(report: &Report) -> String {
    let rules = report
        .findings
        .iter()
        .map(|finding| finding.code)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    let artifact_locations = report
        .files
        .iter()
        .enumerate()
        .map(|(index, file)| match &file.project_path {
            Some(path) => json!({
                "uri": encode_path(path),
                "uriBaseId": SRCROOT,
                "index": index,
            }),
            None => json!({
                "uri": file_uri(&file.absolute_path, false),
                "index": index,
            }),
        })
        .collect::<Vec<_>>();

    let results = report
        .findings
        .iter()
        .map(|finding| {
            // GitHub code scanning requires `startLine` even for file-level results.
            let region = match finding.range {
                Some(range) => json!({
                    "startLine": range.start.line + 1,
                    "startColumn": range.start.character + 1,
                    "endLine": range.end.line + 1,
                    "endColumn": range.end.character + 1,
                }),
                None => json!({ "startLine": 1 }),
            };
            json!({
                "ruleId": finding.code,
                "ruleIndex": rules.binary_search(&finding.code).unwrap_or_default(),
                "level": level_str(finding.level),
                "message": { "text": finding.message },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": artifact_locations[finding.file_index],
                        "region": region,
                    },
                }],
            })
        })
        .collect::<Vec<_>>();

    let sarif = json!({
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "tombi",
                    "informationUri": "https://tombi-toml.github.io/tombi",
                    "version": env!("__TOMBI_VERSION").trim_start_matches('v'),
                    "rules": rules.iter().map(|rule| json!({ "id": rule })).collect::<Vec<_>>(),
                },
            },
            "invocations": [{
                "executionSuccessful": report.execution_successful,
            }],
            "originalUriBaseIds": {
                SRCROOT: { "uri": file_uri(&report.project_root, true) },
            },
            "artifacts": artifact_locations
                .iter()
                .map(|location| json!({ "location": location }))
                .collect::<Vec<_>>(),
            "columnKind": "utf16CodeUnits",
            "results": results,
        }],
    });

    let mut output = serde_json::to_string_pretty(&sarif).unwrap_or_default();
    output.push('\n');
    output
}

/// Percent-encodes each segment of a `/`-separated relative path.
fn encode_path(path: &str) -> String {
    path.split('/')
        .map(|segment| utf8_percent_encode(segment, PATH_SEGMENT).to_string())
        .collect::<Vec<_>>()
        .join("/")
}

/// Converts an absolute path into a `file` URI.
fn file_uri(path: &Path, is_dir: bool) -> String {
    uri_from_slash_path(&to_slash(path), is_dir)
}

/// Splits `server/share/dir` of a UNC path into the server, which is the authority, and the rest.
fn split_authority(unc: &str) -> (&str, &str) {
    unc.split_once('/').unwrap_or((unc, ""))
}

/// Converts an absolute path with `/` separators into a `file` URI.
///
/// A UNC path `//server/share/dir` becomes `file://server/share/dir`,
/// where the server is the authority of the URI.
fn uri_from_slash_path(path: &str, is_dir: bool) -> String {
    // `\\?\C:\dir` and `\\?\UNC\server\share` are the verbatim forms of a drive path and a UNC path.
    let (authority, path) = if let Some(unc) = path.strip_prefix("//?/UNC/") {
        split_authority(unc)
    } else if let Some(verbatim) = path.strip_prefix("//?/") {
        ("", verbatim)
    } else if let Some(unc) = path.strip_prefix("//") {
        split_authority(unc)
    } else {
        ("", path.trim_start_matches('/'))
    };

    let mut uri = format!("file://{}/{}", encode_path(authority), encode_path(path));
    if is_dir && !uri.ends_with('/') {
        uri.push('/');
    }
    uri
}

#[cfg(test)]
mod tests {
    use super::super::tests::*;
    use super::*;

    #[cfg(unix)]
    test_report! {
        #[test]
        fn sarif_diagnostics(
            [clean_file("clean.toml"), lint_file("dir name/a#.toml"), not_formatted_file("b.toml")],
            render,
        ) -> Ok(format!(r#"{{
  "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
  "version": "2.1.0",
  "runs": [
    {{
      "tool": {{
        "driver": {{
          "name": "tombi",
          "informationUri": "https://tombi-toml.github.io/tombi",
          "version": "{version}",
          "rules": [
            {{
              "id": "expected-equal"
            }},
            {{
              "id": "key-unused"
            }},
            {{
              "id": "not-formatted"
            }}
          ]
        }}
      }},
      "invocations": [
        {{
          "executionSuccessful": true
        }}
      ],
      "originalUriBaseIds": {{
        "%SRCROOT%": {{
          "uri": "file:///project/"
        }}
      }},
      "artifacts": [
        {{
          "location": {{
            "uri": "b.toml",
            "uriBaseId": "%SRCROOT%",
            "index": 0
          }}
        }},
        {{
          "location": {{
            "uri": "clean.toml",
            "uriBaseId": "%SRCROOT%",
            "index": 1
          }}
        }},
        {{
          "location": {{
            "uri": "dir%20name/a%23.toml",
            "uriBaseId": "%SRCROOT%",
            "index": 2
          }}
        }}
      ],
      "columnKind": "utf16CodeUnits",
      "results": [
        {{
          "ruleId": "not-formatted",
          "ruleIndex": 2,
          "level": "error",
          "message": {{
            "text": "File is not formatted"
          }},
          "locations": [
            {{
              "physicalLocation": {{
                "artifactLocation": {{
                  "uri": "b.toml",
                  "uriBaseId": "%SRCROOT%",
                  "index": 0
                }},
                "region": {{
                  "startLine": 1
                }}
              }}
            }}
          ]
        }},
        {{
          "ruleId": "expected-equal",
          "ruleIndex": 0,
          "level": "error",
          "message": {{
            "text": "expected '='"
          }},
          "locations": [
            {{
              "physicalLocation": {{
                "artifactLocation": {{
                  "uri": "dir%20name/a%23.toml",
                  "uriBaseId": "%SRCROOT%",
                  "index": 2
                }},
                "region": {{
                  "startLine": 1,
                  "startColumn": 1,
                  "endLine": 1,
                  "endColumn": 4
                }}
              }}
            }}
          ]
        }},
        {{
          "ruleId": "key-unused",
          "ruleIndex": 1,
          "level": "warning",
          "message": {{
            "text": "unused key"
          }},
          "locations": [
            {{
              "physicalLocation": {{
                "artifactLocation": {{
                  "uri": "dir%20name/a%23.toml",
                  "uriBaseId": "%SRCROOT%",
                  "index": 2
                }},
                "region": {{
                  "startLine": 2,
                  "startColumn": 1,
                  "endLine": 2,
                  "endColumn": 2
                }}
              }}
            }}
          ]
        }}
      ]
    }}
  ]
}}
"#, version = env!("__TOMBI_VERSION").trim_start_matches('v')));
    }

    #[test]
    fn sarif_columns_are_utf16() {
        use super::super::FileReport;
        use tombi_diagnostic::Diagnostic;

        let root = test_root();
        let files = vec![FileReport {
            source: "\"😀\" = 1\n".to_owned(),
            diagnostics: vec![Diagnostic::new_error(
                "error",
                "code",
                range((0, 3), (0, 4)),
            )],
            ..clean_file("a.toml")
        }];
        let files = collect_with(files, EncodingKind::Utf16);
        let report = Report::new(&files, true, &root, &root);
        let sarif: serde_json::Value = serde_json::from_str(&render(&report)).unwrap();

        pretty_assertions::assert_eq!(
            sarif["runs"][0]["results"][0]["locations"][0]["physicalLocation"]["region"],
            json!({ "startLine": 1, "startColumn": 5, "endLine": 1, "endColumn": 6 })
        );
    }

    macro_rules! test_uri_from_slash_path {
        ($name:ident: $path:expr, $is_dir:expr => $expected:expr) => {
            #[test]
            fn $name() {
                pretty_assertions::assert_eq!(uri_from_slash_path($path, $is_dir), $expected);
            }
        };
    }

    test_uri_from_slash_path!(unix_path_is_encoded_per_segment: "/a b/c%d", true => "file:///a%20b/c%25d/");
    test_uri_from_slash_path!(unix_file: "/project/a.toml", false => "file:///project/a.toml");
    test_uri_from_slash_path!(drive_path_keeps_colon: "C:/project", true => "file:///C:/project/");
    test_uri_from_slash_path!(unc_server_is_authority: "//server/share/dir", true => "file://server/share/dir/");
    test_uri_from_slash_path!(unc_path_is_encoded: "//server/share/a b", false => "file://server/share/a%20b");
    test_uri_from_slash_path!(unc_share_root: "//server/share", true => "file://server/share/");
    test_uri_from_slash_path!(verbatim_drive_path: "//?/C:/project", true => "file:///C:/project/");
    test_uri_from_slash_path!(verbatim_unc_path: "//?/UNC/server/share/dir", true => "file://server/share/dir/");
}
