use std::fs;
use zed::LanguageServerId;
use zed_extension_api::{self as zed, Result, settings::LspSettings};

const VERSION_DIR_PREFIX: &str = "tombi-";

struct TombiExtension {
    cached_binary_path: Option<String>,
}

impl TombiExtension {
    fn make_language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let lsp_settings = LspSettings::for_worktree(language_server_id.as_ref(), worktree).ok();
        let binary_settings = lsp_settings
            .as_ref()
            .and_then(|lsp_settings| lsp_settings.binary.as_ref());

        let args = binary_settings
            .and_then(|binary_settings| binary_settings.arguments.as_ref())
            .cloned()
            .unwrap_or_else(|| vec!["lsp".to_string()]);

        let env = binary_settings
            .and_then(|binary_settings| binary_settings.env.as_ref())
            .map(|env| {
                env.iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        // Resolve the language server in descending priority:
        // 1. explicit user configuration,
        // 2. project-local installs in a Python virtual environment,
        // 3. project-local installs in node_modules/.bin,
        // 4. project-local installs found via PATH,
        // 5. the process-local cached path from a previous extension-managed resolution,
        // 6. download the latest tombi binary from GitHub,
        // 7. fallback to the newest extension-managed install already on disk.
        //
        // On the first extension-managed resolution in a Zed process, step 5 is skipped
        // because no process-local cache exists yet. In that case, the extension attempts
        // step 6 first and only falls back to step 7 if the download fails.
        if let Some(path) = binary_settings.and_then(|binary_settings| binary_settings.path.clone())
        {
            return Ok(zed::Command {
                command: path,
                args,
                env,
            });
        }

        let binary_name = match zed::current_platform() {
            (zed::Os::Windows, _) => "tombi.exe",
            _ => "tombi",
        };

        if let Some(path) = Self::resolve_project_local_install(worktree, binary_name) {
            return Ok(zed::Command {
                command: path,
                args,
                env,
            });
        }

        let binary_path = if let Some(path) = self.resolve_memory_cached_path() {
            path
        } else {
            match Self::download_fresh_github_release(language_server_id, binary_name) {
                Ok(path) => path,
                Err(err) => Self::resolve_file_cached_path(binary_name).ok_or(err)?,
            }
        };

        self.cached_binary_path = Some(binary_path.clone());

        Ok(zed::Command {
            command: binary_path,
            args,
            env,
        })
    }

    fn resolve_project_local_install(
        worktree: &zed::Worktree,
        binary_name: &str,
    ) -> Option<String> {
        let worktree_root_path = worktree.root_path();
        let worktree_root_path = std::path::Path::new(&worktree_root_path);

        if let Some(path) = Self::resolve_venv_install(worktree_root_path, binary_name) {
            return Some(path);
        }

        if let Some(path) = Self::resolve_node_modules_install(worktree_root_path, binary_name) {
            return Some(path);
        }

        worktree.which("tombi")
    }

    fn resolve_venv_install(
        worktree_root_path: &std::path::Path,
        binary_name: &str,
    ) -> Option<String> {
        let venv_script_dir = match zed::current_platform() {
            (zed::Os::Windows, _) => "Scripts",
            _ => "bin",
        };

        let venv_bin_path =
            worktree_root_path.join(format!(".venv/{venv_script_dir}/{binary_name}"));
        venv_bin_path
            .is_file()
            .then(|| venv_bin_path.to_string_lossy().to_string())
    }

    fn resolve_node_modules_install(
        worktree_root_path: &std::path::Path,
        binary_name: &str,
    ) -> Option<String> {
        let candidate_names: &[&str] = match zed::current_platform() {
            (zed::Os::Windows, _) => &["tombi.cmd", "tombi.ps1", binary_name],
            _ => &[binary_name],
        };

        candidate_names.iter().find_map(|candidate_name| {
            let node_modules_bin_path =
                worktree_root_path.join(format!("node_modules/.bin/{candidate_name}"));
            node_modules_bin_path
                .is_file()
                .then(|| node_modules_bin_path.to_string_lossy().to_string())
        })
    }

    fn resolve_memory_cached_path(&self) -> Option<String> {
        self.cached_binary_path
            .as_ref()
            .filter(|path| fs::metadata(path).is_ok_and(|stat| stat.is_file()))
            .cloned()
    }

    fn download_fresh_github_release(
        language_server_id: &LanguageServerId,
        binary_name: &str,
    ) -> Result<String> {
        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );
        let release = zed::latest_github_release(
            "tombi-toml/tombi",
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        let (platform, arch) = zed::current_platform();
        let version = release
            .version
            .strip_prefix("v")
            .unwrap_or(&release.version);

        let asset_stem = format!(
            "tombi-cli-{version}-{arch}-{os}",
            arch = match arch {
                zed::Architecture::Aarch64 => "aarch64",
                zed::Architecture::X86 => "x86",
                zed::Architecture::X8664 => "x86_64",
            },
            os = match platform {
                zed::Os::Mac => "apple-darwin",
                zed::Os::Linux => "unknown-linux-musl",
                zed::Os::Windows => "pc-windows-msvc",
            }
        );
        let asset_name = format!(
            "{asset_stem}.{suffix}",
            suffix = match platform {
                zed::Os::Windows => "zip",
                _ if Self::uses_legacy_unix_artifact(version) => "gz",
                _ => "tar.gz",
            }
        );

        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == asset_name)
            .ok_or_else(|| format!("no asset found matching {asset_name:?}"))?;

        let version_dir = format!("{VERSION_DIR_PREFIX}{version}");
        fs::create_dir_all(&version_dir)
            .map_err(|err| format!("failed to create directory '{version_dir}': {err}"))?;
        let binary_path = format!("{version_dir}/{binary_name}");

        if !fs::metadata(&binary_path).is_ok_and(|stat| stat.is_file()) {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );
            let file_kind = match platform {
                zed::Os::Windows => zed::DownloadedFileType::Zip,
                _ if Self::uses_legacy_unix_artifact(version) => zed::DownloadedFileType::Gzip,
                _ => zed::DownloadedFileType::GzipTar,
            };

            match file_kind {
                zed::DownloadedFileType::Zip => {
                    zed::download_file(&asset.download_url, &version_dir, file_kind)
                        .map_err(|e| format!("failed to download file: {e}"))?;
                }
                zed::DownloadedFileType::GzipTar => {
                    zed::download_file(&asset.download_url, &version_dir, file_kind)
                        .map_err(|e| format!("failed to download file: {e}"))?;
                    fs::rename(
                        format!("{version_dir}/{asset_stem}/{binary_name}"),
                        &binary_path,
                    )
                    .map_err(|err| format!("failed to relocate extracted binary: {err}"))?;
                    zed::make_file_executable(&binary_path)?;
                }
                _ => {
                    zed::download_file(&asset.download_url, &binary_path, file_kind)
                        .map_err(|e| format!("failed to download file: {e}"))?;
                    zed::make_file_executable(&binary_path)?;
                }
            }

            let entries = fs::read_dir(".")
                .map_err(|err| format!("failed to list working directory {err}"))?;
            for entry in entries {
                let entry = entry.map_err(|err| format!("failed to load directory entry {err}"))?;
                if entry.file_name().to_str() != Some(&version_dir) {
                    fs::remove_dir_all(entry.path()).ok();
                }
            }
        }

