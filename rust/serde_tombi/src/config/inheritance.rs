use std::{collections::HashMap, path::PathBuf, str::FromStr};

use tombi_document::{Key, KeyKind, Table, Value};
use tombi_schema_store::{DefaultHttpClient, HttpClient};
use tombi_toml_version::TomlVersion;
use tombi_uri::Uri;

const MAX_EXTENDS_DEPTH: usize = 32;
const MAX_EXTENDS_REFERENCES: usize = 256;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum ConfigSource {
    File(PathBuf),
    Remote(Uri),
}

impl ConfigSource {
    fn from_path(path: PathBuf) -> Self {
        let path = if path.is_absolute() {
            path
        } else {
            std::env::current_dir()
                .map(|cwd| cwd.join(&path))
                .unwrap_or(path)
        };

        let path = tombi_fs::canonicalize(&path).unwrap_or_else(|_| tombi_fs::normalize(&path));
        Self::File(path)
    }

    fn display(&self) -> std::string::String {
        match self {
            Self::File(path) => path.display().to_string(),
            Self::Remote(uri) => uri.to_string(),
        }
    }

    fn is_pyproject(&self) -> bool {
        match self {
            Self::File(path) => path
                .file_name()
                .is_some_and(|name| name == "pyproject.toml"),
            Self::Remote(uri) => uri.path().ends_with("/pyproject.toml"),
        }
    }

    fn resolve_reference(&self, reference: &str) -> Result<ConfigSource, tombi_config::Error> {
        let invalid = |reason: &str| tombi_config::Error::ConfigExtendsInvalid {
            config_source: self.display(),
            reference: reference.to_owned(),
            reason: reason.to_owned(),
        };

        match self {
            Self::Remote(base) => {
                if reference.starts_with("file:")
                    || reference.contains("://") && !is_http(reference)
                {
                    return Err(invalid("remote configs may only extend HTTP(S) sources"));
                }

                let mut uri = if is_http(reference) {
                    Uri::from_str(reference).map_err(|error| invalid(&error.to_string()))?
                } else {
                    Uri::from(
                        base.join(reference)
                            .map_err(|error| invalid(&error.to_string()))?,
                    )
                };
                validate_remote_uri(&uri).map_err(|reason| invalid(reason))?;
                uri.set_fragment(None);
                Ok(Self::Remote(uri))
            }
            Self::File(source_path) => {
                if reference.starts_with("file:") {
                    let uri =
                        Uri::from_str(reference).map_err(|error| invalid(&error.to_string()))?;
                    if uri.scheme() != "file" {
                        return Err(invalid("expected a file:// URL"));
                    }
                    let path = uri
                        .to_file_path()
                        .map_err(|_| invalid("file URL does not contain a valid local path"))?;
                    return Ok(Self::from_path(path));
                }

                if is_http(reference) {
                    let mut uri =
                        Uri::from_str(reference).map_err(|error| invalid(&error.to_string()))?;
                    validate_remote_uri(&uri).map_err(|reason| invalid(reason))?;
                    uri.set_fragment(None);
                    return Ok(Self::Remote(uri));
                }

                if reference.contains("://") {
                    return Err(invalid(
                        "only file://, http://, and https:// URLs are supported",
                    ));
                }

                let reference_path = PathBuf::from(reference);
                let path = if reference_path.is_absolute() {
                    reference_path
                } else {
                    source_path
                        .parent()
                        .unwrap_or_else(|| std::path::Path::new("."))
                        .join(reference_path)
                };
                Ok(Self::from_path(path))
            }
        }
    }
}

fn is_http(reference: &str) -> bool {
    reference
        .get(..7)
        .is_some_and(|scheme| scheme.eq_ignore_ascii_case("http://"))
        || reference
            .get(..8)
            .is_some_and(|scheme| scheme.eq_ignore_ascii_case("https://"))
}

fn validate_remote_uri(uri: &Uri) -> Result<(), &'static str> {
    if !matches!(uri.scheme(), "http" | "https") {
        return Err("only http:// and https:// URLs are supported");
    }
    if !uri.username().is_empty() || uri.password().is_some() {
        return Err("URL credentials are not allowed");
    }
    Ok(())
}

#[derive(Debug)]
struct ConfigDocument {
    source: ConfigSource,
    extends: Vec<ConfigSource>,
    values: tombi_document::Document,
}

struct ResolutionFrame {
    document: ConfigDocument,
    next_base: usize,
    inherited: tombi_document::Document,
}

