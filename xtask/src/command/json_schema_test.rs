use std::io::Write;

use xshell::Shell;

use crate::utils::project_root_path;

#[derive(clap::Args, Debug)]
pub struct Args {
    /// Drafts to run (repeatable). Default: all Definition A drafts.
    #[arg(long = "draft")]
    drafts: Vec<String>,

    /// Only run suite files whose path contains this substring.
    #[arg(long)]
    filter: Option<String>,

    /// Stop after this many failures.
    #[arg(long)]
    max_failures: Option<usize>,

    /// How many failure details to print.
    #[arg(long, default_value_t = 50)]
    show_failures: usize,

    /// Exit 0 even when there are failures.
    #[arg(long, default_value_t = false)]
    allow_fail: bool,

    /// Fetch/update the vendored suite and exit.
    #[arg(long, default_value_t = false)]
    fetch_only: bool,
}

pub fn run(sh: &Shell, args: Args) -> anyhow::Result<()> {
    let project_root = project_root_path();
    sh.change_dir(&project_root);

    xshell::cmd!(sh, "cargo build -p json-schema-test --bin json-schema-test").run()?;

    let bin = format!("{}/target/debug/json-schema-test", project_root.display());

    let mut cmd_args: Vec<String> = Vec::new();
    for draft in &args.drafts {
        cmd_args.push("--draft".to_string());
        cmd_args.push(draft.clone());
    }
    if let Some(filter) = &args.filter {
        cmd_args.push("--filter".to_string());
        cmd_args.push(filter.clone());
    }
    if let Some(max_failures) = args.max_failures {
        cmd_args.push("--max-failures".to_string());
        cmd_args.push(max_failures.to_string());
    }
    cmd_args.push("--show-failures".to_string());
    cmd_args.push(args.show_failures.to_string());
    if args.allow_fail {
        cmd_args.push("--allow-fail".to_string());
    }
    if args.fetch_only {
        cmd_args.push("--fetch-only".to_string());
    }

    match xshell::cmd!(sh, "{bin} {cmd_args...}")
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
            std::process::exit(1);
        }
    }

    Ok(())
}
