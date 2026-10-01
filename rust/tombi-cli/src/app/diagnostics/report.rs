pub(super) mod github;
pub(super) mod gitlab;
pub(super) mod json;
pub(super) mod json_lines;
pub(super) mod junit;
pub(super) mod sarif;

use std::collections::HashMap;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

use tombi_diagnostic::{Diagnostic, Level};
use tombi_text::{EncodingKind, LineIndex};

use super::format_reporter::FormatReporter;

const NOT_FORMATTED_CODE: &str = "not-formatted";
const NOT_FORMATTED_MESSAGE: &str = "File is not formatted";
const IO_ERROR_CODE: &str = "io-error";

/// A problem with a whole file, reported as a diagnostic without a range.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileProblem {
    /// `tombi format --check` found that the file is not formatted.
    NotFormatted,
    /// The file could not be read or written.
    Io(String),
}

impl FileProblem {
    /// The problem of the file that caused the error, if the error is about the file.
    pub fn from_error(error: &crate::Error) -> Option<Self> {
        match error {
            crate::Error::NotFormatted(_) => Some(Self::NotFormatted),
            // The path is the location of the report, so it is not repeated in the message.
            crate::Error::FileOpenFailed { source, .. } => Some(Self::io("Failed to open", source)),
            crate::Error::FileReadFailed { source, .. } => Some(Self::io("Failed to read", source)),
            crate::Error::FileWriteFailed { source, .. } => {
                Some(Self::io("Failed to write", source))
            }
            crate::Error::TombiGlob(tombi_glob::Error::FileNotFound(_)) => {
                Some(Self::Io("File not found".to_owned()))
            }
            // Listed explicitly, so that a new variant has to be classified.
            crate::Error::TombiGlob(_)
            | crate::Error::Io(_)
            | crate::Error::StdinParseFailed
            | crate::Error::FileParseFailed(_) => None,
        }
    }

    fn io(failure: &str, source: &std::io::Error) -> Self {
        Self::Io(format!("{failure} [{source}]"))
    }
}

/// A diagnostic with its range, converted while the source of the file was still alive.
#[derive(Debug, Clone)]
pub struct ReportedDiagnostic {
    pub diagnostic: Diagnostic,
    /// The range of the span, whose columns are in the unit of the reporter's encoding.
    pub range: tombi_text::Range,
}

/// The result of checking a single file.
#[derive(Debug, Default)]
pub struct FileReport {
    /// `None` only for stdin without `--stdin-filename`, which is rejected for report formats.
    pub path: Option<PathBuf>,
    pub diagnostics: Vec<ReportedDiagnostic>,
    pub problem: Option<FileProblem>,
}

impl FileReport {
    /// Creates a report, attaching `path` to the diagnostics.
    ///
    /// The spans of the diagnostics are converted into ranges here, with the line index of the
    /// source and the encoding the reporter needs, so that the source does not outlive the task
    /// that checked the file. `line_index` is `None` if the file could not be read, in which case
    /// there are no diagnostics.
    pub fn new(
        path: Option<PathBuf>,
        line_index: Option<(&LineIndex<'_>, EncodingKind)>,
        diagnostics: Vec<Diagnostic>,
    ) -> Self {
        debug_assert!(diagnostics.is_empty() || line_index.is_some());
        let diagnostics = match &path {
            Some(path) => diagnostics
                .into_iter()
                .map(|diagnostic| diagnostic.with_source_file(path))
                .collect(),
            None => diagnostics,
        };
        let ranges = match line_index {
            Some((line_index, encoding)) => ranges_in_order(line_index, encoding, &diagnostics),
            None => Vec::new(),
        };
        Self {
            path,
            diagnostics: diagnostics
                .into_iter()
                .zip(ranges)
                .map(|(diagnostic, range)| ReportedDiagnostic { diagnostic, range })
                .collect(),
            problem: None,
        }
    }
}

/// Converts the spans of the diagnostics, keeping their order.
///
/// A cursor converts the spans in ascending order without searching the lines again.
fn ranges_in_order(
    line_index: &LineIndex<'_>,
    encoding: EncodingKind,
    diagnostics: &[Diagnostic],
) -> Vec<tombi_text::Range> {
    let mut order = (0..diagnostics.len()).collect::<Vec<_>>();
    order.sort_by_key(|&index| {
        let span = diagnostics[index].span();
        (span.start, span.end)
    });
    let mut cursor = line_index.cursor(encoding);
    let mut ranges = vec![tombi_text::Range::default(); diagnostics.len()];
    for index in order {
        ranges[index] = cursor.range(diagnostics[index].span());
    }
    ranges
}

/// A format that collects all files and renders them as a single report.
pub(super) trait ReportFormat {
    /// The range of a finding, whose 0-based columns are in the unit of the format.
    ///
    /// The format adds 1 when it writes a line or a column.
    type Range: Copy + Send;

