use std::{env, io, process::Command};

use clap::CommandFactory;

include!("src/args.rs");

fn main() -> io::Result<()> {
    let re = regex::Regex::new(r"^v\d+\.\d+\.\d+$").unwrap();

    // Try to get version from git tag
    let git_version = Command::new("git")
        .args(["tag", "--points-at", "HEAD"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                if let Ok(tags) = String::from_utf8(output.stdout) {
                    for tag in tags.split('\n') {
                        if re.is_match(tag.trim()) {
                            return Some(tag.trim().to_string());
                        }
                    }
                }
            }
            None
        })
        .unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string());

    println!("cargo:rustc-env=__TOMBI_VERSION={git_version}");
    println!("cargo:rerun-if-changed=.git/HEAD");

    if let Ok(out_dir) = env::var("OUT_DIR") {
        let cmd = Args::command().about("TOML Toolkit");
        clap_mangen::generate_to(cmd, out_dir)?;
    }
    Ok(())
}
