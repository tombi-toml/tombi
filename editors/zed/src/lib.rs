use std::fs;
use zed::LanguageServerId;
use zed_extension_api::{self as zed, settings::LspSettings, Result};

struct TombiBinary {
    path: String,
    args: Vec<String>,
}

struct TombiExtension {
    cached_binary_path: Option<String>,
}

impl TombiExtension {
    fn language_server_binary(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<TombiBinary> {
        let lsp_settings = LspSettings::for_worktree(language_server_id.as_ref(), worktree).ok();
        let binary_settings = lsp_settings
            .as_ref()
            .and_then(|lsp_settings| lsp_settings.binary.as_ref());

        let args = binary_settings
            .and_then(|binary_settings| binary_settings.arguments.as_ref())
            .cloned()
            .unwrap_or_else(|| vec!["lsp".to_string()]);

        if let Some(path) = binary_settings
            .and_then(|binary_settings| binary_settings.path.clone())
        {
            return Ok(TombiBinary { path, args });
        }

        let worktree_root_path = worktree.root_path();
        let worktree_root_path = std::path::Path::new(&worktree_root_path);

        let (venv_script_dir, binary_name) = match zed::current_platform() {
            (zed::Os::Windows, _) => ("Scripts", "tombi.exe"),
            _ => ("bin", "tombi"),
        };

        let venv_bin_path =
            worktree_root_path.join(format!(".venv/{venv_script_dir}/{binary_name}"));
        if venv_bin_path.is_file() {
            return Ok(TombiBinary {
                path: venv_bin_path.to_string_lossy().to_string(),
                args,
            });
        }

        let node_modules_bin_path = worktree_root_path.join("node_modules/.bin/tombi");
        if node_modules_bin_path.is_file() {
            return Ok(TombiBinary {
                path: node_modules_bin_path.to_string_lossy().to_string(),
                args,
            });
        }

        if let Some(path) = worktree.which("tombi") {
            return Ok(TombiBinary { path, args });
        }

        if let Some(path) = &self.cached_binary_path {
            if fs::metadata(path).is_ok_and(|stat| stat.is_file()) {
                return Ok(TombiBinary {
                    path: path.clone(),
                    args,
                });
            }
        }

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

        let version_dir = format!("tombi-{version}");
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

        self.cached_binary_path = Some(binary_path.clone());
        Ok(TombiBinary {
            path: binary_path,
            args,
        })
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
        let tombi_binary = self.language_server_binary(language_server_id, worktree)?;
        Ok(zed::Command {
            command: tombi_binary.path,
            args: tombi_binary.args,
            env: vec![],
        })
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
