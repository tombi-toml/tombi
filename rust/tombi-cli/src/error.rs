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
    #[error("{0:?} file not found")]
    FileNotFound(PathBuf),
    #[error("{0:?} is invalid glob pattern")]
    GlobPatternInvalid(String),
    #[error(transparent)]
    GlobSearchFailed(tombi_glob::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
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
            Some(path) => write!(f, "{:?} is not formatted", path),
            None => write!(f, "Input is not formatted"),
        }
    }
}

impl Print<Pretty> for Error {
    fn print(&self, _printer: &mut Pretty) {
        self.print(&mut Simple);
    }
}

impl Print<Simple> for Error {
    fn print(&self, printer: &mut Simple) {
        Level::ERROR.print(printer);
        println!(": {}", Style::new().bold().paint(self.to_string()));
    }
}