enum ResolutionStep {
    Read(ConfigSource),
    Complete(tombi_document::Document),
}

struct ConfigResolution {
    stack: Vec<ResolutionFrame>,
    active: Vec<ConfigSource>,
    completed: HashMap<ConfigSource, tombi_document::Document>,
    pending: Option<ConfigSource>,
    reference_count: usize,
}

impl ConfigResolution {
    fn new(root: ConfigDocument) -> Self {
        let source = root.source.clone();
        Self {
            stack: vec![ResolutionFrame {
                document: root,
                next_base: 0,
                inherited: tombi_document::Document::new(),
            }],
            active: vec![source],
            completed: HashMap::new(),
            pending: None,
            reference_count: 0,
        }
    }

    fn advance(
        &mut self,
        loaded: Option<ConfigDocument>,
    ) -> Result<ResolutionStep, tombi_config::Error> {
        if let Some(expected_source) = self.pending.take() {
            let Some(loaded) = loaded else {
                unreachable!("a requested config source must be loaded before advancing")
            };
            debug_assert_eq!(loaded.source, expected_source);
            self.active.push(expected_source);
            self.stack.push(ResolutionFrame {
                document: loaded,
                next_base: 0,
                inherited: tombi_document::Document::new(),
            });
        } else {
            debug_assert!(loaded.is_none());
        }

        loop {
            let Some(frame) = self.stack.last_mut() else {
                unreachable!("resolution stack is empty before completion")
            };

            if frame.next_base < frame.document.extends.len() {
                let source = frame.document.extends[frame.next_base].clone();
                frame.next_base += 1;

                if self.reference_count >= MAX_EXTENDS_REFERENCES {
                    return Err(tombi_config::Error::ConfigExtendsCountExceeded {
                        limit: MAX_EXTENDS_REFERENCES,
                    });
                }
                self.reference_count += 1;

                if let Some(index) = self.active.iter().position(|active| active == &source) {
                    let mut chain = self.active[index..]
                        .iter()
                        .map(ConfigSource::display)
                        .collect::<Vec<_>>();
                    chain.push(source.display());
                    return Err(tombi_config::Error::ConfigExtendsCycle {
                        chain: chain.join(" -> "),
                    });
                }

                if self.active.len() - 1 >= MAX_EXTENDS_DEPTH {
                    return Err(tombi_config::Error::ConfigExtendsDepthExceeded {
                        limit: MAX_EXTENDS_DEPTH,
                    });
                }

                if let Some(completed) = self.completed.get(&source) {
                    overlay(&mut frame.inherited, completed.clone(), true);
                    continue;
                }

                self.pending = Some(source.clone());
                return Ok(ResolutionStep::Read(source));
            }

            let frame = self.stack.pop().expect("top frame exists");
            let source = frame.document.source.clone();
            self.active.pop();
            let mut resolved = frame.inherited;
            overlay(&mut resolved, frame.document.values, true);
            self.completed.insert(source, resolved.clone());

            if let Some(parent) = self.stack.last_mut() {
                overlay(&mut parent.inherited, resolved, true);
            } else {
                return Ok(ResolutionStep::Complete(resolved));
            }
        }
    }
}

fn overlay(
    inherited: &mut tombi_document::Document,
    higher_priority: tombi_document::Document,
    root: bool,
) {
    overlay_table(inherited, higher_priority.into(), root);
}

fn overlay_table(inherited: &mut Table, higher_priority: Table, root: bool) {
    for (key, higher_value) in higher_priority.key_values().clone() {
        if let Some(inherited_value) = inherited.key_values_mut().get_mut(&key) {
            match (&mut *inherited_value, higher_value) {
                (Value::Table(inherited_table), Value::Table(higher_table)) => {
                    overlay_table(inherited_table, higher_table, false);
                }
                (Value::Array(inherited_array), Value::Array(mut higher_array))
                    if root && matches!(key.value(), "schemas" | "overrides") =>
                {
                    let mut values = std::mem::take(higher_array.values_mut());
                    values.extend(std::mem::take(inherited_array.values_mut()));
                    *higher_array.values_mut() = values;
                    *inherited_value = Value::Array(higher_array);
                }
                (_, higher_value) => *inherited_value = higher_value,
            }
        } else {
            inherited.insert(key, higher_value);
        }
    }
}

fn key(name: &str) -> Key {
    Key::new(KeyKind::BareKey, name.to_owned())
}

