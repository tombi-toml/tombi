pub(super) mod github;
pub(super) mod gitlab;
pub(super) mod json;
pub(super) mod json_lines;
pub(super) mod junit;
pub(super) mod sarif;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use tombi_diagnostic::{Diagnostic, Level};
use unicode_segmentation::UnicodeSegmentation;

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

/// The result of checking a single file.
#[derive(Debug, Default)]
pub struct FileReport {
    /// `None` only for stdin without `--stdin-filename`, which is rejected for report formats.
    pub path: Option<PathBuf>,
    pub source: String,
    pub diagnostics: Vec<Diagnostic>,
    pub problem: Option<FileProblem>,
}

impl FileReport {
    /// Creates a report, attaching `path` to the diagnostics.
    pub fn new(path: Option<PathBuf>, source: String, diagnostics: Vec<Diagnostic>) -> Self {
        let diagnostics = match &path {
            Some(path) => diagnostics
                .into_iter()
                .map(|diagnostic| diagnostic.with_source_file(path))
                .collect(),
            None => diagnostics,
        };
        Self {
            path,
            source,
            diagnostics,
            problem: None,
        }
    }
}

/// Files collected by reporters that write a single report at the end.
#[derive(Debug)]
pub(super) struct CollectedFiles {
    files: Vec<CollectedFile>,
    execution_successful: bool,
}

/// A checked file whose positions are already converted, so that its source can be dropped.
#[derive(Debug)]
pub(super) struct CollectedFile {
    path: Option<PathBuf>,
    /// Sorted by position, code, and message.
    findings: Vec<CollectedFinding>,
}

#[derive(Debug)]
struct CollectedFinding {
    level: Level,
    code: String,
    message: String,
    range: Option<ReportRange>,
}

impl From<FileReport> for CollectedFile {
    fn from(file: FileReport) -> Self {
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
        if !file.diagnostics.is_empty() {
            let lines = SourceLines::new(&file.source);
            findings.extend(file.diagnostics.iter().map(|diagnostic| CollectedFinding {
                level: diagnostic.level(),
                code: diagnostic.code().to_owned(),
                message: diagnostic.message().to_owned(),
                range: Some(lines.range(diagnostic.range())),
            }));
        }
        findings
            .sort_by(|a, b| (a.range, &a.code, &a.message).cmp(&(b.range, &b.code, &b.message)));

        Self {
            path: file.path,
            findings,
        }
    }
}

impl Default for CollectedFiles {
    fn default() -> Self {
        Self {
            files: Vec::new(),
            execution_successful: true,
        }
    }
}

impl CollectedFiles {
    pub(super) fn record(&mut self, file: FileReport) {
        self.files.push(file.into());
    }

    pub(super) fn record_runtime_error(&mut self) {
        self.execution_successful = false;
    }

    /// Renders the collected files into the writer.
    pub(super) fn finish(
        &self,
        writer: &mut dyn std::io::Write,
        render: fn(&Report) -> String,
    ) -> std::io::Result<()> {
        let cwd = std::env::current_dir()?;
        let project_root = project_root(&cwd);
        let report = Report::new(&self.files, self.execution_successful, &cwd, &project_root);
        writer.write_all(render(&report).as_bytes())
    }
}

/// A report shared by all formats, with files and findings in a deterministic order.
pub(super) struct Report<'a> {
    pub files: Vec<ReportFile>,
    pub findings: Vec<Finding<'a>>,
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

pub(super) struct Finding<'a> {
    pub file_index: usize,
    pub level: Level,
    pub code: &'a str,
    pub message: &'a str,
    pub range: Option<ReportRange>,
    /// Number of preceding findings in the same file with the same code and message.
    pub occurrence: usize,
}

/// A 1-based range.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct ReportRange {
    pub start: ReportPosition,
    pub end: ReportPosition,
}

/// A 1-based position.
///
/// `column` counts Unicode code points, and `utf16_column` counts UTF-16 code units.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct ReportPosition {
    pub line: u32,
    pub column: u32,
    pub utf16_column: u32,
}

impl ReportPosition {
    /// `{ "line": _, "column": _ }` with the code point column.
    pub fn to_json(self) -> serde_json::Value {
        serde_json::json!({
            "line": self.line,
            "column": self.column,
        })
    }
}

