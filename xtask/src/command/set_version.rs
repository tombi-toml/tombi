use std::path::PathBuf;

use xshell::Shell;

use crate::utils::project_root_path;

pub const DEV_VERSION: &str = "0.0.0-dev";

pub fn run(sh: &Shell) -> anyhow::Result<()> {
    let version = match std::env::var("TOMBI_VERSION") {
        Ok(version) if !version.is_empty() => version,
        _ => match std::env::var("GITHUB_REF") {
            Ok(github_ref) if github_ref.starts_with("refs/tags/v") => {
                github_ref.trim_start_matches("refs/tags/v").to_owned()
            }
            _ => {
                eprint!("INFO: If you want to set a specific version, please use the GITHUB_REF environment variable.\n\n");
                eprint!("      $ GITHUB_REF=refs/tags/v1.2.3 cargo xtask set-version\n\n");
                DEV_VERSION.to_owned()
            }
        },
    };

    set_cargo_toml_version(sh, &version)?;
    set_editors_vscode_package_json_version(sh, &version)?;
    set_pyproject_toml_version(sh, &version)?;
    set_package_json_versions(sh, &version)?;

    println!("TOMBI_VERSION={version}");

    Ok(())
}

fn set_cargo_toml_version(sh: &Shell, version: &str) -> anyhow::Result<()> {
    let project_root = project_root_path();
    let mut patch = Patch::new(sh, project_root.join("Cargo.toml"))?;
    patch.replace(
        &format!(r#"version = "{DEV_VERSION}""#),
        &format!(r#"version = "{version}""#),
    );
    patch.commit(sh)?;
    Ok(())
}

fn set_editors_vscode_package_json_version(sh: &Shell, version: &str) -> anyhow::Result<()> {
    let mut patch = Patch::new(
        sh,
        project_root_path()
            .join("editors")
            .join("vscode")
            .join("package.json"),
    )?;

    patch.replace(
        &format!(r#""version": "{DEV_VERSION}""#),
        &format!(r#""version": "{version}""#),
    );

    patch.commit(sh)?;
    Ok(())
}

fn set_pyproject_toml_version(sh: &Shell, version: &str) -> anyhow::Result<()> {
    let mut patch = Patch::new(sh, project_root_path().join("pyproject.toml"))?;
    patch.replace(
        &format!(r#"version = "{DEV_VERSION}""#),
        &format!(r#"version = "{version}""#),
    );
    patch.commit(sh)?;
    Ok(())
}

fn set_package_json_versions(sh: &Shell, version: &str) -> anyhow::Result<()> {
    use std::fs;
    let pkgs_dir = project_root_path().join("typescript").join("@tombi-toml");
    for entry in fs::read_dir(&pkgs_dir)? {
        let entry = entry?;
        let path = entry.path();
        let is_main = path.file_name().unwrap() == "tombi";
        let is_cli = path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("cli-");
        if path.is_dir() && (is_main || is_cli) {
            let package_json = path.join("package.json");
            if package_json.exists() {
                let mut patch = Patch::new(sh, &package_json)?;
                patch.replace(&format!(r#""{DEV_VERSION}""#), &format!(r#""{version}""#));
                patch.commit(sh)?;
            }
        }
    }
    Ok(())
}

struct Patch {
    path: PathBuf,
    contents: String,
}

impl Patch {
    fn new(sh: &Shell, path: impl Into<PathBuf>) -> anyhow::Result<Patch> {
        let path = path.into();
        let contents = sh.read_file(&path)?;
        Ok(Patch { path, contents })
    }

    fn replace(&mut self, from: &str, to: &str) -> &mut Patch {
        pretty_assertions::assert_eq!(
            self.contents.contains(from),
            true,
            "{}",
            format!("Expected '{}' to be in '{}'", from, self.path.display())
        );
        self.contents = self.contents.replace(from, to);
        self
    }

    fn commit(&self, sh: &Shell) -> anyhow::Result<()> {
        sh.write_file(&self.path, &self.contents)?;
        Ok(())
    }
}
