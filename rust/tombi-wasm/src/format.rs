use crate::pretty_buf::PrettyBuf;
use itertools::Either::Right;
use std::path::PathBuf;
use tombi_config::{Config, TomlVersion};
use tombi_diagnostic::Print;
use tombi_formatter::formatter::definitions::FormatDefinitions;
use tombi_formatter::Formatter;

pub async fn format(target_path: PathBuf, target_content: String) -> Result<String, String> {
    let toml_version = TomlVersion::V1_0_0;
    let schema_store =
        tombi_schema_store::SchemaStore::new_with_options(tombi_schema_store::Options {
            offline: Some(false),
            strict: None,
        });

    schema_store
        .load_config(&Config::default(), None)
        .await
        .map_err(|e| e.to_string())?;

    let options = Default::default();
    let definitions = FormatDefinitions::default();
    let formatter = Formatter::new(
        toml_version,
        &definitions,
        &options,
        Some(Right(&target_path)),
        &schema_store,
    );

    let mut printer = PrettyBuf::new();

    match formatter.format(&target_content).await {
        Ok(formatted) => Ok(formatted),
        Err(diagnostics) => {
            diagnostics
                .into_iter()
                .map(|diagnostic| diagnostic.with_source_file(target_path.clone()))
                .collect::<Vec<_>>()
                .print(&mut printer);

            Err(printer.get())
        }
    }
}
