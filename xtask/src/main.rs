mod app;
mod codegen;
mod command;
mod glue;
mod utils;

fn main() {
    if let Err(err) = app::run(std::env::args_os()) {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}
