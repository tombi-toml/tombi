mod cache;
mod date_time;
mod path;
pub use cache::*;
pub use date_time::*;
pub use path::*;

#[macro_export]
macro_rules! toml_text_assert_eq {
    ($actual:expr, $expected:expr) => {
        let expected = format!("{}\n", textwrap::dedent($expected).trim());
        pretty_assertions::assert_eq!($actual, expected);
    };
}

pub fn init_log() {
    if let Ok(level) = std::env::var("RUST_LOG") {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(level)
            .pretty()
            .try_init();
    }
}