    /// The unit of the columns of the ranges.
    const ENCODING: EncodingKind;

    /// Converts a range whose columns are counted in [`Self::ENCODING`].
    fn convert_range(range: tombi_text::Range) -> Self::Range;

    fn render(report: &Report<Self::Range>) -> String;
}

/// Collects all files and writes the report of the format `F` at the end.
pub(super) struct CollectingReporter<F: ReportFormat> {
    files: Vec<CollectedFile<F::Range>>,
    execution_successful: bool,
    format: PhantomData<fn() -> F>,
}

impl<F: ReportFormat> Default for CollectingReporter<F> {
    fn default() -> Self {
        Self {
            files: Vec::new(),
            execution_successful: true,
            format: PhantomData,
        }
    }
}

impl<F: ReportFormat> FormatReporter for CollectingReporter<F> {
    fn encoding(&self) -> EncodingKind {
        F::ENCODING
    }

    fn record(
        &mut self,
        file: FileReport,
        _writer: &mut dyn std::io::Write,
    ) -> std::io::Result<()> {
        self.files.push(CollectedFile::new::<F>(file));
        Ok(())
    }

    fn record_runtime_error(&mut self) {
        self.execution_successful = false;
    }

    fn finish(&mut self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
        let cwd = std::env::current_dir()?;
        let project_root = project_root(&cwd);
        let report = Report::new(&self.files, self.execution_successful, &cwd, &project_root);
        writer.write_all(F::render(&report).as_bytes())
    }

    fn reports_file_problems(&self) -> bool {
        true
    }
}

/// A checked file whose spans are already converted into ranges, so that its source can be dropped.
pub(super) struct CollectedFile<R> {
    path: Option<PathBuf>,
    /// Sorted by position, code, and message.
    findings: Vec<CollectedFinding<R>>,
}

struct CollectedFinding<R> {
    level: Level,
    code: String,
    message: String,
    range: Option<R>,
}

impl<R: Copy> CollectedFile<R> {
    pub(super) fn new<F: ReportFormat<Range = R>>(file: FileReport) -> Self {
        let mut findings = Vec::with_capacity(file.diagnostics.len() + 1);
        if let Some(problem) = file.problem {
            let (code, message) = match problem {
                FileProblem::NotFormatted => (NOT_FORMATTED_CODE, NOT_FORMATTED_MESSAGE.to_owned()),
                FileProblem::Io(message) => (IO_ERROR_CODE, message),
            };
            findings.push(CollectedFinding {
                level: Level::ERROR,
                code: code.to_owned(),
                message,
                range: None,
            });
        }

        fn sort_key(
            reported: &ReportedDiagnostic,
        ) -> (tombi_text::Offset, tombi_text::Offset, &str, &str) {
            let span = reported.diagnostic.span();
            (
                span.start,
                span.end,
                reported.diagnostic.code(),
                reported.diagnostic.message(),
            )
        }
        let mut diagnostics = file.diagnostics;
        diagnostics.sort_by(|a, b| sort_key(a).cmp(&sort_key(b)));
        findings.extend(diagnostics.iter().map(|reported| CollectedFinding {
            level: reported.diagnostic.level(),
            code: reported.diagnostic.code().to_owned(),
            message: reported.diagnostic.message().to_owned(),
            range: Some(F::convert_range(reported.range)),
        }));

        Self {
            path: file.path,
            findings,
        }
    }
}

/// A report shared by all formats, with files and findings in a deterministic order.
pub(super) struct Report<'a, R> {
    pub files: Vec<ReportFile>,
    pub findings: Vec<Finding<'a, R>>,
    pub execution_successful: bool,
    /// Absolute path that `ReportFile::project_path` is relative to.
    pub project_root: PathBuf,
}

pub(super) struct ReportFile {
    /// Relative to the current directory, or absolute if the file is outside of it.
    pub display_path: String,
    /// Relative to the project root (repository root in CI), or `None` if the file is outside of it.
    pub project_path: Option<String>,
    pub absolute_path: PathBuf,
}

