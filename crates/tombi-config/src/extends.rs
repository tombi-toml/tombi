use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use tombi_uri::Uri;

use crate::{DOT_TOMBI_TOML_FILENAME, TOMBI_TOML_FILENAME, config_base_dir};

/// Where an `extends` reference points to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtendsTarget {
    File(PathBuf),
    Remote(Uri),
}

/// Only `tombi.toml` and `.tombi.toml` can be extended,
/// so that the schema for Tombi config files applies to every inherited config.
pub fn is_extends_file_name(file_name: &str) -> bool {
    matches!(file_name, TOMBI_TOML_FILENAME | DOT_TOMBI_TOML_FILENAME)
}

pub fn extends_file_name_error() -> String {
    format!("extended configs must be named {TOMBI_TOML_FILENAME} or {DOT_TOMBI_TOML_FILENAME}")
}

impl ExtendsTarget {
    /// Whether the file name is `tombi.toml` or `.tombi.toml`.
    ///
    /// The query and fragment of a URL are not part of the file name.
    pub fn has_extends_file_name(&self) -> bool {
        let file_name = match self {
            Self::File(path) => path.file_name().and_then(|name| name.to_str()),
            Self::Remote(uri) => uri.path().rsplit('/').next(),
        };
        file_name.is_some_and(is_extends_file_name)
    }
}

/// Resolve an `extends` reference declared by the local config `source_path`.
///
/// Local paths are relative to the base directory of the config that declares them,
/// the same one that `schemas` and `schema.catalog` paths use (see [`config_base_dir`]).
/// The loader and the editor features share this, so they agree on what is valid.
pub fn resolve_local_extends(reference: &str, source_path: &Path) -> Result<ExtendsTarget, String> {
    let target = if reference.starts_with("file:") {
        let uri = Uri::from_str(reference).map_err(|error| error.to_string())?;
        if uri.scheme() != "file" {
            return Err("expected a file:// URL".to_owned());
        }
        if uri.query().is_some() || uri.fragment().is_some() {
            return Err("file URLs must not have a query or fragment".to_owned());
        }
        ExtendsTarget::File(
            uri.to_file_path()
                .map_err(|_| "file URL does not contain a valid local path".to_owned())?,
        )
    } else if is_http_url(reference) {
        ExtendsTarget::Remote(parse_remote_uri(reference)?)
    } else if reference.contains("://") {
        return Err("only file://, http://, and https:// URLs are supported".to_owned());
    } else {
        let path = PathBuf::from(reference);
        ExtendsTarget::File(if path.is_absolute() {
            path
        } else {
            config_base_dir(source_path)
                .unwrap_or_else(|| Path::new("."))
                .join(path)
        })
    };

    if !target.has_extends_file_name() {
        return Err(extends_file_name_error());
    }
    Ok(target)
}

pub fn is_http_url(reference: &str) -> bool {
    ["http://", "https://"].iter().any(|scheme| {
        reference
            .get(..scheme.len())
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case(scheme))
    })
}

/// Parse a remote config URL, without its fragment.
pub fn parse_remote_uri(reference: &str) -> Result<Uri, String> {
    let uri = Uri::from_str(reference).map_err(|error| error.to_string())?;
    validate_remote_uri(uri)
}

/// Check that `uri` is an HTTP(S) URL without credentials, and drop its fragment.
pub fn validate_remote_uri(mut uri: Uri) -> Result<Uri, String> {
    if !matches!(uri.scheme(), "http" | "https") {
        return Err("only http:// and https:// URLs are supported".to_owned());
    }
    if !uri.username().is_empty() || uri.password().is_some() {
        return Err("URL credentials are not allowed".to_owned());
    }
    uri.set_fragment(None);
    Ok(uri)
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{ExtendsTarget, resolve_local_extends};

    fn resolved(source_path: &str) -> Result<ExtendsTarget, String> {
        resolve_local_extends("base/tombi.toml", Path::new(source_path))
    }

    #[test]
    fn extends_is_relative_to_the_config_directory() {
        assert_eq!(
            resolved("/tmp/project/tombi.toml"),
            Ok(ExtendsTarget::File(PathBuf::from(
                "/tmp/project/base/tombi.toml"
            )))
        );
    }

    #[test]
    fn extends_of_dot_config_tombi_toml_is_relative_to_the_project_root() {
        assert_eq!(
            resolved("/tmp/project/.config/tombi.toml"),
            Ok(ExtendsTarget::File(PathBuf::from(
                "/tmp/project/base/tombi.toml"
            )))
        );
    }
}
