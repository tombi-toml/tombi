/// Verbosity level for logging
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum VerbosityLevel {
    /// No verbose output (default)
    #[default]
    Default,

    /// Verbose output (-v)
    Verbose,

    /// Very verbose output (-vv)
    VeryVerbose,
}

impl VerbosityLevel {
    /// Convert to tracing log level filter
    pub fn log_level_filter(self) -> tracing_subscriber::filter::LevelFilter {
        match self {
            VerbosityLevel::Default => tracing_subscriber::filter::LevelFilter::INFO,
            VerbosityLevel::Verbose => tracing_subscriber::filter::LevelFilter::DEBUG,
            VerbosityLevel::VeryVerbose => tracing_subscriber::filter::LevelFilter::TRACE,
        }
    }
}

/// Verbosity flag that supports -v and -vv only
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
    #[clap(short = 'v', long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,
}

impl Verbosity {
    /// Get the verbosity level
    pub fn verbosity_level(&self) -> VerbosityLevel {
        match self.verbose {
            0 => VerbosityLevel::Default,
            1 => VerbosityLevel::Verbose,
            2.. => VerbosityLevel::VeryVerbose,
        }
    }

    /// Get the log level filter
    pub fn log_level_filter(&self) -> tracing_subscriber::filter::LevelFilter {
        self.verbosity_level().log_level_filter()
    }
}