impl ReportFile {
    /// Path relative to the project root, falling back to the absolute path.
    pub fn project_path_or_absolute(&self) -> String {
        self.project_path
            .clone()
            .unwrap_or_else(|| to_slash(&self.absolute_path))
    }
}

pub(super) struct Finding<'a, R> {
    pub file_index: usize,
    pub level: Level,
    pub code: &'a str,
    pub message: &'a str,
    /// `None` for a problem with a whole file.
    pub range: Option<R>,
    /// Number of preceding findings in the same file with the same code and message.
    pub occurrence: usize,
}

/// `{ "line": _, "column": _ }` of a 1-based position.
pub(super) fn position_json(position: tombi_text::Position) -> serde_json::Value {
    serde_json::json!({
        "line": position.line + 1,
        "column": position.column + 1,
    })
}

impl<'a, R: Copy> Report<'a, R> {
    pub fn new(
        files: &'a [CollectedFile<R>],
        execution_successful: bool,
        cwd: &Path,
        project_root: &Path,
    ) -> Self {
        let mut report_files = files
            .iter()
            .map(|file| {
                let absolute_path = file
                    .path
                    .as_deref()
                    .map(|path| tombi_fs::normalize(&cwd.join(path)))
                    .unwrap_or_else(|| cwd.join("<stdin>"));
                let report_file = ReportFile {
                    display_path: relative_path(&absolute_path, cwd)
                        .unwrap_or_else(|| to_slash(&absolute_path)),
                    project_path: relative_path(&absolute_path, project_root),
                    absolute_path,
                };
                (report_file, file)
            })
            .collect::<Vec<_>>();
        report_files.sort_by(|(a, _), (b, _)| a.display_path.cmp(&b.display_path));

        let mut findings = Vec::new();
        for (file_index, (_, file)) in report_files.iter().enumerate() {
            let mut occurrences = HashMap::<(&str, &str), usize>::new();
            findings.extend(file.findings.iter().map(|finding| {
                let occurrence = occurrences
                    .entry((&finding.code, &finding.message))
                    .or_default();
                *occurrence += 1;
                Finding {
                    file_index,
                    level: finding.level,
                    code: &finding.code,
                    message: &finding.message,
                    range: finding.range,
                    occurrence: *occurrence - 1,
                }
            }));
        }

        Self {
            files: report_files.into_iter().map(|(file, _)| file).collect(),
            findings,
            execution_successful,
            project_root: project_root.to_owned(),
        }
    }

    #[inline]
    pub fn file(&self, finding: &Finding<R>) -> &ReportFile {
        &self.files[finding.file_index]
    }

    /// Findings of the file at `file_index`, which are contiguous in `findings`.
    pub fn findings_of(&self, file_index: usize) -> &[Finding<'a, R>] {
        let start = self
            .findings
            .partition_point(|finding| finding.file_index < file_index);
        let end = self
            .findings
            .partition_point(|finding| finding.file_index <= file_index);
        &self.findings[start..end]
    }
}

pub(super) const fn level_str(level: Level) -> &'static str {
    match level {
        Level::ERROR => "error",
        Level::WARNING => "warning",
    }
}

/// The root that paths in CI reports are relative to.
///
/// Uses `CI_PROJECT_DIR` (GitLab CI), `GITHUB_WORKSPACE` (GitHub Actions),
/// the nearest directory containing `.git`, or the current directory, in this order.
pub(super) fn project_root(cwd: &Path) -> PathBuf {
    for name in ["CI_PROJECT_DIR", "GITHUB_WORKSPACE"] {
        if let Some(dir) = std::env::var_os(name).filter(|dir| !dir.is_empty()) {
            return tombi_fs::normalize(&cwd.join(dir));
        }
    }

    cwd.ancestors()
        .find(|dir| dir.join(".git").exists())
        .unwrap_or(cwd)
        .to_owned()
}

fn relative_path(path: &Path, base: &Path) -> Option<String> {
    path.strip_prefix(base).ok().map(to_slash)
}

/// Converts a path to a string with `/` separators.
pub(super) fn to_slash(path: &Path) -> String {
    let path = path.to_string_lossy();
    if cfg!(windows) {
        path.replace('\\', "/")
    } else {
        path.into_owned()
    }
}

#[cfg(test)]
mod tests {
    use tombi_text::{EncodingKind, Position, Range};

    use super::json::JsonFormat;
    use super::sarif::SarifFormat;
    use super::*;

