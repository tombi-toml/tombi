use crate::app::arg;
use config::TomlVersion;
use diagnostic::{printer::Pretty, Diagnostic, Print};
use std::io::{Read, Seek, Write};

/// Format TOML files.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// Paths or glob patterns to TOML documents.
    ///
    /// If the only argument is "-", the standard input will be used.
    files: Vec<String>,

    /// Check if the input is formatted.
    #[arg(long, default_value_t = false)]
    check: bool,

    /// TOML version.
    #[arg(long, value_enum, default_value_t = TomlVersion::default())]
    toml_version: TomlVersion,
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn run(args: Args) -> Result<(), crate::Error> {
    let (success_num, error_num) = inner_run(args, Pretty);

    match success_num {
        0 => eprintln!("No files formatted"),
        1 => eprintln!("1 file formatted"),
        _ => eprintln!("{} files formatted", success_num),
    }

    if error_num > 0 {
        std::process::exit(1);
    }

    Ok(())
}

fn inner_run<P>(args: Args, printer: P) -> (usize, usize)
where
    Diagnostic: Print<P>,
    crate::Error: Print<P>,
    P: Copy,
{
    let input = arg::FileInput::from(args.files.as_ref());

    let total_num = input.len();
    let mut success_num = 0;
    let mut error_num = 0;

    match input {
        arg::FileInput::Stdin => {
            tracing::debug!("stdin input formatting...");
            if format_file(FormatFile::from_stdin(), printer, &args) {
                success_num += 1;
            } else {
                error_num += 1;
            }
        }
        arg::FileInput::Files(files) => {
            for file in files {
                match file {
                    Ok(path) => {
                        tracing::debug!("{:?} formatting...", path);
                        match FormatFile::from_file(&path) {
                            Ok(file) => {
                                if format_file(file, printer, &args) {
                                    success_num += 1;
                                    continue;
                                }
                            }
                            Err(err) => {
                                if err.kind() == std::io::ErrorKind::NotFound {
                                    crate::Error::FileNotFound(path).print(printer);
                                } else {
                                    crate::Error::Io(err).print(printer);
                                }
                            }
                        }
                    }
                    Err(err) => err.print(printer),
                }
                error_num += 1;
            }
        }
    };

    assert_eq!(success_num + error_num, total_num);

    (success_num, error_num)
}

fn format_file<P>(mut file: FormatFile, printer: P, args: &Args) -> bool
where
    Diagnostic: Print<P>,
    crate::Error: Print<P>,
    P: Copy,
{
    let mut source = String::new();
    if file.read_to_string(&mut source).is_ok() {
        match formatter::format_with(&source, args.toml_version, &Default::default()) {
            Ok(formatted) => {
                if source != formatted {
                    if args.check {
                        crate::error::NotFormattedError::from(file.source())
                            .into_error()
                            .print(printer);
                    }
                    if let Err(err) = file.reset() {
                        crate::Error::Io(err).print(printer);
                    };
                    match file.write_all(formatted.as_bytes()) {
                        Ok(_) => return true,
                        Err(err) => {
                            crate::Error::Io(err).print(printer);
                        }
                    }
                } else {
                    return true;
                }
            }
            Err(diagnostics) => diagnostics.print(printer),
        }
    }
    false
}

enum FormatFile {
    Stdin(std::io::Stdin),
    File {
        path: std::path::PathBuf,
        file: std::fs::File,
    },
}

impl FormatFile {
    fn from_stdin() -> Self {
        Self::Stdin(std::io::stdin())
    }

    fn from_file(path: &std::path::Path) -> std::io::Result<Self> {
        Ok(Self::File {
            path: path.to_owned(),
            file: std::fs::OpenOptions::new()
                .read(true)
                .append(true)
                .open(path)?,
        })
    }

    #[inline]
    fn source(&self) -> Option<&std::path::Path> {
        match self {
            Self::Stdin(_) => None,
            Self::File { path, .. } => Some(path.as_ref()),
        }
    }

    fn reset(&mut self) -> std::io::Result<()> {
        match self {
            Self::Stdin(_) => Ok(()),
            Self::File { file, .. } => {
                file.seek(std::io::SeekFrom::Start(0))?;
                file.set_len(0)
            }
        }
    }
}

impl std::io::Read for FormatFile {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Self::Stdin(stdin) => stdin.read(buf),
            Self::File { file, .. } => file.read(buf),
        }
    }
}

impl std::io::Write for FormatFile {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Self::Stdin(_) => std::io::stdout().write(buf),
            Self::File { file, .. } => file.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Self::Stdin(_) => std::io::stdout().flush(),
            Self::File { file, .. } => file.flush(),
        }
    }
}
