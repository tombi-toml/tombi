use std::{
    fs::File,
    io::{self, BufWriter},
    path::{Path, PathBuf},
};

use flate2::{Compression, write::GzEncoder};
use tar::Builder as TarBuilder;
use time::OffsetDateTime;
use xshell::Shell;
use zip::{DateTime, ZipWriter, write::FileOptions};

use super::set_version::DEV_VERSION;
use crate::utils::project_root_path;

// Keep this cutoff in sync with:
//   docs/public/install.sh (version_uses_legacy_unix_artifact)
//   editors/zed/src/lib.rs (TombiExtension::uses_legacy_unix_artifact)
//   .github/workflows/release_cli_vscode.yml ("Set CLI artifact extension" step)
const UNIX_ARCHIVE_FORMAT_CUTOFF: (u64, u64, u64) = (0, 9, 23);

pub fn run(sh: &Shell) -> Result<(), anyhow::Error> {
    let project_root = project_root_path();
    let target = Target::get(&project_root);
    let dist = project_root.join("dist");

    println!("Target: {target:#?}");

    sh.remove_path(&dist)?;
    sh.create_dir(&dist)?;

    dist_server(sh, &target)?;
    dist_client(sh, &target)?;

    Ok(())
}

fn dist_server(sh: &Shell, target: &Target) -> Result<(), anyhow::Error> {
    let target_name = &target.target_name;

    if target_name.contains("-linux-") {
        unsafe {
            std::env::set_var("CC", "clang");
        }
    }

    let manifest_path = project_root_path()
        .join("rust")
        .join("tombi-cli")
        .join("Cargo.toml");

    xshell::cmd!(
        sh,
        "cargo build --locked --manifest-path {manifest_path} --bin tombi --target {target_name} --release"
    )
    .run()?;

    let dist = project_root_path().join("dist");
    if target_name.contains("-windows-") {
        zip(
            &target.server_path,
            target.symbols_path.as_ref(),
            &dist.join(&target.cli_artifact_name),
        )?;
    } else if Target::uses_legacy_unix_artifact(&target.version) {
        gzip(&target.server_path, &dist.join(&target.cli_artifact_name))?;
    } else {
        tar_gz(
            &target.server_path,
            &target.cli_artifact_dir_name,
            &dist.join(&target.cli_artifact_name),
        )?;
    }

    Ok(())
}

fn dist_client(sh: &Shell, target: &Target) -> Result<(), anyhow::Error> {
    dist_editor_vscode(sh, target)
}

fn dist_editor_vscode(sh: &Shell, target: &Target) -> Result<(), anyhow::Error> {
    let vscode_path = project_root_path().join("editors").join("vscode");
    let bundle_path = vscode_path.join("server");
    sh.remove_path(&bundle_path)?;
    sh.create_dir(&bundle_path)?;

    let readme_path = vscode_path.join("README.md");
    let readme = sh.read_file(&readme_path)?;
    let readme = readme.replace("tombi.svg", "tombi.jpg");
    sh.write_file(&readme_path, &readme)?;

    if !target.server_path.exists() {
        return Err(anyhow::anyhow!(
            "CLI binary not found at {}. Please run `cargo build --package tombi-cli --release` first.",
            target.server_path.display()
        ));
    }

    sh.copy_file(&target.server_path, bundle_path.join(&target.exe_name))?;
    if let Some(symbols_path) = &target.symbols_path {
        sh.copy_file(symbols_path, &bundle_path)?;
    }

    let vscode_target = &target.vscode_target_name;
    let vscode_artifact_name = &target.vscode_artifact_name;

    let _d = sh.push_dir(vscode_path);

    // FIXME: pnpm cannot exec `cargo xtask dist` on windows.
    //        See https://github.com/matklad/xshell/issues/82
    if !cfg!(target_os = "windows") {
        xshell::cmd!(
            sh,
            "pnpm exec vsce package --no-dependencies -o ../../dist/{vscode_artifact_name} --target {vscode_target}"
        )
        .run()?;
    }

    Ok(())
}

#[derive(Debug)]
struct Target {
    version: String,
    target_name: String,
    vscode_target_name: String,
    exe_name: String,
    server_path: PathBuf,
    symbols_path: Option<PathBuf>,
    cli_artifact_dir_name: String,
    cli_artifact_name: String,
    vscode_artifact_name: String,
}

impl Target {
    fn uses_legacy_unix_artifact(version: &str) -> bool {
        if version == DEV_VERSION {
            return false;
        }

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

        (major, minor, patch) < UNIX_ARCHIVE_FORMAT_CUTOFF
    }

