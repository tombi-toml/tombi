use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    str::FromStr,
};

use tombi_document::{Key, KeyKind, Table, Value};
use tombi_schema_store::{CachedHttpClient, FetchError};
use tombi_toml_version::TomlVersion;
use tombi_uri::Uri;

const MAX_EXTENDS_DEPTH: usize = 32;
const MAX_EXTENDS_REFERENCES: usize = 256;
const MAX_MERGED_ENTRIES: usize = 16_384;
const MAX_CACHED_ENTRIES: usize = 65_536;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum ConfigSource {
    File {
        selected_path: PathBuf,
        identity: PathBuf,
    },
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

        let identity = tombi_fs::canonicalize(&path).unwrap_or_else(|_| tombi_fs::normalize(&path));
        Self::File {
            selected_path: path,
            identity,
        }
    }

    fn same_identity(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::File { identity: left, .. },
                Self::File {
                    identity: right, ..
                },
            ) => left == right,
            (Self::Remote(left), Self::Remote(right)) => left == right,
            _ => false,
        }
    }

    fn local_dependencies(&self) -> impl Iterator<Item = PathBuf> {
        let paths = match self {
            Self::File {
                selected_path,
                identity,
            } => vec![tombi_fs::normalize(selected_path), identity.clone()],
            Self::Remote(_) => Vec::new(),
        };
        paths.into_iter()
    }

    fn display(&self) -> std::string::String {
        match self {
            Self::File {
                selected_path: path,
                ..
            } => path.display().to_string(),
            Self::Remote(uri) => uri.to_string(),
        }
    }

    fn is_pyproject(&self) -> bool {
        match self {
            Self::File {
                selected_path: path,
                ..
            } => path
                .file_name()
                .is_some_and(|name| name == "pyproject.toml"),
            Self::Remote(_) => false,
        }
    }

    /// Resolve an `extends` reference.
    ///
    /// Only `tombi.toml` and `.tombi.toml` can be extended.
    fn resolve_reference(&self, reference: &str) -> Result<ConfigSource, tombi_config::Error> {
        let invalid = |reason: String| tombi_config::Error::ConfigExtendsInvalid {
            config_source: self.display(),
            reference: reference.to_owned(),
            reason,
        };

        let target = match self {
            Self::File {
                selected_path: source_path,
                ..
            } => tombi_config::resolve_local_extends(reference, source_path).map_err(invalid)?,
            Self::Remote(base) => {
                if reference.starts_with("file:")
                    || reference.contains("://") && !tombi_config::is_http_url(reference)
                {
                    return Err(invalid(
                        "remote configs may only extend HTTP(S) sources".to_owned(),
                    ));
                }

                let uri = if tombi_config::is_http_url(reference) {
                    Uri::from_str(reference).map_err(|error| invalid(error.to_string()))?
                } else {
                    Uri::from(
                        base.join(reference)
                            .map_err(|error| invalid(error.to_string()))?,
                    )
                };
                let target = tombi_config::ExtendsTarget::Remote(
                    tombi_config::validate_remote_uri(uri).map_err(invalid)?,
                );
                if !target.has_extends_file_name() {
                    return Err(invalid(tombi_config::extends_file_name_error()));
                }
                target
            }
        };

        Ok(match target {
            tombi_config::ExtendsTarget::File(path) => Self::from_path(path),
            tombi_config::ExtendsTarget::Remote(uri) => Self::Remote(uri),
        })
    }
}

#[derive(Debug, Clone)]
struct ConfigDocument {
    source: ConfigSource,
    extends: Vec<ConfigSource>,
    values: tombi_document::Document,
}

#[derive(Debug, Default)]
pub struct ConfigSourceCache {
    documents: HashMap<ConfigSource, ConfigDocument>,
}

struct ResolutionFrame {
    document: ConfigDocument,
    next_base: usize,
    inherited: tombi_document::Document,
    max_base_depth: usize,
    local_dependencies: HashSet<PathBuf>,
}