impl<'a> Report<'a> {
    pub fn new(
        files: &'a [CollectedFile],
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
    pub fn file(&self, finding: &Finding) -> &ReportFile {
        &self.files[finding.file_index]
    }

    /// Findings of the file at `file_index`, which are contiguous in `findings`.
    pub fn findings_of(&self, file_index: usize) -> &[Finding<'a>] {
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

/// Converts grapheme-based positions of tombi into code point and UTF-16 columns.
struct SourceLines<'a> {
    lines: Vec<&'a str>,
}

impl<'a> SourceLines<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            lines: source
                .split('\n')
                .map(|line| line.strip_suffix('\r').unwrap_or(line))
                .collect(),
        }
    }

    fn range(&self, range: tombi_text::Range) -> ReportRange {
        ReportRange {
            start: self.position(range.start),
            end: self.position(range.end),
        }
    }

    fn position(&self, position: tombi_text::Position) -> ReportPosition {
        let mut column = 0;
        let mut utf16_column = 0;
        let mut graphemes = 0;

        if let Some(line) = self.lines.get(position.line as usize) {
            for grapheme in line.graphemes(true).take(position.column as usize) {
                column += grapheme.chars().count() as u32;
                utf16_column += grapheme.encode_utf16().count() as u32;
                graphemes += 1;
            }
        }

        // Positions past the end of the line are kept as they are.
        let rest = position.column.saturating_sub(graphemes);
        ReportPosition {
            line: position.line + 1,
            column: column + rest + 1,
            utf16_column: utf16_column + rest + 1,
        }
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
    use tombi_text::{Position, Range};

    use super::*;

    /// Renders a report for the given files, relative to the fixed root `/project`.
    macro_rules! test_report {
        (
            #[test]
            fn $name:ident(
                [$($file:expr),* $(,)?],
                $render:path,
            ) -> Ok($expected:expr);
        ) => {
            #[test]
            fn $name() {
                let root = test_root();
                let files = collect(vec![$($file),*]);
        let report = Report::new(&files, true, &root, &root);
                pretty_assertions::assert_eq!($render(&report), $expected);
            }
        };
    }
    pub(super) use test_report;

    pub(super) fn collect(files: Vec<FileReport>) -> Vec<CollectedFile> {
        files.into_iter().map(CollectedFile::from).collect()
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

    /// A file with an error at 1:1-1:4 and a warning at 2:1-2:2.
    pub(super) fn lint_file(path: &str) -> FileReport {
        FileReport {
            path: Some(PathBuf::from(path)),
            source: "key\nb = 1\n".to_owned(),
            diagnostics: vec![
                Diagnostic::new_warning("unused key", "key-unused", range((1, 0), (1, 1))),
                Diagnostic::new_error("expected '='", "expected-equal", range((0, 0), (0, 3))),
            ],
            problem: None,
        }
    }

    pub(super) fn clean_file(path: &str) -> FileReport {
        FileReport {
            path: Some(PathBuf::from(path)),
            source: "a = 1\n".to_owned(),
            ..Default::default()
        }
    }

    pub(super) fn not_formatted_file(path: &str) -> FileReport {
        FileReport {
            path: Some(PathBuf::from(path)),
            source: "a=1\n".to_owned(),
            problem: Some(FileProblem::NotFormatted),
            ..Default::default()
        }
    }

    macro_rules! test_position {
        ($name:ident: $source:expr, ($line:expr, $column:expr) => ($expected_column:expr, $expected_utf16:expr)) => {
            #[test]
            fn $name() {
                let position = SourceLines::new($source).position(Position::new($line, $column));
                pretty_assertions::assert_eq!(
                    (position.line, position.column, position.utf16_column),
                    ($line + 1, $expected_column, $expected_utf16)
                );
            }
        };
    }

    test_position!(ascii: "a = 1", (0, 4) => (5, 5));
    test_position!(cjk: "キー = 1", (0, 5) => (6, 6));
    test_position!(emoji: "\"😀\" = 1", (0, 3) => (4, 5));
    test_position!(combining_mark: "\"e\u{301}\" = 1", (0, 3) => (5, 5));
    test_position!(crlf: "a = 1\r\n\"😀\" = 2", (1, 3) => (4, 5));
    test_position!(past_end_of_line: "a", (0, 3) => (4, 4));
    test_position!(past_end_of_file: "a", (2, 1) => (2, 2));

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
        let files = collect(files);
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
            source: "a = 1\na = 2\n".to_owned(),
            diagnostics: vec![
                Diagnostic::new_error("duplicate key", "key-duplicated", range((1, 0), (1, 1))),
                Diagnostic::new_error("duplicate key", "key-duplicated", range((0, 0), (0, 1))),
            ],
            problem: None,
        }];
        let files = collect(files);
        let report = Report::new(&files, true, &root, &root);

        pretty_assertions::assert_eq!(
            report
                .findings
                .iter()
                .map(|finding| (finding.range.unwrap().start.line, finding.occurrence))
                .collect::<Vec<_>>(),
            vec![(1, 0), (2, 1)]
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
        let files = collect(files);
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
