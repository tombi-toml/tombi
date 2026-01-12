use std::str::FromStr;

use tombi_config::TomlVersion;

use crate::{
    Backend,
    handler::workspace_diagnostic::{WorkspaceDiagnosticOptions, push_workspace_diagnostics},
};

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssociateSchemaParams {
    title: Option<String>,
    description: Option<String>,
    uri: String,
    file_match: Vec<String>,
    toml_version: Option<TomlVersion>,
    /// If true, the schema will be inserted at the beginning of the schema list
    /// to force it to take precedence over catalog schemas. Default is false.
    #[serde(default)]
    force: bool,
}

/// Handle the `tombi/associateSchema` request to associate a schema with a file match pattern.
///
/// This function is used to associate a schema URL with a file match pattern in the TOML Language Server.
///
/// In VSCode Extension, contributors can use this to associate a schema with specific files
/// by providing a regex match pattern.
///
/// ```json
/// {
///   "contributes": {
///     "tomlValidation": [
///       {
///         "regexMatch": "^.*foo.toml$",
///         "url": "https://www.schemastore.org/foo.json"
///       }
///     ]
///   }
/// }
/// ```
#[tracing::instrument(level = "debug", skip_all)]
pub async fn handle_associate_schema(backend: &Backend, params: AssociateSchemaParams) {
    tracing::info!("handle_associate_schema");
    tracing::trace!(?params);

    let Ok(schema_uri) = tombi_schema_store::SchemaUri::from_str(&params.uri) else {
        tracing::warn!("Invalid schema URL: {}", params.uri);
        return;
    };

    backend
        .config_manager
        .associate_schema(
            &schema_uri,
            &params.file_match,
            &tombi_schema_store::AssociateSchemaOptions {
                title: params.title,
                description: params.description,
                toml_version: params.toml_version,
                force: params.force,
            },
        )
        .await;

    // Refresh workspace diagnostics after schema association
    if let Err(err) = push_workspace_diagnostics(
        backend,
        &WorkspaceDiagnosticOptions {
            include_open_files: true,
        },
    )
    .await
    {
        tracing::warn!("Failed to push workspace diagnostics: {err}");
    }
}
