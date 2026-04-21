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

        let node_modules_bin_path = worktree_root_path.join("node_modules/.bin/tombi");
        if node_modules_bin_path.is_file() {
            return Some(node_modules_bin_path.to_string_lossy().to_string());
        }

        worktree.which("tombi")
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
        let asset_name = format!(
            "{asset_stem}.{suffix}",
            suffix = match platform {
                zed::Os::Windows => "zip",
                _ => "gz",
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
                _ => zed::DownloadedFileType::Gzip,
            };

            match platform {
                zed::Os::Windows => {
                    zed::download_file(&asset.download_url, &version_dir, file_kind)
                        .map_err(|e| format!("failed to download file: {e}"))?;
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

                let binary_path = path.join(binary_name);
                let metadata = fs::metadata(&binary_path).ok()?;
                if !metadata.is_file() {
                    return None;
                }

                let modified = metadata.modified().ok()?;
                Some((modified, binary_path))
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