        Ok(binary_path)
    }

    // Keep the 0.9.23 cutoff in sync with docs/public/install.sh
    // (version_uses_legacy_unix_artifact).
    fn uses_legacy_unix_artifact(version: &str) -> bool {
        Self::parse_release_version(version).is_some_and(|version| version < (0, 9, 23))
    }

    fn parse_release_version(version: &str) -> Option<(u64, u64, u64)> {
        let version = version
            .split_once(['-', '+'])
            .map(|(prefix, _)| prefix)
            .unwrap_or(version);

        let mut parts = version.split('.');
        let (Some(major), Some(minor), Some(patch), None) =
            (parts.next(), parts.next(), parts.next(), parts.next())
        else {
            return None;
        };

        Some((
            major.parse::<u64>().ok()?,
            minor.parse::<u64>().ok()?,
            patch.parse::<u64>().ok()?,
        ))
    }

    fn resolve_file_cached_path(binary_name: &str) -> Option<String> {
        fs::read_dir(".")
            .ok()?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                let file_name = path.file_name()?.to_str()?;
                if !file_name.starts_with(VERSION_DIR_PREFIX) {
                    return None;
                }

                let version = file_name.strip_prefix(VERSION_DIR_PREFIX)?;
                let version = Self::parse_release_version(version)?;
                let binary_path = path.join(binary_name);
                let metadata = fs::metadata(&binary_path).ok()?;
                if !metadata.is_file() {
                    return None;
                }

                Some((version, binary_path))
            })
            .max_by_key(|(version, _)| *version)
            .map(|(_, path)| path.to_string_lossy().to_string())
    }
}

impl zed::Extension for TombiExtension {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        self.make_language_server_command(language_server_id, worktree)
    }

    fn language_server_initialization_options(
        &mut self,
        server_id: &LanguageServerId,
        worktree: &zed_extension_api::Worktree,
    ) -> Result<Option<zed_extension_api::serde_json::Value>> {
        let settings = LspSettings::for_worktree(server_id.as_ref(), worktree)
            .ok()
            .and_then(|lsp_settings| lsp_settings.initialization_options.clone())
            .unwrap_or_default();
        Ok(Some(settings))
    }

    fn language_server_workspace_configuration(
        &mut self,
        server_id: &LanguageServerId,
        worktree: &zed_extension_api::Worktree,
    ) -> Result<Option<zed_extension_api::serde_json::Value>> {
        let settings = LspSettings::for_worktree(server_id.as_ref(), worktree)
            .ok()
            .and_then(|lsp_settings| lsp_settings.settings.clone())
            .unwrap_or_default();
        Ok(Some(settings))
    }
}

zed::register_extension!(TombiExtension);