fn source_error(source: &ConfigSource, reason: impl std::fmt::Display) -> tombi_config::Error {
    match source {
        ConfigSource::File(path) => tombi_config::Error::ConfigFileReadFailed {
            config_path: path.clone(),
        },
        ConfigSource::Remote(uri) => tombi_config::Error::ConfigUriFetchFailed {
            config_uri: uri.clone(),
            reason: reason.to_string(),
        },
    }
}

fn layer_error(
    source: &ConfigSource,
    inherited: bool,
    reason: impl std::fmt::Display,
) -> tombi_config::Error {
    if !inherited {
        if let ConfigSource::File(path) = source {
            return tombi_config::Error::ConfigFileParseFailed {
                config_path: path.clone(),
            };
        }
    }
    tombi_config::Error::ConfigExtendsParseFailed {
        config_source: source.display(),
        reason: reason.to_string(),
    }
}

fn parse_document(
    text: &str,
    source: ConfigSource,
    inherited: bool,
) -> Result<Option<ConfigDocument>, tombi_config::Error> {
    let parsed = tombi_parser::parse(text);
    if !parsed.errors.is_empty() {
        return Err(layer_error(&source, inherited, "invalid TOML syntax"));
    }

    let deserializer = crate::Deserializer::new();
    let parsed_document = deserializer
        .try_to_document(
            parsed.root(),
            if source.is_pyproject() {
                TomlVersion::V1_0_0
            } else {
                tombi_config::TOMBI_CONFIG_TOML_VERSION
            },
        )
        .map_err(|error| layer_error(&source, inherited, error))?;

    let mut values = if source.is_pyproject() {
        let Some(config) = extract_pyproject_config(parsed_document)
            .map_err(|reason| layer_error(&source, inherited, reason))?
        else {
            return Ok(None);
        };
        config
    } else {
        parsed_document
    };

    let extends_value = values.key_values_mut().shift_remove(&key("extends"));
    let extends_strings = match extends_value {
        None => Vec::new(),
        Some(Value::String(value)) => vec![value.value().to_owned()],
        Some(Value::Array(values)) => values
            .values()
            .iter()
            .map(|value| match value {
                Value::String(value) => Ok(value.value().to_owned()),
                _ => Err(tombi_config::Error::ConfigExtendsInvalid {
                    config_source: source.display(),
                    reference: "extends".to_owned(),
                    reason: "extends arrays must contain only strings".to_owned(),
                }),
            })
            .collect::<Result<Vec<_>, _>>()?,
        Some(_) => {
            return Err(tombi_config::Error::ConfigExtendsInvalid {
                config_source: source.display(),
                reference: "extends".to_owned(),
                reason: "extends must be a string or an array of strings".to_owned(),
            });
        }
    };

    // Validate every layer on its own so a local override cannot hide invalid inherited keys.
    crate::from_document::<tombi_config::Config>(values.clone())
        .map_err(|error| layer_error(&source, inherited, error))?;

    let extends = extends_strings
        .iter()
        .map(|reference| source.resolve_reference(reference))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Some(ConfigDocument {
        source,
        extends,
        values,
    }))
}

fn extract_pyproject_config(
    mut document: tombi_document::Document,
) -> Result<Option<tombi_document::Document>, std::string::String> {
    let Some(tool) = document.key_values_mut().shift_remove(&key("tool")) else {
        return Ok(None);
    };
    let Value::Table(mut tool) = tool else {
        return Err("the pyproject tool value must be a table".to_owned());
    };
    let Some(config) = tool.key_values_mut().shift_remove(&key("tombi")) else {
        return Ok(None);
    };
    let Value::Table(config) = config else {
        return Err("tool.tombi must be a table".to_owned());
    };
    Ok(Some(config.into()))
}

async fn read_document_async(
    source: ConfigSource,
    inherited: bool,
) -> Result<ConfigDocument, tombi_config::Error> {
    let text = match &source {
        ConfigSource::File(path) => tombi_fs::read_to_string_async(path)
            .await
            .map_err(|error| source_error(&source, error))?,
        ConfigSource::Remote(uri) => {
            let bytes = DefaultHttpClient::new()
                .get_bytes(uri.as_str())
                .await
                .map_err(|error| source_error(&source, error))?;
            std::string::String::from_utf8(bytes.to_vec())
                .map_err(|error| source_error(&source, error))?
        }
    };

    parse_document(&text, source.clone(), inherited)?.ok_or_else(|| {
        tombi_config::Error::ConfigExtendsParseFailed {
            config_source: source.display(),
            reason: "pyproject.toml does not contain [tool.tombi]".to_owned(),
        }
    })
}

