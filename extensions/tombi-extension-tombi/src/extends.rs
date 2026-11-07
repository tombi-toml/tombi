use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

/// Where an `extends` reference points to.
pub(crate) enum ExtendsTarget {
    File(PathBuf),
    Remote(tombi_uri::Uri),
}

/// Resolve an `extends` reference the same way the config loader does.
///
/// Local paths are relative to the directory of the config that declares them.
pub(crate) fn resolve(value: &str, source_path: &Path) -> Result<ExtendsTarget, String> {
    if value.starts_with("file:") {
        let uri = tombi_uri::Uri::from_str(value).map_err(|error| error.to_string())?;
        return uri
            .to_file_path()
            .map(ExtendsTarget::File)
            .map_err(|_| "file URL does not contain a valid local path".to_owned());
    }

    if is_http(value) {
        let uri = tombi_uri::Uri::from_str(value).map_err(|error| error.to_string())?;
        if !uri.username().is_empty() || uri.password().is_some() {
            return Err("URL credentials are not allowed".to_owned());
        }
        return Ok(ExtendsTarget::Remote(uri));
    }

    if value.contains("://") {
        return Err("only file://, http://, and https:// URLs are supported".to_owned());
    }

    let path = Path::new(value);
    if path.is_absolute() {
        return Ok(ExtendsTarget::File(path.to_owned()));
    }
    source_path
        .parent()
        .map(|parent| ExtendsTarget::File(parent.join(path)))
        .ok_or_else(|| "cannot resolve a relative path from this config".to_owned())
}

fn is_http(value: &str) -> bool {
    ["http://", "https://"].iter().any(|scheme| {
        value
            .get(..scheme.len())
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case(scheme))
    })
}
