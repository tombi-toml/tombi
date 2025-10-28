use std::path::PathBuf;

use nu_ansi_term::Style;
use tombi_diagnostic::{
    printer::{Pretty, Simple},
    Level, Print,
};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    NotFormatted(#[from] NotFormattedError),

    #[error(transparent)]
    TombiGlob(#[from] tombi_glob::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("stdin failed to parse")]
    StdinParseFailed,

    #[error("{0:?} failed to parse")]
    FileParseFailed(PathBuf),
}

#[derive(thiserror::Error, Debug)]
pub struct NotFormattedError {
    source_path: Option<PathBuf>,
}

impl NotFormattedError {
    #[inline]
    pub fn from_source(source_path: impl Into<PathBuf>) -> Self {
        Self {
            source_path: Some(source_path.into()),
        }
    }

    #[inline]
    pub fn from_input() -> Self {
        Self { source_path: None }
    }

    #[inline]
    pub fn into_error(self) -> Error {
        Error::NotFormatted(self)
    }
}

impl From<Option<&std::path::Path>> for NotFormattedError {
    #[inline]
    fn from(path: Option<&std::path::Path>) -> Self {
        match path {
            Some(path) => Self::from_source(path),
            None => Self::from_input(),
        }
    }
}

impl std::fmt::Display for NotFormattedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.source_path {
            Some(path) => write!(f, "{path:?} is not formatted"),
            None => write!(f, "Input is not formatted"),
        }
    }
}

impl Print<Pretty> for Error {
    fn print(&self, _printer: &mut Pretty, use_ansi_color: bool) {
        self.print(&mut Simple, use_ansi_color);
    }
}

impl Print<Simple> for Error {
    fn print(&self, printer: &mut Simple, use_ansi_color: bool) {
        let message_style = if use_ansi_color {
            Style::new().bold()
        } else {
            Style::new()
        };

        Level::ERROR.print(printer, use_ansi_color);
        println!(": {}", message_style.paint(self.to_string()));
    }
}
