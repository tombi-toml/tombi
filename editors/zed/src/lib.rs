use std::fs;
use zed::LanguageServerId;
use zed_extension_api::{self as zed, Result, settings::LspSettings};

const VERSION_DIR_PREFIX: &str = "tombi-";

struct TombiExtension {
    cached_binary_path: Option<String>,
}

impl TombiExtension {
    // Keep the 0.9.23 cutoff in sync with:
    //   xtask/src/command/dist.rs (UNIX_ARCHIVE_FORMAT_CUTOFF)
    //   docs/public/install.sh (version_uses_legacy_unix_artifact)
    fn uses_legacy_unix_artifact(version: &str) -> bool {
        let version = version
            .split_once(['-', '+'])
            .map(|(prefix, _)| prefix)
            .unwrap_or(version);

        let mut parts = version.split('.');
        let (Some(major), Some(minor), Some(patch), None) =
            (parts.next(), parts.next(), parts.next(), parts.next())
        else {
            return false;
        };

        let (Ok(major), Ok(minor), Ok(patch)) = (
            major.parse::<u64>(),
            minor.parse::<u64>(),
            patch.parse::<u64>(),
        ) else {
            return false;
        };

        (major, minor, patch) < (0, 9, 23)
    }

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
        // 2. project-local installs (.venv, node_modules, PATH),
        // 3. the process-local cached path from a previous resolution,
        // 4. a fresh GitHub release download,
        // 5. fallback to an already installed extension-managed binary.
        if let Some(path) = binary_settings.and_then(|binary_settings| binary_settings.path.clone())
        {
            return Ok(zed::Command {
                command: path,
                args,
                env,
            });
        }

        let worktree_root_path = worktree.root_path();
        let worktree_root_path = std::path::Path::new(&worktree_root_path);

        let (venv_script_dir, binary_name) = match zed::current_platform() {
            (zed::Os::Windows, _) => ("Scripts", "tombi.exe"),
            _ => ("bin", "tombi"),
        };

        if let Some(path) = Self::resolve_project_local_install(
            worktree,
            worktree_root_path,
            venv_script_dir,
            binary_name,
        ) {
            return Ok(zed::Command {
                command: path,
                args,
                env,
            });
        }

        if let Some(path) = self.resolve_process_local_cached_path() {
            return Ok(zed::Command {
                command: path,
                args,
                env,
            });
        }

        let binary_path = match Self::download_fresh_github_release(language_server_id, binary_name)
        {
            Ok(path) => path,
            Err(err) => Self::resolve_extension_managed_binary_fallback(binary_name).ok_or(err)?,
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
        worktree_root_path: &std::path::Path,
        venv_script_dir: &str,
        binary_name: &str,
    ) -> Option<String> {
        let venv_bin_path =
            worktree_root_path.join(format!(".venv/{venv_script_dir}/{binary_name}"));
        if venv_bin_path.is_file() {
            return Some(venv_bin_path.to_string_lossy().to_string());
        }

        if let Some(path) = Self::resolve_node_modules_install(worktree_root_path, binary_name) {
            return Some(path);
        }

        worktree.which("tombi")
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

    fn resolve_process_local_cached_path(&self) -> Option<String> {
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

        let version_dir = format!("{VERSION_DIR_PREFIX}{version}");
        fs::create_dir_all(&version_dir)
            .map_err(|err| format!("failed to create directory '{version_dir}': {err}"))?;
        let binary_path = format!("{version_dir}/{binary_name}");

        let asset = match platform {
            zed::Os::Windows => release
                .assets
                .iter()
                .find(|asset| asset.name == format!("{asset_stem}.zip"))
                .map(|asset| (asset, zed::DownloadedFileType::Zip))
                .ok_or_else(|| format!("no asset found matching {:?}.zip", asset_stem))?,
            _ if Self::uses_legacy_unix_artifact(version) => release
                .assets
                .iter()
                .find(|asset| asset.name == format!("{asset_stem}.gz"))
                .map(|asset| (asset, zed::DownloadedFileType::Gzip))
                .or_else(|| {
                    release
                        .assets
                        .iter()
                        .find(|asset| asset.name == format!("{asset_stem}.tar.gz"))
                        .map(|asset| (asset, zed::DownloadedFileType::GzipTar))
                })
                .ok_or_else(|| format!("no asset found matching {asset_stem:?}.gz or .tar.gz"))?,
            _ => release
                .assets
                .iter()
                .find(|asset| asset.name == format!("{asset_stem}.tar.gz"))
                .map(|asset| (asset, zed::DownloadedFileType::GzipTar))
                .or_else(|| {
                    release
                        .assets
                        .iter()
                        .find(|asset| asset.name == format!("{asset_stem}.gz"))
                        .map(|asset| (asset, zed::DownloadedFileType::Gzip))
                })
                .ok_or_else(|| format!("no asset found matching {asset_stem:?}.tar.gz or .gz"))?,
        };

        if Self::find_binary_in_dir(std::path::Path::new(&version_dir), binary_name).is_none() {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );

            match asset.1 {
                zed::DownloadedFileType::Gzip => {
                    zed::download_file(&asset.0.download_url, &binary_path, asset.1)
                        .map_err(|e| format!("failed to download file: {e}"))?;
                    zed::make_file_executable(&binary_path)?;
                }
                zed::DownloadedFileType::Zip | zed::DownloadedFileType::GzipTar => {
                    zed::download_file(&asset.0.download_url, &version_dir, asset.1)
                        .map_err(|e| format!("failed to download file: {e}"))?;
                    if platform != zed::Os::Windows {
                        let downloaded_binary_path = Self::find_binary_in_dir(
                            std::path::Path::new(&version_dir),
                            binary_name,
                        )
                        .ok_or_else(|| {
                            format!(
                                "failed to locate {binary_name:?} after extracting {:?}",
                                asset.0.name
                            )
                        })?;
                        zed::make_file_executable(&downloaded_binary_path)?;
                    }
                }
                zed::DownloadedFileType::Uncompressed => {
                    return Err("unexpected uncompressed release asset".to_string());
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

        Self::find_binary_in_dir(std::path::Path::new(&version_dir), binary_name)
            .ok_or_else(|| format!("failed to locate installed binary {binary_name:?}"))
    }

    fn find_binary_in_dir(dir: &std::path::Path, binary_name: &str) -> Option<String> {
        let mut pending = vec![dir.to_path_buf()];

        while let Some(dir) = pending.pop() {
            let entries = fs::read_dir(&dir).ok()?;
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    pending.push(path);
                    continue;
                }

                if path.file_name().and_then(|file_name| file_name.to_str()) == Some(binary_name) {
                    return Some(path.to_string_lossy().to_string());
                }
            }
        }

        None
    }

    fn resolve_extension_managed_binary_fallback(binary_name: &str) -> Option<String> {
        fs::read_dir(".")
            .ok()?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                let file_name = path.file_name()?.to_str()?;
                if !file_name.starts_with(VERSION_DIR_PREFIX) {
                    return None;
                }

                let binary_path = Self::find_binary_in_dir(&path, binary_name)?;
                let metadata = fs::metadata(&binary_path).ok()?;
                if !metadata.is_file() {
                    return None;
                }

                let modified = metadata.modified().ok()?;
                Some((modified, std::path::PathBuf::from(binary_path)))
            })
            .max_by_key(|(modified, _)| *modified)
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
