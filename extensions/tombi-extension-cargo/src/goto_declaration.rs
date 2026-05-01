use crate::{
    feature_key_at_accessors, goto_declaration_for_crate_cargo_toml,
    optional_dependency_value_at_accessors,
};
use tombi_config::TomlVersion;
use tombi_document_tree::dig_keys;
use tombi_schema_store::{Accessor, matches_accessors};

pub async fn goto_declaration(
    text_document_uri: &tombi_uri::Uri,
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[tombi_schema_store::Accessor],
    toml_version: TomlVersion,
    features: Option<&tombi_config::CargoExtensionFeatures>,
) -> Result<Option<Vec<tombi_extension::Location>>, tower_lsp::jsonrpc::Error> {
    // Check if current file is Cargo.toml
    if !text_document_uri.path().ends_with("Cargo.toml") {
        return Ok(Default::default());
    }
    let Some(cargo_toml_path) = text_document_uri.to_file_path().ok() else {
        return Ok(Default::default());
    };

    if !cargo_navigation_enabled(features, accessors) {
        return Ok(None);
    }

    let locations = goto_declaration_for_crate_cargo_toml(
        document_tree,
        accessors,
        &cargo_toml_path,
        toml_version,
        false,
    )?;

    if locations.is_empty() {
        return Ok(Default::default());
    }

    Ok(Some(locations))
}

fn cargo_navigation_enabled(
    features: Option<&tombi_config::CargoExtensionFeatures>,
    accessors: &[tombi_schema_store::Accessor],
) -> bool {
    features
        .and_then(|features| features.lsp())
        .and_then(|lsp| lsp.goto_declaration())
        .and_then(|goto_declaration| {
            if matches!(
                accessors.last(),
                Some(tombi_schema_store::Accessor::Key(key)) if key == "path"
            ) {
                goto_declaration.path()
            } else {
                goto_declaration.dependency()
            }
        })
        .map(|feature| feature.enabled())
        .unwrap_or_default()
        .value()
}

pub fn get_current_declaration(
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[Accessor],
    cargo_toml_uri: &tombi_uri::Uri,
) -> Option<tombi_extension::Location> {
    if !cargo_toml_uri.path().ends_with("Cargo.toml") {
        return None;
    }

    if matches_accessors!(accessors, ["features", _]) {
        let feature_key = feature_key_at_accessors(document_tree, accessors)?;
        return Some(tombi_extension::Location {
            uri: cargo_toml_uri.clone(),
            range: feature_key.unquoted_range(),
        });
    }

    let (dependency_key, _) = if matches_accessors!(accessors, ["dependencies", _, "optional"])
        || matches_accessors!(accessors, ["dev-dependencies", _, "optional"])
        || matches_accessors!(accessors, ["build-dependencies", _, "optional"])
    {
        if !optional_dependency_value_at_accessors(document_tree, accessors).unwrap_or_default() {
            return None;
        }
        dig_keys(
            document_tree,
            &[accessors.first()?.as_key()?, accessors.get(1)?.as_key()?],
        )?
    } else if matches_accessors!(accessors, ["target", _, "dependencies", _, "optional"])
        || matches_accessors!(accessors, ["target", _, "dev-dependencies", _, "optional"])
        || matches_accessors!(
            accessors,
            ["target", _, "build-dependencies", _, "optional"]
        )
    {
        if !optional_dependency_value_at_accessors(document_tree, accessors).unwrap_or_default() {
            return None;
        }
        dig_keys(
            document_tree,
            &[
                "target",
                accessors.get(1)?.as_key()?,
                accessors.get(2)?.as_key()?,
                accessors.get(3)?.as_key()?,
            ],
        )?
    } else {
        return None;
    };

    Some(tombi_extension::Location {
        uri: cargo_toml_uri.clone(),
        range: dependency_key.unquoted_range(),
    })
}
