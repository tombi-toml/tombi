use std::io::Write;

use tombi_config::TomlVersion;
use xshell::Shell;

use crate::utils::project_root_path;

#[derive(clap::Args, Debug)]
pub struct Args {
    #[arg(long, default_value_t= TomlVersion::default())]
    toml_version: TomlVersion,
}

pub fn run(sh: &Shell, args: Args) -> anyhow::Result<()> {
    let project_root = project_root_path();

    sh.change_dir(&project_root);

    xshell::cmd!(sh, "cargo build --bin decode").run()?;

    decode_test(sh, &project_root, args.toml_version);

    Ok(())
}

fn decode_test(sh: &Shell, project_root: &std::path::Path, toml_version: TomlVersion) {
    let toml_test_version = toml_test_version(toml_version);
    let toml_version_str = serde_json::to_string(&toml_version).unwrap_or_default();
    let toml_version_str = toml_version_str.trim_matches('"');

    match xshell::cmd!(
        sh,
        "toml-test -color=never -toml={toml_test_version} -- {project_root}/target/debug/decode --toml-version {toml_version_str}"
    ).ignore_status().output() {
        Ok(output) => {
            std::io::stdout().write_all(&output.stdout).unwrap();
            std::io::stderr().write_all(&output.stderr).unwrap();
            if !output.status.success() {
                std::process::exit(output.status.code().unwrap_or(1));
            }
        }
        Err(err) => {
            eprintln!("{err}");
        }
    }
}

const fn toml_test_version(toml_version: TomlVersion) -> &'static str {
    match toml_version {
        TomlVersion::V1_0_0 => "1.0.0",
        TomlVersion::V1_1_0_Preview => "1.1.0",
    }
}
