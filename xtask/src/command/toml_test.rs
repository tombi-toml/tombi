use std::io::Write;

use tombi_cli_options::VerbosityLevel;
use tombi_config::TomlVersion;
use xshell::Shell;

use crate::utils::project_root_path;

#[derive(clap::Args, Debug)]
pub struct Args {
    #[arg(long, default_value_t= TomlVersion::default())]
    toml_version: TomlVersion,
}

pub fn run(sh: &Shell, verbosity: tombi_cli_options::Verbosity, args: Args) -> anyhow::Result<()> {
    let project_root = project_root_path();

    sh.change_dir(&project_root);

    xshell::cmd!(sh, "cargo build --bin decode").run()?;

    decode_test(sh, &project_root, args.toml_version, verbosity);

    Ok(())
}

fn decode_test(
    sh: &Shell,
    project_root: &std::path::Path,
    toml_version: TomlVersion,
    verbosity: tombi_cli_options::Verbosity,
) {
    let toml_test_version = toml_test_version(toml_version);
    let toml_version_str = serde_json::to_string(&toml_version).unwrap_or_default();
    let toml_version_str = toml_version_str.trim_matches('"');
    let decoder = format!(
        "{}/target/debug/decode --toml-version {toml_version_str}",
        project_root.display()
    );

    let mut options = vec![];
    if verbosity.verbosity_level() != VerbosityLevel::Default {
        options.push("-v");
    };

    match xshell::cmd!(
        sh,
        "toml-test test {options...} -color=never -timeout=10s -toml={toml_test_version} -decoder {decoder}"
    )
    .ignore_status()
    .output()
    {
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

#[allow(deprecated)]
const fn toml_test_version(toml_version: TomlVersion) -> &'static str {
    match toml_version {
        TomlVersion::V1_0_0 => "1.0",
        TomlVersion::V1_1_0_Preview | TomlVersion::V1_1_0 => "1.1",
    }
}
