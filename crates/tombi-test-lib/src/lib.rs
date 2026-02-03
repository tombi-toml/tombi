mod date_time;
mod path;
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
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("off"))
        .try_init();
}
