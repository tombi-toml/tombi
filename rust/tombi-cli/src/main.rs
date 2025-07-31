mod app;
mod error;

pub use error::Error;

fn main() {
    if let Err(err) = app::run(std::env::args_os()) {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}