    /// Renders a report for the given files, relative to the fixed root `/project`.
    macro_rules! test_report {
        (
            #[test]
            fn $name:ident(
                $format:ty,
                [$($file:expr),* $(,)?],
            ) -> Ok($expected:expr);
        ) => {
            #[test]
            fn $name() {
                let root = test_root();
                let files = collect::<$format>(vec![$($file),*]);
                let report = Report::new(&files, true, &root, &root);
                pretty_assertions::assert_eq!(<$format as ReportFormat>::render(&report), $expected);
            }
        };
    }
    pub(super) use test_report;

    pub(super) fn collect<F: ReportFormat>(files: Vec<FileReport>) -> Vec<CollectedFile<F::Range>> {
        files.into_iter().map(CollectedFile::new::<F>).collect()
    }

    pub(super) fn test_root() -> PathBuf {
        if cfg!(windows) {
            PathBuf::from(r"C:\project")
        } else {
            PathBuf::from("/project")
        }
    }

    pub(super) fn range((sl, sc): (u32, u32), (el, ec): (u32, u32)) -> Range {
        Range::new(Position::new(sl, sc), Position::new(el, ec))
    }

    /// The span of `source` at the range whose columns count grapheme clusters.
    pub(super) fn span(source: &str, start: (u32, u32), end: (u32, u32)) -> tombi_text::Span {
        LineIndex::new(source).span(range(start, end), EncodingKind::GraphemeCluster)
    }

    /// The diagnostics in `source` with their ranges converted in `encoding`, as a task does.
    pub(super) fn reported(
        source: &str,
        encoding: EncodingKind,
        diagnostics: Vec<Diagnostic>,
    ) -> Vec<ReportedDiagnostic> {
        FileReport::new(None, Some((&LineIndex::new(source), encoding)), diagnostics).diagnostics
    }

    const LINT_SOURCE: &str = "key\nb = 1\n";

    /// A file with an error at 1:1-1:4 and a warning at 2:1-2:2.
    pub(super) fn lint_file(path: &str) -> FileReport {
        FileReport {
            path: Some(PathBuf::from(path)),
            diagnostics: reported(
                LINT_SOURCE,
                EncodingKind::GraphemeCluster,
                vec![
                    Diagnostic::new_warning(
                        "unused key",
                        "key-unused",
                        span(LINT_SOURCE, (1, 0), (1, 1)),
                    ),
                    Diagnostic::new_error(
                        "expected '='",
                        "expected-equal",
                        span(LINT_SOURCE, (0, 0), (0, 3)),
                    ),
                ],
            ),
            problem: None,
        }
    }

    pub(super) fn clean_file(path: &str) -> FileReport {
        FileReport {
            path: Some(PathBuf::from(path)),
            ..Default::default()
        }
    }

    pub(super) fn not_formatted_file(path: &str) -> FileReport {
        FileReport {
            path: Some(PathBuf::from(path)),
            problem: Some(FileProblem::NotFormatted),
            ..Default::default()
        }
    }

    /// Renders `source` with one diagnostic at `range` through `CollectingReporter<F>`.
    fn render_through_reporter<F: ReportFormat>(source: &str, range: Range) -> serde_json::Value {
        let mut reporter = CollectingReporter::<F>::default();
        let line_index = LineIndex::new(source);
        let file = FileReport::new(
            Some(PathBuf::from("a.toml")),
            Some((&line_index, F::ENCODING)),
            vec![Diagnostic::new_error(
                "message",
                "code",
                line_index.span(range, EncodingKind::GraphemeCluster),
            )],
        );
        reporter.record(file, &mut std::io::sink()).unwrap();
        let mut output = Vec::new();
        reporter.finish(&mut output).unwrap();
        serde_json::from_slice(&output).unwrap()
    }

    /// `"é" = 1` with the `é` as `e` and a combining mark is a single grapheme cluster,
    /// which is 2 UTF-16 code units.
    #[test]
    fn json_counts_graphemes_and_sarif_counts_utf16_code_units() {
        let source = "\"e\u{301}\" = 1";
        let range = range((0, 1), (0, 2));

        let json = render_through_reporter::<JsonFormat>(source, range);
        pretty_assertions::assert_eq!(
            (
                &json[0]["range"]["start"]["column"],
                &json[0]["range"]["end"]["column"]
            ),
            (&serde_json::json!(2), &serde_json::json!(3))
        );

        let sarif = render_through_reporter::<SarifFormat>(source, range);
        let region = &sarif["runs"][0]["results"][0]["locations"][0]["physicalLocation"]["region"];
        pretty_assertions::assert_eq!(
            (&region["startColumn"], &region["endColumn"]),
            (&serde_json::json!(2), &serde_json::json!(4))
        );
    }