#[cfg(target_family = "wasm")]
fn read_document_sync(
    source: ConfigSource,
    inherited: bool,
) -> Result<ConfigDocument, tombi_config::Error> {
    let text = match &source {
        ConfigSource::File(path) => {
            tombi_fs::read_to_string(path).map_err(|error| source_error(&source, error))?
        }
        ConfigSource::Remote(uri) => {
            return Err(tombi_config::Error::ConfigAsyncRequired {
                config_uri: uri.clone(),
            });
        }
    };

    parse_document(&text, source.clone(), inherited)?.ok_or_else(|| {
        tombi_config::Error::ConfigExtendsParseFailed {
            config_source: source.display(),
            reason: "pyproject.toml does not contain [tool.tombi]".to_owned(),
        }
    })
}

async fn resolve_async(root: ConfigDocument) -> Result<tombi_config::Config, tombi_config::Error> {
    let mut resolution = ConfigResolution::new(root);
    let mut step = resolution.advance(None)?;
    loop {
        step = match step {
            ResolutionStep::Read(source) => {
                let document = read_document_async(source, true).await?;
                resolution.advance(Some(document))?
            }
            ResolutionStep::Complete(document) => {
                return crate::from_document(document).map_err(|error| {
                    tombi_config::Error::ConfigExtendsParseFailed {
                        config_source: "merged config".to_owned(),
                        reason: error.to_string(),
                    }
                });
            }
        };
    }
}

#[cfg(target_family = "wasm")]
fn resolve_sync(root: ConfigDocument) -> Result<tombi_config::Config, tombi_config::Error> {
    let mut resolution = ConfigResolution::new(root);
    let mut step = resolution.advance(None)?;
    loop {
        step = match step {
            ResolutionStep::Read(source) => {
                let document = read_document_sync(source, true)?;
                resolution.advance(Some(document))?
            }
            ResolutionStep::Complete(document) => {
                return crate::from_document(document).map_err(|error| {
                    tombi_config::Error::ConfigExtendsParseFailed {
                        config_source: "merged config".to_owned(),
                        reason: error.to_string(),
                    }
                });
            }
        };
    }
}

pub(crate) async fn from_str_async(
    text: &str,
    config_path: &std::path::Path,
) -> Result<tombi_config::Config, tombi_config::Error> {
    let source = ConfigSource::from_path(config_path.to_owned());
    let root = parse_document(text, source.clone(), false)?.ok_or_else(|| {
        tombi_config::Error::ConfigExtendsParseFailed {
            config_source: source.display(),
            reason: "pyproject.toml does not contain [tool.tombi]".to_owned(),
        }
    })?;
    resolve_async(root).await
}

#[cfg(target_family = "wasm")]
pub(crate) fn from_str_sync(
    text: &str,
    config_path: &std::path::Path,
) -> Result<tombi_config::Config, tombi_config::Error> {
    let source = ConfigSource::from_path(config_path.to_owned());
    let root = parse_document(text, source.clone(), false)?.ok_or_else(|| {
        tombi_config::Error::ConfigExtendsParseFailed {
            config_source: source.display(),
            reason: "pyproject.toml does not contain [tool.tombi]".to_owned(),
        }
    })?;
    resolve_sync(root)
}

pub(crate) async fn try_from_path_async(
    config_path: &std::path::Path,
) -> Result<Option<tombi_config::Config>, tombi_config::Error> {
    if !tombi_fs::is_file(config_path) {
        return Err(tombi_config::Error::ConfigFileNotFound {
            config_path: config_path.to_owned(),
        });
    }

    let source = ConfigSource::from_path(config_path.to_owned());
    let text = tombi_fs::read_to_string_async(config_path)
        .await
        .map_err(|error| source_error(&source, error))?;
    let Some(root) = parse_document(&text, source, false)? else {
        return Ok(None);
    };
    resolve_async(root).await.map(Some)
}

#[cfg(target_family = "wasm")]
pub(crate) fn try_from_path_sync(
    config_path: &std::path::Path,
) -> Result<Option<tombi_config::Config>, tombi_config::Error> {
    if !tombi_fs::is_file(config_path) {
        return Err(tombi_config::Error::ConfigFileNotFound {
            config_path: config_path.to_owned(),
        });
    }

    let source = ConfigSource::from_path(config_path.to_owned());
    let text =
        tombi_fs::read_to_string(config_path).map_err(|error| source_error(&source, error))?;
    let Some(root) = parse_document(&text, source, false)? else {
        return Ok(None);
    };
    resolve_sync(root).map(Some)
}
