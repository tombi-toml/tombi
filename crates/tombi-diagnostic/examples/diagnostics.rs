use std::path::PathBuf;

use clap::Parser;
use tombi_diagnostic::{Diagnostic, Print, printer::Pretty};
use tombi_text::LineIndex;

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

    env_logger::Builder::from_default_env()
        .format_timestamp(None)
        .init();

    let source_file = source_file();
    let source_text = std::fs::read_to_string(&source_file)?;
    let line_index = LineIndex::new(&source_text);
    let span = line_index.span(
        ((2, 1), (2, 3)).into(),
        tombi_text::EncodingKind::GraphemeCluster,
    );

    let warning = Diagnostic::new_warning("some warning occured.", "tombi-diagnostic", span);
    let error = Diagnostic::new_error("some error occured.", "tombi-diagnostic", span);

    let printer = Pretty {
        use_ansi_color: true,
    };
    let mut stderr = std::io::stderr();

    warning
        .located(&line_index, tombi_text::EncodingKind::GraphemeCluster)
        .print(&printer, &mut stderr)?;
    warning
        .with_source_file(&source_file)
        .located(&line_index, tombi_text::EncodingKind::GraphemeCluster)
        .print(&printer, &mut stderr)?;
    error
        .located(&line_index, tombi_text::EncodingKind::GraphemeCluster)
        .print(&printer, &mut stderr)?;
    error
        .with_source_file(&source_file)
        .located(&line_index, tombi_text::EncodingKind::GraphemeCluster)
        .print(&printer, &mut stderr)?;

    Ok(())
}