    macro_rules! test_file_problem {
        ($name:ident: $error:expr => $expected:expr) => {
            #[test]
            fn $name() {
                pretty_assertions::assert_eq!(FileProblem::from_error(&$error), $expected);
            }
        };
    }

    test_file_problem!(
        not_formatted_is_file_problem:
        crate::Error::NotFormatted(crate::error::NotFormattedError::from_source("a.toml"))
            => Some(FileProblem::NotFormatted)
    );
    test_file_problem!(
        file_not_found_is_file_problem:
        crate::Error::TombiGlob(tombi_glob::Error::FileNotFound(PathBuf::from("a.toml")))
            => Some(FileProblem::Io("File not found".to_owned()))
    );
    test_file_problem!(
        file_read_failure_is_file_problem_without_path:
        crate::Error::FileReadFailed {
            path: PathBuf::from("a.toml"),
            source: std::io::Error::other("denied"),
        } => Some(FileProblem::Io("Failed to read [denied]".to_owned()))
    );
    #[cfg(unix)]
    test_file_problem!(
        os_error_is_in_brackets:
        crate::Error::FileWriteFailed {
            path: PathBuf::from("a.toml"),
            source: std::io::Error::from_raw_os_error(13),
        } => Some(FileProblem::Io("Failed to write [Permission denied (os error 13)]".to_owned()))
    );
    test_file_problem!(
        io_error_is_not_file_problem:
        crate::Error::Io(std::io::Error::other("denied")) => None
    );
    test_file_problem!(
        parse_failure_is_not_file_problem:
        crate::Error::FileParseFailed(PathBuf::from("a.toml")) => None
    );
    test_file_problem!(
        invalid_pattern_is_not_file_problem:
        crate::Error::TombiGlob(tombi_glob::Error::InvalidPattern { pattern: "[".to_owned() }) => None
    );

    #[test]
    fn findings_are_sorted_by_path_and_position() {
        let root = test_root();
        let files = vec![
            lint_file("b.toml"),
            not_formatted_file("a.toml"),
            lint_file("a.toml"),
        ];
        let files = collect::<JsonFormat>(files);
        let report = Report::new(&files, true, &root, &root);

        pretty_assertions::assert_eq!(
            report
                .findings
                .iter()
                .map(|finding| (report.file(finding).display_path.as_str(), finding.code))
                .collect::<Vec<_>>(),
            vec![
                ("a.toml", NOT_FORMATTED_CODE),
                ("a.toml", "expected-equal"),
                ("a.toml", "key-unused"),
                ("b.toml", "expected-equal"),
                ("b.toml", "key-unused"),
            ]
        );
    }

    #[test]
    fn duplicate_findings_have_increasing_occurrence() {
        let root = test_root();
        let files = vec![FileReport {
            path: Some(PathBuf::from("a.toml")),
            diagnostics: reported(
                "a = 1\na = 2\n",
                EncodingKind::GraphemeCluster,
                vec![
                    Diagnostic::new_error(
                        "duplicate key",
                        "key-duplicated",
                        span("a = 1\na = 2\n", (1, 0), (1, 1)),
                    ),
                    Diagnostic::new_error(
                        "duplicate key",
                        "key-duplicated",
                        span("a = 1\na = 2\n", (0, 0), (0, 1)),
                    ),
                ],
            ),
            problem: None,
        }];
        let files = collect::<JsonFormat>(files);
        let report = Report::new(&files, true, &root, &root);

        pretty_assertions::assert_eq!(
            report
                .findings
                .iter()
                .map(|finding| (finding.range.unwrap().start.line, finding.occurrence))
                .collect::<Vec<_>>(),
            vec![(0, 0), (1, 1)]
        );
    }

    #[test]
    fn paths_outside_of_the_project_are_absolute() {
        let root = test_root();
        let outside = if cfg!(windows) {
            r"C:\other\a.toml"
        } else {
            "/other/a.toml"
        };
        let files = vec![clean_file(outside), clean_file("./sub/../b.toml")];
        let files = collect::<JsonFormat>(files);
        let report = Report::new(&files, true, &root, &root);

        pretty_assertions::assert_eq!(
            report
                .files
                .iter()
                .map(|file| (file.display_path.as_str(), file.project_path.as_deref()))
                .collect::<Vec<_>>(),
            vec![
                (to_slash(Path::new(outside)).as_str(), None),
                ("b.toml", Some("b.toml")),
            ]
        );
    }
}