#[derive(Clone)]
struct ResolvedConfig {
    values: tombi_document::Document,
    depth: usize,
    local_dependencies: HashSet<PathBuf>,
}

enum ResolutionStep {
    Read(ConfigSource),
    Complete(tombi_document::Document, HashSet<PathBuf>),
}

struct ConfigResolution {
    stack: Vec<ResolutionFrame>,
    active: Vec<ConfigSource>,
    completed: HashMap<ConfigSource, ResolvedConfig>,
    pending: Option<ConfigSource>,
    reference_count: usize,
    cached_entries: usize,
}

impl ConfigResolution {
    fn new(root: ConfigDocument) -> Self {
        let source = root.source.clone();
        Self {
            stack: vec![ResolutionFrame {
                local_dependencies: source.local_dependencies().collect(),
                document: root,
                next_base: 0,
                inherited: tombi_document::Document::new(),
                max_base_depth: 0,
            }],
            active: vec![source],
            completed: HashMap::new(),
            pending: None,
            reference_count: 0,
            cached_entries: 0,
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
            self.active.push(expected_source.clone());
            self.stack.push(ResolutionFrame {
                local_dependencies: expected_source.local_dependencies().collect(),
                document: loaded,
                next_base: 0,
                inherited: tombi_document::Document::new(),
                max_base_depth: 0,
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

                if let Some(index) = self
                    .active
                    .iter()
                    .position(|active| active.same_identity(&source))
                {
                    let mut chain = self.active[index..]
                        .iter()
                        .map(ConfigSource::display)
                        .collect::<Vec<_>>();
                    chain.push(source.display());
                    return Err(tombi_config::Error::ConfigExtendsCycle {
                        chain: chain.join(" -> "),
                    });
                }

                if self.active.len() > MAX_EXTENDS_DEPTH {
                    return Err(tombi_config::Error::ConfigExtendsDepthExceeded {
                        limit: MAX_EXTENDS_DEPTH,
                    });
                }

                if let Some(completed) = self.completed.get(&source) {
                    if self.active.len() + completed.depth > MAX_EXTENDS_DEPTH {
                        return Err(tombi_config::Error::ConfigExtendsDepthExceeded {
                            limit: MAX_EXTENDS_DEPTH,
                        });
                    }
                    frame.max_base_depth = frame.max_base_depth.max(completed.depth + 1);
                    preflight_merge(&frame.inherited, &completed.values)?;
                    overlay(&mut frame.inherited, completed.values.clone());
                    frame
                        .local_dependencies
                        .extend(completed.local_dependencies.iter().cloned());
                    continue;
                }

                self.pending = Some(source.clone());
                return Ok(ResolutionStep::Read(source));
            }

            let frame = self.stack.pop().expect("top frame exists");
            let source = frame.document.source.clone();
            self.active.pop();
            let mut resolved = frame.inherited;
            preflight_merge(&resolved, &frame.document.values)?;
            overlay(&mut resolved, frame.document.values);
            let entries = special_entries(&resolved);
            check_limit(self.cached_entries + entries, MAX_CACHED_ENTRIES)?;
            self.cached_entries += entries;
            self.completed.insert(
                source,
                ResolvedConfig {
                    values: resolved.clone(),
                    depth: frame.max_base_depth,
                    local_dependencies: frame.local_dependencies.clone(),
                },
            );

            if let Some(parent) = self.stack.last_mut() {
                parent.max_base_depth = parent.max_base_depth.max(frame.max_base_depth + 1);
                preflight_merge(&parent.inherited, &resolved)?;
                overlay(&mut parent.inherited, resolved);
                parent.local_dependencies.extend(frame.local_dependencies);
            } else {
                return Ok(ResolutionStep::Complete(resolved, frame.local_dependencies));
            }
        }
    }
}

impl ConfigSourceCache {
    pub async fn try_from_path<P: AsRef<std::path::Path>>(
        &mut self,
        config_path: P,
        http_client: &CachedHttpClient,
    ) -> Result<Option<super::ConfigLoadReport>, tombi_config::Error> {
        let config_path = config_path.as_ref();
        super::validate_config_path(config_path)?;
        if !tombi_fs::is_file(config_path) {
            return Err(tombi_config::Error::ConfigFileNotFound {
                config_path: config_path.to_owned(),
            });
        }

        let source = ConfigSource::from_path(config_path.to_owned());
        let root = if let Some(document) = self.documents.get(&source) {
            Some(document.clone())
        } else {
            let text = tombi_fs::read_to_string_async(config_path)
                .await
                .map_err(|error| source_error(&source, error))?;
            let document = parse_document(&text, source.clone(), false)?;
            if let Some(document) = &document {
                self.documents.insert(source, document.clone());
            }
            document
        };

        let Some(root) = root else {
            return Ok(None);
        };

        resolve(root, http_client, self).await.map(Some)
    }

