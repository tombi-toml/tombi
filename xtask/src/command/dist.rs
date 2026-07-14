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

#[derive(clap::Args, Debug)]
pub struct Args {
    /// Build and package only the CLI tarball, skipping the VS Code extension.
    #[arg(long)]
    pub cli_only: bool,
}

pub fn run(sh: &Shell, args: Args) -> Result<(), anyhow::Error> {
    let project_root = project_root_path();
    let target = Target::get(&project_root);
    let vscode_target = resolve_vscode_target(args.cli_only);
    let dist = project_root.join("dist");

    println!("Target: {target:#?}");
    println!("VSCode target: {vscode_target:#?}");

    sh.remove_path(&dist)?;
    sh.create_dir(&dist)?;

    dist_server(sh, &target)?;
    if let Some(vscode_target) = &vscode_target {
        dist_client(sh, &target, vscode_target)?;
    }

    Ok(())
}

fn dist_server(sh: &Shell, target: &Target) -> Result<(), anyhow::Error> {
    let target_name = &target.target_name;

    if target_name.contains("-linux-") {
        unsafe {
            std::env::set_var("CC", "clang");
        }
    }

    // Setting TOMBI_DIST_CARGO=cross allows for cross-compilation to other platforms.
    let cargo = std::env::var("TOMBI_DIST_CARGO").unwrap_or_else(|_| "cargo".to_owned());

    let _dir = sh.push_dir(project_root_path());
    xshell::cmd!(
        sh,
        // cross mounts the workspace at /project in its container, so deriving
        // a manifest path from `project_root_path()` and passing it in as
        // `--manifest-path` is not going to work. Select the tombi-cli crate by
        // name instead, which should work in all cases.
        "{cargo} build --locked -p tombi-cli --bin tombi --target {target_name} --release"
    )
    .run()?;

    let dist = project_root_path().join("dist");
    if target_name.contains("-windows-") {
        zip(
            &target.server_path,
            target.symbols_path.as_ref(),
            &dist.join(&target.cli_artifact_name),
        )?;
    } else {
        tar_gz(
            &target.server_path,
            &target.cli_artifact_dir_name,
            &dist.join(&target.cli_artifact_name),
        )?;
    }

    Ok(())
}

fn dist_client(
    sh: &Shell,
    target: &Target,
    vscode_target: &VscodeTarget,
) -> Result<(), anyhow::Error> {
    dist_editor_vscode(sh, target, vscode_target)
}

fn dist_editor_vscode(
    sh: &Shell,
    target: &Target,
    vscode_target: &VscodeTarget,
) -> Result<(), anyhow::Error> {
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

    let vscode_target_name = &vscode_target.vscode_target_name;
    let vscode_artifact_name = &vscode_target.vscode_artifact_name;

    let _d = sh.push_dir(vscode_path);

    // FIXME: pnpm cannot exec `cargo xtask dist` on windows.
    //        See https://github.com/matklad/xshell/issues/82
    if !cfg!(target_os = "windows") {
        xshell::cmd!(
            sh,
            "pnpm exec vsce package --no-dependencies -o ../../dist/{vscode_artifact_name} --target {vscode_target_name}"
        )
        .run()?;
    }

    Ok(())
}

fn dist_version() -> String {
    std::env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| DEV_VERSION.to_owned())
}

#[derive(Debug)]
struct Target {
    target_name: String,
    exe_name: String,
    server_path: PathBuf,
    symbols_path: Option<PathBuf>,
    cli_artifact_dir_name: String,
    cli_artifact_name: String,
}

impl Target {
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
        let version = dist_version();

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
        } else {
            (String::new(), ".tar.gz".to_string(), None)
        };
        let exe_name = format!("tombi{exe_suffix}");
        let server_path = out_path.join(&exe_name);
        let cli_artifact_dir_name = format!("tombi-cli-{version}-{target_name}");
        let cli_artifact_name = format!("{cli_artifact_dir_name}{cli_artifact_suffix}");

        Self {
            target_name,
            exe_name,
            server_path,
            symbols_path,
            cli_artifact_dir_name,
            cli_artifact_name,
        }
    }
}

#[derive(Debug)]
struct VscodeTarget {
    vscode_target_name: String,
    vscode_artifact_name: String,
}

impl VscodeTarget {
    fn get() -> Self {
        let vscode_target_name = match std::env::var("VSCODE_TARGET") {
            Ok(target) if !target.is_empty() => target,
            Ok(_) => panic!(
                "VSCODE_TARGET is set but empty. Pass in --cli-only to \
                 skip the VS Code extension, or set VSCODE_TARGET to a vsce target."
            ),
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
        let version = dist_version();
        let vscode_artifact_name = format!("tombi-vscode-{version}-{vscode_target_name}.vsix");

        Self {
            vscode_target_name,
            vscode_artifact_name,
        }
    }
}

fn resolve_vscode_target(cli_only: bool) -> Option<VscodeTarget> {
    (!cli_only).then(VscodeTarget::get)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::glue::pushenv;

    #[test]
    fn unix_targets_always_use_tar_gz() {
        let _darwin = pushenv("TOMBI_TARGET", "aarch64-apple-darwin");
        let target = Target::get(Path::new("."));
        assert_eq!(
            target.cli_artifact_name,
            format!("tombi-cli-{DEV_VERSION}-aarch64-apple-darwin.tar.gz")
        );

        let _illumos = pushenv("TOMBI_TARGET", "x86_64-unknown-illumos");
        let target = Target::get(Path::new("."));
        assert_eq!(
            target.cli_artifact_name,
            format!("tombi-cli-{DEV_VERSION}-x86_64-unknown-illumos.tar.gz")
        );
    }

    #[test]
    fn empty_vscode_target_only_panics_when_vscode_dist_is_requested() {
        let _guard = pushenv("VSCODE_TARGET", "");

        assert!(resolve_vscode_target(true).is_none());

        let panic = std::panic::catch_unwind(|| resolve_vscode_target(false))
            .expect_err("resolving the VS Code target with empty VSCODE_TARGET panicked");
        let message = panic
            .downcast_ref::<&str>()
            .copied()
            .expect("panic payload was a str literal");
        assert!(message.contains("VSCODE_TARGET is set but empty"));
    }
}
