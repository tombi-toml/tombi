use std::path::PathBuf;

use clap::Parser;
use tombi_diagnostic::{printer::Pretty, Diagnostic, Print};
use tracing_subscriber::prelude::*;

#[derive(clap::Parser)]
pub struct Args {}

pub fn project_root_path() -> PathBuf {
    let dir = std::env::var("CARGO_MANIFEST_DIR")
        .unwrap_or_else(|_| env!("CARGO_MANIFEST_DIR").to_owned());
    PathBuf::from(dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_owned()
}

pub fn source_file() -> PathBuf {
    project_root_path().join("Cargo.toml")
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _args = Args::parse_from(std::env::args_os());

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().pretty().without_time())
        .init();

    let source_file = source_file();

    let warning = Diagnostic::new_warning(
        "Some warning occured.",
        "tombi-diagnostic",
        ((2, 1), (2, 3)),
    );
    let error = Diagnostic::new_error("Some error occured.", "tombi-diagnostic", ((2, 1), (2, 3)));

    warning.print(&mut Pretty, true);
    warning
        .with_source_file(&source_file)
        .print(&mut Pretty, true);
    error.print(&mut Pretty, true);
    error
        .with_source_file(&source_file)
        .print(&mut Pretty, true);

    Ok(())
}
