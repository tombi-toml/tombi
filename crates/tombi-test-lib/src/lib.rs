mod cache;
mod date_time;
mod path;
#[cfg(feature = "pixi-channel")]
mod pixi_channel;
pub use cache::*;
pub use date_time::*;
pub use path::*;
#[cfg(feature = "pixi-channel")]
pub use pixi_channel::*;

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
