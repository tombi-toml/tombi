/// Verbosity level for logging
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum VerbosityLevel {
    /// Errors only (-qq)
    VeryQuiet,

    /// Warnings and errors (-q)
    Quiet,

    /// No verbosity option (default)
    #[default]
    Default,

    /// Verbose output (-v)
    Verbose,

    /// Very verbose output (-vv)
    VeryVerbose,
}

impl VerbosityLevel {
    /// Convert to log level filter
    pub fn log_level_filter(self) -> log::LevelFilter {
        match self {
            VerbosityLevel::VeryQuiet => log::LevelFilter::Error,
            VerbosityLevel::Quiet => log::LevelFilter::Warn,
            VerbosityLevel::Default => log::LevelFilter::Info,
            VerbosityLevel::Verbose => log::LevelFilter::Debug,
            VerbosityLevel::VeryVerbose => log::LevelFilter::Trace,
        }
    }

    /// Whether the output is turned down by `-q`.
    pub fn is_quiet(self) -> bool {
        self < VerbosityLevel::Default
    }

    /// Whether the output is turned up by `-v`.
    pub fn is_verbose(self) -> bool {
        self > VerbosityLevel::Default
    }
}

/// Verbosity flag that supports -v, -vv, -q and -qq
#[derive(clap::Args, Debug, Clone)]
pub struct Verbosity {
    /// Change the logging level
    ///
    /// -v: DEBUG
    ///
    /// -vv: TRACE
    ///
    /// [default: INFO]
    ///
    #[clap(short = 'v', long, action = clap::ArgAction::Count, global = true, conflicts_with = "quiet", value_parser = clap::value_parser!(u8).range(..=2))]
    verbose: u8,

    /// Lower the logging level and omit the summary
    ///
    /// -q: WARN
    ///
    /// -qq: ERROR
    ///
    /// The exit code of `format` and `lint` is not affected.
    ///
    #[clap(short = 'q', long, action = clap::ArgAction::Count, global = true, value_parser = clap::value_parser!(u8).range(..=2))]
    quiet: u8,
}

impl Verbosity {
    /// Get the verbosity level
    pub fn verbosity_level(&self) -> VerbosityLevel {
        match (self.verbose, self.quiet) {
            (0, 0) => VerbosityLevel::Default,
            (0, 1) => VerbosityLevel::Quiet,
            (0, _) => VerbosityLevel::VeryQuiet,
            (1, _) => VerbosityLevel::Verbose,
            (_, _) => VerbosityLevel::VeryVerbose,
        }
    }

    /// Get the log level filter
    pub fn log_level(&self) -> log::LevelFilter {
        self.verbosity_level().log_level_filter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser as _;

    #[derive(clap::Parser)]
    struct Cli {
        #[command(flatten)]
        verbosity: Verbosity,
    }

    macro_rules! test_verbosity {
        (#[test] fn $name:ident($($arg:expr),*) -> Err) => {
            #[test]
            fn $name() {
                assert!(Cli::try_parse_from(["tombi" $(, $arg)*]).is_err());
            }
        };
        (#[test] fn $name:ident($($arg:expr),*) -> $level:ident) => {
            #[test]
            fn $name() {
                let cli = Cli::try_parse_from(["tombi" $(, $arg)*]).unwrap();
                assert_eq!(cli.verbosity.verbosity_level(), VerbosityLevel::$level);
            }
        };
    }

    test_verbosity!(#[test] fn default() -> Default);
    test_verbosity!(#[test] fn verbose("-v") -> Verbose);
    test_verbosity!(#[test] fn verbose_long("--verbose") -> Verbose);
    test_verbosity!(#[test] fn very_verbose("-vv") -> VeryVerbose);
    test_verbosity!(#[test] fn too_verbose("-vvv") -> Err);
    test_verbosity!(#[test] fn quiet("-q") -> Quiet);
    test_verbosity!(#[test] fn quiet_long("--quiet") -> Quiet);
    test_verbosity!(#[test] fn very_quiet("-qq") -> VeryQuiet);
    test_verbosity!(#[test] fn too_quiet("-qqq") -> Err);
    test_verbosity!(#[test] fn verbose_conflicts_with_quiet("-v", "-q") -> Err);
}