    pub fn invalidate_local_paths(&mut self, paths: &[PathBuf]) -> bool {
        let changed_paths = paths
            .iter()
            .flat_map(|path| {
                let selected = tombi_fs::normalize(path);
                let identity = tombi_fs::canonicalize(path).unwrap_or_else(|_| selected.clone());
                [selected, identity]
            })
            .collect::<HashSet<_>>();
        let before = self.documents.len();
        self.documents.retain(|source, _| {
            !source
                .local_dependencies()
                .any(|path| changed_paths.contains(&path))
        });
        before != self.documents.len()
    }

    pub fn clear(&mut self) {
        self.documents.clear();
    }

    async fn read_document(
        &mut self,
        source: ConfigSource,
        inherited: bool,
        http_client: &CachedHttpClient,
    ) -> Result<ConfigDocument, tombi_config::Error> {
        if let Some(document) = self.documents.get(&source) {
            return Ok(document.clone());
        }

        let document = read_document(source.clone(), inherited, http_client).await?;
        self.documents.insert(source, document.clone());
        Ok(document)
    }
}

fn special_entries(values: &tombi_document::Document) -> usize {
    ["schemas", "overrides"]
        .iter()
        .map(|name| match values.key_values().get(&key(name)) {
            Some(Value::Array(array)) => array.values().len(),
            _ => 0,
        })
        .sum()
}

fn check_limit(entries: usize, limit: usize) -> Result<(), tombi_config::Error> {
    if entries > limit {
        Err(tombi_config::Error::ConfigInheritanceEntriesExceeded { limit })
    } else {
        Ok(())
    }
}

fn preflight_merge(
    inherited: &tombi_document::Document,
    higher: &tombi_document::Document,
) -> Result<(), tombi_config::Error> {
    check_limit(
        special_entries(inherited) + special_entries(higher),
        MAX_MERGED_ENTRIES,
    )
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TableScope {
    Root,
    Ordinary,
    /// The `extensions` table.
    Extensions,
    /// Below `extensions`: a table with `enabled` is a different form than one with features,
    /// so the higher priority one replaces the inherited one instead of mixing both.
    ExtensionFeature,
}

fn overlay(inherited: &mut tombi_document::Document, higher_priority: tombi_document::Document) {
    overlay_table(inherited, higher_priority.into(), TableScope::Root);
}

fn overlay_table(inherited: &mut Table, higher_priority: Table, scope: TableScope) {
    for (key, higher_value) in higher_priority.key_values().clone() {
        if let Some(inherited_value) = inherited.key_values_mut().get_mut(&key) {
            match (&mut *inherited_value, higher_value) {
                (Value::Table(inherited_table), Value::Table(higher_table)) => {
                    let child_scope = match scope {
                        TableScope::Root if key.value() == "extensions" => TableScope::Extensions,
                        TableScope::Extensions | TableScope::ExtensionFeature => {
                            TableScope::ExtensionFeature
                        }
                        _ => TableScope::Ordinary,
                    };
                    if matches!(scope, TableScope::Extensions | TableScope::ExtensionFeature)
                        && (inherited_table
                            .key_values()
                            .contains_key(&self::key("enabled"))
                            || higher_table
                                .key_values()
                                .contains_key(&self::key("enabled")))
                    {
                        *inherited_table = higher_table;
                    } else {
                        overlay_table(inherited_table, higher_table, child_scope);
                    }
                }
                (Value::Array(inherited_array), Value::Array(mut higher_array))
                    if scope == TableScope::Root
                        && matches!(key.value(), "schemas" | "overrides") =>
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
        ConfigSource::File {
            selected_path: path,
            ..
        } => tombi_config::Error::ConfigFileReadFailed {
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
    if !inherited
        && let ConfigSource::File {
            selected_path: path,
            ..
        } = source
    {
        return tombi_config::Error::ConfigFileParseFailed {
            config_path: path.clone(),
        };
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
        Some(Value::Array(values)) => {
            if values.values().len() > MAX_EXTENDS_REFERENCES {
                return Err(tombi_config::Error::ConfigExtendsCountExceeded {
                    limit: MAX_EXTENDS_REFERENCES,
                });
            }
            values
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
                .collect::<Result<Vec<_>, _>>()?
        }
        Some(_) => {
            return Err(tombi_config::Error::ConfigExtendsInvalid {
                config_source: source.display(),
                reference: "extends".to_owned(),
                reason: "extends must be an array of strings".to_owned(),
            });
        }
    };

    check_limit(special_entries(&values), MAX_MERGED_ENTRIES)?;
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

async fn read_document(
    source: ConfigSource,
    inherited: bool,
    http_client: &CachedHttpClient,
) -> Result<ConfigDocument, tombi_config::Error> {
    let text = match &source {
        ConfigSource::File {
            selected_path: path,
            ..
        } => tombi_fs::read_to_string_async(path)
            .await
            .map_err(|error| source_error(&source, error))?,
        ConfigSource::Remote(uri) => {
            let is_valid = |bytes: &[u8]| {
                std::str::from_utf8(bytes)
                    .is_ok_and(|text| parse_document(text, source.clone(), inherited).is_ok())
            };
            let bytes = http_client
                .get_validated(uri.as_str(), &is_valid)
                .await
                .map_err(|error| match error {
                    FetchError::Offline => tombi_config::Error::ConfigOffline {
                        config_uri: uri.clone(),
                    },
                    error => source_error(&source, error),
                })?;
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

async fn resolve(
    root: ConfigDocument,
    http_client: &CachedHttpClient,
    source_cache: &mut ConfigSourceCache,
) -> Result<super::ConfigLoadReport, tombi_config::Error> {
    let mut resolution = ConfigResolution::new(root);
    let mut step = resolution.advance(None)?;
    loop {
        step = match step {
            ResolutionStep::Read(source) => {
                let document = source_cache
                    .read_document(source, true, http_client)
                    .await?;
                resolution.advance(Some(document))?
            }
            ResolutionStep::Complete(document, local_dependencies) => {
                let config = crate::from_document(document).map_err(|error| {
                    tombi_config::Error::ConfigExtendsParseFailed {
                        config_source: "merged config".to_owned(),
                        reason: error.to_string(),
                    }
                })?;
                let mut local_dependencies = local_dependencies.into_iter().collect::<Vec<_>>();
                local_dependencies.sort();
                return Ok(super::ConfigLoadReport {
                    config,
                    local_dependencies,
                });
            }
        };
    }
}

pub(crate) async fn from_str(
    text: &str,
    config_path: &std::path::Path,
    http_client: &CachedHttpClient,
) -> Result<tombi_config::Config, tombi_config::Error> {
    let source = ConfigSource::from_path(config_path.to_owned());
    let root = parse_document(text, source.clone(), false)?.ok_or_else(|| {
        tombi_config::Error::ConfigExtendsParseFailed {
            config_source: source.display(),
            reason: "pyproject.toml does not contain [tool.tombi]".to_owned(),
        }
    })?;
    let mut source_cache = ConfigSourceCache::default();
    resolve(root, http_client, &mut source_cache)
        .await
        .map(|report| report.config)
}
