use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

const STDOUT_FD: i32 = 1;
const STDERR_FD: i32 = 2;

/// Where diagnostics are written.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum Destination {
    /// `--diagnostics-file` is not specified.
    DefaultStderr,

    /// A regular path, truncated on open.
    File(PathBuf),

    /// An inherited file descriptor such as `/dev/fd/3`.
    ///
    /// The descriptor is duplicated instead of opening the path,
    /// because `open("/proc/self/fd/N")` fails with `ENXIO` on Linux when it is a socket
    /// (e.g. Node.js `child_process.spawn` with `stdio: "pipe"`).
    Fd(i32),
}

impl Destination {
    pub(super) fn new(path: Option<&Path>) -> Self {
        match path {
            None => Self::DefaultStderr,
            Some(path) => match fd_path(path) {
                Some(fd) => Self::Fd(fd),
                None => Self::File(path.to_owned()),
            },
        }
    }

    #[inline]
    pub(super) fn is_default_stderr(&self) -> bool {
        *self == Self::DefaultStderr
    }

    #[inline]
    pub(super) fn is_stderr(&self) -> bool {
        matches!(self, Self::DefaultStderr | Self::Fd(STDERR_FD))
    }

    #[inline]
    pub(super) fn is_stdout(&self) -> bool {
        *self == Self::Fd(STDOUT_FD)
    }

    pub(super) fn open(&self) -> std::io::Result<Box<dyn Write + Send>> {
        match self {
            Self::DefaultStderr => Ok(Box::new(std::io::stderr())),
            Self::File(path) => Ok(Box::new(BufWriter::new(
                std::fs::File::create(path).map_err(|error| with_path(error, path))?,
            ))),
            Self::Fd(fd) => open_fd(*fd),
        }
    }
}

#[cfg(unix)]
fn fd_path(path: &Path) -> Option<i32> {
    let path = path.to_str()?;
    match path {
        "/dev/stdout" => Some(STDOUT_FD),
        "/dev/stderr" => Some(STDERR_FD),
        _ => path
            .strip_prefix("/dev/fd/")
            .or_else(|| path.strip_prefix("/proc/self/fd/"))
            .and_then(|fd| fd.parse::<u16>().ok())
            .map(i32::from),
    }
}

#[cfg(not(unix))]
fn fd_path(_path: &Path) -> Option<i32> {
    None
}

#[cfg(unix)]
fn open_fd(fd: i32) -> std::io::Result<Box<dyn Write + Send>> {
    use std::os::fd::BorrowedFd;

    let path = PathBuf::from(format!("/dev/fd/{fd}"));

    // `stat` succeeds even for sockets, so this only checks that the descriptor is open.
    std::fs::metadata(&path).map_err(|error| with_path(error, &path))?;

    // SAFETY: The descriptor was checked to be open above, and it is only borrowed
    // until it is duplicated. The duplicate is owned and closed independently,
    // so the inherited descriptor (including stdout and stderr) is never closed.
    let borrowed = unsafe { BorrowedFd::borrow_raw(fd) };
    let owned = borrowed
        .try_clone_to_owned()
        .map_err(|error| with_path(error, &path))?;

    Ok(Box::new(BufWriter::new(std::fs::File::from(owned))))
}

#[cfg(not(unix))]
fn open_fd(_fd: i32) -> std::io::Result<Box<dyn Write + Send>> {
    unreachable!("file descriptor paths are only recognized on Unix")
}

fn with_path(error: std::io::Error, path: &Path) -> std::io::Error {
    std::io::Error::new(
        error.kind(),
        format!("failed to open {:?} for diagnostics: {error}", path),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_destination {
        ($name:ident: $path:expr => $expected:expr) => {
            #[test]
            fn $name() {
                pretty_assertions::assert_eq!(Destination::new($path.map(Path::new)), $expected);
            }
        };
    }

    test_destination!(default_is_stderr: None::<&str> => Destination::DefaultStderr);
    test_destination!(regular_file: Some("report.json") => Destination::File(PathBuf::from("report.json")));
    test_destination!(dash_is_a_regular_file: Some("-") => Destination::File(PathBuf::from("-")));

    #[cfg(unix)]
    test_destination!(dev_stdout: Some("/dev/stdout") => Destination::Fd(1));
    #[cfg(unix)]
    test_destination!(dev_stderr: Some("/dev/stderr") => Destination::Fd(2));
    #[cfg(unix)]
    test_destination!(dev_fd: Some("/dev/fd/3") => Destination::Fd(3));
    #[cfg(unix)]
    test_destination!(proc_self_fd: Some("/proc/self/fd/4") => Destination::Fd(4));
    #[cfg(unix)]
    test_destination!(invalid_fd: Some("/dev/fd/x") => Destination::File(PathBuf::from("/dev/fd/x")));
}
