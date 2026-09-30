use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

const STDOUT_FD: i32 = 1;
const STDERR_FD: i32 = 2;

/// Where diagnostics are written.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum Destination {
    /// `--diagnostics-file` is not specified.
    DefaultStderr,

    /// A regular path, truncated right before the first write.
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

    /// The path if the destination is a regular file.
    #[inline]
    pub(super) fn file_path(&self) -> Option<&Path> {
        match self {
            Self::File(path) => Some(path),
            _ => None,
        }
    }

    pub(super) fn open(&self) -> std::io::Result<Box<dyn Write + Send>> {
        match self {
            Self::DefaultStderr => Ok(Box::new(std::io::stderr())),
            Self::File(path) => {
                // Opened here to report errors such as permissions early,
                // but not truncated because the path may be a file that is read by the command.
                let file = std::fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(false)
                    .open(path)
                    .map_err(|error| with_path(error, path))?;
                Ok(Box::new(BufWriter::new(TruncateOnWrite {
                    file,
                    pending: true,
                })))
            }
            Self::Fd(fd) => open_fd(*fd),
        }
    }
}

/// Truncates the file right before the first write or flush.
struct TruncateOnWrite {
    file: std::fs::File,
    pending: bool,
}

impl TruncateOnWrite {
    fn truncate(&mut self) -> std::io::Result<()> {
        if self.pending {
            // Devices and FIFOs such as `/dev/null` cannot be truncated (`EINVAL` on Linux).
            if self.file.metadata()?.is_file() {
                self.file.set_len(0)?;
            }
            self.pending = false;
        }
        Ok(())
    }
}

impl Write for TruncateOnWrite {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.truncate()?;
        self.file.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        // Flushed at the end even if nothing was written, so that an old report does not remain.
        self.truncate()?;
        self.file.flush()
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
            // Digits only, because `parse` also accepts a sign.
            .filter(|fd| fd.bytes().all(|byte| byte.is_ascii_digit()))
            .and_then(|fd| fd.parse::<i32>().ok()),
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
    #[cfg(unix)]
    test_destination!(large_fd: Some("/dev/fd/65536") => Destination::Fd(65536));
    #[cfg(unix)]
    test_destination!(negative_fd: Some("/dev/fd/-1") => Destination::File(PathBuf::from("/dev/fd/-1")));
    #[cfg(unix)]
    test_destination!(signed_fd: Some("/dev/fd/+3") => Destination::File(PathBuf::from("/dev/fd/+3")));
    #[cfg(unix)]
    test_destination!(overflowing_fd: Some("/dev/fd/99999999999") => Destination::File(PathBuf::from("/dev/fd/99999999999")));

    /// The file keeps its content until the first write or flush, and is truncated at the end.
    macro_rules! test_truncate_on_write {
        ($name:ident: $existing:expr, $written:expr => $expected:expr) => {
            #[test]
            fn $name() {
                let dir = tempfile::tempdir().unwrap();
                let path = dir.path().join("report");
                if let Some(existing) = $existing {
                    std::fs::write(&path, existing).unwrap();
                }

                let mut writer = Destination::File(path.clone()).open().unwrap();
                pretty_assertions::assert_eq!(
                    std::fs::read_to_string(&path).unwrap(),
                    $existing.unwrap_or_default()
                );

                if let Some(written) = $written {
                    writer.write_all(written).unwrap();
                }
                writer.flush().unwrap();
                pretty_assertions::assert_eq!(std::fs::read_to_string(&path).unwrap(), $expected);
            }
        };
    }

    #[cfg(unix)]
    #[test]
    fn device_files_are_written_without_truncation() {
        let mut writer = Destination::File(PathBuf::from("/dev/null"))
            .open()
            .unwrap();
        writer.write_all(b"report").unwrap();
        writer.flush().unwrap();
    }

    test_truncate_on_write!(keeps_content_until_write_and_replaces_it: Some("old content"), Some(b"new") => "new");
    test_truncate_on_write!(truncates_even_if_nothing_is_written: Some("old content"), None::<&[u8]> => "");
    test_truncate_on_write!(creates_missing_file: None::<&str>, Some(b"new") => "new");
}
