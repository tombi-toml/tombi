use std::collections::BTreeSet;
use std::path::Path;

use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};
use serde_json::json;
use tombi_text::EncodingKind;

use super::{Report, ReportFormat, level_str, to_slash};

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

/// Characters that are not allowed in a segment of a relative reference.
///
/// A colon is encoded too, because a first segment that contains one, such as `dir:name/a.toml`,
/// would be taken as a URI scheme.
const RELATIVE_PATH_SEGMENT: &AsciiSet = &PATH_SEGMENT.add(b':');

/// Collects all files and writes a SARIF 2.1.0 report.
pub(in crate::app::diagnostics) struct SarifFormat;

impl ReportFormat for SarifFormat {
    /// Columns count UTF-16 code units, as declared by `columnKind: utf16CodeUnits`.
    type Range = tower_lsp::lsp_types::Range;

    const ENCODING: EncodingKind = EncodingKind::Utf16;

    fn convert_range(range: tombi_text::Range) -> Self::Range {
        tower_lsp::lsp_types::Range::new(
            tower_lsp::lsp_types::Position::new(range.start.line, range.start.column),
            tower_lsp::lsp_types::Position::new(range.end.line, range.end.column),
        )
    }

    /// Renders a SARIF 2.1.0 report.
    ///
    /// Columns count UTF-16 code units, as declared by `columnKind`.
    /// Artifact URIs are relative to `%SRCROOT%`, the project root.
    fn render(report: &Report<tower_lsp::lsp_types::Range>) -> String {
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
                    "uri": encode_path(path, RELATIVE_PATH_SEGMENT),
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
}

/// Percent-encodes each segment of a `/`-separated path.
fn encode_path(path: &str, set: &'static AsciiSet) -> String {
    path.split('/')
        .map(|segment| utf8_percent_encode(segment, set).to_string())
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

    let mut uri = format!(
        "file://{}/{}",
        encode_path(authority, PATH_SEGMENT),
        encode_path(path, PATH_SEGMENT)
    );
    if is_dir && !uri.ends_with('/') {
        uri.push('/');
    }
    uri
}

#[cfg(test)]
mod tests {
    use tombi_diagnostic::Diagnostic;
    use tombi_text::LineIndex;

    use super::super::{CollectedFile, FileReport, tests::*};
    use super::*;

    #[cfg(unix)]
    test_report! {
        #[test]
        fn sarif_diagnostics(
            SarifFormat,
            [clean_file("clean.toml"), lint_file("dir name/a#.toml"), not_formatted_file("b.toml")],
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
        let root = test_root();
        let files = vec![FileReport {
            diagnostics: reported(
                "\"😀\" = 1\n",
                EncodingKind::Utf16,
                vec![Diagnostic::new_error(
                    "error",
                    "code",
                    span("\"😀\" = 1\n", (0, 3), (0, 4)),
                )],
            ),
            ..clean_file("a.toml")
        }];
        let files = collect::<SarifFormat>(files);
        let report = Report::new(&files, true, &root, &root);
        let sarif: serde_json::Value = serde_json::from_str(&SarifFormat::render(&report)).unwrap();

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

    /// Converts `$range` of a diagnostic in `$source` into UTF-16 columns.
    macro_rules! test_utf16_columns {
        ($name:ident: $source:expr, $range:expr => $expected:expr) => {
            #[test]
            fn $name() {
                let (start, end) = $range;
                let line_index = LineIndex::new($source);
                let file = FileReport::new(
                    Some(std::path::PathBuf::from("a.toml")),
                    Some((&line_index, EncodingKind::Utf16)),
                    vec![Diagnostic::new_error(
                        "message",
                        "code",
                        span($source, start, end),
                    )],
                );
                let range = CollectedFile::new::<SarifFormat>(file).findings[0]
                    .range
                    .unwrap();
                pretty_assertions::assert_eq!(
                    (
                        range.start.line,
                        range.start.character,
                        range.end.line,
                        range.end.character
                    ),
                    $expected
                );
            }
        };
    }

    test_utf16_columns!(ascii: "a = 1", ((0, 0), (0, 4)) => (0, 0, 0, 4));
    test_utf16_columns!(cjk: "キー = 1", ((0, 0), (0, 2)) => (0, 0, 0, 2));
    test_utf16_columns!(emoji: "\"😀\" = 1", ((0, 1), (0, 2)) => (0, 1, 0, 3));
    test_utf16_columns!(combining_mark: "\"e\u{301}\" = 1", ((0, 1), (0, 2)) => (0, 1, 0, 3));
    test_utf16_columns!(crlf: "a = 1\r\n\"😀\" = 2", ((1, 1), (1, 2)) => (1, 1, 1, 3));
    test_utf16_columns!(multiple_lines: "a = \"\"\"\n😀\n\"\"\"", ((0, 4), (2, 3)) => (0, 4, 2, 3));
    test_utf16_columns!(past_end_of_line_is_clamped: "a", ((0, 0), (0, 3)) => (0, 0, 0, 1));

    macro_rules! test_encode_relative_path {
        ($name:ident: $path:expr => $expected:expr) => {
            #[test]
            fn $name() {
                pretty_assertions::assert_eq!(encode_path($path, RELATIVE_PATH_SEGMENT), $expected);
            }
        };
    }

    test_encode_relative_path!(colon_in_first_segment: "dir:name/a.toml" => "dir%3Aname/a.toml");
    test_encode_relative_path!(colon_in_file_name: "dir/a:b.toml" => "dir/a%3Ab.toml");
    test_encode_relative_path!(spaces_and_hashes: "a b/c#d.toml" => "a%20b/c%23d.toml");
    test_encode_relative_path!(plain_path: "sub/a.toml" => "sub/a.toml");

    #[cfg(unix)]
    #[test]
    fn relative_artifact_uri_has_no_scheme_like_colon() {
        let root = test_root();
        let files = collect::<SarifFormat>(vec![clean_file("dir:name/a.toml")]);
        let report = Report::new(&files, true, &root, &root);
        let sarif: serde_json::Value = serde_json::from_str(&SarifFormat::render(&report)).unwrap();

        pretty_assertions::assert_eq!(
            sarif["runs"][0]["artifacts"][0]["location"]["uri"],
            "dir%3Aname/a.toml"
        );
    }
}