    fn get(project_root: &Path) -> Self {
        let target_name = match std::env::var("TOMBI_TARGET") {
            Ok(target) => target,
            _ => {
                if cfg!(target_os = "linux") {
                    "x86_64-unknown-linux-gnu".to_owned()
                } else if cfg!(target_os = "windows") {
                    "x86_64-pc-windows-msvc".to_owned()
                } else if cfg!(target_os = "macos") {
                    "aarch64-apple-darwin".to_owned()
                } else {
                    panic!("Unsupported OS, maybe try setting TOMBI_TARGET")
                }
            }
        };
        let vscode_target_name = match std::env::var("VSCODE_TARGET") {
            Ok(target) => target,
            _ => {
                if cfg!(target_os = "linux") {
                    "linux-x64".to_owned()
                } else if cfg!(target_os = "windows") {
                    "win32-x64".to_owned()
                } else if cfg!(target_os = "macos") {
                    "darwin-arm64".to_owned()
                } else {
                    panic!("Unsupported OS, maybe try setting VSCODE_TARGET")
                }
            }
        };
        let version = std::env::var("CARGO_PKG_VERSION").unwrap_or(DEV_VERSION.to_owned());

        let out_path = project_root
            .join("target")
            .join(&target_name)
            .join("release");
        let (exe_suffix, cli_artifact_suffix, symbols_path) = if target_name.contains("-windows-") {
            (
                ".exe".into(),
                ".zip".to_string(),
                Some(out_path.join("tombi.pdb")),
            )
        } else if Self::uses_legacy_unix_artifact(&version) {
            (String::new(), ".gz".to_string(), None)
        } else {
            (String::new(), ".tar.gz".to_string(), None)
        };
        let exe_name = format!("tombi{exe_suffix}");
        let server_path = out_path.join(&exe_name);
        let cli_artifact_dir_name = format!("tombi-cli-{version}-{target_name}");
        let cli_artifact_name = format!("{cli_artifact_dir_name}{cli_artifact_suffix}");
        let vscode_artifact_name = format!("tombi-vscode-{version}-{vscode_target_name}.vsix");

        Self {
            version,
            target_name,
            vscode_target_name,
            exe_name,
            server_path,
            symbols_path,
            cli_artifact_dir_name,
            cli_artifact_name,
            vscode_artifact_name,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_cutoff_boundaries() {
        assert!(Target::uses_legacy_unix_artifact("0.9.22"));
        assert!(Target::uses_legacy_unix_artifact("0.8.99"));
        assert!(Target::uses_legacy_unix_artifact("0.0.1"));
        assert!(!Target::uses_legacy_unix_artifact("0.9.23"));
        assert!(!Target::uses_legacy_unix_artifact("0.9.24"));
        assert!(!Target::uses_legacy_unix_artifact("0.10.0"));
        assert!(!Target::uses_legacy_unix_artifact("1.0.0"));
    }

    #[test]
    fn legacy_cutoff_invalid_inputs() {
        assert!(!Target::uses_legacy_unix_artifact("0.9"));
        assert!(!Target::uses_legacy_unix_artifact("0.9.23.1"));
        assert!(!Target::uses_legacy_unix_artifact("invalid"));
        assert!(!Target::uses_legacy_unix_artifact(""));
        assert!(!Target::uses_legacy_unix_artifact("0.9.x"));
    }

    #[test]
    fn legacy_cutoff_prereleases_follow_base_version() {
        assert!(Target::uses_legacy_unix_artifact("0.9.22-rc.1"));
        assert!(Target::uses_legacy_unix_artifact("0.9.22+build.7"));
        assert!(!Target::uses_legacy_unix_artifact("0.9.23-rc.1"));
    }

    #[test]
    fn legacy_cutoff_dev_version_uses_new_format() {
        assert!(!Target::uses_legacy_unix_artifact(DEV_VERSION));
    }
}

fn gzip(src_path: &Path, dest_path: &Path) -> anyhow::Result<()> {
    let mut encoder = GzEncoder::new(File::create(dest_path)?, Compression::best());
    let mut input = std::io::BufReader::new(File::open(src_path)?);
    std::io::copy(&mut input, &mut encoder)?;
    encoder.finish()?;
    Ok(())
}

fn tar_gz(src_path: &Path, root_dir: &str, dest_path: &Path) -> anyhow::Result<()> {
    let encoder = GzEncoder::new(File::create(dest_path)?, Compression::best());
    let mut archive = TarBuilder::new(encoder);
    let archive_path = format!(
        "{root_dir}/{}",
        src_path.file_name().unwrap().to_str().unwrap()
    );
    archive.append_path_with_name(src_path, archive_path)?;
    archive.finish()?;
    archive.into_inner()?.finish()?;
    Ok(())
}

fn zip(src_path: &Path, symbols_path: Option<&PathBuf>, dest_path: &Path) -> anyhow::Result<()> {
    let file = File::create(dest_path)?;
    let mut writer = ZipWriter::new(BufWriter::new(file));
    writer.start_file(
        src_path.file_name().unwrap().to_str().unwrap(),
        FileOptions::<()>::default()
            .last_modified_time(
                DateTime::try_from(OffsetDateTime::from(
                    std::fs::metadata(src_path)?.modified()?,
                ))
                .unwrap(),
            )
            .unix_permissions(0o755)
            .compression_method(zip::CompressionMethod::Deflated)
            .compression_level(Some(9)),
    )?;
    let mut input = io::BufReader::new(File::open(src_path)?);
    io::copy(&mut input, &mut writer)?;
    if let Some(symbols_path) = symbols_path {
        writer.start_file(
            symbols_path.file_name().unwrap().to_str().unwrap(),
            FileOptions::<()>::default()
                .last_modified_time(
                    DateTime::try_from(OffsetDateTime::from(
                        std::fs::metadata(src_path)?.modified()?,
                    ))
                    .unwrap(),
                )
                .compression_method(zip::CompressionMethod::Deflated)
                .compression_level(Some(9)),
        )?;
        let mut input = io::BufReader::new(File::open(symbols_path)?);
        io::copy(&mut input, &mut writer)?;
    }
    writer.finish()?;
    Ok(())
}
